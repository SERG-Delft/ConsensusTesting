use std::collections::HashSet;
use std::sync::Arc;
use log::error;
use itertools::Itertools;
use crate::message_handler::ParsedValidation;
use crate::node_state::{MutexNodeStates};
use crate::protos::ripple::{NodeEvent, TMStatusChange};

pub struct ConsensusProperties {}

impl ConsensusProperties {
    pub fn check_proposal_integrity_property(node_states: &Arc<MutexNodeStates>, status_change: &TMStatusChange, sender: usize) -> bool {
        if status_change.has_newEvent() && status_change.get_newEvent() == NodeEvent::neACCEPTED_LEDGER {
            let mut node_states_vec = node_states.node_states.lock();
            let already_present = node_states_vec.add_consensus_constructed_ledger(status_change.clone(), sender);
            if let Some(earlier_status_change) = already_present {
                if earlier_status_change != *status_change {
                    error!("(I1) Node has declared consensus on two transaction sets for the same ledger sequence\nOld: {:?}\nNew: {:?}", earlier_status_change, status_change);
                    return false;
                }
            }
        }
        true
    }

    pub fn check_validation_integrity_property(node_states: &Arc<MutexNodeStates>, validation: ParsedValidation, sender: usize) -> bool {
        let mut node_states_vec = node_states.node_states.lock();
        let already_present = node_states_vec.add_sent_validation(validation.clone(), sender);
        if let Some(earlier_validation) = already_present {
            if earlier_validation != validation {
                error!("(I2) Node validated twice for one ledger sequence\nOld: {:?}\nNew: {:?}", earlier_validation, validation);
                return false
            }
        }
        true
    }

    pub fn check_agreement_properties(node_states: &Arc<MutexNodeStates>) -> bool {
        let mut agreement = true;
        let node_states_vec = &node_states.node_states.lock().node_states;
        let seqs = node_states_vec.iter()
            .map(|node| node.validated_ledgers.keys().map(|key| *key).collect::<HashSet<usize>>())
            .flatten()
            .collect::<HashSet<usize>>();
        for seq in seqs {
            let validation_agreement = node_states_vec.iter()
                .filter_map(|node_state| node_state.validated_ledgers.get(&seq))
                .all_equal();
            let proposal_agreement = node_states_vec.iter()
                .filter_map(|node_state| node_state.consensus_constructed_ledgers.get(&seq))
                .map(|status_change| status_change.get_ledgerHash())
                .all_equal();
            if !validation_agreement {
                error!("(A2) Conflicting ledgers validated");
                agreement = false
            }
            if !proposal_agreement {
                error!("(A1) Conflicting ledgers created");
                agreement = false
            }
        }
        agreement
    }

    /// Check validity consensus properties
    /// V1 Check whether the transaction sets on which the nodes declared consensus are actually in the proposed transaction sets
    /// V2 Check whether the transaction sets (consensus_hash) in nodes' validation messages are actually in the proposed transaction sets
    pub fn check_validity_properties(node_states: &Arc<MutexNodeStates>) {
        let node_states_vec = &node_states.node_states.lock().node_states;
        let seqs = node_states_vec.iter().map(|node| node.proposed_tx_sets.keys().map(|key| *key).collect::<HashSet<usize>>()).flatten().collect::<HashSet<usize>>();
        for seq in seqs {
            let proposed_tx_sets = node_states_vec.iter()
                .filter_map(|node| node.proposed_tx_sets.get(&seq)).flatten()
                .map(|tx_sets| tx_sets.clone()).collect::<HashSet<Vec<u8>>>();
            // V1
            let consensus_tx_sets = node_states_vec.iter()
                .filter_map(|node| node.consensus_transaction_sets.get(&seq))
                .map(|tx_set| tx_set.clone()).collect::<HashSet<Vec<u8>>>();
            let is_v1_violated = consensus_tx_sets.difference(&proposed_tx_sets).count() > 0;
            // V2
            let validations = node_states_vec.iter()
                .filter_map(|node| node.validations_sent.get(&seq))
                .filter_map(|validation| match hex::decode(&validation.consensus_hash) {
                    Ok(consensus_tx_set) => Some(consensus_tx_set),
                    Err(_) => None,
                }).collect::<HashSet<Vec<u8>>>();
            let is_v2_violated = validations.difference(&proposed_tx_sets).count() > 0;
            if is_v1_violated {
                error!("(V1) Node declared consensus on a tx_set that was never proposed");
            }
            if is_v2_violated {
                error!("(V2) Node sent a validation for a ledger that was never constructed");
            }
        }
    }
}

#[cfg(test)]
mod consensus_properties_tests {
    use std::sync::Arc;
    use itertools::Itertools;
    use crate::client::ValidatedLedger;
    use crate::consensus_properties::ConsensusProperties;
    use crate::message_handler::ParsedValidation;
    use crate::node_state::{MutexNodeStates, NodeState, NodeStates};
    use crate::protos::ripple::{NodeEvent, TMStatusChange};

    #[test]
    fn test_check_agreement_properties() {
        let mut node_states_vec = setup_node_states(3);
        node_states_vec[0].consensus_constructed_ledgers.insert(0, TMStatusChange::default());
        node_states_vec[1].consensus_constructed_ledgers.insert(0, TMStatusChange::default());
        node_states_vec[0].validated_ledgers.insert(0, ValidatedLedger::default());
        node_states_vec[1].validated_ledgers.insert(0, ValidatedLedger::default());
        node_states_vec[2].validated_ledgers.insert(1, ValidatedLedger::default());
        let node_states = Arc::new(MutexNodeStates::new(NodeStates::new(node_states_vec)));
        assert!(ConsensusProperties::check_agreement_properties(&node_states));
        let mut different_validation = ValidatedLedger::default();
        different_validation.ledger_hash = "Different ledger hash".to_string();
        node_states.node_states.lock().node_states[1].validated_ledgers.insert(1, different_validation);
        assert!(!ConsensusProperties::check_agreement_properties(&node_states));
        node_states.node_states.lock().node_states[1].validated_ledgers.insert(1, ValidatedLedger::default());
        assert!(ConsensusProperties::check_agreement_properties(&node_states));
        let mut different_status_change = TMStatusChange::default();
        different_status_change.set_ledgerHash(vec![1, 2, 3]);
        node_states.node_states.lock().node_states[2].consensus_constructed_ledgers.insert(0, different_status_change);
        assert!(!ConsensusProperties::check_agreement_properties(&node_states));
    }

    #[test]
    fn test_check_proposal_integrity_properties() {
        let node_states_vec = setup_node_states(3);
        let node_states = Arc::new(MutexNodeStates::new(NodeStates::new(node_states_vec)));
        let mut status_change_1 = TMStatusChange::default();
        status_change_1.set_newEvent(NodeEvent::neACCEPTED_LEDGER);
        status_change_1.set_ledgerSeq(0);
        status_change_1.set_ledgerHash(vec![1]);
        assert_eq!(ConsensusProperties::check_proposal_integrity_property(&node_states, &status_change_1, 0), true);
        let mut status_change_2 = TMStatusChange::default();
        status_change_2.set_newEvent(NodeEvent::neACCEPTED_LEDGER);
        status_change_2.set_ledgerSeq(1);
        status_change_2.set_ledgerHash(vec![1, 2, 3]);
        assert_eq!(ConsensusProperties::check_proposal_integrity_property(&node_states, &status_change_2, 0), true);
        status_change_1.set_ledgerSeq(1);
        assert_eq!(ConsensusProperties::check_proposal_integrity_property(&node_states, &status_change_1, 0), false);
    }

    #[test]
    fn test_check_validation_integrity_properties() {
        let node_states_vec = setup_node_states(3);
        let node_states = Arc::new(MutexNodeStates::new(NodeStates::new(node_states_vec)));
        let mut validation_1 = ParsedValidation::default();
        validation_1.ledger_sequence = 0;
        validation_1.hash = "hash1".to_string();
        assert_eq!(ConsensusProperties::check_validation_integrity_property(&node_states, validation_1.clone(), 0), true);
        let mut validation_2 = ParsedValidation::default();
        validation_2.ledger_sequence = 1;
        validation_2.hash = "hash2".to_string();
        assert_eq!(ConsensusProperties::check_validation_integrity_property(&node_states, validation_2.clone(), 0), true);
        validation_1.ledger_sequence = 1;
        assert_eq!(ConsensusProperties::check_validation_integrity_property(&node_states, validation_1.clone(), 0), false);
    }

    #[test]
    fn test_check_validity_properties() {
        let node_states_vec = setup_node_states(3);
        let node_states = Arc::new(MutexNodeStates::new(NodeStates::new(node_states_vec)));
        let proposal_1 = [1];
        node_states.node_states.lock().add_proposed_tx_set(&proposal_1, 1);
        let mut validation_1 = ParsedValidation::default();
        validation_1.ledger_sequence = 3;
        validation_1.consensus_hash = "01".to_string();
        node_states.node_states.lock().add_sent_validation(validation_1, 1);
        // No violations
        ConsensusProperties::check_validity_properties(&node_states);
        let mut validation_1 = ParsedValidation::default();
        validation_1.ledger_sequence = 3;
        validation_1.consensus_hash = "02".to_string();
        node_states.node_states.lock().add_sent_validation(validation_1, 0);
        // Violations
        ConsensusProperties::check_validity_properties(&node_states);
    }

    fn setup_node_states(peer: usize) -> Vec<NodeState> {
        (0..peer).map(|x| NodeState::new(x)).collect_vec()
    }
}
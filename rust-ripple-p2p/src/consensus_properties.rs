use std::sync::Arc;
use log::error;
use itertools::Itertools;
use crate::message_handler::ParsedValidation;
use crate::node_state::{MutexNodeStates, NodeState};
use crate::protos::ripple::{NodeEvent, TMStatusChange};

pub struct ConsensusProperties {}

impl ConsensusProperties {
    pub fn check_proposal_integrity_property(node_states: &Arc<MutexNodeStates>, status_change: &TMStatusChange, sender: usize) -> bool {
        if status_change.has_newEvent() && status_change.get_newEvent() == NodeEvent::neACCEPTED_LEDGER {
            let mut node_states_vec = node_states.node_states.lock();
            let already_present = node_states_vec.add_consensus_transaction_set(status_change.get_ledgerSeq() as usize, status_change.clone(), sender);
            Self::check_agreement_properties(&node_states_vec.node_states);
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
        let already_present = node_states_vec.add_sent_validation(validation.ledger_sequence as usize, validation.clone(), sender);
        Self::check_agreement_properties(&node_states_vec.node_states);
        if let Some(earlier_validation) = already_present {
            if earlier_validation != validation {
                error!("(I2) Node validated twice for one ledger sequence\nOld: {:?}\nNew: {:?}", earlier_validation, validation);
                return false
            }
        }
        true
    }

    pub fn check_agreement_properties(node_states_vec: &Vec<NodeState>) -> bool {
        let validation_agreement = node_states_vec.iter()
            .flat_map(|state: &NodeState| state.validations_sent.clone())
            .into_group_map()
            .iter()
            .all(|(seq, validations)| if validations.iter()
                .map(|val| &val.hash).all_equal()
            {
                true
            } else {
                error!("Conflicting validations sent for ledger_seq: {}", seq);
                for val in validations {
                    println!("{:?}", val);
                }
                false
            });
        let proposal_agreement = node_states_vec.iter()
            .flat_map(|state: &NodeState| state.consensus_transaction_sets.clone())
            .into_group_map()
            .iter()
            .all(|(seq, status_changes)| if status_changes.iter()
                .map(|sc| sc.get_ledgerHash().clone()).all_equal()
            {
                true
            } else {
                error!("Conflicting transaction sets sent for ledger_seq: {}", seq);
                for sc in status_changes {
                    println!("{:?}", sc);
                }
                false
            });
        validation_agreement && proposal_agreement
    }
}

#[cfg(test)]
mod consensus_properties_tests {
    use std::sync::Arc;
    use itertools::Itertools;
    use crate::consensus_properties::ConsensusProperties;
    use crate::message_handler::ParsedValidation;
    use crate::node_state::{MutexNodeStates, NodeState, NodeStates};
    use crate::protos::ripple::{NodeEvent, TMStatusChange};

    #[test]
    fn test_check_agreement_properties() {
        let mut node_states_vec = setup_node_states(3);
        node_states_vec[0].consensus_transaction_sets.insert(0, TMStatusChange::default());
        node_states_vec[1].consensus_transaction_sets.insert(0, TMStatusChange::default());
        node_states_vec[0].validations_sent.insert(0, ParsedValidation::default());
        node_states_vec[1].validations_sent.insert(0, ParsedValidation::default());
        node_states_vec[2].validations_sent.insert(1, ParsedValidation::default());
        assert!(ConsensusProperties::check_agreement_properties(&node_states_vec));
        let mut different_validation = ParsedValidation::default();
        different_validation.hash = "Different ledger hash".to_string();
        node_states_vec[1].validations_sent.insert(1, different_validation);
        assert!(!ConsensusProperties::check_agreement_properties(&node_states_vec));
        node_states_vec[1].validations_sent.insert(1, ParsedValidation::default());
        assert!(ConsensusProperties::check_agreement_properties(&node_states_vec));
        let mut different_status_change = TMStatusChange::default();
        different_status_change.set_ledgerHash(vec![1, 2, 3]);
        node_states_vec[2].consensus_transaction_sets.insert(0, different_status_change);
        assert!(!ConsensusProperties::check_agreement_properties(&node_states_vec));
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

        fn setup_node_states(peer: usize) -> Vec<NodeState> {
        (0..peer).map(|x| NodeState::new(x)).collect_vec()
    }
}
#![allow(dead_code)]
use parking_lot::{Mutex, Condvar};
use crate::client::ValidatedLedger;

/// Contains the state for a particular node at a particular time
#[derive(Clone, Debug)]
pub struct NodeState {
    pub peer: usize,
    pub consensus_phase: ConsensusPhase,
    pub current_consensus_round: u32,
    pub last_validated_ledger: ValidatedLedger,
    pub unvalidated_transactions: Vec<String>,
    pub validated_transactions: Vec<String>,
}

impl NodeState {
    pub(crate) fn new(peer: usize) -> Self {
        Self {
            peer,
            consensus_phase: ConsensusPhase::Open,
            current_consensus_round: 3,
            last_validated_ledger: ValidatedLedger::default(),
            unvalidated_transactions: vec![],
            validated_transactions: vec![],
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum ConsensusPhase {
    Open,
    Establish,
    Accepted,
}

/// Combine individual node states for aggregate functions
#[derive(Clone, Debug)]
pub struct NodeStates {
    pub node_states: Vec<NodeState>,
}

impl NodeStates {
    pub fn max_current_round(&self) -> u32 {
        self.node_states.iter().map(|x| x.current_consensus_round).max().expect("node states is empty")
    }

    pub fn min_current_round(&self) -> u32 {
        self.node_states.iter().map(|x| x.current_consensus_round).min().expect("node states is empty")
    }

    pub fn diff_rounds(&self) -> u32 {
        self.max_current_round() - self.min_current_round()
    }

    pub fn min_validated_ledger(&self) -> u32 {
        self.node_states.iter().map(|state| state.last_validated_ledger.ledger_index).min().expect("node states is empty")
    }

    pub fn max_validated_ledger(&self) -> u32 {
        self.node_states.iter().map(|state| state.last_validated_ledger.ledger_index).max().expect("node states is empty")
    }
}

/// Wrap NodeStates in a Mutex with Condvars for convenient access in the rest of the program.
/// This struct should always be wrapped in a Arc
/// This struct should always be used to access and modify node_state data
#[derive(Debug)]
pub struct MutexNodeStates {
    pub node_states: Mutex<NodeStates>,
    pub round_cvar: Condvar,
    pub consensus_phase_cvar: Condvar,
    pub validated_ledger_cvar: Condvar,
    pub transactions_cvar: Condvar,
}

impl MutexNodeStates {
    pub fn new(node_states: NodeStates) -> Self {
        MutexNodeStates {
            node_states: Mutex::new(node_states),
            round_cvar: Condvar::new(),
            consensus_phase_cvar: Condvar::new(),
            validated_ledger_cvar: Condvar::new(),
            transactions_cvar: Condvar::new(),
        }
    }

    pub fn set_current_round(&self, peer: usize, new_round: u32) {
        self.node_states.lock().node_states[peer].current_consensus_round = new_round;
        self.round_cvar.notify_all();
    }

    pub fn set_consensus_phase(&self, peer: usize, new_phase: ConsensusPhase) {
        self.node_states.lock().node_states[peer].consensus_phase = new_phase;
        self.consensus_phase_cvar.notify_all();
    }

    pub fn set_validated_ledger(&self, peer: usize, new_validated_ledger: ValidatedLedger) {
        self.node_states.lock().node_states[peer].last_validated_ledger = new_validated_ledger;
        self.validated_ledger_cvar.notify_all();
    }

    pub fn clear_transactions(&self) {
        let mut node_states = self.node_states.lock();
        for i in 0..node_states.node_states.len() {
            node_states.node_states[i].unvalidated_transactions.clear();
            node_states.node_states[i].validated_transactions.clear();
        }
        self.transactions_cvar.notify_all();
    }

    pub fn add_unvalidated_transaction(&self, peer: usize, transaction_signature: String) {
        self.node_states.lock().node_states[peer].unvalidated_transactions.push(transaction_signature);
        self.transactions_cvar.notify_all();
    }

    pub fn add_validated_transaction(&self, peer: usize, transaction_signature: String) {
        self.node_states.lock().node_states[peer].validated_transactions.push(transaction_signature);
        self.transactions_cvar.notify_all();
    }

    pub fn get_current_round(&self, peer: usize) -> u32 {
        self.node_states.lock().node_states[peer].current_consensus_round
    }

    pub fn get_consensus_phase(&self, peer: usize) -> ConsensusPhase {
        self.node_states.lock().node_states[peer].consensus_phase.clone()
    }

    pub fn get_validated_ledger(&self, peer: usize) -> ValidatedLedger {
        self.node_states.lock().node_states[peer].last_validated_ledger.clone()
    }

    pub fn get_min_validated_transactions(&self) -> usize {
        self.node_states.lock().node_states.iter().map(|x| x.validated_transactions.len()).min().unwrap()
    }

    pub fn get_max_validated_transaction(&self) -> usize {
        self.node_states.lock().node_states.iter().map(|x| x.validated_transactions.len()).max().unwrap()
    }

    pub fn get_unvalidated_transaction(&self, peer: usize) -> Vec<String> {
        self.node_states.lock().node_states[peer].unvalidated_transactions.clone()
    }

    pub fn get_validated_transaction(&self, peer: usize) -> Vec<String> {
        self.node_states.lock().node_states[peer].validated_transactions.clone()
    }

    pub fn min_validated_ledger(&self) -> u32 {
        self.node_states.lock().min_validated_ledger()
    }

    pub fn max_validated_ledger(&self) -> u32 {
        self.node_states.lock().max_validated_ledger()
    }

    pub fn max_current_round(self) -> u32 {
        self.node_states.lock().max_current_round()
    }

    pub fn min_current_round(self) -> u32 {
        self.node_states.lock().min_current_round()
    }

    pub fn diff_rounds(self) -> u32 {
        self.node_states.lock().diff_rounds()
    }
}
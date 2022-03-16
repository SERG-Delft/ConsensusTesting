#![allow(dead_code)]

use std::fmt::{Debug, Formatter};
use parking_lot::{Mutex, Condvar};
use petgraph::Graph;
use petgraph::prelude::NodeIndex;
use crate::client::{PeerServerStateObject, ServerStateObject, ValidatedLedger};
use crate::collector::RippleMessage;
use crate::ga::genetic_algorithm::DelaysGenotype;

/// Contains the state for a particular node at a particular time
#[derive(Clone, Debug)]
pub struct NodeState {
    pub peer: usize,
    pub consensus_phase: ConsensusPhase,
    pub current_consensus_round: u32,
    pub last_validated_ledger: ValidatedLedger,
    pub unvalidated_transactions: Vec<String>,
    pub validated_transactions: Vec<String>,
    pub number_of_failed_consensus_rounds: u32,
    pub unreceived_message_sends: Vec<(RippleMessage, Option<DependencyNode>)>,
    pub latest_message_received: Option<DependencyNode>,
    pub server_state: ServerStateObject,
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
            number_of_failed_consensus_rounds: 0,
            unreceived_message_sends: vec![],
            latest_message_received: None,
            server_state: ServerStateObject::default(),
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
    pub number_of_nodes: usize,
    pub node_states: Vec<NodeState>,
    pub executions: Vec<RippleMessage>,
    pub dependency_graph: Graph<DependencyEvent, ()>,
    pub current_delays: DelaysGenotype,
    pub server_state_updates: Vec<bool>,
}

impl NodeStates {
    pub fn new(node_states: Vec<NodeState>) -> Self {
        let number_of_nodes = node_states.len();
        NodeStates {
            number_of_nodes,
            node_states,
            executions: vec![],
            dependency_graph: petgraph::Graph::new(),
            current_delays: vec![],
            server_state_updates: vec![false; number_of_nodes],
        }
    }

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

    pub fn total_number_of_failed_consensus_rounds(&self) -> u32 {
        self.node_states.iter().map(|state| state.number_of_failed_consensus_rounds).sum()
    }

    pub fn clear_number_of_failed_consensus_rounds(&mut self) {
        for i in 0..self.node_states.len() {
            self.node_states[i].number_of_failed_consensus_rounds = 0;
        }
    }

    pub fn clear_transactions(&mut self) {
        for i in 0..self.node_states.len() {
            self.node_states[i].unvalidated_transactions.clear();
            self.node_states[i].validated_transactions.clear();
        }
    }

    pub fn clear_dependency_graph(&mut self) {
        self.dependency_graph = petgraph::Graph::new();
        for i in 0..self.node_states.len() {
            self.node_states[i].unreceived_message_sends = vec![];
            self.node_states[i].latest_message_received = None;
        }
    }

    pub fn add_receive_dependency(&mut self, ripple_message: RippleMessage) {
        let dependency_event = DependencyEvent { ripple_message: ripple_message.clone() };
        let node_index = self.dependency_graph.add_node(dependency_event.clone());
        let dependency_node = DependencyNode { event: dependency_event, index: node_index };

        let mut receiver_node_state = self.node_states[ripple_message.receiver_index()].clone();
        let mut sender_node_state = self.node_states[ripple_message.sender_index()].clone();

        if let Some(pos) = sender_node_state.unreceived_message_sends
            .iter()
            .position(|x| x.0 == ripple_message)
        {
            let (_, dependency_node) = sender_node_state.unreceived_message_sends.remove(pos);
            match dependency_node {
                None => {}
                Some(node) => { self.dependency_graph.add_edge(node.index, node_index, ()); }
            }
        }

        match receiver_node_state.latest_message_received {
            None => {}
            Some(receive_dependency) => { self.dependency_graph.add_edge(receive_dependency.index, node_index, ()); }
        }

        receiver_node_state.latest_message_received = Some(dependency_node);

        self.node_states[ripple_message.receiver_index()] = receiver_node_state;
        self.node_states[ripple_message.sender_index()] = sender_node_state;
    }

    pub fn add_send_dependency(&mut self, ripple_message: RippleMessage) {
        let mut sender_node_state = self.node_states[ripple_message.sender_index()].clone();
        sender_node_state.unreceived_message_sends.push((ripple_message.clone(), sender_node_state.latest_message_received.clone()));
        self.node_states[ripple_message.sender_index()] = sender_node_state;
    }

    pub fn set_current_delays(&mut self, delays: DelaysGenotype) {
        self.current_delays = delays;
    }

    /// Returns true when the server states for all nodes have been updated
    pub fn set_server_state(&mut self, server_state: PeerServerStateObject) -> bool {
        self.server_state_updates[server_state.peer as usize] = true;
        self.node_states[server_state.peer as usize].server_state = server_state.server_state_object;
        if self.server_state_updates.iter().all(|x| *x) {
            self.server_state_updates = vec![false; self.node_states.len()];
            return true;
        }
        false
    }

    pub fn get_server_state(&self, peer: usize) -> ServerStateObject {
        self.node_states[peer].server_state.clone()
    }
}

/// Wrap NodeStates in a Mutex with Condvars for convenient access in the rest of the program.
/// This struct should always be wrapped in a Arc
/// This struct should always be used to access and modify node_state data
#[derive(Debug)]
pub struct MutexNodeStates {
    pub number_of_nodes: usize,
    pub node_states: Mutex<NodeStates>,
    pub round_cvar: Condvar,
    pub consensus_phase_cvar: Condvar,
    pub validated_ledger_cvar: Condvar,
    pub transactions_cvar: Condvar,
    pub server_state_cvar: Condvar,
}

impl MutexNodeStates {
    pub fn new(node_states: NodeStates) -> Self {
        MutexNodeStates {
            number_of_nodes: node_states.node_states.len(),
            node_states: Mutex::new(node_states),
            round_cvar: Condvar::new(),
            consensus_phase_cvar: Condvar::new(),
            validated_ledger_cvar: Condvar::new(),
            transactions_cvar: Condvar::new(),
            server_state_cvar: Condvar::new(),
        }
    }

    pub fn set_current_round(&self, peer: usize, new_round: u32) {
        self.node_states.lock().node_states[peer].current_consensus_round = new_round;
        self.round_cvar.notify_all();
    }

    /// Take care not to get a deadlock here, calls to associated methods acquire locks
    pub fn set_consensus_phase(&self, peer: usize, new_phase: ConsensusPhase) {
        let current_phase = self.node_states.lock().node_states[peer].consensus_phase.clone();
        if new_phase == ConsensusPhase::Open {
            let old_round = self.get_current_round(peer);
            self.set_current_round(peer, old_round + 1);
            if current_phase != ConsensusPhase::Accepted {
                self.node_states.lock().node_states[peer].number_of_failed_consensus_rounds += 1;
                println!("Failed consensus round peer {}: establish -> open", peer);
            }
        } else if new_phase == ConsensusPhase::Establish && current_phase != ConsensusPhase::Open {
            self.node_states.lock().node_states[peer].number_of_failed_consensus_rounds += 1;
            println!("Failed consensus round peer {}: accepted -> establish", peer);
        }
        self.node_states.lock().node_states[peer].consensus_phase = new_phase;
        self.consensus_phase_cvar.notify_all();
    }

    pub fn set_validated_ledger(&self, peer: usize, new_validated_ledger: ValidatedLedger) {
        self.node_states.lock().node_states[peer].last_validated_ledger = new_validated_ledger;
        self.validated_ledger_cvar.notify_all();
    }

    pub fn clear_transactions(&self) {
        self.node_states.lock().clear_transactions();
        self.node_states.lock().clear_dependency_graph();
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

    pub fn get_number_of_failed_consensus_rounds(&self, peer: usize) -> u32 {
        self.node_states.lock().node_states[peer].number_of_failed_consensus_rounds
    }

    pub fn get_total_number_of_failed_consensus_rounds(&self) -> u32 {
        self.node_states.lock().total_number_of_failed_consensus_rounds()
    }

    pub fn clear_number_of_failed_consensus_rounds(&self) {
        self.node_states.lock().clear_number_of_failed_consensus_rounds();
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

    pub fn add_execution(&self, ripple_message: RippleMessage) {
        let mut states = self.node_states.lock();
        states.executions.push(ripple_message.clone());
        states.add_receive_dependency(ripple_message.clone());
    }

    pub fn get_executions(&self) -> Vec<RippleMessage> {
        self.node_states.lock().executions.clone()
    }

    pub fn clear_executions(&self) {
        self.node_states.lock().executions.clear();
    }

    pub fn add_send_dependency(&self, ripple_message: RippleMessage) {
        self.node_states.lock().add_send_dependency(ripple_message.clone());
    }

    pub fn get_dependency_graph(&self) -> Graph<DependencyEvent, ()> {
        self.node_states.lock().dependency_graph.clone()
    }

    pub fn set_current_delays(&self, delays: DelaysGenotype) {
        self.node_states.lock().set_current_delays(delays);
    }

    pub fn get_current_delays(&self) -> DelaysGenotype {
        self.node_states.lock().current_delays.clone()
    }

    pub fn set_server_state(&self, server_state: PeerServerStateObject) {
        if self.node_states.lock().set_server_state(server_state) {
            self.server_state_cvar.notify_all();
        }
    }

    pub fn get_server_state(&self, peer: usize) -> ServerStateObject {
        self.node_states.lock().get_server_state(peer)
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash, Default)]
pub struct DependencyEvent {
    pub ripple_message: RippleMessage,
}

impl Debug for DependencyEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} -> {}\n", self.ripple_message.message_type(), self.ripple_message.from_node, self.ripple_message.to_node)
    }
}

#[derive(Debug, Clone)]
pub struct DependencyNode {
    pub event: DependencyEvent,
    pub index: NodeIndex,
}

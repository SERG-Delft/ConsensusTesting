#![allow(dead_code)]

use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use parking_lot::{Mutex, Condvar};
use petgraph::Graph;
use petgraph::prelude::NodeIndex;
use crate::client::ValidatedLedger;
use crate::collector::RippleMessage;

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
    pub executions: Vec<RippleMessage>,
    pub dependency_graph: Graph<DependencyEvent, ()>,
    latest_message_sent: HashMap<usize, DependencyNode>,
    latest_messages_received: Vec<Vec<DependencyNode>>,
    unreceived_message_sends: Vec<Vec<DependencyNode>>,
}

impl NodeStates {
    pub fn new(node_states: Vec<NodeState>) -> Self {
        NodeStates {
            node_states,
            executions: vec![],
            dependency_graph: petgraph::Graph::new(),
            latest_message_sent: HashMap::new(),
            latest_messages_received: vec![vec![]; 5],
            unreceived_message_sends: vec![vec![]; 5]
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

    /// This receive event is dependent on its send event, nothing more
    pub fn add_receive_dependency(&mut self, ripple_message: RippleMessage) {
        let dependency_event = DependencyEvent { ripple_message: ripple_message.clone(), event_type: EventType::Receive };
        let node_index = self.dependency_graph.add_node(dependency_event.clone());
        let dependency_node = DependencyNode { event: dependency_event, index: node_index };
        self.latest_messages_received[ripple_message.receiver_index()].push(dependency_node);
        if let Some(pos) = self.unreceived_message_sends[ripple_message.sender_index()]
            .iter()
            .position(|x| x.event.ripple_message == ripple_message)
        {
            let node = self.unreceived_message_sends[ripple_message.sender_index()].remove(pos);
            self.dependency_graph.add_edge(node.index, node_index, ());
        }
    }

    /// This send event is dependent on all messages received in this node prior to this send event
    /// Subsequently all receive events are cleared as they now have a child
    /// This send event is also dependent on the previous send event
    pub fn add_send_dependency(&mut self, ripple_message: RippleMessage) {
        let dependency_event = DependencyEvent { ripple_message: ripple_message.clone(), event_type: EventType::Send };
        let node_index = self.dependency_graph.add_node(dependency_event.clone());
        let dependency_node = DependencyNode { event: dependency_event, index: node_index };
        if let Some(node) = self.latest_message_sent.get(&ripple_message.sender_index()) {
            self.dependency_graph.add_edge(node.index, dependency_node.index, ());
        }
        for node in &self.latest_messages_received[ripple_message.sender_index()] {
            self.dependency_graph.add_edge(node.index, dependency_node.index, ());
        }
        self.latest_message_sent.insert(ripple_message.sender_index(), dependency_node.clone());
        self.latest_messages_received[ripple_message.sender_index()].clear();
        self.unreceived_message_sends[ripple_message.sender_index()].push(dependency_node);
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

    /// Take care not to get a deadlock here, calls to associated methods acquire locks
    pub fn set_consensus_phase(&self, peer: usize, new_phase: ConsensusPhase) {
        let current_phase = self.node_states.lock().node_states[peer].consensus_phase.clone();
        if new_phase == ConsensusPhase::Open {
            let old_round = self.get_current_round(peer);
            self.set_current_round(peer, old_round + 1);
            if current_phase != ConsensusPhase::Accepted {
                self.node_states.lock().node_states[peer].number_of_failed_consensus_rounds += 1;
                println!("Failed consensus round");
            }
        }
        else if new_phase == ConsensusPhase::Establish && current_phase != ConsensusPhase::Open {
            self.node_states.lock().node_states[peer].number_of_failed_consensus_rounds += 1;
            println!("Failed consensus round");
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
        self.node_states.lock().add_send_dependency(ripple_message);
    }

    pub fn get_dependency_graph(&self) -> Graph<DependencyEvent, ()> {
        self.node_states.lock().dependency_graph.clone()
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash)]
pub struct DependencyEvent {
    pub ripple_message: RippleMessage,
    pub event_type: EventType,
}

impl Debug for DependencyEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} {} {} -> {}\n", self.event_type, self.ripple_message.message_type(), self.ripple_message.from_node, self.ripple_message.to_node)
    }
}

#[derive(Debug, Clone)]
struct DependencyNode {
    pub event: DependencyEvent,
    pub index: NodeIndex,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash)]
pub enum EventType {
    Send,
    Receive,
}
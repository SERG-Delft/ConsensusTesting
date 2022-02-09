use std::fmt::{Display, Formatter};
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::{BufWriter, Write};
use std::path::Path;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender};
use serde_json::json;
use crate::client::{ConsensusChange, PeerSubscriptionObject, SubscriptionObject};
use crate::message_handler::RippleMessageObject;
use chrono::{DateTime, MAX_DATETIME, Utc};
use itertools::Itertools;
use crate::message_handler::RippleMessageObject::TMProposeSet;
use crate::node_state::{ConsensusPhase, MutexNodeStates};
use crate::protos::ripple::TMTransaction;

/// Collects and writes data to files and the scheduler
/// Execution file stores all messages sent from the proxy
/// Subscription file stores all subscription messages received from the client
/// Node states contains the current state of all nodes individually, info is received from subscriptions
pub struct Collector {
    ripple_message_receiver: Receiver<Box<RippleMessage>>,
    subscription_receiver: Receiver<PeerSubscriptionObject>,
    control_receiver: Receiver<String>,
    _scheduler_sender: Sender<SubscriptionObject>,
    execution_file: BufWriter<File>,
    subscription_files: Vec<BufWriter<File>>,
    node_states: Arc<MutexNodeStates>,
}

impl Collector {
    pub fn new(
        number_of_nodes: u16,
        ripple_message_receiver: Receiver<Box<RippleMessage>>,
        subscription_receiver: Receiver<PeerSubscriptionObject>,
        control_receiver: Receiver<String>,
        scheduler_sender: Sender<SubscriptionObject>,
        node_states: Arc<MutexNodeStates>,
    ) -> Self {
        let execution_file = File::create(Path::new("execution.txt")).expect("Opening execution file failed");
        let mut subscription_files = vec![];
        for peer in 0..number_of_nodes {
            let mut subscription_file = BufWriter::new(File::create(Path::new(format!("subscription_{}.json", peer).as_str())).expect("Opening subscription file failed"));
            subscription_file.write_all(String::from("[\n").as_bytes()).unwrap();
            subscription_files.push(subscription_file);
        }
        Collector {
            ripple_message_receiver,
            subscription_receiver,
            control_receiver,
            _scheduler_sender: scheduler_sender,
            execution_file: BufWriter::new(execution_file),
            subscription_files,
            node_states,
        }
    }

    pub fn start(&mut self) {
        loop {
            // Stop writing to file if any control message is received
            // Can be extended to start writing to file later, currently not implemented
            match self.control_receiver.try_recv() {
                Ok(_) => {
                    break;
                }
                _ => {}
            }
            // Write all messages sent by the scheduler to peers to "execution.txt". After delay!
            match self.ripple_message_receiver.try_recv() {
                Ok(mut message) => {
                    self.write_to_file(&mut message);
                }
                _ => {}
            }
            // Handle subscription streams in a central place, TODO: refactor to own associated method.
            match self.subscription_receiver.try_recv() {
                Ok(subscription_object) => match subscription_object.subscription_object {
                    SubscriptionObject::ValidatedLedger(ledger) => {
                        self.node_states.set_validated_ledger(subscription_object.peer as usize,ledger.clone());
                        self.write_to_subscription_file(subscription_object.peer, json!({"LedgerValidated": ledger}).to_string());
                        // self.scheduler_sender.send(SubscriptionObject::ValidatedLedger(ledger.clone())).expect("Scheduler send failed");
                    }
                    SubscriptionObject::ReceivedValidation(validation) =>
                        self.write_to_subscription_file(subscription_object.peer, json!({"ValidationReceived": validation}).to_string()),
                    SubscriptionObject::PeerStatusChange(peer_status) =>
                        self.write_to_subscription_file(subscription_object.peer, json!({"PeerStatus": peer_status}).to_string()),
                    SubscriptionObject::ConsensusChange(consensus_change) => {
                        // Use new consensus phase to determine the current round of consensus of the node
                        let new_consensus_phase = Self::parse_consensus_change(consensus_change.clone());
                        self.node_states.set_consensus_phase(subscription_object.peer as usize, new_consensus_phase);
                        // println!("{:?}", self.node_states.node_states.lock().node_states[subscription_object.peer as usize]);
                        self.write_to_subscription_file(subscription_object.peer, json!({"ConsensusChange": consensus_change}).to_string());
                    }
                    SubscriptionObject::Transaction(transaction_subscription) => {
                        // Transactions can be validated or unvalidated
                        if transaction_subscription.validated {
                            self.node_states.add_validated_transaction(
                                subscription_object.peer as usize,
                                transaction_subscription.transaction.txn_signature.clone().unwrap()
                            );
                        } else {
                            self.node_states.add_unvalidated_transaction(
                                subscription_object.peer as usize,
                                transaction_subscription.transaction.txn_signature.clone().unwrap()
                            );
                        }
                        self.write_to_subscription_file(subscription_object.peer, json!({"Transaction": transaction_subscription}).to_string())
                    }
                    SubscriptionObject::ServerStatus(server_status) => {
                        self.write_to_subscription_file(subscription_object.peer, json!({"ServerStatus": server_status}).to_string())
                    }
                },
                _ => {}
            }
        }
    }

    fn write_to_file(&mut self, ripple_message: &mut RippleMessage) {
        match &ripple_message.message {
            TMProposeSet(tm_propose) => if tm_propose.get_proposeSeq() > 1 { println!("{}", ripple_message.to_string()) }
            _ => {},
        }
        self.execution_file.write_all(ripple_message.to_string().as_bytes()).unwrap();
    }

    fn write_to_subscription_file(&mut self, peer: u16, text: String) {
        self.subscription_files[peer as usize].write_all((text + ",\n").as_bytes()).unwrap();
    }

    fn parse_consensus_change(consensus_change: ConsensusChange) -> ConsensusPhase {
        match consensus_change.consensus.as_str() {
            "open" => ConsensusPhase::Open,
            "establish" => ConsensusPhase::Establish,
            "accepted" => ConsensusPhase::Accepted,
            _ => ConsensusPhase::Open
        }
    }
}

/// Struct for writing clearly to execution.txt, should definitely rename
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RippleMessage {
    pub from_node: String,
    pub to_node: String,
    timestamp: DateTime<Utc>,
    pub message: RippleMessageObject
}

impl RippleMessage {
    pub fn new(from_node: String, to_node: String, timestamp: DateTime<Utc>, message: RippleMessageObject) -> Box<Self> {
        Box::from(RippleMessage { from_node, to_node, timestamp, message })
    }

    pub fn message_type(&self) -> String {
        self.message.message_type().to_string()
    }

    pub fn sender_index(&self) -> usize {
        self.from_node.as_str().chars().next_back().unwrap().to_digit(10).unwrap() as usize - 1
    }

    pub fn receiver_index(&self) -> usize {
        self.to_node.as_str().chars().next_back().unwrap().to_digit(10).unwrap() as usize - 1
    }

    pub fn simple_str(&self) -> String {
        let message = self.message.to_string();
        let message_type = message.split(" ").collect_vec()[0];
        format!("{}{}{}\n", self.from_node.as_str().chars().next_back().unwrap(), self.to_node.as_str().chars().next_back().unwrap(), message_type[0..message_type.len() - 1].to_string())
    }
}

impl Hash for RippleMessage {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.to_node.hash(state);
        self.from_node.hash(state);
        self.message.hash(state);
    }
}

impl PartialEq for RippleMessage {
    fn eq(&self, other: &Self) -> bool {
        self.to_node == other.to_node && self.from_node == other.from_node && self.message.message_type() == other.message.message_type()
    }
}
impl Eq for RippleMessage {}

impl Display for RippleMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // Use the ripple_epoch (seconds since 2000-01-01T00:00:00+00:00) for easier cross-referencing with ripple logs
        // TODO: Store actual time and perhaps node_state information at time of send. (This is after delay!)
        let ripple_epoch = DateTime::parse_from_rfc3339("2000-01-01T00:00:00+00:00").unwrap();
        let from_node_buf = &self.from_node;
        let to_node_buf = &self.to_node;
        let time_since = self.timestamp.signed_duration_since(ripple_epoch).num_seconds();
        let message_buf = self.message.to_string();
        write!(f, "{} {} -> {} sent {}\n", time_since, from_node_buf, to_node_buf, message_buf)
    }
}

impl Default for RippleMessage {
    fn default() -> Self {
        *Self::new("".to_string(), "".to_string(), MAX_DATETIME, RippleMessageObject::TMTransaction(TMTransaction::default()))
    }
}
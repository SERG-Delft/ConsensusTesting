use std::fmt::{Display, Formatter};
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::{BufWriter, Write};
use std::path::Path;
use std::sync::Arc;
use std::sync::mpsc::{Receiver};
use std::thread;
use serde_json::json;
use crate::client::{ConsensusChange, PeerServerStateObject, PeerSubscriptionObject, SubscriptionObject};
use crate::message_handler::RippleMessageObject;
use chrono::{DateTime, Duration, MAX_DATETIME, Utc};
use itertools::Itertools;
use log::error;
use serde_with::{serde_as, DurationSecondsWithFrac};
use crate::LOG_FOLDER;
use crate::message_handler::RippleMessageObject::TMProposeSet;
use crate::node_state::{ConsensusPhase, MutexNodeStates};
use crate::protos::ripple::{TMTransaction};
use crate::test_harness::TransactionResultCode;

/// Collects and writes data to files and the scheduler
/// Execution file stores all messages sent from the proxy
/// Subscription file stores all subscription messages received from the client
/// Node states contains the current state of all nodes individually, info is received from subscriptions
pub struct Collector {
    subscription_receiver: Receiver<PeerSubscriptionObject>,
    subscription_files: Vec<BufWriter<File>>,
    node_states: Arc<MutexNodeStates>,
}

impl Collector {
    pub fn new(
        number_of_nodes: u16,
        subscription_receiver: Receiver<PeerSubscriptionObject>,
        node_states: Arc<MutexNodeStates>,
    ) -> Self {
        let mut subscription_files = vec![];
        for peer in 0..number_of_nodes {
            let mut subscription_file = BufWriter::new(File::create(Path::new(format!("{}\\subscription_{}.json", *LOG_FOLDER, peer).as_str())).expect("Opening subscription file failed"));
            subscription_file.write_all(String::from("[\n").as_bytes()).unwrap();
            subscription_files.push(subscription_file);
        }
        Collector {
            subscription_receiver,
            subscription_files,
            node_states,
        }
    }

    pub fn start(&mut self, ripple_message_receiver: Receiver<Box<RippleMessage>>, server_state_receiver: Receiver<PeerServerStateObject>) {
        let node_states_clone = self.node_states.clone();
        let node_state_clone_2 = self.node_states.clone();
        thread::spawn(move || Self::execution_writer(ripple_message_receiver, node_state_clone_2));
        thread::spawn(move || Self::server_state_handler(server_state_receiver, node_states_clone));
        loop {
            // Handle subscription streams in a central place, TODO: refactor to own associated method.
            match self.subscription_receiver.recv() {
                Ok(subscription_object) => match subscription_object.subscription_object {
                    SubscriptionObject::ValidatedLedger(ledger) => {
                        self.node_states.set_validated_ledger(subscription_object.peer as usize,ledger.clone());
                        self.write_to_subscription_file(subscription_object.peer, json!({"LedgerValidated": ledger}).to_string());
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
                        self.write_to_subscription_file(subscription_object.peer, json!({"Transaction": transaction_subscription}).to_string());
                        if transaction_subscription.validated {
                            let result = TransactionResultCode::parse(&transaction_subscription.engine_result);
                            self.node_states.add_validated_transaction(
                                subscription_object.peer as usize,
                                transaction_subscription.transaction,
                                result
                            );
                        } else {
                            self.node_states.add_unvalidated_transaction(
                                subscription_object.peer as usize,
                                transaction_subscription.transaction
                            );
                        }
                    }
                    SubscriptionObject::ServerStatus(server_status) => {
                        self.write_to_subscription_file(subscription_object.peer, json!({"ServerStatus": server_status}).to_string())
                    }
                },
                _ => {}
            }
        }
    }

    fn execution_writer(ripple_message_receiver: Receiver<Box<RippleMessage>>, node_states: Arc<MutexNodeStates>) {
        let mut execution_file = File::create(Path::new(format!("{}\\execution.txt", *LOG_FOLDER).as_str())).expect("Opening execution file failed");
        // let mut execution_writer = BufWriter::new(execution_file);
        loop {
            // Write all messages sent by the scheduler to peers to "execution.txt". After delay!
            let mut buf = vec![];
            for _ in 0..30 {
                match ripple_message_receiver.recv() {
                    Ok(message) => {
                        match &message.message {
                            TMProposeSet(tm_propose) => {
                                node_states.set_highest_propose_seq(tm_propose.get_proposeSeq(), message.sender_index());
                                if tm_propose.get_proposeSeq() > 1 { error!("{}", message.to_string()) }
                            },
                            _ => {},
                        }
                        buf.extend(message.to_string().as_bytes());
                        // execution_writer.flush().unwrap();
                    }
                    _ => {}
                }
            }
            execution_file.write_all(&buf).unwrap();
        }
    }

    fn server_state_handler(server_state_receiver: Receiver<PeerServerStateObject>, node_states: Arc<MutexNodeStates>) {
        loop {
            match server_state_receiver.recv() {
                Ok(server_state_object) => {
                    node_states.set_server_state(server_state_object)
                }
                _ => {}
            }
        }
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
#[serde_as]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RippleMessage {
    pub from_node: String,
    pub to_node: String,
    #[serde_as(as = "DurationSecondsWithFrac<f64>")]
    delay: Duration,
    timestamp: DateTime<Utc>,
    pub message: RippleMessageObject
}

impl RippleMessage {
    pub fn new(from_node: String, to_node: String, delay: Duration, timestamp: DateTime<Utc>, message: RippleMessageObject) -> Box<Self> {
        Box::from(RippleMessage { from_node, to_node, delay, timestamp, message })
    }

    pub fn message_type(&self) -> String {
        self.message.message_type().to_string()
    }

    pub fn sender_index(&self) -> usize {
        self.from_node.as_str().split_at(6).1.parse::<usize>().unwrap() - 1
    }

    pub fn receiver_index(&self) -> usize {
        self.to_node.as_str().split_at(6).1.parse::<usize>().unwrap() - 1
    }

    #[allow(unused)]
    pub fn simple_str(&self) -> String {
        let message = self.message.to_string();
        let message_type = message.split(" ").collect_vec()[0];
        format!("{}{}{}\n", self.from_node.as_str().split_at(6).1, self.to_node.as_str().split_at(6).1, message_type[0..message_type.len() - 1].to_string())
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
        // let ripple_epoch = DateTime::parse_from_rfc3339("2000-01-01T00:00:00+00:00").unwrap();
        let from_node_buf = &self.from_node;
        let to_node_buf = &self.to_node;
        // let time_since = self.timestamp.signed_duration_since(ripple_epoch).num_seconds();
        let message_buf = self.message.to_string();
        write!(f, "After {}, at {} {} -> {} sent {}\n", self.delay, self.timestamp, from_node_buf, to_node_buf, message_buf)
    }
}

impl Default for RippleMessage {
    fn default() -> Self {
        *Self::new("".to_string(), "".to_string(), Duration::zero(), MAX_DATETIME, RippleMessageObject::TMTransaction(TMTransaction::default()))
    }
}
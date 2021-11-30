use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender};
use serde_json::json;
use crate::client::{ConsensusChange, PeerSubscriptionObject, SubscriptionObject};
use crate::message_handler::RippleMessageObject;
use chrono::{DateTime, Utc};
use parking_lot::{Condvar, Mutex};
use crate::node_state::{ConsensusPhase, MutexNodeStates, NodeState, NodeStates};

/// Collects and writes data to files
/// Execution file stores all messages sent from the proxy
/// Subscription file stores all subscription messages received from the client
pub struct Collector {
    ripple_message_receiver: Receiver<Box<RippleMessage>>,
    subscription_receiver: Receiver<PeerSubscriptionObject>,
    control_receiver: Receiver<String>,
    scheduler_sender: Sender<SubscriptionObject>,
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
            scheduler_sender,
            execution_file: BufWriter::new(execution_file),
            subscription_files,
            node_states,
        }
    }

    pub fn start(&mut self) {
        let mut latest_ledger = 0;
        // let &(ref node_states_lock, ref node_states_cvar) = &*self.node_states.clone();
        loop {
            // Stop writing to file if any control message is received
            // Can be extended to start writing to file later
            match self.control_receiver.try_recv() {
                Ok(_) => {
                    break;
                }
                _ => {}
            }
            match self.ripple_message_receiver.try_recv() {
                Ok(mut message) => {
                    self.write_to_file(&mut message);
                }
                _ => {}
            }
            match self.subscription_receiver.try_recv() {
                Ok(subscription_object) => match subscription_object.subscription_object {
                    SubscriptionObject::ValidatedLedger(ledger) => {
                        if ledger.ledger_index > latest_ledger {
                            println!("Ledger {} is validated", ledger.ledger_index);
                            latest_ledger = ledger.ledger_index;
                        }
                        self.node_states.set_validated_ledger(subscription_object.peer as usize,ledger.clone());
                        node_states_cvar.notify_all();
                        self.write_to_subscription_file(subscription_object.peer, json!({"LedgerValidated": ledger}).to_string());
                        self.scheduler_sender.send(SubscriptionObject::ValidatedLedger(ledger.clone())).expect("Scheduler send failed");
                    }
                    SubscriptionObject::ReceivedValidation(validation) =>
                        self.write_to_subscription_file(subscription_object.peer, json!({"ValidationReceived": validation}).to_string()),
                    SubscriptionObject::PeerStatusChange(peer_status) =>
                        self.write_to_subscription_file(subscription_object.peer, json!({"PeerStatus": peer_status}).to_string()),
                    SubscriptionObject::ConsensusChange(consensus_change) => {
                        let new_consensus_phase = Self::parse_consensus_change(consensus_change.clone());
                        let node_state = &mut (*node_states_lock.lock())[subscription_object.peer as usize];
                        if new_consensus_phase == ConsensusPhase::Open && node_state.consensus_phase == ConsensusPhase::Accepted { node_state.current_consensus_round += 1 }
                        node_state.consensus_phase = new_consensus_phase;
                        println!("{:?}", node_state);
                        node_states_cvar.notify_all();
                        self.write_to_subscription_file(subscription_object.peer, json!({"ConsensusChange": consensus_change}).to_string());
                    }
                    SubscriptionObject::Transaction(transaction_subscription) => {
                        // if subscription_object.peer == 4 { println!("Transaction subscription received: {:?}", transaction_subscription); }
                        self.write_to_subscription_file(subscription_object.peer, json!({"Transaction": transaction_subscription}).to_string())
                    }
                },
                _ => {}
            }
        }
    }

    fn write_to_file(&mut self, ripple_message: &mut RippleMessage) {
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

pub struct RippleMessage {
    from_node: String,
    to_node: String,
    timestamp: DateTime<Utc>,
    message: RippleMessageObject
}

impl RippleMessage {
    pub fn new(from_node: String, to_node: String, timestamp: DateTime<Utc>, message: RippleMessageObject) -> Box<Self> {
        Box::from(RippleMessage { from_node, to_node, timestamp, message })
    }
}

impl Display for RippleMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let ripple_epoch = DateTime::parse_from_rfc3339("2000-01-01T00:00:00+00:00").unwrap();
        let from_node_buf = &self.from_node;
        let to_node_buf = &self.to_node;
        let time_since = self.timestamp.signed_duration_since(ripple_epoch).num_seconds();
        let message_buf = self.message.to_string();
        write!(f, "{} {} -> {} sent {}\n", time_since, from_node_buf, to_node_buf, message_buf)
    }
}
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::mpsc::{Receiver, TryRecvError};
use std::time::Instant;
use protobuf::Message;
use serde_json::json;
use crate::client::{ConsensusChange, PeerStatusEvent, PeerSubscriptionObject, ReceivedValidation, SubscriptionObject, ValidatedLedger};

/// Collects and writes data to files
/// Execution file stores all messages sent from the proxy
/// Subscription file stores all subscription messages received from the client
pub struct Collector {
    number_of_nodes: u16,
    ripple_message_receiver: Receiver<Box<RippleMessage>>,
    subscription_receiver: Receiver<PeerSubscriptionObject>,
    control_receiver: Receiver<String>,
    execution_file: File,
    subscription_files: Vec<File>,
    start: Instant
}

impl Collector {
    pub fn new(number_of_nodes: u16, ripple_message_receiver: Receiver<Box<RippleMessage>>, subscription_receiver: Receiver<PeerSubscriptionObject>, control_receiver: Receiver<String>) -> Self {
        let execution_file = File::create(Path::new("execution.txt")).expect("Opening execution file failed");
        let mut subscription_files = vec![];
        for peer in 0..number_of_nodes {
            let mut subscription_file = File::create(Path::new(format!("subscription_{}.json", peer).as_str())).expect("Opening subscription file failed");
            subscription_file.write_all(String::from("[\n").as_bytes()).unwrap();
            subscription_files.push(subscription_file);
        }
        Collector {
            number_of_nodes,
            ripple_message_receiver,
            subscription_receiver,
            control_receiver,
            execution_file,
            subscription_files,
            start: Instant::now()
        }
    }

    pub fn start(&mut self) {
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
                Ok(mut subscription_object) => match subscription_object.subscription_object {
                    SubscriptionObject::ValidatedLedger(ledger) =>
                        self.write_to_subscription_file(subscription_object.peer, json!({"LedgerValidated": ledger}).to_string()),
                    SubscriptionObject::ReceivedValidation(validation) =>
                        self.write_to_subscription_file(subscription_object.peer, json!({"ValidationReceived": validation}).to_string()),
                    SubscriptionObject::PeerStatusChange(peer_status) =>
                        self.write_to_subscription_file(subscription_object.peer, json!({"PeerStatus": peer_status}).to_string()),
                    SubscriptionObject::ConsensusChange(consensus_change) =>
                        self.write_to_subscription_file(subscription_object.peer, json!({"ConsensusChange": consensus_change}).to_string())
                },
                _ => {}
            }
        }
    }

    fn write_to_file(&mut self, ripple_message: &mut RippleMessage) {
        ripple_message.set_start(self.start);
        self.execution_file.write_all(ripple_message.to_string().as_bytes()).unwrap();
    }

    fn write_to_subscription_file(&mut self, peer: u16, text: String) {
        self.subscription_files[peer as usize].write_all((text + ",\n").as_bytes()).unwrap();
    }
}

pub struct RippleMessage {
    from_node: String,
    timestamp: Instant,
    message: Box<dyn Message>,
    start: Option<Instant>
}

impl RippleMessage {
    pub fn new(from_node: String, timestamp: Instant, message: Box<dyn Message>) -> Box<Self> {
        Box::from(RippleMessage { from_node, timestamp, message, start: None })
    }

    fn set_start(&mut self, start: Instant) {
        self.start = Option::from(start);
    }
}

impl Display for RippleMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let from_node_buf = &self.from_node;
        let time_since = if self.start.is_some() {
            self.timestamp.duration_since(self.start.unwrap()).as_millis()
        } else {
            0
        };
        let message_buf = self.message.descriptor().name();
        write!(f, "{} {} sent {}\n", time_since, from_node_buf, message_buf)
    }
}
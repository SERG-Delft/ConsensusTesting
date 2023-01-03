use serde_json::json;
use serialize::RippleMessage;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::client::{PeerSubscriptionObject, SubscriptionObject};

/// Collects and writes data to files
/// Execution file stores all messages sent from the proxy
/// Subscription file stores all subscription messages received from the client
pub struct Collector {
    ripple_message_receiver: tokio::sync::mpsc::Receiver<Box<RippleMessage>>,
    subscription_receiver: tokio::sync::mpsc::Receiver<PeerSubscriptionObject>,
    scheduler_sender: tokio::sync::mpsc::Sender<PeerSubscriptionObject>,
    execution_file: BufWriter<File>,
    subscription_files: Vec<BufWriter<File>>,
}

impl Collector {
    pub fn new(
        number_of_nodes: u16,
        ripple_message_receiver: tokio::sync::mpsc::Receiver<Box<RippleMessage>>,
        subscription_receiver: tokio::sync::mpsc::Receiver<PeerSubscriptionObject>,
        scheduler_sender: tokio::sync::mpsc::Sender<PeerSubscriptionObject>,
    ) -> Self {
        let execution_file =
            File::create(Path::new("execution.txt")).expect("Opening execution file failed");
        let mut subscription_files = vec![];
        for peer in 0..number_of_nodes {
            let mut subscription_file = BufWriter::new(
                File::create(Path::new(format!("subscription_{}.json", peer).as_str()))
                    .expect("Opening subscription file failed"),
            );
            subscription_file
                .write_all(String::from("[\n").as_bytes())
                .unwrap();
            subscription_files.push(subscription_file);
        }
        Collector {
            ripple_message_receiver,
            subscription_receiver,
            scheduler_sender,
            execution_file: BufWriter::new(execution_file),
            subscription_files,
        }
    }

    pub async fn start(self) {
        let mut ripple_message_receiver = self.ripple_message_receiver;
        let mut execution_file = self.execution_file;
        tokio::spawn(async move {
            loop {
                if let Some(message) = ripple_message_receiver.recv().await {
                    execution_file
                        .write_all(message.to_string().as_bytes())
                        .unwrap();
                }
            }
        });

        let mut subscription_receiver = self.subscription_receiver;
        let scheduler_sender = self.scheduler_sender;
        let mut subscription_files = self.subscription_files;
        tokio::spawn(async move {
            let mut write_to_subscription_file = |peer: u16, text: String| {
                subscription_files[peer as usize]
                    .write_all((text + ",\n").as_bytes())
                    .unwrap();
            };
            loop {
                if let Some(subscription_object) = subscription_receiver.recv().await {
                    match &subscription_object.subscription_object {
                        SubscriptionObject::ValidatedLedger(ledger) => {
                            println!(
                                "Ledger {} is validated with {} txns and {} hash",
                                ledger.ledger_index, ledger.txn_count, ledger.ledger_hash
                            );
                            write_to_subscription_file(
                                subscription_object.peer,
                                json!({ "LedgerValidated": ledger }).to_string(),
                            );
                            scheduler_sender
                                .send(subscription_object)
                                .await
                                .expect("Scheduler send failed");
                        }
                        SubscriptionObject::ReceivedValidation(validation) => {
                            write_to_subscription_file(
                                subscription_object.peer,
                                json!({ "ValidationReceived": validation }).to_string(),
                            )
                        }
                        SubscriptionObject::PeerStatusChange(peer_status) => {
                            write_to_subscription_file(
                                subscription_object.peer,
                                json!({ "PeerStatus": peer_status }).to_string(),
                            )
                        }
                        SubscriptionObject::ConsensusChange(consensus_change) => {
                            write_to_subscription_file(
                                subscription_object.peer,
                                json!({ "ConsensusChange": consensus_change }).to_string(),
                            )
                        }
                    }
                }
            }
        });
    }
}

use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::sync::mpsc::Sender;

use chrono::{DateTime, Utc};
use openssl::sha::sha256;
use serde_json::json;

use crate::client::{PeerSubscriptionObject, SubscriptionObject};
use crate::deserialization::parse2;
use crate::message_handler::RippleMessageObject;

/// Collects and writes data to files
/// Execution file stores all messages sent from the proxy
/// Subscription file stores all subscription messages received from the client
pub struct Collector {
    ripple_message_receiver: tokio::sync::mpsc::Receiver<Box<RippleMessage>>,
    subscription_receiver: tokio::sync::mpsc::Receiver<PeerSubscriptionObject>,
    scheduler_sender: Sender<PeerSubscriptionObject>,
    execution_file: BufWriter<File>,
    subscription_files: Vec<BufWriter<File>>,
}

impl Collector {
    pub fn new(
        number_of_nodes: u16,
        ripple_message_receiver: tokio::sync::mpsc::Receiver<Box<RippleMessage>>,
        subscription_receiver: tokio::sync::mpsc::Receiver<PeerSubscriptionObject>,
        scheduler_sender: Sender<PeerSubscriptionObject>,
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

#[derive(Debug)]
pub struct RippleMessage {
    from_node: String,
    to_node: String,
    timestamp: DateTime<Utc>,
    message: RippleMessageObject,
}

impl RippleMessage {
    pub fn new(
        from_node: String,
        to_node: String,
        timestamp: DateTime<Utc>,
        message: RippleMessageObject,
    ) -> Box<Self> {
        Box::from(RippleMessage {
            from_node,
            to_node,
            timestamp,
            message,
        })
    }
}

impl Display for RippleMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.message {
            RippleMessageObject::TMValidation(validation) => {
                let ripple_epoch =
                    DateTime::parse_from_rfc3339("2000-01-01T00:00:00+00:00").unwrap();
                let from_node_buf = &self.from_node;
                let to_node_buf = &self.to_node;
                let time_since = self
                    .timestamp
                    .signed_duration_since(ripple_epoch)
                    .num_seconds();
                // write!(f, "{}\n", hex::encode(validation.get_validation()));
                let parsed = parse2(validation.get_validation()).unwrap().1;
                // let pub_key = self.message.node_pub_key();
                let type_prefixed_key = [
                    &[28u8],
                    hex::decode(parsed["SigningPubKey"].as_str().unwrap())
                        .unwrap()
                        .as_slice(),
                ]
                .concat();
                let checksum = sha256(&sha256(&type_prefixed_key));
                let key = [&type_prefixed_key, &checksum[..4]].concat();
                let node_key = Some(
                    bs58::encode(key)
                        .with_alphabet(bs58::Alphabet::RIPPLE)
                        .into_string(),
                );
                let node = match node_key {
                    Some(ref key) => match key.as_str() {
                        "n9KkgT2SFxpQGic7peyokvkXcAmNLFob1AZXeErMFHxJ71q5MGaK" => "0",
                        "n9M6ouZU7cLwRHPiVZjgJdEgrVyx2uv9euZzepdb34wDoj1RP5uS" => "1",
                        "n9LJhBqLGTjPQa2KJtJmkHUubaHs1Y1ENYKZVmzZYhNb7GXh9m4j" => "2",
                        "n9KgN4axJo1WC3fjFoUSkJ4gtZX4Pk2jPZzGR5CE9ddo16ewAPjN" => "3",
                        "n9MsRMobdfpGvpXeGb3F6bm7WZbCiPrxzc1qBPP7wQox3NJzs5j2" => "4",
                        "n9JFX46v3d3WgQW8DJQeBwqTk8vaCR7LufApEy65J1eK4X7dZbR3" => "5",
                        "n9LFueHyYVJSyDArog2qtR42NixmeGxpaqFEFFp1xjxGU9aYRDZc" => "6",
                        _ => key.as_str(),
                    },
                    None => panic!("needs node key"),
                };
                writeln!(
                    f,
                    "{} [{}->{}] Validation {} validates {}",
                    time_since, from_node_buf, to_node_buf, node, parsed
                )
            }
            RippleMessageObject::TMProposeSet(proposal) => {
                let ripple_epoch =
                    DateTime::parse_from_rfc3339("2000-01-01T00:00:00+00:00").unwrap();
                let from_node_buf = &self.from_node;
                let to_node_buf = &self.to_node;
                let time_since = self
                    .timestamp
                    .signed_duration_since(ripple_epoch)
                    .num_seconds();
                let pub_key = self.message.node_pub_key();
                let node = match pub_key {
                    Some(ref key) => match key.as_str() {
                        "n9KkgT2SFxpQGic7peyokvkXcAmNLFob1AZXeErMFHxJ71q5MGaK" => "0",
                        "n9M6ouZU7cLwRHPiVZjgJdEgrVyx2uv9euZzepdb34wDoj1RP5uS" => "1",
                        "n9LJhBqLGTjPQa2KJtJmkHUubaHs1Y1ENYKZVmzZYhNb7GXh9m4j" => "2",
                        "n9KgN4axJo1WC3fjFoUSkJ4gtZX4Pk2jPZzGR5CE9ddo16ewAPjN" => "3",
                        "n9MsRMobdfpGvpXeGb3F6bm7WZbCiPrxzc1qBPP7wQox3NJzs5j2" => "4",
                        "n9JFX46v3d3WgQW8DJQeBwqTk8vaCR7LufApEy65J1eK4X7dZbR3" => "5",
                        "n9LFueHyYVJSyDArog2qtR42NixmeGxpaqFEFFp1xjxGU9aYRDZc" => "6",
                        _ => key.as_str(),
                    },
                    None => panic!("needs node key"),
                };
                writeln!(
                    f,
                    "{} [{}->{}] ProposeSet<{} proposes {}, seq={}, prev={}>",
                    time_since,
                    from_node_buf,
                    to_node_buf,
                    node,
                    hex::encode(&proposal.get_currentTxHash()[..2]),
                    proposal.get_proposeSeq(),
                    hex::encode(proposal.get_previousledger())
                )
            }
            _ => {
                let ripple_epoch =
                    DateTime::parse_from_rfc3339("2000-01-01T00:00:00+00:00").unwrap();
                let from_node_buf = &self.from_node;
                let to_node_buf = &self.to_node;
                let time_since = self
                    .timestamp
                    .signed_duration_since(ripple_epoch)
                    .num_seconds();
                let message_buf = self.message.to_string();
                writeln!(
                    f,
                    "{} {} [{}->{}] sent {}",
                    time_since,
                    self.message.node_pub_key().get_or_insert("".to_string()),
                    from_node_buf,
                    to_node_buf,
                    message_buf
                )
            }
        }
    }
}

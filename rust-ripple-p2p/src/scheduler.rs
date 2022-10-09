use std::collections::{HashMap, HashSet};
use std::sync::mpsc::Receiver as STDReceiver;
use std::sync::{Arc, Mutex};

use crate::spec_checker::{SpecChecker, Status};
use crate::ByzzFuzz;
use byteorder::{BigEndian, ByteOrder};
use chrono::Utc;
use log::error;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::mpsc::{Receiver as TokioReceiver, Sender as TokioSender};

use crate::client::{PeerSubscriptionObject, SubscriptionObject};
use crate::collector::RippleMessage;
use crate::message_handler::{from_bytes, invoke_protocol_message};

type P2PConnections = HashMap<usize, HashMap<usize, PeerChannel>>;

pub struct Scheduler {
    p2p_connections: P2PConnections,
    collector_sender: TokioSender<Box<RippleMessage>>,
    stable: Arc<Mutex<bool>>,
    latest_validated_ledger: Arc<Mutex<u32>>,
    byzz_fuzz: ByzzFuzz,
    spec_checker: SpecChecker,
    shutdown_tx: Sender<(HashMap<usize, String>, usize, String)>,
    shutdown_rx: Receiver<(HashMap<usize, String>, usize, String)>,
}

impl Scheduler {
    pub fn new(
        p2p_connections: P2PConnections,
        collector_sender: TokioSender<Box<RippleMessage>>,
        byzz_fuzz: ByzzFuzz,
        shutdown_tx: Sender<(HashMap<usize, String>, usize, String)>,
        shutdown_rx: Receiver<(HashMap<usize, String>, usize, String)>,
        public_key_to_index: HashMap<String, usize>,
    ) -> Self {
        Scheduler {
            p2p_connections,
            collector_sender,
            stable: Arc::new(Mutex::new(false)),
            latest_validated_ledger: Arc::new(Mutex::new(0)),
            byzz_fuzz,
            spec_checker: SpecChecker::new(public_key_to_index),
            shutdown_tx,
            shutdown_rx,
        }
    }

    pub async fn start(
        mut self,
        mut receiver: TokioReceiver<Event>,
        collector_receiver: STDReceiver<PeerSubscriptionObject>,
    ) {
        let stable_clone = self.stable.clone();
        let latest_validated_ledger_clone = self.latest_validated_ledger.clone();
        let transactions = Arc::new(Mutex::new(HashSet::new()));
        let collector_txs = transactions.clone();
        let disagree_message = Arc::new(Mutex::new(String::new()));
        let mutex_sequences_hashes = Arc::new(Mutex::new(HashMap::new()));
        let message_clone = disagree_message.clone();
        let mutex_clone = mutex_sequences_hashes.clone();
        let task = tokio::spawn(async move {
            Self::listen_to_collector(
                collector_receiver,
                stable_clone,
                latest_validated_ledger_clone,
                collector_txs,
                mutex_clone,
                message_clone,
            )
        });
        loop {
            tokio::select! {
                event_option = receiver.recv() => match event_option {
                    Some(event) => {
                        let (keep_going, reason) = self.execute_event(event).await;
                        let committed = transactions.lock().unwrap().len();
                        let message = disagree_message.lock().unwrap();
                        if !keep_going || committed == 7 || !message.is_empty() {
                            self.shutdown_tx.send((mutex_sequences_hashes.lock().unwrap().clone(), committed, if !message.is_empty() { message.clone() } else {if committed == 7 { "all committed".to_string() } else { reason.to_string() }})).unwrap();
                        }
                    },
                    None => error!("Peer senders failed")
                },
                _ = self.shutdown_rx.recv() => {
                    break;
                }
            }
        }
        task.await.unwrap();
    }

    async fn execute_event(&mut self, mut event: Event) -> (bool, &'static str) {
        event = self.byzz_fuzz.on_message(event).await;
        match self
            .spec_checker
            .check(event.from, from_bytes(&event.message))
        {
            Err(Status::Timeout) => return (false, "timeout_messages"),
            Err(Status::Liveness) => {
                println!("detected liveness violation");
                return (true, "liveness_violation");
            }
            Ok(()) => {}
        };
        self.p2p_connections
            .get(&event.to)
            .unwrap()
            .get(&event.from)
            .unwrap()
            .send(event.message.clone())
            .await;
        if self.byzz_fuzz.baseline {
            return (true, "everything good");
        }
        let collector_message = RippleMessage::new(
            event.from.to_string(),
            event.to.to_string(),
            Utc::now(),
            invoke_protocol_message(
                BigEndian::read_u16(&event.message[4..6]),
                &event.message[6..],
            ),
        );
        self.collector_sender
            .send(collector_message)
            .await
            .expect("Collector receiver failed");
        (true, "everything good")
    }

    #[allow(unused)]
    fn create_event(from: usize, to: usize, message: Vec<u8>) -> Event {
        Event { from, to, message }
    }

    fn listen_to_collector(
        collector_receiver: STDReceiver<PeerSubscriptionObject>,
        stable: Arc<Mutex<bool>>,
        latest_validated_ledger: Arc<Mutex<u32>>,
        transactions: Arc<Mutex<HashSet<u16>>>,
        mutex_sequences_hashes: Arc<Mutex<HashMap<usize, String>>>,
        mutex_disagree_messages: Arc<Mutex<String>>,
    ) {
        let mut set_stable = false;
        let mut local_latest_validated_ledger = 0;
        loop {
            match collector_receiver.recv() {
                Ok(subscription_object) => match subscription_object.subscription_object {
                    SubscriptionObject::ValidatedLedger(ledger) => {
                        if ledger.txn_count == 1 {
                            transactions
                                .lock()
                                .unwrap()
                                .insert(subscription_object.peer);
                            println!("transactions {:?}", transactions);
                        }

                        let sequence = ledger.ledger_index as usize;

                        let mut sequences_hashes = mutex_sequences_hashes.lock().unwrap();

                        if sequences_hashes.contains_key(&sequence) {
                            let sequence_hash = sequences_hashes.get(&sequence).unwrap();
                            if !sequence_hash.eq(&ledger.ledger_hash) {
                                mutex_disagree_messages.lock().unwrap().push_str(
                                    format!(
                                        "node {} validated {} whereas we stored {}",
                                        subscription_object.peer, ledger.ledger_hash, sequence_hash
                                    )
                                    .as_str(),
                                )
                            }
                        } else {
                            sequences_hashes.insert(sequence, ledger.ledger_hash);
                        }

                        if !set_stable {
                            *stable.lock().unwrap() = true;
                            set_stable = true;
                        }
                        if local_latest_validated_ledger < ledger.ledger_index {
                            *latest_validated_ledger.lock().unwrap() = ledger.ledger_index;
                            local_latest_validated_ledger = ledger.ledger_index;
                        }
                    }
                    _ => {}
                },
                Err(_) => {
                    break;
                }
            }
        }
    }
}

pub struct PeerChannel {
    sender: TokioSender<Vec<u8>>,
}

impl PeerChannel {
    pub fn new(sender: TokioSender<Vec<u8>>) -> Self {
        PeerChannel { sender }
    }

    pub async fn send(&self, message: Vec<u8>) {
        match self.sender.send(message).await {
            Ok(_) => {}
            Err(_err) => error!("Failed to send message to peer {}", _err),
        }
    }
}

pub struct Event {
    pub from: usize,
    pub to: usize,
    pub message: Vec<u8>,
}

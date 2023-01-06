use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use crate::specs::{Flags, SpecChecker};
use crate::ByzzFuzz;
use chrono::Utc;
use log::error;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::mpsc::{Receiver as TokioReceiver, Sender as TokioSender};

use crate::client::{PeerSubscriptionObject, SubscriptionObject};
use crate::message_handler::from_bytes;
use serialize::RippleMessage;

type P2PConnections = HashMap<usize, HashMap<usize, PeerChannel>>;

pub struct Scheduler {
    p2p_connections: P2PConnections,
    collector_sender: TokioSender<Box<RippleMessage>>,
    latest_validated_ledger: Arc<Mutex<u32>>,
    byzz_fuzz: ByzzFuzz,
    spec_checker: SpecChecker,
    shutdown_tx: Sender<(HashMap<usize, String>, usize, String)>,
    shutdown_rx: Receiver<(HashMap<usize, String>, usize, String)>,
    flags_tx: tokio::sync::broadcast::Sender<Flags>,
}

impl Scheduler {
    pub async fn new(
        p2p_connections: P2PConnections,
        collector_sender: TokioSender<Box<RippleMessage>>,
        byzz_fuzz: ByzzFuzz,
        shutdown_tx: Sender<(HashMap<usize, String>, usize, String)>,
        shutdown_rx: Receiver<(HashMap<usize, String>, usize, String)>,
        public_key_to_index: HashMap<String, usize>,
        flags_tx: Sender<Flags>,
    ) -> Self {
        Scheduler {
            p2p_connections,
            collector_sender,
            latest_validated_ledger: Arc::new(Mutex::new(0)),
            byzz_fuzz,
            spec_checker: SpecChecker::new(public_key_to_index, flags_tx.clone()).await,
            shutdown_tx,
            shutdown_rx,
            flags_tx,
        }
    }

    pub async fn start(
        &mut self,
        mut receiver: TokioReceiver<Event>,
        collector_receiver: TokioReceiver<PeerSubscriptionObject>,
    ) {
        let latest_validated_ledger_clone = self.latest_validated_ledger.clone();
        let transactions = Arc::new(Mutex::new(HashSet::new()));
        let collector_txs = transactions.clone();
        let disagree_message = Arc::new(Mutex::new(String::new()));
        let mutex_sequences_hashes = Arc::new(Mutex::new(HashMap::new()));
        let message_clone = disagree_message.clone();
        let mutex_clone = mutex_sequences_hashes.clone();
        let task = tokio::spawn(Self::listen_to_collector(
            collector_receiver,
            latest_validated_ledger_clone,
            collector_txs,
            mutex_clone,
            message_clone,
        ));
        let mut flags_rx = self.flags_tx.subscribe();
        let shutdown_tx = self.shutdown_tx.clone();
        tokio::spawn(async move {
            while let Ok(flag) = flags_rx.recv().await {
                if let Flags::Timeout = flag {
                    shutdown_tx
                        .send((HashMap::new(), 0, "flags".to_owned()))
                        .unwrap();
                    break;
                }
            }
        });
        loop {
            tokio::select! {
                event_option = receiver.recv() => match event_option {
                    Some(event) => {
                        // println!("event incoming");
                        self.execute_event(event).await;
                        let committed = transactions.lock().unwrap().len();
                        let message = disagree_message.lock().unwrap();
                        if committed == 7 || !message.is_empty() {
                            self.shutdown_tx.send((mutex_sequences_hashes.lock().unwrap().clone(), committed, if !message.is_empty() { message.clone() } else { "all committed".to_string() })).unwrap();
                        }
                        // println!("executed event");
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

    async fn execute_event(&mut self, mut event: Event) {
        event = self.byzz_fuzz.on_message(event).await;
        if let Ok(message) = from_bytes(&event.message) {
            self.spec_checker.check(event.from, &message);
            let collector_message = RippleMessage::new(
                event.from.to_string(),
                event.to.to_string(),
                Utc::now(),
                message,
            );
            self.collector_sender
                .send(collector_message)
                .await
                .expect("Collector receiver failed");
        }
        self.p2p_connections
            .get(&event.to)
            .unwrap()
            .get(&event.from)
            .unwrap()
            .send(event.message.clone())
            .await;
    }

    async fn listen_to_collector(
        mut collector_receiver: tokio::sync::mpsc::Receiver<PeerSubscriptionObject>,
        latest_validated_ledger: Arc<Mutex<u32>>,
        transactions: Arc<Mutex<HashSet<u16>>>,
        mutex_sequences_hashes: Arc<Mutex<HashMap<usize, String>>>,
        mutex_disagree_messages: Arc<Mutex<String>>,
    ) {
        let mut local_latest_validated_ledger = 0;
        while let Some(subscription_object) = collector_receiver.recv().await {
            if let SubscriptionObject::ValidatedLedger(ledger) =
                subscription_object.subscription_object
            {
                if ledger.txn_count == 1 {
                    transactions
                        .lock()
                        .unwrap()
                        .insert(subscription_object.peer);
                    println!("transactions {:?}", transactions.lock().unwrap());
                }

                let sequence = ledger.ledger_index as usize;

                let mut sequences_hashes = mutex_sequences_hashes.lock().unwrap();

                if let Entry::Vacant(e) = sequences_hashes.entry(sequence) {
                    e.insert(ledger.ledger_hash);
                } else {
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
                }

                if local_latest_validated_ledger < ledger.ledger_index {
                    *latest_validated_ledger.lock().unwrap() = ledger.ledger_index;
                    local_latest_validated_ledger = ledger.ledger_index;
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

#[derive(Debug)]
pub struct Event {
    pub from: usize,
    pub to: usize,
    pub message: Vec<u8>,
}

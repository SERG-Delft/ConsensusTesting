use bs58::Alphabet;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::sync::mpsc::{Receiver as STDReceiver, Sender as STDSender};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::ByzzFuzz;
use byteorder::{BigEndian, ByteOrder};
use chrono::{DateTime, Utc};
use itertools::Itertools;
use json::parse;
use log::error;
use openssl::sha::sha512;
use protobuf::Message;
use rippled_binary_codec::serialize::serialize_tx;
use secp256k1::{PublicKey, Secp256k1, SecretKey};
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::mpsc::{Receiver as TokioReceiver, Sender as TokioSender};
use xrpl::core::keypairs::utils::sha512_first_half;
use xrpl::indexmap::serde_seq::deserialize;

use crate::client::SubscriptionObject;
use crate::collector::RippleMessage;
use crate::container_manager::NodeKeys;
use crate::deserialization::{parse2, parse_canonical_binary_format};
use crate::message_handler::RippleMessageObject::{
    TMHaveTransactionSet, TMProposeSet, TMStatusChange, TMTransaction, TMValidation,
};
use crate::message_handler::{invoke_protocol_message, RippleMessageObject};
use crate::protos::ripple::NodeEvent;

type P2PConnections = HashMap<usize, HashMap<usize, PeerChannel>>;

pub struct Scheduler {
    p2p_connections: P2PConnections,
    collector_sender: STDSender<Box<RippleMessage>>,
    stable: Arc<Mutex<bool>>,
    latest_validated_ledger: Arc<Mutex<u32>>,
    byzz_fuzz: ByzzFuzz,
    message_count: usize,
    shutdown_tx: Sender<()>,
    shutdown_rx: Receiver<()>,
}

impl Scheduler {
    pub fn new(
        p2p_connections: P2PConnections,
        collector_sender: STDSender<Box<RippleMessage>>,
        byzz_fuzz: ByzzFuzz,
        mut shutdown_tx: Sender<()>,
        shutdown_rx: Receiver<()>,
    ) -> Self {
        Scheduler {
            p2p_connections,
            collector_sender,
            stable: Arc::new(Mutex::new(false)),
            latest_validated_ledger: Arc::new(Mutex::new(0)),
            byzz_fuzz,
            message_count: 0,
            shutdown_tx,
            shutdown_rx,
        }
    }

    pub async fn start(
        mut self,
        mut receiver: TokioReceiver<Event>,
        collector_receiver: STDReceiver<SubscriptionObject>,
    ) {
        let stable_clone = self.stable.clone();
        let latest_validated_ledger_clone = self.latest_validated_ledger.clone();
        let task = tokio::spawn(async move {
            Self::listen_to_collector(
                collector_receiver,
                stable_clone,
                latest_validated_ledger_clone,
            )
        });
        loop {
            tokio::select! {
                event_option = receiver.recv() => match event_option {
                    Some(event) => if !self.execute_event(event).await {
                        self.shutdown_tx.send(()).unwrap();
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

    async fn execute_event(&mut self, mut event: Event) -> bool {
        event = self.byzz_fuzz.on_message(event).await;
        self.message_count += 1;
        if self.message_count >= 10_000 {
            return false;
        }
        self.p2p_connections
            .get(&event.to)
            .unwrap()
            .get(&event.from)
            .unwrap()
            .send(event.message.clone())
            .await;
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
            .expect("Collector receiver failed");
        true
    }

    #[allow(unused)]
    fn create_event(from: usize, to: usize, message: Vec<u8>) -> Event {
        Event { from, to, message }
    }

    fn listen_to_collector(
        collector_receiver: STDReceiver<SubscriptionObject>,
        stable: Arc<Mutex<bool>>,
        latest_validated_ledger: Arc<Mutex<u32>>,
    ) {
        let mut set_stable = false;
        let mut local_latest_validated_ledger = 0;
        loop {
            match collector_receiver.recv() {
                Ok(subscription_object) => match subscription_object {
                    SubscriptionObject::ValidatedLedger(ledger) => {
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

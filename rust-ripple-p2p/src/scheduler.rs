use log::error;
use std::collections::HashMap;
use chrono::Utc;
use tokio::sync::mpsc::{Sender as TokioSender, Receiver as TokioReceiver};
use std::sync::mpsc::{Sender as STDSender, Receiver as STDReceiver};
use std::sync::{Arc, Mutex};
use std::thread;
use byteorder::{BigEndian, ByteOrder};
use crate::client::{SubscriptionObject};
use crate::collector::RippleMessage;
use crate::message_handler::{invoke_protocol_message, RippleMessageObject};
use crate::deserialization::deserialize;

type P2PConnections = HashMap<usize, HashMap<usize, PeerChannel>>;

pub struct Scheduler {
    p2p_connections: P2PConnections,
    collector_sender: STDSender<Box<RippleMessage>>,
    stable: Arc<Mutex<bool>>,
    latest_validated_ledger: Arc<Mutex<u32>>
}

impl Scheduler {
    pub fn new(p2p_connections: P2PConnections, collector_sender: STDSender<Box<RippleMessage>>) -> Self {
        Scheduler { p2p_connections, collector_sender, stable: Arc::new(Mutex::new(false)), latest_validated_ledger: Arc::new(Mutex::new(0)) }
    }

    pub fn start(self, mut receiver: TokioReceiver<Event>, collector_receiver: STDReceiver<SubscriptionObject>) {
        let stable_clone = self.stable.clone();
        let latest_validated_ledger_clone = self.latest_validated_ledger.clone();
        thread::spawn(move || Self::listen_to_collector(collector_receiver, stable_clone, latest_validated_ledger_clone));
        loop {
            match receiver.blocking_recv() {
                Some(event) => self.execute_event(event),
                None => error!("Peer senders failed")
            }
        }
    }

    fn execute_event(&self, event: Event) {
        let rmo: RippleMessageObject = invoke_protocol_message(BigEndian::read_u16(&event.message[4..6]), &event.message[6..]);
        match rmo {
            RippleMessageObject::TMValidation(ref x) => {deserialize(x.get_validation())}
            _ => {}
        }
        self.collector_sender.send(RippleMessage::new(format!("Ripple{}", event.from+1), format!("Ripple{}", event.to+1), Utc::now(), rmo)).expect("Collector receiver failed");
        self.p2p_connections.get(&event.to).unwrap().get(&event.from).unwrap().send(event.message);
    }

    #[allow(unused)]
    fn create_event(from: usize, to: usize, message: Vec<u8>) -> Event {
        Event { from, to, message }
    }

    fn listen_to_collector(collector_receiver: STDReceiver<SubscriptionObject>, stable: Arc<Mutex<bool>>, latest_validated_ledger: Arc<Mutex<u32>>) {
        let mut set_stable = false;
        let mut local_latest_validated_ledger = 0;
        loop {
            match collector_receiver.recv() {
                Ok(subscription_object) => {
                    match subscription_object {
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
                    }
                }
                Err(_) => {}
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

    pub fn send(&self, message: Vec<u8>) {
        match self.sender.blocking_send(message) {
            Ok(_) => { }
            Err(_err) => error!("Failed to send message to peer {}", _err)
        }
    }
}

pub struct Event {
    pub from: usize,
    pub to: usize,
    pub message: Vec<u8>
}

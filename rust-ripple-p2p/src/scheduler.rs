use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver as STDReceiver, Sender as STDSender};
use std::thread;

use byteorder::{BigEndian, ByteOrder};
use chrono::Utc;
use log::error;
use tokio::sync::mpsc::{Receiver as TokioReceiver, Sender as TokioSender};

use crate::client::SubscriptionObject;
use crate::collector::RippleMessage;
use crate::deserialization::deserialize;
use crate::message_handler::{invoke_protocol_message, RippleMessageObject};

type P2PConnections = HashMap<usize, HashMap<usize, PeerChannel>>;

pub struct Scheduler {
    p2p_connections: P2PConnections,
    collector_sender: STDSender<Box<RippleMessage>>,
    stable: Arc<Mutex<bool>>,
    latest_validated_ledger: Arc<Mutex<u32>>,
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
        let mut rmo: RippleMessageObject = invoke_protocol_message(BigEndian::read_u16(&event.message[4..6]), &event.message[6..]);
        println!("[{}->{}] {}", event.from + 1, event.to + 1, rmo);
        self.p2p_connections.get(&event.to).unwrap().get(&event.from).unwrap().send(event.message);
        // match rmo {
        //     RippleMessageObject::TMTransaction(_) => {
        //         let bin = deserialize(&mut rmo, event.from, event.to);
        //         let res = [event.message[0..6].to_vec(), bin].concat();
        //         println!("{:?}", event.message);
        //         println!("{:?}", res);
        //         // assert!(event.message.eq(&res));
        //         self.p2p_connections.get(&event.to).unwrap().get(&event.from).unwrap().send(res);
        //     }
        //     // RippleMessageObject::TMLedgerData(_) => {
        //     //     let bin = deserialize(&mut rmo, event.from, event.to);
        //     //     let res = [event.message[0..6].to_vec(), bin].concat();
        //     //     println!("{:?}", event.message);
        //     //     println!("{:?}", res);
        //     //     // assert!(event.message.eq(&res));
        //     //     self.p2p_connections.get(&event.to).unwrap().get(&event.from).unwrap().send(res);
        //     // }
        //     _ => {
        //         self.p2p_connections.get(&event.to).unwrap().get(&event.from).unwrap().send(event.message);
        //     }
        // }
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
            Ok(_) => {}
            Err(_err) => error!("Failed to send message to peer {}", _err)
        }
    }
}

pub struct Event {
    pub from: usize,
    pub to: usize,
    pub message: Vec<u8>,
}

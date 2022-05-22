use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver as STDReceiver, Sender as STDSender};
use std::thread;

use byteorder::{BigEndian, ByteOrder};
use chrono::Utc;
use log::error;
use protobuf::Message;
use tokio::sync::mpsc::{Receiver as TokioReceiver, Sender as TokioSender};

use crate::client::SubscriptionObject;
use crate::collector::RippleMessage;
use crate::container_manager::NodeKeys;
use crate::deserialization::{parse_canonical_binary_format};
use crate::message_handler::{invoke_protocol_message, RippleMessageObject};
use crate::message_handler::RippleMessageObject::{TMTransaction, TMValidation, TMProposeSet, TMHaveTransactionSet};

type P2PConnections = HashMap<usize, HashMap<usize, PeerChannel>>;

pub struct Scheduler {
    p2p_connections: P2PConnections,
    collector_sender: STDSender<Box<RippleMessage>>,
    stable: Arc<Mutex<bool>>,
    latest_validated_ledger: Arc<Mutex<u32>>,
    node_keys: Vec<NodeKeys>
}

impl Scheduler {
    pub fn new(p2p_connections: P2PConnections, collector_sender: STDSender<Box<RippleMessage>>, node_keys: Vec<NodeKeys>) -> Self {
        Scheduler { p2p_connections, collector_sender, stable: Arc::new(Mutex::new(false)), latest_validated_ledger: Arc::new(Mutex::new(0)), node_keys }
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

    fn execute_event(&self, mut event: Event) {
        let mut rmo: RippleMessageObject = invoke_protocol_message(BigEndian::read_u16(&event.message[4..6]), &event.message[6..]);
        let mutated_unl: Vec<usize> = vec![4, 5, 6];
        match rmo {
            TMHaveTransactionSet(_) => {
                // if event.from == 3 { return () }
            }
            TMTransaction(ref mut trx) => {
                if event.from == 3 && mutated_unl.contains(&event.to) {
                    // println!("pre  {}", hex::encode(&event.message));
                    trx.set_rawTransaction(hex::decode(parse_canonical_binary_format(trx.get_rawTransaction())).unwrap());
                    // println!("post {}", hex::encode([&event.message[0..6], &trx.write_to_bytes().unwrap()].concat()));
                    event.message = [&event.message[0..6], &trx.write_to_bytes().unwrap()].concat();
                }
                println!("[{}->{}] {}", event.from + 1, event.to + 1, rmo);
            }
            TMValidation(_) => {
                // if event.from == 3 { return () }
                // println!("[{}->{}] {}", event.from + 1, event.to + 1, rmo);
            }
            TMProposeSet(ref mut proposal) => {
                if event.from == 3 && mutated_unl.contains(&event.to) && !proposal.get_currentTxHash().starts_with(&[0]) {
                    proposal.set_currentTxHash(hex::decode("E803E1999369975AED1BFD2444A3552A73383C03A2004CB784CE07E13EBD7D7C").unwrap());
                    proposal.set_signature(hex::decode("3045022100a36058cae09aa725515fa94363372f2542a70015ee7cff640d6690b5f552575902207be2137c73559c788f8eaab50c29bdae8b525191b9d7641d3e3690561cdd721a").unwrap());
                }
                println!("[{}->{}] {}", event.from + 1, event.to + 1, rmo);
            }
            _ => ()
        }
        // println!("[{}->{}] {}", event.from + 1, event.to + 1, rmo);
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

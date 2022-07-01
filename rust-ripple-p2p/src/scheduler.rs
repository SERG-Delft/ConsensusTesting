use std::collections::HashMap;
use std::convert::TryFrom;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver as STDReceiver, Sender as STDSender};
use std::thread;
use bs58::Alphabet;

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
use crate::ByzzFuzz;

use crate::client::SubscriptionObject;
use crate::collector::RippleMessage;
use crate::container_manager::NodeKeys;
use crate::deserialization::{parse2, parse_canonical_binary_format};
use crate::message_handler::{invoke_protocol_message, RippleMessageObject};
use crate::message_handler::RippleMessageObject::{TMHaveTransactionSet, TMProposeSet, TMStatusChange, TMTransaction, TMValidation};
use crate::protos::ripple::NodeEvent;

type P2PConnections = HashMap<usize, HashMap<usize, PeerChannel>>;

pub struct Scheduler {
    p2p_connections: P2PConnections,
    collector_sender: STDSender<Box<RippleMessage>>,
    stable: Arc<Mutex<bool>>,
    latest_validated_ledger: Arc<Mutex<u32>>,
    node_keys: Vec<NodeKeys>,
    mutated_ledger_hash: Vec<u8>,
    byzz_fuzz: ByzzFuzz,
    message_count: usize,
    shutdown_tx: Sender<()>,
    shutdown_rx: Receiver<()>,
}

impl Scheduler {
    pub fn new(
        p2p_connections: P2PConnections, collector_sender: STDSender<Box<RippleMessage>>, node_keys: Vec<NodeKeys>,
        byzz_fuzz: ByzzFuzz,
        mut shutdown_tx: Sender<()>,
        shutdown_rx: Receiver<()>,
    ) -> Self {
        Scheduler {
            p2p_connections,
            collector_sender,
            stable: Arc::new(Mutex::new(false)),
            latest_validated_ledger: Arc::new(Mutex::new(0)),
            node_keys,
            mutated_ledger_hash: hex::decode("0000000000000000000000000000000000000000000000000000000000000000").unwrap(),
            byzz_fuzz,
            message_count: 0,
            shutdown_tx,
            shutdown_rx,
        }
    }

    pub async fn start(
        mut self,
        mut receiver: TokioReceiver<Event>,
        collector_receiver: STDReceiver<SubscriptionObject>
    ) {
        let stable_clone = self.stable.clone();
        let latest_validated_ledger_clone = self.latest_validated_ledger.clone();
        let task = tokio::spawn(async move {
            Self::listen_to_collector(collector_receiver, stable_clone, latest_validated_ledger_clone)
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
        // println!("messages {}", self.message_count);
        if self.message_count >= 10_000 {
            return false;
        }
        // this code should go in the byzzfuzz on_message method
        // let mut rmo: RippleMessageObject = invoke_protocol_message(BigEndian::read_u16(&event.message[4..6]), &event.message[6..]);
        // let mutated_unl: Vec<usize> = vec![4, 5, 6];
        // match rmo {
        //     TMTransaction(ref mut trx) => {
        //         if event.from == 3 && mutated_unl.contains(&event.to) {
        //             let mutation = "1200002280000000240000000161400000000BED48A068400000000000000A73210330E7FC9D56BB25D6893BA3F317AE5BCF33B3291BD63DB32654A313222F7FD02074473045022100F1D8AA686F6241A5F39106FFDA94AA218118D385B58A00E633425D882B17205902200B38092D3F990359928F393485DC352CD0F3C22E4559280354FB423BC7F08BEC8114B5F762798A53D543A014CAF8B297CFF8F2F937E883149D94BFF9BAAA5267D5733CA2B59950B4C9A01564";
        //             trx.set_rawTransaction(hex::decode(mutation).unwrap());
        //             event.message = [&event.message[0..6], &trx.write_to_bytes().unwrap()].concat();
        //
        //         }
        //     }
        //     TMProposeSet(ref mut proposal) => {
        //         if event.from == 3 && (!proposal.get_currentTxHash().starts_with(&[0]) || proposal.get_proposeSeq() != 0) && proposal.get_nodePubKey()[1] == 149 {
        //             if mutated_unl.contains(&event.to) {
        //                 proposal.set_currentTxHash(hex::decode("e803e1999369975aed1bfd2444a3552a73383c03a2004cb784ce07e13ebd7d7c").unwrap());
        //             } else {
        //                 proposal.set_currentTxHash(hex::decode("FE0E71183243245E3619EFCBE073F2D7EEDE9B0F0BF1A1B2B7D9F1E22B4A5C2A").unwrap());
        //             }
        //             let hash = sha512_first_half([
        //                 &[80, 82, 80, 00],
        //                 &proposal.get_proposeSeq().to_be_bytes(),
        //                 &proposal.get_closeTime().to_be_bytes(),
        //                 proposal.get_previousledger(),
        //                 proposal.get_currentTxHash()
        //             ].concat().as_slice());
        //             let keys = &self.node_keys[3];
        //             let algo = Secp256k1::new();
        //             let priv_key = bs58::decode(&keys.validation_private_key).with_alphabet(Alphabet::RIPPLE).into_vec().unwrap();
        //             let message = secp256k1::Message::from_slice(&hash).unwrap();
        //             let signature = algo.sign_ecdsa(&message, &SecretKey::from_slice(&priv_key[1..33]).unwrap());
        //             proposal.set_signature(signature.serialize_der().to_vec());
        //         }
        //         event.message = [&event.message[0..6], &proposal.write_to_bytes().unwrap()].concat();
        //         let bytes = ((event.message.len() - 6) as u32).to_be_bytes();
        //         event.message[0] = bytes[0];
        //         event.message[1] = bytes[1];
        //         event.message[2] = bytes[2];
        //         event.message[3] = bytes[3];
        //     }
        //     TMStatusChange(ref mut status) => {
        //         if event.from > 3 && status.get_newEvent() == NodeEvent::neACCEPTED_LEDGER {
        //             if !status.get_ledgerHash().to_vec().eq(&self.mutated_ledger_hash) {
        //                 self.mutated_ledger_hash = status.get_ledgerHash().to_vec().clone();
        //                 println!("cached ledger {}", hex::encode(&self.mutated_ledger_hash))
        //             }
        //         }
        //     }
        //     TMValidation(ref mut validation) => {
        //         let (_, mut parsed) = parse2(validation.get_validation()).unwrap();
        //         if event.from == 3 && parsed["ConsensusHash"].as_str().unwrap().eq_ignore_ascii_case("fe0e71183243245e3619efcbe073f2d7eede9b0f0bf1a1b2b7d9f1e22b4a5c2a") && mutated_unl.contains(&event.to) && parsed["SigningPubKey"].as_str().unwrap().eq_ignore_ascii_case("02954103E420DA5361F00815929207B36559492B6C37C62CB2FE152CCC6F3C11C5") {
        //             let secp256k1 = Secp256k1::new();
        //             let private_key = SecretKey::from_slice(
        //                 &bs58::decode(&self.node_keys[3].validation_private_key)
        //                     .with_alphabet(Alphabet::RIPPLE)
        //                     .into_vec()
        //                     .unwrap()[1..33]
        //             ).unwrap();
        //
        //             let mutated_validation = hex::decode(
        //                 format!("22{}26{}29{}3A{}51{}5017{}5019{}7321{}",
        //                         hex::encode(parsed["Flags"].as_u32().unwrap().to_be_bytes()),
        //                         hex::encode(parsed["LedgerSequence"].as_u32().unwrap().to_be_bytes()),
        //                         hex::encode(parsed["SigningTime"].as_u32().unwrap().to_be_bytes()),
        //                         hex::encode(parsed["Cookie"].as_u64().unwrap().to_be_bytes()),
        //                         hex::encode(&self.mutated_ledger_hash),
        //                         "E803E1999369975AED1BFD2444A3552A73383C03A2004CB784CE07E13EBD7D7C",
        //                         parsed["ValidatedHash"].as_str().unwrap(),
        //                         parsed["SigningPubKey"].as_str().unwrap()
        //                 )
        //             ).unwrap();
        //
        //             let mutated_signing_hash = sha512_first_half([
        //                 &[86, 65, 76, 00],
        //                 mutated_validation.as_slice()
        //             ].concat().as_slice());
        //             let mutated_message = secp256k1::Message::from_slice(&mutated_signing_hash).unwrap();
        //             let mutated_signature = secp256k1.sign_ecdsa(&mutated_message, &private_key);
        //             let der_sign = mutated_signature.serialize_der().to_vec();
        //
        //             let val = hex::decode(
        //                 format!("22{}26{}29{}3A{}51{}5017{}5019{}7321{}76{}{}",
        //                         hex::encode(parsed["Flags"].as_u32().unwrap().to_be_bytes()),
        //                         hex::encode(parsed["LedgerSequence"].as_u32().unwrap().to_be_bytes()),
        //                         hex::encode(parsed["SigningTime"].as_u32().unwrap().to_be_bytes()),
        //                         hex::encode(parsed["Cookie"].as_u64().unwrap().to_be_bytes()),
        //                         hex::encode(&self.mutated_ledger_hash),
        //                         "E803E1999369975AED1BFD2444A3552A73383C03A2004CB784CE07E13EBD7D7C",
        //                         parsed["ValidatedHash"].as_str().unwrap(),
        //                         parsed["SigningPubKey"].as_str().unwrap(),
        //                         hex::encode((der_sign.len() as u8).to_be_bytes()),
        //                         hex::encode(der_sign)
        //                 )
        //             ).unwrap();
        //
        //             validation.set_validation(val);
        //             event.message = [&event.message[0..6], &validation.write_to_bytes().unwrap()].concat();
        //             let bytes = ((event.message.len() - 6) as u32).to_be_bytes();
        //             event.message[0] = bytes[0];
        //             event.message[1] = bytes[1];
        //             event.message[2] = bytes[2];
        //             event.message[3] = bytes[3];
        //         }
        //     }
        //     _ => ()
        // }
        self.p2p_connections.get(&event.to).unwrap().get(&event.from).unwrap().send(event.message.clone()).await;
        let collector_message = RippleMessage::new(event.from.to_string(),
                                                   event.to.to_string(), Utc::now(), invoke_protocol_message(BigEndian::read_u16(&event.message[4..6]), &event.message[6..]));
        self.collector_sender.send(collector_message).expect("Collector receiver failed");
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
            Err(_err) => error!("Failed to send message to peer {}", _err)
        }
    }
}

pub struct Event {
    pub from: usize,
    pub to: usize,
    pub message: Vec<u8>,
}

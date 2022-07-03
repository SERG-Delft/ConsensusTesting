use bs58::Alphabet;
use itertools::Itertools;
use nom::AsBytes;
use protobuf::Message;
use std::cmp::max;
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;
use std::net::TcpStream;
use std::sync::Arc;
use std::time::Duration;
use websocket::Message as WsMessage;

use crate::client::Client;
use crate::container_manager::NodeKeys;
use crate::deserialization::parse2;
use crate::message_handler::{from_bytes, invoke_protocol_message, RippleMessageObject};
use crate::protos::ripple::{NodeEvent, TMTransaction};
use crate::scheduler::Event;
use crate::toxiproxy::ToxiproxyClient;
use rand::prelude::*;
use secp256k1::{Secp256k1, SecretKey};
use serde_json::json;
use set_partitions::{set_partitions, ArrayVecSetPartition, HashSubsets};
use tokio::time::sleep;
use websocket::receiver::Reader;
use websocket::sender::Writer;
use websocket::ClientBuilder;
use xrpl::core::keypairs::utils::sha512_first_half;
use RippleMessageObject::{TMProposeSet, TMStatusChange, TMValidation};

const LARGE_SCOPE: bool = false;

pub struct ByzzFuzz {
    n: usize, // number of processes
    c: usize, // bound on the #rounds with process faults
    d: usize, // bound on the #rounds with network faults
    r: usize, // bound on the #rounds with faults
    current_index: usize,
    current_round: usize,
    applied_partitions: bool,
    pub process_faults: HashMap<usize, (HashSet<usize>, u32)>,
    pub network_faults: HashMap<usize, Vec<HashSet<u8>>>,
    pub toxiproxy: Arc<ToxiproxyClient>,
    mutated_ledger_hash: Vec<u8>,
    node_keys: Vec<NodeKeys>,
    pub sequences_hashes: HashMap<usize, String>,
    byzantine_sender: Writer<TcpStream>,
}

impl ByzzFuzz {
    pub fn new(n: usize, c: usize, d: usize, r: usize, node_keys: Vec<NodeKeys>) -> Self {
        assert_eq!(n, 7);
        let mut process_faults = HashMap::with_capacity(c);
        (0..c).for_each(|_| {
            let round = thread_rng().gen_range(2..r + 2);
            let sublist = if thread_rng().gen_bool(0.5) {
                (0..5)
            } else {
                (1..7)
            }
            .powerset()
            .collect_vec();
            let mut subset = HashSet::new();
            for peer in sublist
                .get(thread_rng().gen_range(1..(sublist.len())))
                .unwrap()
            {
                subset.insert(*peer);
            }
            // (4..7).for_each(|i| {
            //     subset.insert(i);
            // });
            process_faults.insert(round, (subset, thread_rng().gen()));
        });

        // // {4: {2}, 2: {1, 2}, 5: {6}}
        // process_faults.insert(2, HashSet::from([1, 2]));
        // process_faults.insert(4, HashSet::from([2]));
        // process_faults.insert(5, HashSet::from([6]));

        let mut network_faults = HashMap::with_capacity(d);
        (0..d)
            .map(|_| NetworkFault::sample_network_fault(n, r)) //TODO network faults right time
            .for_each(|fault| {
                network_faults.insert(fault.round, fault.partition);
            });
        let mut sequences_hashes: HashMap<usize, String> = HashMap::new();
        let client = ClientBuilder::new("ws://127.0.0.1:6008")
            .unwrap()
            .connect_insecure()
            .unwrap();
        let (_, sender) = client.split().unwrap();
        Self {
            n,
            c,
            d,
            r,
            current_index: 0,
            current_round: 0,
            applied_partitions: true,
            process_faults,
            network_faults,
            toxiproxy: Arc::new(ToxiproxyClient::new("http://localhost:8474/")),
            mutated_ledger_hash: hex::decode(
                "0000000000000000000000000000000000000000000000000000000000000000",
            )
            .unwrap(),
            node_keys,
            sequences_hashes,
            byzantine_sender: sender,
        }
    }

    pub async fn on_message(&mut self, mut event: Event) -> Event {
        let mut message = from_bytes(&event.message);
        self.update_round(&message).await;
        self.apply_partition().await;
        if self.process_faults.contains_key(&self.current_round)
            && self
                .process_faults
                .get(&self.current_round)
                .unwrap()
                .0
                .contains(&event.from)
        {
            match message {
                TMStatusChange(ref mut status) => {
                    if status.get_newEvent() == NodeEvent::neACCEPTED_LEDGER {
                        if !status
                            .get_ledgerHash()
                            .to_vec()
                            .eq(&self.mutated_ledger_hash)
                        {
                            self.mutated_ledger_hash = status.get_ledgerHash().to_vec().clone();
                            println!("cached ledger {}", hex::encode(&self.mutated_ledger_hash))
                        }
                    }
                }
                _ => (),
            }
        }
        if event.from == 3
            && self.process_faults.contains_key(&self.current_round)
            && self
                .process_faults
                .get(&self.current_round)
                .unwrap()
                .0
                .contains(&event.to)
        {
            let seed = self.process_faults.get(&self.current_round).unwrap().1;
            event = self.apply_mutation(event, &mut message, seed);
        }
        event
    }

    async fn update_round(&mut self, message: &RippleMessageObject) {
        if let Some(index) = self.get_index(message) {
            if self.current_index < index {
                self.current_index = index;
            }
        }
        if let Some(round) = self.get_round(message) {
            if self.current_round < round {
                println!("round {}", round);
                self.current_round = round;
                self.applied_partitions = false;
                if self.current_round == 2 {
                    let json = json!({
                        "id": "Ripple TXN",
                        "command": "submit",
                        "tx_json": Client::create_payment_transaction(200000000, crate::client::_ACCOUNT_ID, crate::client::_GENESIS_ADDRESS),
                        "secret": crate::client::_GENESIS_SEED
                    });
                    self.byzantine_sender
                        .send_message(&WsMessage::text(json.to_string()))
                        .unwrap();
                    println!("submitted");
                }
            }
        }
    }

    fn get_index(&self, message: &RippleMessageObject) -> Option<usize> {
        match message {
            TMStatusChange(status) => {
                let index = status.get_ledgerSeq() as usize;
                if index < 5 {
                    return None;
                }
                match status.get_newEvent() {
                    NodeEvent::neACCEPTED_LEDGER => Some(index - 5),
                    NodeEvent::neCLOSING_LEDGER => Some(index + 1 - 5),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn get_round(&self, message: &RippleMessageObject) -> Option<usize> {
        match message {
            TMProposeSet(_) => Some(self.current_index * 2),
            TMValidation(validation) => {
                let (_, validation) = parse2(validation.get_validation()).unwrap();
                if validation["LedgerSequence"].as_usize().unwrap() != self.current_index + 5 {
                    return None;
                }
                Some(self.current_index * 2 + 1)
            }
            _ => None,
        }
    }

    async fn apply_partition(&mut self) {
        if !self.applied_partitions && self.network_faults.contains_key(&self.current_round) {
            self.applied_partitions = true;
            self.toxiproxy
                .partition(self.network_faults.get(&self.current_round).unwrap())
                .await;
        } else if !self.applied_partitions {
            self.applied_partitions = true;
            self.toxiproxy
                .partition(&vec![(0..self.n as u8).collect()])
                .await;
        }
    }

    fn apply_mutation(
        &mut self,
        mut event: Event,
        message: &mut RippleMessageObject,
        seed: u32,
    ) -> Event {
        let mutate_sequence_ids = seed % 2 == 0;
        match message {
            RippleMessageObject::TMTransaction(ref mut transaction) => {
                let mutation = "1200002280000000240000000161400000000BED48A068400000000000000A73210330E7FC9D56BB25D6893BA3F317AE5BCF33B3291BD63DB32654A313222F7FD02074473045022100F1D8AA686F6241A5F39106FFDA94AA218118D385B58A00E633425D882B17205902200B38092D3F990359928F393485DC352CD0F3C22E4559280354FB423BC7F08BEC8114B5F762798A53D543A014CAF8B297CFF8F2F937E883149D94BFF9BAAA5267D5733CA2B59950B4C9A01564";
                transaction.set_rawTransaction(hex::decode(mutation).unwrap());
                event.message =
                    [&event.message[0..6], &transaction.write_to_bytes().unwrap()].concat();
            }
            TMProposeSet(ref mut proposal) => {
                if proposal.get_nodePubKey()[1] == 149 {
                    if !mutate_sequence_ids {
                        proposal.set_currentTxHash(
                            hex::decode(
                                "e803e1999369975aed1bfd2444a3552a73383c03a2004cb784ce07e13ebd7d7c",
                            )
                            .unwrap(),
                        );
                    } else {
                        let initial_propose_seq = proposal.get_proposeSeq();
                        let mut corrupted_propose_seq = initial_propose_seq + 1;
                        if LARGE_SCOPE {
                            corrupted_propose_seq = seed % 2;
                        }
                        proposal.set_proposeSeq(corrupted_propose_seq);
                    }
                    let hash = sha512_first_half(
                        [
                            &[80, 82, 80, 00],
                            &proposal.get_proposeSeq().to_be_bytes(),
                            &proposal.get_closeTime().to_be_bytes(),
                            proposal.get_previousledger(),
                            proposal.get_currentTxHash(),
                        ]
                        .concat()
                        .as_slice(),
                    );
                    let keys = &self.node_keys[3];
                    let algo = Secp256k1::new();
                    let priv_key = bs58::decode(&keys.validation_private_key)
                        .with_alphabet(Alphabet::RIPPLE)
                        .into_vec()
                        .unwrap();
                    let message = secp256k1::Message::from_slice(&hash).unwrap();
                    let signature = algo
                        .sign_ecdsa(&message, &SecretKey::from_slice(&priv_key[1..33]).unwrap());
                    proposal.set_signature(signature.serialize_der().to_vec());
                }
                event.message =
                    [&event.message[0..6], &proposal.write_to_bytes().unwrap()].concat();
                let bytes = ((event.message.len() - 6) as u32).to_be_bytes();
                event.message[0] = bytes[0];
                event.message[1] = bytes[1];
                event.message[2] = bytes[2];
                event.message[3] = bytes[3];
            }
            TMValidation(ref mut validation) => {
                let (_, mut parsed) = parse2(validation.get_validation()).unwrap();
                if event.from == 3
                    // && parsed["ConsensusHash"]
                    //     .as_str()
                    //     .unwrap()
                    //     .eq_ignore_ascii_case(
                    //         "fe0e71183243245e3619efcbe073f2d7eede9b0f0bf1a1b2b7d9f1e22b4a5c2a",
                    //     )
                    && parsed["SigningPubKey"]
                    .as_str()
                    .unwrap()
                    .eq_ignore_ascii_case(
                        "02954103E420DA5361F00815929207B36559492B6C37C62CB2FE152CCC6F3C11C5",
                    )
                {
                    let secp256k1 = Secp256k1::new();
                    let private_key = SecretKey::from_slice(
                        &bs58::decode(&self.node_keys[3].validation_private_key)
                            .with_alphabet(Alphabet::RIPPLE)
                            .into_vec()
                            .unwrap()[1..33],
                    )
                    .unwrap();

                    if !mutate_sequence_ids {
                        let mutated_validation = hex::decode(format!(
                            "22{}26{}29{}3A{}51{}5017{}5019{}7321{}",
                            hex::encode(parsed["Flags"].as_u32().unwrap().to_be_bytes()),
                            hex::encode(parsed["LedgerSequence"].as_u32().unwrap().to_be_bytes()),
                            hex::encode(parsed["SigningTime"].as_u32().unwrap().to_be_bytes()),
                            hex::encode(parsed["Cookie"].as_u64().unwrap().to_be_bytes()),
                            hex::encode(&self.mutated_ledger_hash),
                            "E803E1999369975AED1BFD2444A3552A73383C03A2004CB784CE07E13EBD7D7C",
                            parsed["ValidatedHash"].as_str().unwrap(),
                            parsed["SigningPubKey"].as_str().unwrap()
                        ))
                        .unwrap();

                        let mutated_signing_hash = sha512_first_half(
                            [&[86, 65, 76, 00], mutated_validation.as_slice()]
                                .concat()
                                .as_slice(),
                        );
                        let mutated_message =
                            secp256k1::Message::from_slice(&mutated_signing_hash).unwrap();
                        let mutated_signature =
                            secp256k1.sign_ecdsa(&mutated_message, &private_key);
                        let der_sign = mutated_signature.serialize_der().to_vec();

                        let val = hex::decode(format!(
                            "22{}26{}29{}3A{}51{}5017{}5019{}7321{}76{}{}",
                            hex::encode(parsed["Flags"].as_u32().unwrap().to_be_bytes()),
                            hex::encode(parsed["LedgerSequence"].as_u32().unwrap().to_be_bytes()),
                            hex::encode(parsed["SigningTime"].as_u32().unwrap().to_be_bytes()),
                            hex::encode(parsed["Cookie"].as_u64().unwrap().to_be_bytes()),
                            hex::encode(&self.mutated_ledger_hash),
                            "E803E1999369975AED1BFD2444A3552A73383C03A2004CB784CE07E13EBD7D7C",
                            parsed["ValidatedHash"].as_str().unwrap(),
                            parsed["SigningPubKey"].as_str().unwrap(),
                            hex::encode((der_sign.len() as u8).to_be_bytes()),
                            hex::encode(der_sign)
                        ))
                        .unwrap();

                        validation.set_validation(val);
                    } else {
                        let ledger_sequence = parsed["LedgerSequence"].as_u32().unwrap();
                        let mut new_ledger_sequence = ledger_sequence + 1;
                        if LARGE_SCOPE {
                            new_ledger_sequence = seed % 2;
                        }

                        let mutated_validation = hex::decode(format!(
                            "22{}26{}29{}3A{}51{}5017{}5019{}7321{}",
                            hex::encode(parsed["Flags"].as_u32().unwrap().to_be_bytes()),
                            hex::encode(new_ledger_sequence.to_be_bytes()),
                            hex::encode(parsed["SigningTime"].as_u32().unwrap().to_be_bytes()),
                            hex::encode(parsed["Cookie"].as_u64().unwrap().to_be_bytes()),
                            parsed["hash"].as_str().unwrap(),
                            parsed["ConsensusHash"].as_str().unwrap(),
                            parsed["ValidatedHash"].as_str().unwrap(),
                            parsed["SigningPubKey"].as_str().unwrap()
                        ))
                        .unwrap();

                        let mutated_signing_hash = sha512_first_half(
                            [&[86, 65, 76, 00], mutated_validation.as_slice()]
                                .concat()
                                .as_slice(),
                        );
                        let mutated_message =
                            secp256k1::Message::from_slice(&mutated_signing_hash).unwrap();
                        let mutated_signature =
                            secp256k1.sign_ecdsa(&mutated_message, &private_key);
                        let der_sign = mutated_signature.serialize_der().to_vec();

                        let val = hex::decode(format!(
                            "22{}26{}29{}3A{}51{}5017{}5019{}7321{}76{}{}",
                            hex::encode(parsed["Flags"].as_u32().unwrap().to_be_bytes()),
                            hex::encode(new_ledger_sequence.to_be_bytes()),
                            hex::encode(parsed["SigningTime"].as_u32().unwrap().to_be_bytes()),
                            hex::encode(parsed["Cookie"].as_u64().unwrap().to_be_bytes()),
                            parsed["hash"].as_str().unwrap(),
                            parsed["ConsensusHash"].as_str().unwrap(),
                            parsed["ValidatedHash"].as_str().unwrap(),
                            parsed["SigningPubKey"].as_str().unwrap(),
                            hex::encode((der_sign.len() as u8).to_be_bytes()),
                            hex::encode(der_sign)
                        ))
                        .unwrap();

                        validation.set_validation(val);
                    }

                    event.message =
                        [&event.message[0..6], &validation.write_to_bytes().unwrap()].concat();
                    let bytes = ((event.message.len() - 6) as u32).to_be_bytes();
                    event.message[0] = bytes[0];
                    event.message[1] = bytes[1];
                    event.message[2] = bytes[2];
                    event.message[3] = bytes[3];
                }
            }
            _ => (),
        }
        event
    }
}

#[derive(Debug)]
struct NetworkFault {
    round: usize,
    partition: Vec<HashSet<u8>>,
}

impl NetworkFault {
    pub fn sample_network_fault(n: usize, r: usize) -> Self {
        assert!(n <= 16);

        let bell_number = set_partitions(n).unwrap();
        let partition = thread_rng().gen_range(0..bell_number);

        let mut partitions: ArrayVecSetPartition<u8, HashSubsets<u8>, 16> =
            ArrayVecSetPartition::try_with_size(n).unwrap();
        for _ in 0..partition {
            partitions.increment();
        }

        Self {
            round: thread_rng().gen_range(2..r + 2),
            partition: partitions.subsets().subsets().to_vec(),
        }
    }
}

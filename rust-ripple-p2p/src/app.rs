use std::collections::HashMap;
use std::net::{SocketAddr, Ipv4Addr, IpAddr};
use std::thread;

use log::*;
use itertools::Itertools;

use super::{EmptyResult};
use crate::client::{Client};
use crate::collector::{Collector};
use crate::peer_connection::PeerConnection;
use crate::scheduler::{PeerChannel, Scheduler};
use crate::genetic_algorithm;


const _NODE_PRIVATE_KEY: &str = "e55dc8f3741ac9668dbe858409e5d64f5ce88380f7228eccfe82b92b2c7848ba";
const _NODE_PUBLIC_KEY_BASE58: &str = "n9KAa2zVWjPHgfzsE3iZ8HAbzJtPrnoh4H2M2HgE7dfqtvyEb1KJ";
// Account and its keys to send transaction to
const _ACCOUNT_ID: &str = "rE4DHSdcXafD7DkpJuFCAvc3CvsgXHjmEJ";
const _MASTER_KEY: &str = "BUSY MARS SLED SNUG OBOE REID SUNK NEW GYM LAD LICE FEAT";
const _MASTER_SEED: &str = "saNSJMEBKisBr6phJtGXUcV85RBZ3";
const _MASTER_SEED_HEX: &str = "FDDE6A91607445E59C6F7CF07AF7B661";
const _PUBLIC_KEY_HEX: &str = "03137FF01C82A1CF507CC243EBF629A99F2256FA43BCB7A458F638AF9A5488CD87";
const _PUBLIC_KEY: &str = "aBQsqGF1HEduKrHrSVzNE5yeCTJTGgrsKgyjNLgabS2Rkq7CgZiq";

// Genesis account with initial supply of XRP
const _GENESIS_SEED: &str = "snoPBrXtMeMyMHUVTgbuqAfg1SUTb";
const _GENESIS_ADDRESS: &str = "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh";

const _AMOUNT: u32 = 2u32.pow(31);

// Peer identities
const PRIVATE_KEYS: [&'static str; 5] = ["ssiNcpPcuBEwAyranF3wLt9UgefZv",
                                       "ssen1bRpA3ig9PPD7NwPVkxLcvgBW",
                                       "shXDCbJnfKbKze177MWPduTXQ5wsv",
                                       "snwB8RcbpEwzgJFUeAoSPDaXbtHDx",
                                       "saakCgDucD2q31GYHYdEbZLWJxVir"];

const PUBLIC_KEYS: [&'static str; 5] = ["n9MY9K6YBuPJm7mYFtQYYYSetRTAnR1SnGaQ3uTdcppQYkdQ6SnD",
                                      "n9MUM9gZ5HLCJY35ebgMCVpSbPm1ftAxdbyiq5ZzZR2rUWMvoc9H",
                                      "n9Ljh4A9A6PzhEFi7YLFG5du1tVx7E5wA2c9roZNZ6uMnJgezR7q",
                                      "n9MVitj842zxST7LLnNBiVhLEbQ7pgmvLZqDwMv5enpgAHxYyD3M",
                                      "n9J8Mp1mrT8ovunq3hoZzan2uacr9iM3o7Wsx3BctbPiTwNmwi9s"];

pub struct App {
    peers: u16,
    only_subscribe: bool
}

impl App {
    pub fn new(peers: u16, only_subscribe: bool) -> Self {
        App { peers, only_subscribe }
    }

    /// Start proxy
    /// Starts a separate thread per p2p connection, which in turn starts one thread per peer,
    /// which in turn start an extra thread for sending to that peer
    /// Every p2p connection has two senders and receivers for relaying messages to and from the scheduler
    /// Every message gets relayed by the scheduler
    /// A separate thread is created for each node which handles websocket client requests
    pub async fn start(&self) -> EmptyResult {
        let mut tokio_tasks = vec![];
        let mut threads = vec![];
        let (collector_tx, collector_rx) = std::sync::mpsc::channel();
        let (_control_tx, control_rx) = std::sync::mpsc::channel();
        let (subscription_tx, subscription_rx) = std::sync::mpsc::channel();
        let (collector_state_tx, scheduler_state_rx) = std::sync::mpsc::channel();
        let peer = self.peers.clone();
        // Start the collector which writes output to files
        let collector_task = thread::spawn(move || {
            Collector::new(peer, collector_rx, subscription_rx, control_rx, collector_state_tx).start();
        });
        threads.push(collector_task);

        // Start p2p connections
        if !self.only_subscribe {
            let addrs = self.get_addrs(self.peers);
            let mut peer_senders = HashMap::new();
            let mut peer_receivers = HashMap::new();
            let mut scheduler_peer_channels = HashMap::new();
            let (scheduler_sender, scheduler_receiver) = tokio::sync::mpsc::channel(32);
            let (ga_scheduler_sender, ga_scheduler_receiver) = std::sync::mpsc::channel();
            let (scheduler_ga_sender, scheduler_ga_receiver) = std::sync::mpsc::channel();

            thread::spawn(||genetic_algorithm::run(ga_scheduler_sender, scheduler_ga_receiver));

            for pair in (0..peer).into_iter().combinations(2).into_iter() {
                let i = pair[0] as usize;
                let j = pair[1] as usize;
                let tx_peer_i = scheduler_sender.clone();
                let tx_peer_j = scheduler_sender.clone();
                let (tx_scheduler_i, rx_peer_i) = tokio::sync::mpsc::channel(32);
                let (tx_scheduler_j, rx_peer_j) = tokio::sync::mpsc::channel(32);
                peer_senders.entry(i).or_insert(HashMap::new()).insert(j, tx_peer_i);
                peer_senders.entry(j).or_insert(HashMap::new()).insert(i, tx_peer_j);
                peer_receivers.entry(i).or_insert(HashMap::new()).insert(j, rx_peer_i);
                peer_receivers.entry(j).or_insert(HashMap::new()).insert(i, rx_peer_j);
                scheduler_peer_channels.entry(i).or_insert(HashMap::new()).insert(j, PeerChannel::new(tx_scheduler_i));
                scheduler_peer_channels.entry(j).or_insert(HashMap::new()).insert(i, PeerChannel::new(tx_scheduler_j));
            }

            let scheduler = Scheduler::new(scheduler_peer_channels, collector_tx);
            let scheduler_thread = thread::spawn(move || {
                scheduler.start(scheduler_receiver, scheduler_state_rx, scheduler_ga_sender, ga_scheduler_receiver);
            });
            threads.push(scheduler_thread);

            for pair in (0..peer).into_iter().combinations(2).into_iter() {
                let i = pair[0] as usize;
                let j = pair[1] as usize;
                let peer_receiver_i = peer_receivers.get_mut(&i).unwrap().remove(&j).unwrap();
                let peer_sender_i = peer_senders.get_mut(&i).unwrap().remove(&j).unwrap();
                let peer_receiver_j = peer_receivers.get_mut(&j).unwrap().remove(&i).unwrap();
                let peer_sender_j = peer_senders.get_mut(&j).unwrap().remove(&i).unwrap();

                let name = format!("ripple{}, ripple{}", i+1, j+1);
                let address_i = addrs[i].clone();
                let address_j = addrs[j].clone();
                // let thread = thread::Builder::new().name(String::from(name.clone())).spawn(move || {
                let peer = PeerConnection::new(
                    &name,
                    address_i,
                    address_j,
                    String::from(PRIVATE_KEYS[i]),
                    String::from(PRIVATE_KEYS[j]),
                    String::from(PUBLIC_KEYS[i]),
                    String::from(PUBLIC_KEYS[j])
                );
                let (thread1, thread2) = peer.connect(
                    i,
                    j,
                    peer_sender_i,
                    peer_sender_j,
                    peer_receiver_i,
                    peer_receiver_j
                ).await;
                tokio_tasks.push(thread1);
                tokio_tasks.push(thread2);
            }
        }
        // Connect websocket client to ripples
        for i in 0..self.peers {
            let _client = Client::new(i, format!("ws://127.0.0.1:600{}", 5+i).as_str(), subscription_tx.clone());
            // let sender_clone = client.sender_channel.clone();
            // threads.push(thread::spawn(move || {
            //     let mut counter = 0;
            //     // Send payment transaction every 10 seconds
            //     loop {
            //         sleep(Duration::from_secs(10));
            //         Client::sign_and_submit(
            //             &sender_clone,
            //             format!("Ripple{}: {}", i, &*counter.to_string()).as_str(),
            //             &Client::create_payment_transaction(_AMOUNT, _ACCOUNT_ID, _GENESIS_ADDRESS),
            //             _GENESIS_SEED
            //         );
            //         counter += 1;
            //     }
            // }));
        }

        for tokio_task in tokio_tasks {
            tokio_task.await.expect("task failed");
        }
        for thread in threads {
            thread.join().unwrap();
        }
        Ok(())
    }

    fn get_addrs(&self, peers: u16) -> Vec<SocketAddr> {
        let nodes = (0..peers).map(|x| SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127,0,0,1)), 51235 + x)).collect();
        debug!("{:?}", nodes);
        nodes
    }
}
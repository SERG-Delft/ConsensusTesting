use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use itertools::Itertools;
use log::*;

use crate::ByzzFuzz;
use crate::client::Client;
use crate::collector::Collector;
use crate::container_manager::NodeKeys;
use crate::peer_connection::PeerConnection;
use crate::scheduler::{PeerChannel, Scheduler};
use crate::toxiproxy::ToxiproxyClient;

use super::EmptyResult;

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

pub struct App {
    peers: u16,
    only_subscribe: bool,
    node_keys: Vec<NodeKeys>,
    toxiproxy: ToxiproxyClient,
}

impl App {
    pub fn new(peers: u16, only_subscribe: bool, node_keys: Vec<NodeKeys>) -> Self {
        let toxiproxy = ToxiproxyClient::new("http://localhost:8474/");
        App { peers, only_subscribe, node_keys, toxiproxy }
    }

    /// Start proxy
    /// Starts a separate thread per p2p connection, which in turn starts one thread per peer,
    /// which in turn start an extra thread for sending to that peer
    /// Every p2p connection has two senders and receivers for relaying messages to and from the scheduler
    /// Every message gets relayed by the scheduler
    /// A separate thread is created for each node which handles websocket client requests
    pub async fn start(&self, byzz_fuzz: ByzzFuzz) -> EmptyResult {
        let mut tokio_tasks = vec![];
        let (collector_tx, collector_rx) = std::sync::mpsc::channel();
        let (_control_tx, control_rx) = std::sync::mpsc::channel();
        let (subscription_tx, subscription_rx) = std::sync::mpsc::channel();
        let (collector_state_tx, scheduler_state_rx) = std::sync::mpsc::channel();
        let peer = self.peers.clone();
        // Start the collector which writes output to files
        let collector_task = tokio::spawn(async move {
            Collector::new(peer, collector_rx, subscription_rx, control_rx, collector_state_tx).start();
        });
        tokio_tasks.push(collector_task);

        // Start p2p connections
        if !self.only_subscribe {
            let addrs = self.get_addrs(self.peers);
            let mut peer_senders = HashMap::new();
            let mut peer_receivers = HashMap::new();
            let mut scheduler_peer_channels = HashMap::new();
            let (scheduler_sender, scheduler_receiver) = tokio::sync::mpsc::channel(32);

            let peer_pairs = (0..peer).into_iter().combinations(2).collect_vec();

            for pair in &peer_pairs {
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

            let scheduler = Scheduler::new(
                scheduler_peer_channels, collector_tx, self.node_keys.clone(),
                byzz_fuzz,
            );
            let scheduler_thread = tokio::spawn(async move {
                scheduler.start(scheduler_receiver, scheduler_state_rx).await;
            });
            tokio_tasks.push(scheduler_thread);

            self.toxiproxy.populate(&peer_pairs).await;

            for pair in &peer_pairs {

                let i = pair[0] as usize;
                let j = pair[1] as usize;

                let peer_receiver_i = peer_receivers.get_mut(&i).unwrap().remove(&j).unwrap();
                let peer_sender_i = peer_senders.get_mut(&i).unwrap().remove(&j).unwrap();
                let peer_receiver_j = peer_receivers.get_mut(&j).unwrap().remove(&i).unwrap();
                let peer_sender_j = peer_senders.get_mut(&j).unwrap().remove(&i).unwrap();

                let name = format!("ripple{}, ripple{}", i + 1, j + 1);
                let address_i = addrs[i].clone();
                let address_j = addrs[j].clone();
                // let thread = thread::Builder::new().name(String::from(name.clone())).spawn(move || {
                let peer = PeerConnection::new(
                    &name,
                    address_i,
                    address_j,
                    self.node_keys[i].validation_seed.clone(),
                    self.node_keys[j].validation_seed.clone(),
                    self.node_keys[i].validation_public_key.clone(),
                    self.node_keys[j].validation_public_key.clone()
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
            if i < 5 {
                let _client = Client::new(i, format!("ws://127.0.0.1:600{}", 5+i).as_str(), subscription_tx.clone());
                // if i == 0 {
                //     let tx = Client::create_payment_transaction(_AMOUNT, _ACCOUNT_ID, _GENESIS_ADDRESS);
                //     Client::sign_and_submit(&_client.sender_channel, "waddup", &tx, _GENESIS_SEED)
                // }
                // if i == 0 {
                //     let sender_clone = _client.sender_channel.clone();
                //     threads.push(thread::spawn(move || {
                //         let mut counter = 0;
                //         // Send payment transaction every 10 seconds
                //         loop {
                //             sleep(Duration::from_secs(10));
                //             Client::sign_and_submit(
                //                 &sender_clone,
                //                 format!("Ripple{}: {}", i, &*counter.to_string()).as_str(),
                //                 &Client::create_payment_transaction(_AMOUNT, _ACCOUNT_ID, _GENESIS_ADDRESS),
                //                 _GENESIS_SEED
                //             );
                //             counter += 1;
                //         }
                //     }));
                // }
            }
            else {
                let _client = Client::new(i, format!("ws://127.0.0.1:60{}", 5+i).as_str(), subscription_tx.clone());
            }
        }

        for tokio_task in tokio_tasks {
            tokio_task.await.expect("task failed");
        }
        Ok(())
    }

    fn get_addrs(&self, peers: u16) -> Vec<SocketAddr> {
        let nodes = (0..peers).map(|x| SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127,0,0,1)), 51235 + x)).collect();
        debug!("{:?}", nodes);
        nodes
    }
}
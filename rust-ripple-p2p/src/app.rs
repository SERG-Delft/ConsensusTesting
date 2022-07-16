#![allow(unused_imports)]
use std::collections::HashMap;
use std::net::{SocketAddr, Ipv4Addr, IpAddr};
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender};
use tokio::sync::mpsc::{Receiver as TokioReceiver};
use std::thread;
use genevo::operator::prelude::{MaximizeSelector, MultiPointCrossBreeder, RouletteWheelSelector};

use log::*;
use itertools::Itertools;
use websocket::Message;

use super::{EmptyResult};
use crate::client::{AccountInfo, Client, Payment, Transaction};
use crate::collector::{Collector, RippleMessage};
use crate::container_manager::NodeKeys;
use crate::failure_writer::{ConsensusPropertyTypes, FailureWriter};
use crate::ga::crossover::NoCrossoverOperator;
use crate::ga::encoding::delay_encoding::{DelayMapPhenotype, DelayGenotype};
use crate::ga::encoding::{ExtendedGenotype, ExtendedPhenotype};
use crate::ga::fitness::ExtendedFitness;
use crate::ga::genetic_algorithm;
use crate::ga::genetic_algorithm::{CurrentFitness, run_default_mu_lambda_priorities};
use crate::ga::parameters::{default_mu_lambda_delays, default_mu_lambda_priorities, Parameter};
use crate::ga::population_builder::{build_delays_population, build_priorities_population};
use crate::ga::encoding::priority_encoding::{PriorityGenotype, PriorityMapPhenotype};
use crate::locality::{run_locality_experiment_delays, run_locality_experiment_priorities};
use crate::peer_connection::PeerConnection;
use crate::scaling::{run_priority_scaling_experiment, run_scaling_experiment};
use crate::scheduler::{Event, P2PConnections, PeerChannel, Scheduler};
use crate::node_state::{MutexNodeStates, NodeState, NodeStates};
use crate::{CONFIG, Configuration, NUM_NODES};
use crate::scheduler::delay_scheduler::DelayScheduler;
use crate::scheduler::priority_scheduler::PriorityScheduler;
use crate::trace_comparisons::{run_fitness_comparison, run_no_delays, run_predetermined_delays, run_delay_trace_graph_creation, run_priority_trace_graph_creation, run_predetermined_priorities, run_random_priorities, run_random_delays};

const _NODE_PRIVATE_KEY: &str = "e55dc8f3741ac9668dbe858409e5d64f5ce88380f7228eccfe82b92b2c7848ba";
const _NODE_PUBLIC_KEY_BASE58: &str = "n9KAa2zVWjPHgfzsE3iZ8HAbzJtPrnoh4H2M2HgE7dfqtvyEb1KJ";
// Account and its keys to send transaction to
const _ACCOUNT_ADDRESS: &str = "rE4DHSdcXafD7DkpJuFCAvc3CvsgXHjmEJ";
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
    node_keys: Vec<NodeKeys>
}

impl App {
    pub fn new(peers: u16, node_keys: Vec<NodeKeys>) -> Self {
        App { peers, node_keys }
    }

    /// Start proxy
    /// Starts a separate thread per p2p connection, which in turn starts one thread per peer,
    /// which in turn start an extra thread for sending to that peer
    /// Every p2p connection has two senders and receivers for relaying messages to and from the scheduler
    /// Every message gets relayed by the scheduler
    /// A separate thread is created for each node which handles websocket client requests
    pub async fn start(&self, scheduler_type: SchedulerType) -> EmptyResult {
        let mut tokio_tasks = vec![];
        let mut threads = vec![];
        let (collector_tx, collector_rx) = std::sync::mpsc::channel();
        let (subscription_tx, subscription_rx) = std::sync::mpsc::channel();
        let (server_state_tx, server_state_rx) = std::sync::mpsc::channel();
        let (test_harness_tx, test_harness_rx) = std::sync::mpsc::channel();
        let (account_info_tx, account_info_rx) = std::sync::mpsc::channel();
        let (balance_sender, balance_receiver) = std::sync::mpsc::channel();
        let (failure_sender, failure_receiver) = std::sync::mpsc::channel();
        let peer = self.peers.clone();

        let mut node_state_vec = vec![NodeState::new(0); peer as usize];
        for i in 0..peer { node_state_vec[i as usize].peer = i as usize }
        let node_states = NodeStates::new(node_state_vec);
        let mutex_node_states = Arc::new(MutexNodeStates::new(node_states));
        let mutex_node_states_clone = mutex_node_states.clone();

        // Start the collector which writes output to files and collects information on nodes
        let collector_task = thread::spawn(move || {
            Collector::new(peer, subscription_rx, mutex_node_states_clone).start(collector_rx, server_state_rx);
        });
        threads.push(collector_task);

        let failure_mutex_node_states = mutex_node_states.clone();
        FailureWriter::start_failure_writer(failure_receiver, failure_mutex_node_states);

        // Create a client for each peer, which subscribes (among others) to certain streams
        let mut clients = vec![];
        for i in 0..self.peers {
            clients.push(Client::new(i, format!("ws://127.0.0.1:{}", 6005+i).as_str(), subscription_tx.clone(), server_state_tx.clone(), test_harness_tx.clone(), account_info_tx.clone(), balance_sender.clone()));
        }
        let client_senders = clients.iter().map(|client| client.sender_channel.clone()).collect_vec();

        // Start p2p connections
        let addrs = self.get_addrs(self.peers);
        let mut peer_senders = HashMap::new();
        let mut peer_receivers = HashMap::new();
        let mut scheduler_peer_channels = HashMap::new();
        let (scheduler_sender, scheduler_receiver) = tokio::sync::mpsc::channel(1000);
        let (scheduler_ga_sender, scheduler_ga_receiver) = std::sync::mpsc::channel::<CurrentFitness>();

        // For every combination (exclusive) of peers, create the necessary senders and receivers
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

        let scheduler_data = SchedulerData::new(
            scheduler_peer_channels,
            collector_tx,
            mutex_node_states,
            self.node_keys.clone(),
            failure_sender,
            scheduler_receiver,
            scheduler_ga_sender,
            client_senders,
            test_harness_rx,
            account_info_rx,
            balance_receiver
        );

        // Start GA and scheduler
        match scheduler_type {
            SchedulerType::Priority => {
                Self::start_default_mu_lambda_priorities(
                    4,
                    4,
                    scheduler_data,
                    scheduler_ga_receiver,
                );
            }
            SchedulerType::Delay => {
                Self::start_default_mu_lambda_delays(
                    4,
                    4,
                    scheduler_data,
                    scheduler_ga_receiver,
                );
            }
            SchedulerType::RandomPriority => {
                let ga_scheduler_sender = Self::start_scheduler::<PriorityScheduler, PriorityGenotype, PriorityMapPhenotype>(scheduler_data);
                threads.push(thread::spawn(|| run_random_priorities(ga_scheduler_sender, scheduler_ga_receiver, CONFIG.search_budget)));
            }
            SchedulerType::RandomDelay => {
                let ga_scheduler_sender = Self::start_scheduler::<DelayScheduler, DelayGenotype, DelayMapPhenotype>(scheduler_data);
                threads.push(thread::spawn(|| run_random_delays(ga_scheduler_sender, scheduler_ga_receiver, CONFIG.search_budget)));
            }
            SchedulerType::DelayTraceGraph => {
                let mutex_node_states_clone_2 = scheduler_data.mutex_node_states.clone();
                let ga_scheduler_sender = Self::start_scheduler::<DelayScheduler, DelayGenotype, DelayMapPhenotype>(scheduler_data);
                threads.push(thread::spawn(|| run_delay_trace_graph_creation(ga_scheduler_sender, scheduler_ga_receiver, mutex_node_states_clone_2)));
            }
            SchedulerType::PriorityTraceGraph => {
                let mutex_node_states_clone_2 = scheduler_data.mutex_node_states.clone();
                let ga_scheduler_sender = Self::start_scheduler::<PriorityScheduler, PriorityGenotype, PriorityMapPhenotype>(scheduler_data);
                threads.push(thread::spawn(|| run_priority_trace_graph_creation(ga_scheduler_sender, scheduler_ga_receiver, mutex_node_states_clone_2)));
            }
            SchedulerType::FitnessComparison => {
                let ga_scheduler_sender = Self::start_scheduler::<DelayScheduler, DelayGenotype, DelayMapPhenotype>(scheduler_data);
                threads.push(thread::spawn(|| run_fitness_comparison(ga_scheduler_sender, scheduler_ga_receiver)));
            }
            SchedulerType::PredeterminedDelay => {
                let ga_scheduler_sender = Self::start_scheduler::<DelayScheduler, DelayGenotype, DelayMapPhenotype>(scheduler_data);
                threads.push(thread::spawn(|| run_predetermined_delays(ga_scheduler_sender, scheduler_ga_receiver, 100)));
            }
            SchedulerType::PredeterminedPriority => {
                let ga_scheduler_sender = Self::start_scheduler::<PriorityScheduler, PriorityGenotype, PriorityMapPhenotype>(scheduler_data);
                threads.push(thread::spawn(|| run_predetermined_priorities(ga_scheduler_sender, scheduler_ga_receiver, 100)));
            }
            SchedulerType::DelayLocalityExperiment => {
                let mutex_node_states_clone_2= scheduler_data.mutex_node_states.clone();
                let ga_scheduler_sender = Self::start_scheduler::<DelayScheduler, DelayGenotype, DelayMapPhenotype>(scheduler_data);
                threads.push(thread::spawn(|| run_locality_experiment_delays(ga_scheduler_sender, scheduler_ga_receiver, mutex_node_states_clone_2)));
            }
            SchedulerType::PriorityLocalityExperiment => {
                let mutex_node_states_clone_2 = scheduler_data.mutex_node_states.clone();
                let ga_scheduler_sender = Self::start_scheduler::<PriorityScheduler, PriorityGenotype, PriorityMapPhenotype>(scheduler_data);
                threads.push(thread::spawn(|| run_locality_experiment_priorities(ga_scheduler_sender, scheduler_ga_receiver, mutex_node_states_clone_2)));
            }
            SchedulerType::ScalingExperiment => {
                let mutex_node_states_clone_2 = scheduler_data.mutex_node_states.clone();
                let ga_scheduler_sender = Self::start_scheduler::<PriorityScheduler, PriorityGenotype, PriorityMapPhenotype>(scheduler_data);
                threads.push(thread::spawn(|| run_priority_scaling_experiment(ga_scheduler_sender, scheduler_ga_receiver, mutex_node_states_clone_2)));
            }
            SchedulerType::None => {
                let ga_scheduler_sender = Self::start_scheduler::<DelayScheduler, DelayGenotype, DelayMapPhenotype>(scheduler_data);
                threads.push(thread::spawn(|| run_no_delays(ga_scheduler_sender, scheduler_ga_receiver, 20)));
            }
        }

        // For every combination (exclusive) of peers, create connections between the peers and scheduler
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

        for tokio_task in tokio_tasks {
            match tokio_task.await {
                Ok(_) => error!("A tokio task finished with ok"),
                Err(err) => error!("A tokio task finished with an error: {:?}", err)
            }
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

    /// Start the mu lambda GA and delay scheduler
    #[allow(unused)]
    fn start_default_mu_lambda_delays(
        mu: usize,
        lambda: usize,
        scheduler_data: SchedulerData,
        scheduler_ga_receiver: Receiver<CurrentFitness>,
    )
    {
        // Start the scheduler
        let ga_scheduler_sender = Self::start_scheduler::<DelayScheduler, DelayGenotype, DelayMapPhenotype>(scheduler_data);
        // Start the GA
        thread::spawn(move || genetic_algorithm::run_default_mu_lambda_delays(mu, lambda, ga_scheduler_sender, scheduler_ga_receiver));
    }

    /// Start the mu lambda GA and priority scheduler
    #[allow(unused)]
    fn start_default_mu_lambda_priorities(
        mu: usize,
        lambda: usize,
        scheduler_data: SchedulerData,
        scheduler_ga_receiver: Receiver<CurrentFitness>,
    )
    {
        // Start the scheduler
        let ga_scheduler_sender = Self::start_scheduler::<PriorityScheduler, PriorityGenotype, PriorityMapPhenotype>(scheduler_data);
        // Start the GA
        thread::spawn(move || genetic_algorithm::run_default_mu_lambda_priorities(mu, lambda, ga_scheduler_sender, scheduler_ga_receiver));
    }

    fn start_scheduler<S: Scheduler<IndividualPhenotype = P> + Send + 'static, G: ExtendedGenotype, P: ExtendedPhenotype<G> + 'static>(
        scheduler_data: SchedulerData,
    ) -> Sender<P> {
        let (ga_scheduler_sender, ga_scheduler_receiver): (Sender<P>, Receiver<P>) = std::sync::mpsc::channel();
        let scheduler = S::new(scheduler_data.collector_tx, scheduler_data.mutex_node_states, scheduler_data.node_keys, scheduler_data.failure_sender);
        thread::spawn(move || scheduler.start(
            scheduler_data.scheduler_receiver,
            scheduler_data.scheduler_peer_channels,
            scheduler_data.scheduler_ga_sender,
            ga_scheduler_receiver,
            scheduler_data.client_senders,
            scheduler_data.test_harness_rx,
            scheduler_data.account_info_rx,
            scheduler_data.balance_receiver
        ));
        ga_scheduler_sender
    }
}

struct SchedulerData {
    scheduler_peer_channels: P2PConnections,
    collector_tx: Sender<Box<RippleMessage>>,
    mutex_node_states: Arc<MutexNodeStates>,
    node_keys: Vec<NodeKeys>,
    failure_sender: Sender<Vec<ConsensusPropertyTypes>>,
    scheduler_receiver: TokioReceiver<Event>,
    scheduler_ga_sender: Sender<CurrentFitness>,
    client_senders: Vec<Sender<Message<'static>>>,
    test_harness_rx: Receiver<(Transaction, String)>,
    account_info_rx: Receiver<AccountInfo>,
    balance_receiver: Receiver<u32>,
}

impl SchedulerData {
    pub fn new(scheduler_peer_channels: P2PConnections,
               collector_tx: Sender<Box<RippleMessage>>,
               mutex_node_states: Arc<MutexNodeStates>,
               node_keys: Vec<NodeKeys>,
               failure_sender: Sender<Vec<ConsensusPropertyTypes>>,
               scheduler_receiver: TokioReceiver<Event>,
               scheduler_ga_sender: Sender<CurrentFitness>,
               client_senders: Vec<Sender<Message<'static>>>,
               test_harness_rx: Receiver<(Transaction, String)>,
               account_info_rx: Receiver<AccountInfo>,
               balance_receiver: Receiver<u32>) -> Self {
        Self {
            scheduler_peer_channels,
            collector_tx,
            mutex_node_states,
            node_keys,
            failure_sender,
            scheduler_receiver,
            scheduler_ga_sender,
            client_senders,
            test_harness_rx,
            account_info_rx,
            balance_receiver
        }
    }
}

#[allow(unused)]
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub enum SchedulerType {
    Priority,
    Delay,
    RandomPriority,
    RandomDelay,
    DelayTraceGraph,
    PriorityTraceGraph,
    FitnessComparison,
    PredeterminedDelay,
    PredeterminedPriority,
    DelayLocalityExperiment,
    PriorityLocalityExperiment,
    ScalingExperiment,
    None,
}
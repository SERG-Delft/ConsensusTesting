use log::{debug, trace, error};
use std::collections::HashMap;
use std::sync::Arc;
use chrono::Utc;
use tokio::sync::mpsc::{Sender as TokioSender, Receiver as TokioReceiver};
use std::sync::mpsc::{Sender as STDSender, Receiver as STDReceiver, channel};
use parking_lot::{Mutex, Condvar};
use std::thread;
use std::time::Duration;
use byteorder::{BigEndian, ByteOrder};
use websocket::Message;
use crate::client::{SubscriptionObject};
use crate::collector::RippleMessage;
use crate::ga::genetic_algorithm::{CurrentFitness, DelayMapPhenotype, MessageType};
use crate::message_handler::{parse_protocol_message, RippleMessageObject};
use crate::node_state::{MutexNodeStates};
use crate::test_harness::TestHarness;

type P2PConnections = HashMap<usize, HashMap<usize, PeerChannel>>;

/// Scheduler module responsible for scheduling execution of events (message receivals in peers)
/// p2p_connections: Contains the senders for sending from a peer to another peer
/// collector_sender: Sender for sending the executed events to the collector (execution.txt)
/// latest_validated_ledger: The latest validated ledger
/// current_round: The latest round for which a message is sent by one of the peers
pub struct Scheduler {
    p2p_connections: P2PConnections,
    collector_sender: STDSender<Box<RippleMessage>>,
    run: Arc<(Mutex<bool>, Condvar)>,
    latest_validated_ledger: Arc<(Mutex<u32>, Condvar)>,
    current_round: Arc<(Mutex<u32>, Condvar)>,
    current_delays: Arc<Mutex<DelayMapPhenotype>>,
    node_states: Arc<MutexNodeStates>,
}

impl Scheduler {
    pub fn new(p2p_connections: P2PConnections, collector_sender: STDSender<Box<RippleMessage>>, node_states: Arc<MutexNodeStates>) -> Self {
        Scheduler {
            p2p_connections,
            collector_sender,
            run: Arc::new((Mutex::new(false), Condvar::new())),
            latest_validated_ledger: Arc::new((Mutex::new(0), Condvar::new())),
            current_round: Arc::new((Mutex::new(0), Condvar::new())),
            current_delays: Arc::new(Mutex::new(DelayMapPhenotype::default())),
            node_states,
        }
    }

    /// Starts peer and collector listening threads and listens to the scheduler for executing messages after delay
    pub fn start(self,
                 receiver: TokioReceiver<Event>,
                 _collector_receiver: STDReceiver<SubscriptionObject>,
                 ga_sender: STDSender<CurrentFitness>,
                 ga_receiver: STDReceiver<DelayMapPhenotype>,
                 client_senders: Vec<STDSender<Message<'static>>>
    )
    {
        let latest_validated_ledger_clone = self.latest_validated_ledger.clone();
        let latest_validated_ledger_clone_2 = self.latest_validated_ledger.clone();
        let (event_schedule_sender, event_schedule_receiver) = channel();
        let current_round_clone = self.current_round.clone();
        let current_round_clone_2 = self.current_round.clone();
        let current_delays_clone = self.current_delays.clone();
        let current_delays_clone_2 = self.current_delays.clone();
        let run_clone = self.run.clone();
        let run_clone_2 = self.run.clone();
        let node_states_clone = self.node_states.clone();
        let node_states_clone_2 = self.node_states.clone();
        let node_states_clone_3 = self.node_states.clone();
        // thread::spawn(move || Self::listen_to_collector(collector_receiver, latest_validated_ledger_clone));
        thread::spawn(move || Self::listen_to_peers(run_clone_2, current_delays_clone, receiver, event_schedule_sender));
        thread::spawn(move || Self::listen_to_ga(current_delays_clone_2, ga_receiver));
        thread::spawn(move || Self::update_current_round(node_states_clone, current_round_clone));
        thread::spawn(move || Self::update_latest_validated_ledger(node_states_clone_3, latest_validated_ledger_clone));
        thread::spawn(move || Self::harness_controller(ga_sender, client_senders, latest_validated_ledger_clone_2, current_round_clone_2, run_clone, node_states_clone_2));
        loop {
            match event_schedule_receiver.recv() {
                Ok(event) => self.execute_event(event),
                Err(_) => panic!("Scheduler sender failed")
            }
        }
    }

    /// Execute event and report to collector (execution.txt)
    fn execute_event(&self, event: Event) {
        let rmo = parse_protocol_message(BigEndian::read_u16(&event.message[4..6]), &event.message[6..]);
        self.collector_sender.send(RippleMessage::new(format!("Ripple{}", event.from+1), format!("Ripple{}", event.to+1), Utc::now(), rmo)).expect("Collector receiver failed");
        self.p2p_connections.get(&event.to).unwrap().get(&event.from).unwrap().send(event.message);
    }

    /// Listen to messages sent by peers
    /// If the network is not stable, immediately relay messages
    /// Else schedule messages with a certain delay
    fn listen_to_peers(run: Arc<(Mutex<bool>, Condvar)>, current_delays: Arc<Mutex<DelayMapPhenotype>>, mut receiver: TokioReceiver<Event>, event_schedule_sender: STDSender<Event>) {
        let (run_lock, _run_cvar) = &*run;
        loop {
            match receiver.blocking_recv() {
                Some(event) => {
                    let rmo = parse_protocol_message(BigEndian::read_u16(&event.message[4..6]), &event.message[6..]);
                    let ms: u32;
                    // If the network is ready to apply the test case, determine delay of message, else delay = 0
                    if *run_lock.lock() {
                        let message_type_map = current_delays.lock().delay_map.get(&event.from).unwrap().get(&event.to).unwrap().clone();
                        ms = match rmo {
                            RippleMessageObject::TMTransaction(_) => message_type_map.get(&MessageType::TMTransaction).unwrap().clone(),
                            RippleMessageObject::TMProposeSet(_) => message_type_map.get(&MessageType::TMProposeSet).unwrap().clone(),
                            RippleMessageObject::TMStatusChange(_) => message_type_map.get(&MessageType::TMStatusChange).unwrap().clone(),
                            RippleMessageObject::TMHaveTransactionSet(_) => message_type_map.get(&MessageType::TMHaveTransactionSet).unwrap().clone(),
                            _ => 0
                        };
                    } else {
                        ms = 0;
                    }
                    let duration = Duration::from_millis(ms as u64);
                    ScheduledEvent::schedule_execution(
                        event,
                        duration,
                        event_schedule_sender.clone()
                    )
                },
                None => error!("Peer senders failed")
            }
        }
    }

    /// Listen to messages from the collector
    /// Responsible for determining stability and latest validated ledger of the network
    // fn listen_to_collector(
    //     collector_receiver: STDReceiver<SubscriptionObject>,
    //     latest_validated_ledger: Arc<(Mutex<u32>, Condvar)>,
    // )
    // {
    //     let mut local_latest_validated_ledger = 0;
    //     let (ledger_lock, ledger_cvar) = &*latest_validated_ledger;
    //     loop {
    //         match collector_receiver.recv() {
    //             Ok(subscription_object) => {
    //                 match subscription_object {
    //                     SubscriptionObject::ValidatedLedger(ledger) => {
    //                         if local_latest_validated_ledger < ledger.ledger_index {
    //                             *ledger_lock.lock() = ledger.ledger_index;
    //                             ledger_cvar.notify_all();
    //                             local_latest_validated_ledger = ledger.ledger_index;
    //                         }
    //                     }
    //                     _ => {}
    //                 }
    //             }
    //             Err(_) => {}
    //         }
    //     }
    // }

    /// Listen to the genetic algorithm for new individuals to test
    fn listen_to_ga(current_delays: Arc<Mutex<DelayMapPhenotype>>, ga_receiver: STDReceiver<DelayMapPhenotype>) {
        loop {
            match ga_receiver.recv() {
                Ok(new_delays) => {
                    *current_delays.lock() = new_delays;
                    debug!("New delays received");
                },
                Err(_) => {}
            }
        }
    }

    /// Update the current round if a message is received with a higher ledger sequence number
    fn update_current_round(node_states: Arc<MutexNodeStates>, current_round: Arc<(Mutex<u32>, Condvar)>) {
        loop {
            let mut node_states_mutex = node_states.node_states.lock();
            node_states.round_cvar.wait(&mut node_states_mutex);
            let round = node_states_mutex.max_current_round();
            let (ref lock, ref cvar) = &*current_round;
            let mut locked_round = lock.lock();
            if round > *locked_round {
                println!("Updating round to {}", round);
                *locked_round = round;
                cvar.notify_all();
            }
        }
    }

    /// Update the latest validated ledger if all nodes have validated a next ledger
    fn update_latest_validated_ledger(node_states: Arc<MutexNodeStates>, latest_validated_ledger: Arc<(Mutex<u32>, Condvar)>) {
        loop {
            let mut node_states_mutex = node_states.node_states.lock();
            node_states.validated_ledger_cvar.wait(&mut node_states_mutex);
            let validated_ledger_index = node_states_mutex.min_validated_ledger();
            let (ref lock, ref cvar) = &*latest_validated_ledger;
            let mut locked_ledger_index = lock.lock();
            if validated_ledger_index > *locked_ledger_index {
                println!("Updating latest validated ledger to {}", validated_ledger_index);
                *locked_ledger_index = validated_ledger_index;
                cvar.notify_all();
            }
        }
    }

    /// Responsible for
    /// 1. Checking/updating stability of network (through validated ledger after harness)
    /// 2. Checking progress of harness
    /// 3. Relaying fitness of chromosome over harness
    fn harness_controller(
        ga_sender: STDSender<CurrentFitness>,
        client_senders: Vec<STDSender<Message<'static>>>,
        latest_validated_ledger: Arc<(Mutex<u32>, Condvar)>,
        current_round: Arc<(Mutex<u32>, Condvar)>,
        run: Arc<(Mutex<bool>, Condvar)>,
        node_states: Arc<MutexNodeStates>,
    )
    {
        let (ledger_lock, ledger_cvar) = &*latest_validated_ledger;
        let (round_lock, round_cvar) = &*current_round;
        let (run_lock, run_cvar) = &*run;
        let mut execution_sequence = 0;
        // Every loop is one execution of the test harness
        loop {
            let test_harness = TestHarness::parse_test_harness(client_senders.clone(), execution_sequence);
            let mut ledger_number = ledger_lock.lock();
            let first_validated_ledger = *ledger_number;
            println!("Waiting for network stabilization");
            ledger_cvar.wait(&mut ledger_number);
            // If another ledger has been validated, continue
            if *ledger_number > first_validated_ledger {
                drop(ledger_number);
                let mut round_number = round_lock.lock();
                let first_round = *round_number;
                println!("Waiting on round update: {}", first_round);
                round_cvar.wait(&mut round_number);
                println!("Round update received: {}", *round_number);
                // Start test as a node starts a new round
                if *round_number > first_round {
                    node_states.clear_transactions();
                    *run_lock.lock() = true;
                    drop(round_number);
                    println!("Starting test harness run");
                    run_cvar.notify_all();
                    let fitness = CurrentFitness::run_harness(test_harness, node_states.clone());
                    // Send duration of test case to GA
                    ga_sender.send(fitness).expect("GA receiver failed");
                    *run_lock.lock() = false;
                    run_cvar.notify_all();
                }
            }
            execution_sequence += 1;
        }
    }
}

/// Struct for sending from a peer to another peer
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

/// Event is a message event, where the 'message' is sent from peer 'from' and received by 'to'
pub struct Event {
    pub from: usize,
    pub to: usize,
    pub message: Vec<u8>
}

/// ScheduledEvent is a struct with functionality for scheduling the sending of a message after a certain duration
pub struct ScheduledEvent {}

impl ScheduledEvent {
    fn schedule_execution(event: Event, duration: Duration, sender: STDSender<Event>) {
        thread::spawn(move || {
            trace!("Sleeping for {} ms for message: {} -> {}: {:?}", duration.as_millis(), event.from, event.to, event.message);
            thread::sleep(duration);
            trace!("Sending event to executor: {} -> {}: {:?}", event.from, event.to, event.message);
            sender.send(event).expect("Scheduler receiver failed");
        });
    }
}

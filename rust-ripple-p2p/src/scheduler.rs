pub mod delay_scheduler;
pub mod priority_scheduler;

use std::cmp::Ordering;
use log::{error};
use std::collections::{HashMap};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use chrono::Utc;
use tokio::sync::mpsc::{Sender as TokioSender, Receiver as TokioReceiver};
use std::sync::mpsc::{Sender as STDSender, Receiver as STDReceiver, channel};
use std::thread;
use parking_lot::{Mutex, Condvar};
use byteorder::{BigEndian, ByteOrder};
use websocket::Message;
use crate::collector::RippleMessage;
use crate::ga::fitness::ExtendedFitness;
use crate::ga::genetic_algorithm::{CurrentFitness, ConsensusMessageType};
use crate::message_handler::{parse_protocol_message, RippleMessageObject, rmo_to_bytes};
use crate::node_state::{MutexNodeStates};
use crate::test_harness::TestHarness;

type P2PConnections = HashMap<usize, HashMap<usize, PeerChannel>>;

pub trait Scheduler: Sized {
    type IndividualPhenotype: Default + Send + 'static;

    fn start(self, receiver: TokioReceiver<Event>,
             ga_sender: STDSender<CurrentFitness>,
             ga_receiver: STDReceiver<Self::IndividualPhenotype>,
             client_senders: Vec<STDSender<Message<'static>>>
    )
    {
        let latest_validated_ledger_clone = self.get_state().latest_validated_ledger.clone();
        let latest_validated_ledger_clone_2 = self.get_state().latest_validated_ledger.clone();
        let current_round_clone = self.get_state().current_round.clone();
        let current_round_clone_2 = self.get_state().current_round.clone();
        let run_clone = self.get_state().run.clone();
        let node_states_clone = self.get_state().node_states.clone();
        let node_states_clone_2 = self.get_state().node_states.clone();
        let node_states_clone_3 = self.get_state().node_states.clone();

        thread::spawn(move || Self::update_current_round(node_states_clone, current_round_clone));
        thread::spawn(move || Self::update_latest_validated_ledger(node_states_clone_3, latest_validated_ledger_clone));
        thread::spawn(move || Self::harness_controller(ga_sender, client_senders, latest_validated_ledger_clone_2, current_round_clone_2, run_clone, node_states_clone_2));

        // self.start_extension(receiver, ga_receiver);
        let (event_schedule_sender, event_schedule_receiver) = channel();
        let run_clone = self.get_state().run.clone();
        let node_states_clone = self.get_state().node_states.clone();
        let node_states_clone_2 = self.get_state().node_states.clone();
        let current_priorities = Arc::new(Mutex::new(Self::IndividualPhenotype::default()));
        let current_priorities_2 = current_priorities.clone();
        thread::spawn(move || Self::schedule_controller(receiver, run_clone, current_priorities, node_states_clone, event_schedule_sender));
        thread::spawn(move || Self::listen_to_ga(current_priorities_2, ga_receiver, node_states_clone_2));
        loop {
            match event_schedule_receiver.recv() {
                Ok(event) => self.execute_event(event),
                Err(_) => panic!("Scheduler sender failed")
            }
        }
    }

    fn schedule_controller(receiver: TokioReceiver<Event>,
                           run: Arc<(Mutex<bool>, Condvar)>,
                           current_individual: Arc<Mutex<Self::IndividualPhenotype>>,
                           node_states: Arc<MutexNodeStates>,
                           event_schedule_sender: STDSender<RMOEvent>
    );

    fn listen_to_ga(current_individual: Arc<Mutex<Self::IndividualPhenotype>>, ga_receiver: STDReceiver<Self::IndividualPhenotype>, node_states: Arc<MutexNodeStates>);

    fn get_state(&self) -> &SchedulerState;

    /// Execute event and report to collector
    fn execute_event(&self, event: RMOEvent) {
        let collector_message = RippleMessage::new(format!("Ripple{}", event.from+1), format!("Ripple{}", event.to+1), Utc::now(), event.message.clone());
        self.get_state().collector_sender.send(collector_message.clone()).expect("Collector receiver failed");
        let (ref run_lock, ref _run_cvar) = &*self.get_state().run;
        if *run_lock.lock() {
            match event.message {
                RippleMessageObject::TMTransaction(_) | RippleMessageObject::TMProposeSet(_) | RippleMessageObject::TMStatusChange(_) | RippleMessageObject::TMHaveTransactionSet(_) => self.get_state().node_states.add_execution(collector_message.as_ref().clone()),
                _ => {}
            }
        }
        self.get_state().p2p_connections.get(&event.to).unwrap().get(&event.from).unwrap().send(event);
    }

    fn is_consensus_rmo(rmo: &RippleMessageObject) -> bool {
        ConsensusMessageType::RMO_MESSAGE_TYPE.contains(&rmo.message_type())
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
            println!("Validated ledgers: {:?}, fork: {}, liveness: {}", node_states_mutex.validated_ledgers(), node_states_mutex.check_for_fork(), node_states_mutex.check_liveness());
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
                    drop(round_number);
                    *run_lock.lock() = true;
                    println!("Starting test harness run");
                    run_cvar.notify_all();
                    let fitness = CurrentFitness::run_harness(test_harness, node_states.clone());
                    // Send fitness of test case to GA
                    ga_sender.send(fitness).expect("GA receiver failed");
                    *run_lock.lock() = false;
                    run_cvar.notify_all();
                }
            }
            execution_sequence += 1;
        }
    }
}

pub struct SchedulerState {
    pub p2p_connections: P2PConnections,
    pub collector_sender: STDSender<Box<RippleMessage>>,
    pub run: Arc<(Mutex<bool>, Condvar)>,
    pub latest_validated_ledger: Arc<(Mutex<u32>, Condvar)>,
    pub current_round: Arc<(Mutex<u32>, Condvar)>,
    pub node_states: Arc<MutexNodeStates>,
}

impl SchedulerState {
    pub fn new(p2p_connections: P2PConnections, collector_sender: STDSender<Box<RippleMessage>>, node_states: Arc<MutexNodeStates>) -> Self {
        SchedulerState {
            p2p_connections,
            collector_sender,
            run: Arc::new((Mutex::new(false), Condvar::new())),
            latest_validated_ledger: Arc::new((Mutex::new(0), Condvar::new())),
            current_round: Arc::new((Mutex::new(0), Condvar::new())),
            node_states,
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

    pub fn send(&self, message: RMOEvent) {
        match self.sender.blocking_send(Event::from(message).message) {
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

impl Event {
    pub fn from(event: RMOEvent) -> Self {
        Self {
            from: event.from,
            to: event.to,
            message: rmo_to_bytes(event.message)
        }
    }
}

#[derive(Default, Clone, PartialEq, Debug)]
pub struct RMOEvent {
    pub from: usize,
    pub to: usize,
    pub message: RippleMessageObject,
}

impl RMOEvent {
    pub fn from(event: &Event) -> Self {
        Self {
            from: event.from,
            to: event.to,
            message: parse_protocol_message(BigEndian::read_u16(&event.message[4..6]), &event.message[6..]),
        }
    }
}

impl PartialOrd<Self> for RMOEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let mut self_hasher = DefaultHasher::new();
        self.message.hash(&mut self_hasher);
        let mut other_hasher = DefaultHasher::new();
        other.message.hash(&mut other_hasher);
        Some(self_hasher.finish().cmp(&other_hasher.finish()))
    }
}

impl Eq for RMOEvent {}

impl Ord for RMOEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        let mut self_hasher = DefaultHasher::new();
        self.message.hash(&mut self_hasher);
        let mut other_hasher = DefaultHasher::new();
        other.message.hash(&mut other_hasher);
        self_hasher.finish().cmp(&other_hasher.finish())
    }
}

#[cfg(test)]
mod scheduler_tests {
    use std::thread;
    use std::time::Duration;
    use crate::ga::genetic_algorithm::DROP_THRESHOLD;
    use crate::message_handler::RippleMessageObject;
    use crate::protos::ripple::{TMTransaction as PBTransaction, TransactionStatus};
    use crate::scheduler::{Event, RMOEvent};
    use crate::scheduler::delay_scheduler::ScheduledEvent;

    #[test]
    fn test_event_transformation() {
        let mut transaction = PBTransaction::new();
        transaction.set_rawTransaction(vec![]);
        transaction.set_status(TransactionStatus::tsCOMMITED);
        let rmo_event = RMOEvent { from: 0, to: 1, message: RippleMessageObject::TMTransaction(transaction) };
        let event = Event::from(rmo_event.clone());
        let transformed_event = RMOEvent::from(&event);
        assert_eq!(rmo_event.message, transformed_event.message);
    }

    #[test]
    fn test_drop_threshold() {
        let rmo_event = RMOEvent { from: 0, to: 1, message: RippleMessageObject::TMTransaction(PBTransaction::new()) };
        let (sender, receiver) = std::sync::mpsc::channel();
        ScheduledEvent::schedule_execution(rmo_event, Duration::from_millis(DROP_THRESHOLD as u64 + 1), sender.clone());
        thread::sleep(Duration::from_millis(DROP_THRESHOLD as u64 + 500));
        let result = receiver.try_recv();
        assert!(result.is_err());
        let rmo_event = RMOEvent { from: 0, to: 1, message: RippleMessageObject::TMTransaction(PBTransaction::default()) };
        ScheduledEvent::schedule_execution(rmo_event, Duration::from_millis(100), sender);
        thread::sleep(Duration::from_millis(1000));
        let result = receiver.try_recv();
        assert!(result.is_ok());
    }
}

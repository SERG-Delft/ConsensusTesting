pub mod delay_scheduler;
pub mod priority_scheduler;

use std::cmp::Ordering;
use log::{debug, error, trace};
use std::collections::{HashMap};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, RwLock};
use chrono::{DateTime, MAX_DATETIME, Utc};
use tokio::sync::mpsc::{Sender as TokioSender, Receiver as TokioReceiver};
use std::sync::mpsc::{Sender as STDSender, Receiver as STDReceiver, channel, Sender};
use std::thread;
use std::time::Duration;
use parking_lot::{Mutex, Condvar};
use byteorder::{BigEndian, ByteOrder};
use websocket::Message;
use crate::client::{AccountInfo, Transaction};
use crate::collector::RippleMessage;
use crate::consensus_properties::ConsensusProperties;
use crate::failure_writer::ConsensusPropertyTypes;
use crate::ga::fitness::ExtendedFitness;
use crate::ga::genetic_algorithm::{ConsensusMessageType};
use crate::message_handler::{parse_protocol_message, ParsedValidation, RippleMessageObject, rmo_to_bytes};
use crate::node_state::{MutexNodeStates};
use crate::NodeKeys;
use crate::test_harness::TestHarness;

pub type P2PConnections = HashMap<usize, HashMap<usize, PeerChannel>>;

pub trait Scheduler: Sized {
    type IndividualPhenotype: Default + Send + 'static;

    fn new(
        collector_sender: STDSender<Box<RippleMessage>>,
        node_states: Arc<MutexNodeStates>,
        node_keys: Vec<NodeKeys>,
        failure_sender: STDSender<Vec<ConsensusPropertyTypes>>,
    ) -> Self;

    fn start<F: ExtendedFitness>(self,
             receiver: TokioReceiver<Event>,
             p2p_connections: P2PConnections,
             ga_sender: STDSender<F>,
             ga_receiver: STDReceiver<Self::IndividualPhenotype>,
             client_senders: Vec<STDSender<Message<'static>>>,
             client_receiver: STDReceiver<(Transaction, String)>,
             account_receiver: STDReceiver<AccountInfo>,
             balance_receiver: STDReceiver<u32>,
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
        let failure_sender_clone = self.get_state().failure_sender.clone();
        let failure_sender_clone_2 = self.get_state().failure_sender.clone();
        let (round_update_sender, round_update_receiver) = std::sync::mpsc::channel();
        let (ripple_message_sender, ripple_message_receiver) = std::sync::mpsc::channel();
        let (consensus_property_checker_sender, consensus_property_checker_receiver) = std::sync::mpsc::channel();

        thread::spawn(move || Self::update_current_round(node_states_clone, current_round_clone));
        thread::spawn(move || Self::update_latest_validated_ledger(node_states_clone_3, latest_validated_ledger_clone, failure_sender_clone));
        thread::spawn(move || Self::harness_controller(ga_sender, client_senders, failure_sender_clone_2, client_receiver, account_receiver, balance_receiver,latest_validated_ledger_clone_2, current_round_clone_2, run_clone, node_states_clone_2));
        Self::check_message_for_round_update(round_update_receiver, self.get_state().node_states.clone());
        Self::update_send_dependency(ripple_message_receiver, self.get_state().node_states.clone());

        let (event_schedule_sender, event_schedule_receiver) = channel();
        let run_clone = self.get_state().run.clone();
        let node_states_clone = self.get_state().node_states.clone();
        let current_individual = Arc::new(Mutex::new(Self::IndividualPhenotype::default()));
        let current_individual_2 = current_individual.clone();
        thread::spawn(move || Self::schedule_controller(receiver, run_clone, current_individual, round_update_sender, event_schedule_sender, ripple_message_sender));
        thread::spawn(move || Self::listen_to_ga(current_individual_2, ga_receiver, node_states_clone));
        Self::listen_to_scheduler(event_schedule_receiver, consensus_property_checker_sender, p2p_connections);
        self.consensus_property_checker(consensus_property_checker_receiver);
    }

    fn schedule_controller(
        receiver: TokioReceiver<Event>,
        run: Arc<(RwLock<bool>, Condvar)>,
        current_individual: Arc<Mutex<Self::IndividualPhenotype>>,
        round_update_sender: STDSender<RMOEvent>,
        event_schedule_sender: STDSender<RMOEvent>,
        send_dependency_sender: STDSender<RippleMessage>,
    );

    fn listen_to_ga(current_individual: Arc<Mutex<Self::IndividualPhenotype>>, ga_receiver: STDReceiver<Self::IndividualPhenotype>, node_states: Arc<MutexNodeStates>);

    fn get_state(&self) -> &SchedulerState;

    /// Execute event and report to collector
    fn listen_to_scheduler(event_schedule_receiver: STDReceiver<RMOEvent>, consensus_property_sender: Sender<Box<RippleMessage>>, p2p_connections: P2PConnections) {
        thread::spawn(move || {
            loop {
                match event_schedule_receiver.recv() {
                    Ok(event) => {
                        let collector_message = RippleMessage::new(format!("Ripple{}", event.from + 1), format!("Ripple{}", event.to + 1),
                                                                   Utc::now().signed_duration_since(event.time_in), Utc::now(), event.message.clone());
                        consensus_property_sender.send(collector_message).expect("Consensus property sender failed");
                        p2p_connections.get(&event.to).unwrap().get(&event.from).unwrap().send(event);
                    },
                    Err(_) => panic!("Scheduler sender failed")
                }
            }
        });
    }

    fn consensus_property_checker(self, receiver: STDReceiver<Box<RippleMessage>>) {
        let (ref run_lock, ref _run_cvar) = &*self.get_state().run;
        loop {
            let collector_message = receiver.recv().expect("consensus property receiver failed");
            self.get_state().collector_sender.send(collector_message.clone()).expect("collector sender failed");
            if *run_lock.read().unwrap() {
                if Self::is_consensus_rmo(&collector_message.message) {
                    self.get_state().node_states.add_execution(collector_message.as_ref().clone());
                    if Self::is_own_message(&collector_message.message, &self.get_state().node_keys[collector_message.sender_index()].validation_public_key) {
                        let mut consensus_property_violations = vec![];
                        match &collector_message.message {
                            RippleMessageObject::TMStatusChange(status_change) => {
                                consensus_property_violations.append(
                                    &mut ConsensusProperties::check_proposal_integrity_property(
                                        &self.get_state().node_states,
                                        &status_change,
                                        collector_message.sender_index()
                                    ));
                            }
                            RippleMessageObject::TMValidation(validation) => {
                                let parsed_validation = ParsedValidation::new(validation);
                                consensus_property_violations.append(
                                    &mut ConsensusProperties::check_validation_integrity_property(
                                        &self.get_state().node_states,
                                        parsed_validation,
                                        collector_message.sender_index()
                                    ));
                            }
                            RippleMessageObject::TMProposeSet(proposal) => {
                                self.get_state().node_states.node_states.lock().add_proposed_tx_set(proposal.get_currentTxHash(), collector_message.sender_index());
                            }
                            _ => {}
                        }
                        if !consensus_property_violations.is_empty() {
                            match self.get_state().failure_sender.send(consensus_property_violations) {
                                Ok(_) => {}
                                Err(_) => error!("Failure channel failed")
                            };
                        }
                    }
                }
            }
        }
    }

    fn is_consensus_rmo(rmo: &RippleMessageObject) -> bool {
        ConsensusMessageType::RMO_MESSAGE_TYPE.contains(&rmo.message_type())
    }

    fn is_own_message(rmo: &RippleMessageObject, sender_pub_key: &str) -> bool {
        match rmo.node_pub_key() {
            Some(message_pub_key) => {
                sender_pub_key == &message_pub_key
            }
            None => true
        }
    }

    /// Update round number based on ledgerAccept message.
    /// The node has accepted the new ledger and is building/validating that ledger
    /// We consider the node to have moved on to the next round
    fn check_message_for_round_update(message_listener: STDReceiver<RMOEvent>, node_states: Arc<MutexNodeStates>) {
        thread::spawn(move || {
            loop {
                let rmo_event = message_listener.recv().expect("round_update_receiver failed");
                match rmo_event.message {
                    crate::message_handler::RippleMessageObject::TMStatusChange(ref status_change) => {
                        if status_change.has_newEvent() {
                            match status_change.get_newEvent() {
                                crate::protos::ripple::NodeEvent::neACCEPTED_LEDGER => {
                                    trace!("Setting node {}'s round to {}", rmo_event.from, status_change.get_ledgerSeq() + 1);
                                    node_states.set_current_round(rmo_event.from, status_change.get_ledgerSeq() + 1);
                                }
                                _ => {}
                            }
                        }
                    },
                    _ => {}
                }
            }
        });
    }

    fn update_send_dependency(ripple_message_receiver: STDReceiver<RippleMessage>, node_states: Arc<MutexNodeStates>) {
        thread::spawn(move || {
            loop {
                let ripple_message = ripple_message_receiver.recv().expect("ripple_message_receiver failed");
                node_states.add_send_dependency(ripple_message);
            }
        });
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
    fn update_latest_validated_ledger(
        node_states: Arc<MutexNodeStates>,
        latest_validated_ledger: Arc<(Mutex<u32>, Condvar)>,
        failure_sender: STDSender<Vec<ConsensusPropertyTypes>>
    ) {
        let mut liveness = true;
        loop {
            let mut node_states_mutex = node_states.node_states.lock();
            // Liveness check!
            let now = Utc::now();
            node_states.validated_ledger_cvar.wait_for(&mut node_states_mutex, Duration::from_secs(65));
            if Utc::now() - chrono::Duration::seconds(65) >= now {
                error!("Bounded liveness bug");
                match failure_sender.send(vec![ConsensusPropertyTypes::Termination]) {
                    Ok(_) => {}
                    Err(err) => error!("Failure channel failed: {}", err)
                };
                liveness = false;
            } else if !liveness {
                liveness = true;
            }
            let validated_ledger_index = node_states_mutex.min_validated_ledger();
            let (ref lock, ref cvar) = &*latest_validated_ledger;
            let mut locked_ledger_index = lock.lock();
            if validated_ledger_index > *locked_ledger_index {
                println!("Updating latest validated ledger to {}", validated_ledger_index);
                *locked_ledger_index = validated_ledger_index;
                cvar.notify_all();
            }
            println!("Validated ledgers: {:?}, fork: {}, liveness: {}", node_states_mutex.validated_ledgers(), node_states_mutex.check_for_fork(), liveness);
        }
    }

    /// Responsible for
    /// 1. Checking/updating stability of network (through validated ledger after harness)
    /// 2. Checking progress of harness
    /// 3. Relaying fitness of chromosome over harness
    fn harness_controller<F: ExtendedFitness>(
        ga_sender: STDSender<F>,
        client_senders: Vec<STDSender<Message<'static>>>,
        failure_sender: STDSender<Vec<ConsensusPropertyTypes>>,
        client_receiver: STDReceiver<(Transaction, String)>,
        account_receiver: STDReceiver<AccountInfo>,
        balance_receiver: STDReceiver<u32>,
        latest_validated_ledger: Arc<(Mutex<u32>, Condvar)>,
        current_round: Arc<(Mutex<u32>, Condvar)>,
        run: Arc<(RwLock<bool>, Condvar)>,
        node_states: Arc<MutexNodeStates>,
    )
    {
        let (ledger_lock, ledger_cvar) = &*latest_validated_ledger;
        let (round_lock, round_cvar) = &*current_round;
        let (run_lock, run_cvar) = &*run;
        let mut test_harness = TestHarness::parse_test_harness(client_senders.clone(), client_receiver, balance_receiver, failure_sender, None);
        node_states.set_harness_transactions(test_harness.transactions.clone());
        Self::stabilize_network(&mut test_harness, node_states.clone(), latest_validated_ledger.clone(), account_receiver);
        // Every loop is one execution of the test harness
        loop {
            let mut ledger_number = ledger_lock.lock();
            println!("Waiting for network stabilization");
            ledger_cvar.wait(&mut ledger_number);
            // If another ledger has been validated and the ledgers have caught up, continue
            if *ledger_number > *round_lock.lock() - 2 {
                drop(ledger_number);
                let mut round_number = round_lock.lock();
                let first_round = *round_number;
                println!("Waiting on round update: {}", first_round);
                round_cvar.wait(&mut round_number);
                println!("Round update received: {}", *round_number);
                // Start test as a node starts a new round
                if *round_number > first_round {
                    drop(round_number);
                    test_harness.setup_balances(&node_states);
                    {
                        *run_lock.write().unwrap() = true;
                    }
                    println!("Starting test harness run");
                    run_cvar.notify_all();
                    let fitness = F::run_harness(&mut test_harness, node_states.clone());
                    // Send fitness of test case to GA
                    ga_sender.send(fitness).expect("GA receiver failed");
                    {
                        *run_lock.write().unwrap() = false;
                    }
                    run_cvar.notify_all();
                }
            }
        }
    }

    fn stabilize_network(
        test_harness: &mut TestHarness<'static>,
        node_states: Arc<MutexNodeStates>,
        latest_validated_ledger: Arc<(Mutex<u32>, Condvar)>,
        account_receiver: STDReceiver<AccountInfo>,
    )
    {
        let (ledger_lock, ledger_cvar) = &*latest_validated_ledger;
        let mut ledger_number = ledger_lock.lock();
        let validated_ledger = *ledger_number;
        debug!("Waiting for network to stabilize");
        ledger_cvar.wait(&mut ledger_number);
        if *ledger_number > validated_ledger {
            drop(ledger_number);
            debug!("Network stable, creating accounts");
            test_harness.create_accounts();
        }
        // Wait for transactions to be in validated ledger
        while node_states.get_min_validated_transactions().len() < test_harness.accounts.len()-1 {
            let mut ledger_number = ledger_lock.lock();
            debug!("Validated ledger increased to: {}", *ledger_number);
            ledger_cvar.wait(&mut ledger_number);
        }
        // Empty transaction queue
        while let Ok(_) = test_harness.client_receiver.try_recv() {}
        // Fetch account sequence numbers
        crate::client::Client::account_info("account_info", &test_harness.client_senders[0], test_harness.accounts[1].account_keys.account_id.clone());
        let account_seqs = match account_receiver.recv() {
            Ok(account_info) => account_info.account_data.sequence,
            Err(_) => {
                error!("Client hung up");
                0
            }
        };
        debug!("Accounts created in ledger: {}", account_seqs);
        for i in 1..test_harness.accounts.len() {
            test_harness.accounts[i].transaction_sequence = account_seqs;
        }
    }
}

pub struct SchedulerState {
    pub collector_sender: STDSender<Box<RippleMessage>>,
    pub run: Arc<(RwLock<bool>, Condvar)>,
    pub latest_validated_ledger: Arc<(Mutex<u32>, Condvar)>,
    pub current_round: Arc<(Mutex<u32>, Condvar)>,
    pub node_states: Arc<MutexNodeStates>,
    pub node_keys: Vec<NodeKeys>,
    pub failure_sender: Sender<Vec<ConsensusPropertyTypes>>,
}

impl SchedulerState {
    pub fn new(
        collector_sender: STDSender<Box<RippleMessage>>,
        node_states: Arc<MutexNodeStates>,
        node_keys: Vec<NodeKeys>,
        failure_sender: Sender<Vec<ConsensusPropertyTypes>>,
    ) -> Self {
        SchedulerState {
            collector_sender,
            run: Arc::new((RwLock::new(false), Condvar::new())),
            latest_validated_ledger: Arc::new((Mutex::new(0), Condvar::new())),
            current_round: Arc::new((Mutex::new(0), Condvar::new())),
            node_states,
            node_keys,
            failure_sender
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

#[derive(Clone, PartialEq, Debug, Hash)]
pub struct RMOEvent {
    pub from: usize,
    pub to: usize,
    pub message: RippleMessageObject,
    pub time_in: DateTime<Utc>,
}

impl RMOEvent {
    pub fn from(event: &Event) -> Self {
        Self {
            from: event.from,
            to: event.to,
            message: parse_protocol_message(BigEndian::read_u16(&event.message[4..6]), &event.message[6..]),
            time_in: Utc::now(),
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

impl Default for RMOEvent {
    fn default() -> Self {
        Self { from: 0, to: 0, message: RippleMessageObject::default(), time_in: MAX_DATETIME }
    }
}

#[cfg(test)]
mod scheduler_tests {
    use std::thread;
    use std::time::Duration;
    use chrono::{TimeZone, Utc};
    use crate::ga::encoding::delay_encoding::DROP_THRESHOLD;
    use crate::message_handler::RippleMessageObject;
    use crate::protos::ripple::{TMTransaction as PBTransaction, TransactionStatus};
    use crate::scheduler::{Event, RMOEvent};
    use crate::scheduler::delay_scheduler::ScheduledEvent;

    #[test]
    fn test_event_transformation() {
        let mut transaction = PBTransaction::new();
        transaction.set_rawTransaction(vec![]);
        transaction.set_status(TransactionStatus::tsCOMMITED);
        let rmo_event = RMOEvent { from: 0, to: 1, message: RippleMessageObject::TMTransaction(transaction), time_in: Utc.timestamp(1431648000, 0) };
        let event = Event::from(rmo_event.clone());
        let transformed_event = RMOEvent::from(&event);
        assert_eq!(rmo_event.message, transformed_event.message);
    }

    #[test]
    fn test_drop_threshold() {
        let rmo_event = RMOEvent { from: 0, to: 1, message: RippleMessageObject::TMTransaction(PBTransaction::new()), time_in: Utc.timestamp(1431648000, 0) };
        let (sender, receiver) = std::sync::mpsc::channel();
        ScheduledEvent::schedule_execution(rmo_event, DROP_THRESHOLD as u64 + 1, sender.clone());
        thread::sleep(Duration::from_millis(DROP_THRESHOLD as u64 + 500));
        let result = receiver.try_recv();
        assert!(result.is_err());
        let rmo_event = RMOEvent { from: 0, to: 1, message: RippleMessageObject::TMTransaction(PBTransaction::default()), time_in: Utc.timestamp(1431648000, 0) };
        ScheduledEvent::schedule_execution(rmo_event, 100, sender);
        thread::sleep(Duration::from_millis(1000));
        let result = receiver.try_recv();
        assert!(result.is_ok());
    }
}

use std::cmp::Ordering;
use std::sync::{Arc, RwLock};
use std::sync::mpsc::{Receiver as STDReceiver, Sender as STDSender};
use std::thread;
use chrono::{Duration, Utc};
use hashbrown::hash_map::DefaultHashBuilder;
use log::{debug, error, trace};
use parking_lot::{Condvar, Mutex};
use priority_queue::priority_queue::PriorityQueue;
use tokio::sync::mpsc::Receiver as TokioReceiver;
use spin_sleep::SpinSleeper;
use crate::collector::RippleMessage;
use crate::failure_writer::ConsensusPropertyTypes;
use crate::ga::encoding::{ExtendedPhenotype, num_genes};
use crate::ga::genetic_algorithm::ConsensusMessageType;
use crate::ga::encoding::priority_encoding::{PriorityMapPhenotype};
use crate::message_handler::RippleMessageObject;
use crate::node_state::MutexNodeStates;
use crate::NodeKeys;
use crate::scheduler::{Event, P2PConnections, RMOEvent, Scheduler, SchedulerState};

pub struct PriorityScheduler {
    state: SchedulerState,
}

impl PriorityScheduler {
    /// Execute events every t seconds based on size of the inbox.
    /// Do we have a target size of the inbox? ~30 (10% of the different types of events maybe?)
    /// How to determine base t? 1/(num_nodes * num_nodes-1) = 1 / 20? We assume a node broadcasts one message per second
    /// If inbox reaches 150% of desired capacity, increase rate (decrease t) by 10%? t / 1.1
    /// If inbox reaches 50% of desired capacity, decrease rate (increase t) by 10%? t * 1.1
    fn inbox_controller(
        inbox_rx: STDReceiver<OrderedRMOEvent>,
        run: Arc<(RwLock<bool>, Condvar)>,
        event_schedule_sender: STDSender<RMOEvent>,
    ) {
        let (run_lock, _run_cvar) = &*run;
        let sleeper = SpinSleeper::default();
        let mut inbox = PriorityQueue::<RMOEvent, usize, DefaultHashBuilder>::with_default_hasher();
        let mut rate = 0.5 * num_genes() as f64; // Rate at which events are executed from the queue. Base rate of num_genes / second -> too low?
        let target_inbox_size = 0.2 * num_genes() as f64; // Target inbox size of 10% of the events mapped -> higher?
        let sensitivity_ratio = 1.01; // Change rate by 3% at a time
        let rate_change_percentage = 0.5; // 50% less or more than desired size of inbox
        // let target_duration_in_inbox = Duration::seconds(6);
        loop {
            while let Ok(ordered_event) = inbox_rx.try_recv() {
                let priority = ordered_event.priority;
                inbox.push(ordered_event.rmo_event, priority);
            }
            if *run_lock.read().unwrap() {
                let inbox_size = inbox.len();
                // rate changes
                if inbox_size > (target_inbox_size + rate_change_percentage * target_inbox_size) as usize {
                        rate = (rate * sensitivity_ratio).min(num_genes() as f64 * 1.0f64);
                        trace!("size: {}, Increasing rate to {}", inbox_size, rate);
                } else if inbox_size < (target_inbox_size - rate_change_percentage * target_inbox_size) as usize {
                    trace!("size: {}, Decreasing rate to {}", inbox_size, rate);
                    rate = (rate / sensitivity_ratio).max(num_genes() as f64 / 6f64);
                }
                // Execute event with highest priority
                if inbox_size > 0 {
                    let rmo_event = inbox.pop().unwrap().0;
                    event_schedule_sender.send(rmo_event).expect("Event scheduler failed");
                }
            } else {
                while let Some((event, _)) = inbox.pop() {
                    trace!("Emptying inbox");
                    event_schedule_sender.send(event).expect("Event scheduler failed");
                }
            }
            // We sleep for 1 / rate seconds
            let duration_s = 1.0 / rate;
            sleeper.sleep_s(duration_s);
        }
    }
}

impl Scheduler for PriorityScheduler {
    type IndividualPhenotype = PriorityMapPhenotype;

    fn new(
        p2p_connections: P2PConnections,
        collector_sender: STDSender<Box<RippleMessage>>,
        node_states: Arc<MutexNodeStates>,
        node_keys: Vec<NodeKeys>,
        failure_sender: STDSender<Vec<ConsensusPropertyTypes>>,
    ) -> Self {
        Self {
            state: SchedulerState::new(p2p_connections, collector_sender, node_states, node_keys, failure_sender)
        }
    }

    /// Wait for new messages delivered by peers
    /// If the network is not stable, immediately relay messages
    /// Else collect messages in inbox and schedule based on priority
    fn schedule_controller(mut receiver: TokioReceiver<Event>,
                           run: Arc<(RwLock<bool>, Condvar)>,
                           current_individual: Arc<Mutex<Self::IndividualPhenotype>>,
                           node_states: Arc<MutexNodeStates>,
                           event_schedule_sender: STDSender<RMOEvent>
    )
    {
        let (run_lock, _run_cvar) = &*run;
        let (inbox_tx, inbox_rx) = std::sync::mpsc::channel();
        let event_schedule_sender_2 = event_schedule_sender.clone();
        let run_2 = run.clone();
        thread::spawn(move || Self::inbox_controller(inbox_rx, run_2, event_schedule_sender_2));
        loop {
            match receiver.blocking_recv() {
                Some(event) => {
                    let rmo_event = RMOEvent::from(&event);
                    Self::check_message_for_round_update(&rmo_event, &node_states);
                    // If the network is ready to apply the test case, collect messages in inbox, else immediately relay
                    if *run_lock.read().unwrap() {
                        if Self::is_consensus_rmo(&rmo_event.message) {
                            node_states.add_send_dependency(*RippleMessage::new(format!("Ripple{}", rmo_event.from + 1), format!("Ripple{}", rmo_event.to + 1), Duration::zero(), Utc::now(), rmo_event.message.clone()));
                            let message_type_map = current_individual.lock().priority_map.get(&rmo_event.from).unwrap().get(&rmo_event.to).unwrap().clone();

                            let priority = match &rmo_event.message {
                                RippleMessageObject::TMValidation(_) => message_type_map.get(&ConsensusMessageType::TMValidation).unwrap(),
                                RippleMessageObject::TMProposeSet(proposal) => {
                                    match proposal.get_proposeSeq() {
                                        0 => message_type_map.get(&ConsensusMessageType::TMProposeSet0).unwrap(),
                                        1 => message_type_map.get(&ConsensusMessageType::TMProposeSet1).unwrap(),
                                        2 => message_type_map.get(&ConsensusMessageType::TMProposeSet2).unwrap(),
                                        3 => message_type_map.get(&ConsensusMessageType::TMProposeSet3).unwrap(),
                                        4 => message_type_map.get(&ConsensusMessageType::TMProposeSet4).unwrap(),
                                        5 => message_type_map.get(&ConsensusMessageType::TMProposeSet5).unwrap(),
                                        4294967295 => message_type_map.get(&ConsensusMessageType::TMProposeSetBowOut).unwrap(),
                                        _ => message_type_map.get(&ConsensusMessageType::TMProposeSet0).unwrap(),
                                    }
                                },
                                RippleMessageObject::TMStatusChange(_) => message_type_map.get(&ConsensusMessageType::TMStatusChange).unwrap(),
                                RippleMessageObject::TMHaveTransactionSet(_) => message_type_map.get(&ConsensusMessageType::TMHaveTransactionSet).unwrap(),
                                RippleMessageObject::TMTransaction(_) => message_type_map.get(&ConsensusMessageType::TMTransaction).unwrap(),
                                RippleMessageObject::TMLedgerData(_) => message_type_map.get(&ConsensusMessageType::TMLedgerData).unwrap(),
                                RippleMessageObject::TMGetLedger(_) => message_type_map.get(&ConsensusMessageType::TMGetLedger).unwrap(),
                                _ => &0
                            };
                            inbox_tx.send(OrderedRMOEvent::new(rmo_event, *priority)).expect("Inbox sender failed");
                        } else {
                            event_schedule_sender.send(rmo_event).expect("Event scheduler failed");
                        }
                    } else {
                        event_schedule_sender.send(rmo_event).expect("Event scheduler failed");
                    }
                },
                None => error!("Peer senders failed")
            }
        }
    }

    fn listen_to_ga(current_individual: Arc<Mutex<Self::IndividualPhenotype>>, ga_receiver: STDReceiver<Self::IndividualPhenotype>, node_states: Arc<MutexNodeStates>) {
        loop {
            match ga_receiver.recv() {
                Ok(new_priority) => {
                    node_states.set_current_individual(current_individual.lock().display_genotype_by_message());
                    *current_individual.lock() = new_priority;
                    debug!("New priorities received");
                },
                Err(_) => {}
            }
        }
    }

    fn get_state(&self) -> &SchedulerState {
        &self.state
    }
}

#[derive(Debug, Clone, Hash)]
pub struct OrderedRMOEvent {
    rmo_event: RMOEvent,
    priority: usize,
}

impl OrderedRMOEvent {
    pub fn new(rmo_event: RMOEvent, priority: usize) -> Self {
        Self { rmo_event, priority }
    }
}

impl Eq for OrderedRMOEvent{}

impl PartialEq<Self> for OrderedRMOEvent {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.rmo_event == other.rmo_event
    }
}

impl PartialOrd<Self> for OrderedRMOEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.priority.partial_cmp(&other.priority)
    }
}

impl Ord for OrderedRMOEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.priority.partial_cmp(&other.priority) {
            None => self.rmo_event.cmp(&other.rmo_event),
            Some(cmp) => cmp ,
        }
    }
}

#[cfg(test)]
mod priority_scheduler_tests {
    use std::sync::{Arc, RwLock};
    use std::sync::mpsc::channel;
    use std::thread;
    use std::time::Duration;
    use parking_lot::{Condvar};
    use crate::message_handler::RippleMessageObject;
    use crate::protos::ripple::{TMStatusChange, TMValidation};
    use crate::scheduler::priority_scheduler::{OrderedRMOEvent, PriorityScheduler};
    use crate::scheduler::RMOEvent;

    #[test]
    fn test_inbox_controller() {
        let (inbox_tx, inbox_rx) = std::sync::mpsc::channel();
        let run = Arc::new((RwLock::new(false), Condvar::new()));
        let (event_schedule_sender, event_scheduler_receiver) = channel();
        thread::spawn(move || PriorityScheduler::inbox_controller(inbox_rx, run, event_schedule_sender,));
        thread::sleep(Duration::from_millis(100));
        let mut rmo_event_size = RMOEvent::default();
        rmo_event_size.message = RippleMessageObject::TMStatusChange(TMStatusChange::new());
        let mut rmo_event_time = RMOEvent::default();
        rmo_event_time.message = RippleMessageObject::TMValidation(TMValidation::new());
        inbox_tx.send(OrderedRMOEvent::new(rmo_event_size.clone(), 2)).unwrap();
        inbox_tx.send(OrderedRMOEvent::new(rmo_event_time.clone(), 1)).unwrap();
        thread::sleep(Duration::from_millis(100));
        let res = event_scheduler_receiver.recv();
        assert_eq!(res, Ok(rmo_event_size));
        let res = event_scheduler_receiver.recv();
        assert_eq!(res, Ok(rmo_event_time));
    }
}
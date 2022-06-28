use std::cmp::Ordering;
use std::collections::{BinaryHeap};
use std::sync::Arc;
use std::sync::mpsc::{Receiver as STDReceiver, Sender as STDSender};
use std::thread;
use chrono::Utc;
use log::{debug, error, trace};
use parking_lot::{Condvar, Mutex};
use tokio::sync::mpsc::Receiver as TokioReceiver;
use crate::collector::RippleMessage;
use crate::ga::encoding::{ExtendedPhenotype};
use crate::ga::genetic_algorithm::ConsensusMessageType;
use crate::ga::encoding::priority_encoding::{PriorityMapPhenotype};
use crate::message_handler::RippleMessageObject;
use crate::node_state::MutexNodeStates;
use crate::NodeKeys;
use crate::scheduler::{Event, P2PConnections, RMOEvent, Scheduler, SchedulerState};

pub struct PriorityScheduler {
    state: SchedulerState,
}

const MAX_INBOX_SIZE: usize = 100;
const MAX_DURATION_MILLIS: i64 = 30;

impl PriorityScheduler {
    #[allow(unused)]
    pub fn new(p2p_connections: P2PConnections, collector_sender: STDSender<Box<RippleMessage>>, node_states: Arc<MutexNodeStates>, node_keys: Vec<NodeKeys>) -> Self {
        PriorityScheduler {
            state: SchedulerState::new(p2p_connections, collector_sender, node_states, node_keys)
        }
    }

    fn inbox_controller(inbox: Arc<(Mutex<BinaryHeap<OrderedRMOEvent>>, Condvar)>, event_schedule_sender: STDSender<RMOEvent>, max_inbox_size: usize, max_duration_millis: i64) {
        let (inbox_lock, inbox_cvar) = &*inbox;
        let mut time = Utc::now();
        let mut size_counter = 0;
        let mut time_counter = 0;
        loop {
            let mut inbox_heap = inbox_lock.lock();
            // inbox_cvar.wait_for(&mut inbox_heap, std::time::Duration::from_millis(MAX_DURATION_MILLIS as u64));
            inbox_cvar.wait(&mut inbox_heap);
            {
                while inbox_heap.len() > max_inbox_size {
                    event_schedule_sender.send(inbox_heap.pop().unwrap().rmo_event).expect("Event scheduler failed");
                    size_counter += 1;
                    time = Utc::now();
                    if size_counter % 10 == 0 {
                        trace!("{}", size_counter);
                    }
                }
                if Utc::now().signed_duration_since(time) > chrono::Duration::milliseconds(max_duration_millis) {
                    if !inbox_heap.is_empty() {
                        event_schedule_sender.send(inbox_heap.pop().unwrap().rmo_event).expect("Event scheduler failed");
                    }
                    time_counter += 1;
                    if time_counter % 10 == 0 {
                        trace!("{}", time_counter);
                    }
                    time = Utc::now();
                }
            }
        }
    }
}

impl Scheduler for PriorityScheduler {
    type IndividualPhenotype = PriorityMapPhenotype;

    /// Wait for new messages delivered by peers
    /// If the network is not stable, immediately relay messages
    /// Else collect messages in inbox and schedule based on priority
    fn schedule_controller(mut receiver: TokioReceiver<Event>,
                           run: Arc<(Mutex<bool>, Condvar)>,
                           current_individual: Arc<Mutex<Self::IndividualPhenotype>>,
                           node_states: Arc<MutexNodeStates>,
                           event_schedule_sender: STDSender<RMOEvent>
    )
    {
        let (run_lock, _run_cvar) = &*run;
        let inbox = Arc::new((Mutex::new(BinaryHeap::new()), Condvar::new()));
        let event_schedule_sender_2 = event_schedule_sender.clone();
        let inbox_2 = inbox.clone();
        thread::spawn(move || Self::inbox_controller(inbox_2, event_schedule_sender_2, MAX_INBOX_SIZE, MAX_DURATION_MILLIS));
        let (inbox_lock, inbox_cvar) = &*inbox;
        loop {
            match receiver.blocking_recv() {
                Some(event) => {
                    let rmo_event = RMOEvent::from(&event);
                    Self::check_message_for_round_update(&rmo_event, &node_states);
                    // If the network is ready to apply the test case, collect messages in inbox, else immediately relay
                    if *run_lock.lock() {
                        if Self::is_consensus_rmo(&rmo_event.message) {
                            node_states.add_send_dependency(*RippleMessage::new(format!("Ripple{}", rmo_event.from + 1), format!("Ripple{}", rmo_event.to + 1), Utc::now(), rmo_event.message.clone()));
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
                            inbox_lock.lock().push(OrderedRMOEvent::new(rmo_event, *priority));
                            inbox_cvar.notify_all();
                        } else {
                            event_schedule_sender.send(rmo_event).expect("Event scheduler failed");
                        }
                    } else {
                        event_schedule_sender.send(rmo_event).expect("Event scheduler failed");
                        while let Some(event) =  inbox_lock.lock().pop() {
                            event_schedule_sender.send(event.rmo_event).expect("Event scheduler failed");
                        }
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

#[derive(Debug, Clone)]
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
    use std::collections::BinaryHeap;
    use std::sync::Arc;
    use std::sync::mpsc::channel;
    use std::thread;
    use std::time::Duration;
    use parking_lot::{Condvar, Mutex};
    use crate::message_handler::RippleMessageObject;
    use crate::protos::ripple::{TMStatusChange, TMValidation};
    use crate::scheduler::priority_scheduler::{OrderedRMOEvent, PriorityScheduler};
    use crate::scheduler::RMOEvent;

    #[test]
    fn test_inbox_controller() {
        let inbox = Arc::new((Mutex::new(BinaryHeap::new()), Condvar::new()));
        let inbox_clone = inbox.clone();
        let (inbox_lock, inbox_cvar) = &*inbox;
        let (event_schedule_sender, event_scheduler_receiver) = channel();
        let max_inbox_size = 10usize;
        let max_duration_millis = 500;
        thread::spawn(move || PriorityScheduler::inbox_controller(inbox_clone, event_schedule_sender, max_inbox_size, max_duration_millis));
        thread::sleep(Duration::from_millis(100));
        let mut rmo_event_size = RMOEvent::default();
        rmo_event_size.message = RippleMessageObject::TMStatusChange(TMStatusChange::new());
        let mut rmo_event_time = RMOEvent::default();
        rmo_event_time.message = RippleMessageObject::TMValidation(TMValidation::new());
        for i in 0..max_inbox_size+1 {
            let mut inbox_heap = inbox_lock.lock();
            if i == max_inbox_size {
                inbox_heap.push(OrderedRMOEvent::new(rmo_event_size.clone(), i));
                println!("Sending message {} to inbox", i);
            } else if i == max_inbox_size - 1 {
                inbox_heap.push(OrderedRMOEvent::new(rmo_event_time.clone(), i));
            } else {
                inbox_heap.push(OrderedRMOEvent::new(RMOEvent::default(), i));
                println!("Sending message {} to inbox", i);
            }
            println!("woke up {} threads", inbox_cvar.notify_all());
        }
        let res = event_scheduler_receiver.recv();
        assert_eq!(res, Ok(rmo_event_size));
        let res = event_scheduler_receiver.recv();
        assert_eq!(res, Ok(rmo_event_time));
    }
}
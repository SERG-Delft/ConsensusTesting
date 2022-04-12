use std::cmp::Ordering;
use std::collections::{BinaryHeap};
use std::sync::Arc;
use std::sync::mpsc::{channel, Receiver as STDReceiver, Sender as STDSender};
use std::thread;
use chrono::Utc;
use log::{debug, error};
use parking_lot::{Condvar, Mutex};
use tokio::sync::mpsc::Receiver as TokioReceiver;
use crate::collector::RippleMessage;
use crate::ga::genetic_algorithm::ConsensusMessageType;
use crate::ga::priority_encoding::{Priority, PriorityMapPhenotype};
use crate::node_state::MutexNodeStates;
use crate::scheduler::{Event, P2PConnections, RMOEvent, Scheduler, SchedulerState};

pub struct InboxScheduler {
    state: SchedulerState,
}

impl InboxScheduler {
    pub fn new(p2p_connections: P2PConnections, collector_sender: STDSender<Box<RippleMessage>>, node_states: Arc<MutexNodeStates>) -> Self {
        InboxScheduler {
            state: SchedulerState {
                p2p_connections,
                collector_sender,
                run: Arc::new((Mutex::new(false), Condvar::new())),
                latest_validated_ledger: Arc::new((Mutex::new(0), Condvar::new())),
                current_round: Arc::new((Mutex::new(0), Condvar::new())),
                node_states,
            }
        }
    }

    /// Wait for new messages delivered by peers
    /// If the network is not stable, immediately relay messages
    /// Else collect messages in inbox and schedule based on priority
    fn schedule_controller(mut receiver: TokioReceiver<Event>,
                           run: Arc<(Mutex<bool>, Condvar)>,
                           current_individual: Arc<Mutex<PriorityMapPhenotype>>,
                           node_states: Arc<MutexNodeStates>,
                           event_schedule_sender: STDSender<RMOEvent>
    )
    {
        let (run_lock, _run_cvar) = &*run;
        let inbox = Arc::new((Mutex::new(BinaryHeap::new()), Condvar::new()));
        let event_schedule_sender_2 = event_schedule_sender.clone();
        let inbox_2 = inbox.clone();
        thread::spawn(move || Self::inbox_controller(inbox_2, event_schedule_sender_2));
        let (inbox_lock, inbox_cvar) = &*inbox;
        loop {
            match receiver.blocking_recv() {
                Some(event) => {
                    let rmo_event = RMOEvent::from(&event);
                    // If the network is ready to apply the test case, collect messages in inbox, else immediately relay
                    if *run_lock.lock() {
                        if Self::is_consensus_rmo(&rmo_event.message) {
                            node_states.add_send_dependency(*RippleMessage::new(format!("Ripple{}", rmo_event.from + 1), format!("Ripple{}", rmo_event.to + 1), Utc::now(), rmo_event.message.clone()));
                            let message_type_map = current_individual.lock().priority_map.get(&rmo_event.from).unwrap().get(&rmo_event.to).unwrap().clone();
                            let priority = match rmo_event.message.message_type() {
                                "Validation" => message_type_map.get(&ConsensusMessageType::TMValidation).unwrap(),
                                "ProposeSet" => message_type_map.get(&ConsensusMessageType::TMProposeSet).unwrap(),
                                "StatusChange" => message_type_map.get(&ConsensusMessageType::TMStatusChange).unwrap(),
                                "HaveTransactionSet" => message_type_map.get(&ConsensusMessageType::TMHaveTransactionSet).unwrap(),
                                _ => {
                                    error!("Should be consensus message, but not matched!");
                                    &Priority(0f32)
                                }
                            };
                            inbox_lock.lock().push(OrderedRMOEvent::new(rmo_event, *priority));
                            inbox_cvar.notify_all();
                        } else {
                            event_schedule_sender.send(rmo_event);
                        }
                    } else {
                        event_schedule_sender.send(rmo_event);
                        while let Some(event) =  inbox_lock.lock().pop() {
                            event_schedule_sender.send(event.rmo_event);
                        }
                    }
                },
                None => error!("Peer senders failed")
            }
        }
    }

    fn inbox_controller(inbox: Arc<(Mutex<BinaryHeap<OrderedRMOEvent>>, Condvar)>, event_schedule_sender: STDSender<RMOEvent>) {
        let max_inbox_size = 10;
        let max_duration_millis = 200;
        let (inbox_lock, inbox_cvar) = &*inbox;
        let mut time = Utc::now();
        loop {
            let mut inbox_heap = inbox_lock.lock();
            inbox_cvar.wait(&mut inbox_heap);
            if inbox_heap.len() > max_inbox_size ||
                Utc::now().signed_duration_since(time) > chrono::Duration::milliseconds(max_duration_millis)
            {
                event_schedule_sender.send(inbox_heap.pop().unwrap().rmo_event);
                time = Utc::now();
            }
        }
    }
}

impl Scheduler for InboxScheduler {
    type IndividualPhenotype = PriorityMapPhenotype;

    fn start_extension(self, receiver: TokioReceiver<Event>, ga_receiver: STDReceiver<Self::IndividualPhenotype>) {
        let (event_schedule_sender, event_schedule_receiver) = channel();
        let run_clone = self.get_state().run.clone();
        let node_states_clone = self.get_state().node_states.clone();
        let node_states_clone_2 = self.get_state().node_states.clone();
        let current_delays = Arc::new(Mutex::new(Self::IndividualPhenotype::default()));
        let current_delays_2 = current_delays.clone();
        thread::spawn(move || Self::schedule_controller(receiver, run_clone, current_delays, node_states_clone, event_schedule_sender));
        thread::spawn(move || Self::listen_to_ga(current_delays_2, ga_receiver, node_states_clone_2));
        loop {
            match event_schedule_receiver.recv() {
                Ok(event) => self.execute_event(event),
                Err(_) => panic!("Scheduler sender failed")
            }
        }
    }

    fn listen_to_ga(current_individual: Arc<Mutex<Self::IndividualPhenotype>>, ga_receiver: STDReceiver<Self::IndividualPhenotype>, node_states: Arc<MutexNodeStates>) {
        loop {
            match ga_receiver.recv() {
                Ok(new_priority) => {
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

struct OrderedRMOEvent {
    rmo_event: RMOEvent,
    priority: Priority,
}

impl OrderedRMOEvent {
    pub fn new(rmo_event: RMOEvent, priority: Priority) -> Self {
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
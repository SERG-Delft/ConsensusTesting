use std::sync::Arc;
use std::sync::mpsc::{Sender as STDSender, Receiver as STDReceiver, channel};
use tokio::sync::mpsc::{Receiver as TokioReceiver};
use std::thread;
use std::time::Duration;
use chrono::Utc;
use genevo::genetic::{Phenotype};
use log::{debug, error, trace};
use parking_lot::{Condvar, Mutex};
use crate::collector::RippleMessage;
use crate::ga::genetic_algorithm::{ConsensusMessageType, DelayMapPhenotype, DROP_THRESHOLD};
use crate::message_handler::RippleMessageObject;
use crate::node_state::MutexNodeStates;
use crate::scheduler::{Event, P2PConnections, RMOEvent, Scheduler, SchedulerState};

/// Scheduler module responsible for scheduling execution of events (message receivals in peers)
/// p2p_connections: Contains the senders for sending from a peer to another peer
/// collector_sender: Sender for sending the executed events to the collector (execution.txt)
/// latest_validated_ledger: The latest validated ledger
/// current_round: The latest round for which a message is sent by one of the peers
pub struct DelayScheduler {
    state: SchedulerState
}

impl DelayScheduler {
    pub fn new(p2p_connections: P2PConnections, collector_sender: STDSender<Box<RippleMessage>>, node_states: Arc<MutexNodeStates>) -> Self {
        DelayScheduler {
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
    /// Else schedule messages with a certain delay
    fn schedule_controller(mut receiver: TokioReceiver<Event>,
                           run: Arc<(Mutex<bool>, Condvar)>,
                           current_delays: Arc<Mutex<DelayMapPhenotype>>,
                           node_states: Arc<MutexNodeStates>,
                           event_schedule_sender: STDSender<RMOEvent>
    )
    {
        let (run_lock, _run_cvar) = &*run;
        loop {
            match receiver.blocking_recv() {
                Some(event) => {
                    let rmo_event = RMOEvent::from(&event);
                    let ms: u32;
                    // If the network is ready to apply the test case, determine delay of message, else delay = 0
                    if *run_lock.lock() {
                        if Self::is_consensus_rmo(&rmo_event.message) {
                            node_states.add_send_dependency(*RippleMessage::new(format!("Ripple{}", rmo_event.from + 1), format!("Ripple{}", rmo_event.to + 1), Utc::now(), rmo_event.message.clone()));
                        }
                        let message_type_map = current_delays.lock().delay_map.get(&rmo_event.from).unwrap().get(&rmo_event.to).unwrap().clone();
                        ms = match rmo_event.message {
                            RippleMessageObject::TMValidation(_) => message_type_map.get(&ConsensusMessageType::TMValidation).unwrap().clone(),
                            RippleMessageObject::TMProposeSet(_) => message_type_map.get(&ConsensusMessageType::TMProposeSet).unwrap().clone(),
                            RippleMessageObject::TMStatusChange(_) => message_type_map.get(&ConsensusMessageType::TMStatusChange).unwrap().clone(),
                            RippleMessageObject::TMHaveTransactionSet(_) => message_type_map.get(&ConsensusMessageType::TMHaveTransactionSet).unwrap().clone(),
                            _ => 0
                        };
                    } else {
                        ms = 0;
                    }
                    let duration = Duration::from_millis(ms as u64);
                    ScheduledEvent::schedule_execution(
                        rmo_event,
                        duration,
                        event_schedule_sender.clone()
                    )
                },
                None => error!("Peer senders failed")
            }
        }
    }
}

impl Scheduler for DelayScheduler {
    type IndividualPhenotype = DelayMapPhenotype;

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

    /// Listen to the genetic algorithm for new individuals to test
    fn listen_to_ga(current_individual: Arc<Mutex<Self::IndividualPhenotype>>, ga_receiver: STDReceiver<Self::IndividualPhenotype>, node_states: Arc<MutexNodeStates>) {
        loop {
            match ga_receiver.recv() {
                Ok(new_delays) => {
                    node_states.set_current_delays(new_delays.genes());
                    *current_individual.lock() = new_delays;
                    debug!("New delays received");
                },
                Err(_) => {}
            }
        }
    }

    fn get_state(&self) -> &SchedulerState {
        &self.state
    }
}



/// ScheduledEvent is a struct with functionality for scheduling the sending of a message after a certain duration
pub struct ScheduledEvent {}

impl ScheduledEvent {
    pub(crate) fn schedule_execution(event: RMOEvent, duration: Duration, sender: STDSender<RMOEvent>) {
        if duration.as_millis() > DROP_THRESHOLD as u128 {
            return
        } else {
            thread::spawn(move || {
                trace!("Sleeping for {} ms for message: {} -> {}: {:?}", duration.as_millis(), event.from, event.to, event.message);
                thread::sleep(duration);
                trace!("Sending event to executor: {} -> {}: {:?}", event.from, event.to, event.message);
                sender.send(event).expect("Scheduler receiver failed");
            });
        }
    }
}
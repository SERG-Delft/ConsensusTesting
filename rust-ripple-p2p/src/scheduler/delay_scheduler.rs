use std::sync::{Arc, RwLock};
use std::sync::mpsc::{Sender as STDSender, Receiver as STDReceiver};
use tokio::sync::mpsc::{Receiver as TokioReceiver};
use std::thread;
use genevo::genetic::Phenotype;
use log::{debug, error, trace};
use parking_lot::{Condvar, Mutex};
use spin_sleep::SpinSleeper;
use crate::collector::RippleMessage;
use crate::failure_writer::ConsensusPropertyTypes;
use crate::ga::genetic_algorithm::{ConsensusMessageType};
#[allow(unused_imports)]
use crate::ga::encoding::delay_encoding::{DelayMapPhenotype, DROP_THRESHOLD};
use crate::ga::encoding::ExtendedPhenotype;
use crate::node_state::MutexNodeStates;
use crate::NodeKeys;
use crate::scheduler::{Event, RMOEvent, Scheduler, SchedulerState};

/// Scheduler module responsible for scheduling execution of events (message receivals in peers)
/// p2p_connections: Contains the senders for sending from a peer to another peer
/// collector_sender: Sender for sending the executed events to the collector (execution.txt)
/// latest_validated_ledger: The latest validated ledger
/// current_round: The latest round for which a message is sent by one of the peers
pub struct DelayScheduler {
    state: SchedulerState
}

impl Scheduler for DelayScheduler {
    type IndividualPhenotype = DelayMapPhenotype;

    fn new(
        collector_sender: STDSender<Box<RippleMessage>>,
        node_states: Arc<MutexNodeStates>,
        node_keys: Vec<NodeKeys>,
        failure_sender: STDSender<Vec<ConsensusPropertyTypes>>,
    ) -> Self {
        Self {
            state: SchedulerState::new(collector_sender, node_states, node_keys, failure_sender)
        }
    }

    /// Wait for new messages delivered by peers
    /// If the network is not stable, immediately relay messages
    /// Else schedule messages with a certain delay
    fn schedule_controller(
        mut receiver: TokioReceiver<Event>,
        run: Arc<(RwLock<bool>, Condvar)>,
        current_delays: Arc<Mutex<Self::IndividualPhenotype>>,
        round_update_sender: STDSender<RMOEvent>,
        event_schedule_sender: STDSender<RMOEvent>,
        send_dependency_sender: STDSender<RippleMessage>,
    )
    {
        let (run_lock, _run_cvar) = &*run;
        loop {
            match receiver.blocking_recv() {
                Some(event) => {
                    let rmo_event = RMOEvent::from(&event);
                    if Self::is_consensus_rmo(&rmo_event.message) {
                        round_update_sender.send(rmo_event.clone()).expect("Round update sender failed");
                        // If the network is ready to apply the test case, determine delay of message, else delay = 0
                        if *run_lock.read().unwrap() {
                            send_dependency_sender.send(RippleMessage::from_rmo_event(rmo_event.clone())).expect("send dependency sender failed");
                            let consensus_message_type_option = ConsensusMessageType::create_consensus_message_type(&rmo_event.message);
                            if let Some(consensus_message_type) = consensus_message_type_option {
                                let ms = current_delays.lock().get_delay(&rmo_event.from, &rmo_event.to, &consensus_message_type) as u64;
                                if ms > 0 {
                                    ScheduledEvent::schedule_execution(
                                        rmo_event,
                                        ms,
                                        event_schedule_sender.clone()
                                    );
                                    continue;
                                }
                            }
                        }
                    }
                    event_schedule_sender.send(rmo_event).expect("Event schedule sender failed");
                },
                None => error!("Peer senders failed")
            }
        }
    }

    /// Listen to the genetic algorithm for new individuals to test
    fn listen_to_ga(current_individual: Arc<Mutex<Self::IndividualPhenotype>>, ga_receiver: STDReceiver<Self::IndividualPhenotype>, node_states: Arc<MutexNodeStates>) {
        loop {
            match ga_receiver.recv() {
                Ok(new_delays) => {
                    node_states.set_current_delays(new_delays.genes());
                    node_states.set_current_individual(current_individual.lock().display_genotype_by_message());
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
    pub(crate) fn schedule_execution(event: RMOEvent, duration: u64, sender: STDSender<RMOEvent>) {
        // if duration.as_millis() > DROP_THRESHOLD as u128 {
        //     return
        // } else {
            thread::spawn(move || {
                let sleeper = SpinSleeper::default();
                trace!("Sleeping for {} ms for message: {} -> {}: {:?}", duration, event.from, event.to, event.message);
                let ns = match duration.checked_mul(1000 * 1000) {
                    None => {
                        error!("The delay ms to ns caused a u64 overflow, sleeping max ns");
                        u64::MAX
                    }
                    Some(ns) => ns
                };
                sleeper.sleep_ns(ns);
                trace!("Sending event to executor: {} -> {}: {:?}", event.from, event.to, event.message);
                sender.send(event).expect("Scheduler receiver failed");
            });
        // }
    }
}
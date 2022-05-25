use std::sync::Arc;
use std::sync::mpsc::{Sender as STDSender, Receiver as STDReceiver};
use tokio::sync::mpsc::{Receiver as TokioReceiver};
use std::thread;
use std::time::Duration;
use chrono::Utc;
use genevo::genetic::Phenotype;
use log::{debug, error, trace};
use parking_lot::{Condvar, Mutex};
use crate::collector::RippleMessage;
use crate::ga::genetic_algorithm::{ConsensusMessageType};
#[allow(unused_imports)]
use crate::ga::encoding::delay_encoding::{DelayMapPhenotype, DROP_THRESHOLD};
use crate::ga::encoding::ExtendedPhenotype;
use crate::message_handler::RippleMessageObject;
use crate::node_state::MutexNodeStates;
use crate::NodeKeys;
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
    #[allow(unused)]
    pub fn new(p2p_connections: P2PConnections, collector_sender: STDSender<Box<RippleMessage>>, node_states: Arc<MutexNodeStates>, node_keys: Vec<NodeKeys>) -> Self {
        DelayScheduler {
            state: SchedulerState::new(p2p_connections, collector_sender, node_states, node_keys)
        }
    }
}

impl Scheduler for DelayScheduler {
    type IndividualPhenotype = DelayMapPhenotype;

    /// Wait for new messages delivered by peers
    /// If the network is not stable, immediately relay messages
    /// Else schedule messages with a certain delay
    fn schedule_controller(mut receiver: TokioReceiver<Event>,
                           run: Arc<(Mutex<bool>, Condvar)>,
                           current_delays: Arc<Mutex<Self::IndividualPhenotype>>,
                           node_states: Arc<MutexNodeStates>,
                           event_schedule_sender: STDSender<RMOEvent>
    )
    {
        let (run_lock, _run_cvar) = &*run;
        loop {
            match receiver.blocking_recv() {
                Some(event) => {
                    let rmo_event = RMOEvent::from(&event);
                    Self::check_message_for_round_update(&rmo_event, &node_states);
                    let ms: u32;
                    // If the network is ready to apply the test case, determine delay of message, else delay = 0
                    if *run_lock.lock() {
                        if Self::is_consensus_rmo(&rmo_event.message) {
                            node_states.add_send_dependency(*RippleMessage::new(format!("Ripple{}", rmo_event.from + 1), format!("Ripple{}", rmo_event.to + 1), Utc::now(), rmo_event.message.clone()));
                        }
                        let message_type_map = current_delays.lock().delay_map.get(&rmo_event.from).unwrap().get(&rmo_event.to).unwrap().clone();
                        ms = match &rmo_event.message {
                            RippleMessageObject::TMValidation(_) => message_type_map.get(&ConsensusMessageType::TMValidation).unwrap().clone(),
                            RippleMessageObject::TMProposeSet(proposal) => {
                                match proposal.get_proposeSeq() {
                                    0 => {
                                        // println!("Proposal Delay!!!! {}", message_type_map.get(&ConsensusMessageType::TMProposeSet0).unwrap());
                                        message_type_map.get(&ConsensusMessageType::TMProposeSet0).unwrap().clone()
                                    },
                                    1 => message_type_map.get(&ConsensusMessageType::TMProposeSet1).unwrap().clone(),
                                    2 => message_type_map.get(&ConsensusMessageType::TMProposeSet2).unwrap().clone(),
                                    3 => message_type_map.get(&ConsensusMessageType::TMProposeSet3).unwrap().clone(),
                                    4 => message_type_map.get(&ConsensusMessageType::TMProposeSet4).unwrap().clone(),
                                    5 => message_type_map.get(&ConsensusMessageType::TMProposeSet5).unwrap().clone(),
                                    4294967295 => message_type_map.get(&ConsensusMessageType::TMProposeSetBowOut).unwrap().clone(),
                                    _ => message_type_map.get(&ConsensusMessageType::TMProposeSet0).unwrap().clone(),
                                }
                            },
                            RippleMessageObject::TMStatusChange(_) => message_type_map.get(&ConsensusMessageType::TMStatusChange).unwrap().clone(),
                            RippleMessageObject::TMHaveTransactionSet(_) => message_type_map.get(&ConsensusMessageType::TMHaveTransactionSet).unwrap().clone(),
                            RippleMessageObject::TMTransaction(_) => message_type_map.get(&ConsensusMessageType::TMTransaction).unwrap().clone(),
                            RippleMessageObject::TMLedgerData(_) => message_type_map.get(&ConsensusMessageType::TMLedgerData).unwrap().clone(),
                            RippleMessageObject::TMGetLedger(_) => message_type_map.get(&ConsensusMessageType::TMGetLedger).unwrap().clone(),
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
    pub(crate) fn schedule_execution(event: RMOEvent, duration: Duration, sender: STDSender<RMOEvent>) {
        // if duration.as_millis() > DROP_THRESHOLD as u128 {
        //     return
        // } else {
            thread::spawn(move || {
                trace!("Sleeping for {} ms for message: {} -> {}: {:?}", duration.as_millis(), event.from, event.to, event.message);
                thread::sleep(duration);
                trace!("Sending event to executor: {} -> {}: {:?}", event.from, event.to, event.message);
                sender.send(event).expect("Scheduler receiver failed");
            });
        // }
    }
}
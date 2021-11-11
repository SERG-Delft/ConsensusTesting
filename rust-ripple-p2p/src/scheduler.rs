use log::{trace, error};
use std::collections::HashMap;
use chrono::Utc;
use tokio::sync::mpsc::{Sender as TokioSender, Receiver as TokioReceiver};
use std::sync::mpsc::{Sender as STDSender, Receiver as STDReceiver, channel};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use byteorder::{BigEndian, ByteOrder};
use rand::Rng;
use crate::client::{SubscriptionObject};
use crate::collector::RippleMessage;
use crate::message_handler::{parse_protocol_message, RippleMessageObject};

type P2PConnections = HashMap<usize, HashMap<usize, PeerChannel>>;

/// Scheduler module responsible for scheduling execution of events (message receivals in peers)
/// p2p_connections: Contains the senders for sending from a peer to another peer
/// collector_sender: Sender for sending the executed events to the collector (execution.txt)
/// stable: Is the network in a stable state
/// latest_validated_ledger: The latest validated ledger
/// current_round: The latest round for which a message is sent by one of the peers
pub struct Scheduler {
    p2p_connections: P2PConnections,
    collector_sender: STDSender<Box<RippleMessage>>,
    stable: Arc<Mutex<bool>>,
    latest_validated_ledger: Arc<Mutex<u32>>,
    current_round: Arc<Mutex<u32>>
}

impl Scheduler {
    pub fn new(p2p_connections: P2PConnections, collector_sender: STDSender<Box<RippleMessage>>) -> Self {
        Scheduler {
            p2p_connections,
            collector_sender,
            stable: Arc::new(Mutex::new(false)),
            latest_validated_ledger: Arc::new(Mutex::new(0)),
            current_round: Arc::new(Mutex::new(0))
        }
    }

    /// Starts peer and collector listening threads and listens to the scheduler for executing messages afer delay
    pub fn start(self, receiver: TokioReceiver<Event>, collector_receiver: STDReceiver<SubscriptionObject>) {
        let stable_clone = self.stable.clone();
        let latest_validated_ledger_clone = self.latest_validated_ledger.clone();
        let (event_schedule_sender, event_schedule_receiver) = channel();
        let stable_clone_2 = self.stable.clone();
        let current_round_clone = self.current_round.clone();
        thread::spawn(move || Self::listen_to_collector(collector_receiver, stable_clone, latest_validated_ledger_clone));
        thread::spawn(move || Self::listen_to_peers(stable_clone_2, current_round_clone, receiver, event_schedule_sender));
        loop {
            match event_schedule_receiver.recv() {
                Ok(event) => self.execute_event(event),
                Err(_) => error!("Scheduler sender failed")
            }
        }
    }

    /// Execute event and report to collector (execution.txt)
    fn execute_event(&self, event: Event) {
        let rmo = parse_protocol_message(BigEndian::read_u16(&event.message[4..6]), &event.message[6..]);
        self.collector_sender.send(RippleMessage::new(format!("Ripple{}", event.from+1), format!("Ripple{}", event.to+1), Utc::now(), rmo)).expect("Collector receiver failed");
        self.p2p_connections.get(&event.to).unwrap().get(&event.from).unwrap().send(event.message);
    }

    /// Create a random delay in ms between lower- and upper_bound
    fn random_delay(lower_bound: f32, upper_bound: f32) -> Duration {
        let mut rng = rand::thread_rng();
        Duration::from_millis(rng.gen_range(lower_bound..upper_bound) as u64)
    }

    /// Listen to messages sent by peers
    /// If the network is not stable, immediately relay messages
    /// Else schedule messages with a certain delay
    fn listen_to_peers(stable: Arc<Mutex<bool>>, current_round: Arc<Mutex<u32>>, mut receiver: TokioReceiver<Event>, event_schedule_sender: STDSender<Event>) {
        loop {
            while !*stable.lock().unwrap() {
                match receiver.blocking_recv() {
                    Some(event) => ScheduledEvent::schedule_execution(
                        event,
                        Duration::ZERO,
                        event_schedule_sender.clone()
                    ),
                    None => error!("Peer senders failed")
                }
            }
            match receiver.blocking_recv() {
                Some(event) => {
                    let rmo = parse_protocol_message(BigEndian::read_u16(&event.message[4..6]), &event.message[6..]);
                    Self::update_current_round(rmo, current_round.clone());
                    ScheduledEvent::schedule_execution(
                        event,
                        Self::random_delay(0f32, 500f32),
                        event_schedule_sender.clone()
                    )
                },
                None => error!("Peer senders failed")
            }
        }
    }

    /// Listen to messages from the collector
    /// Responsible for determining stability and latest validated ledger of the network
    fn listen_to_collector(collector_receiver: STDReceiver<SubscriptionObject>, stable: Arc<Mutex<bool>>, latest_validated_ledger: Arc<Mutex<u32>>) {
        let mut set_stable = false;
        let mut local_latest_validated_ledger = 0;
        loop {
            match collector_receiver.recv() {
                Ok(subscription_object) => {
                    match subscription_object {
                        SubscriptionObject::ValidatedLedger(ledger) => {
                            if !set_stable {
                                *stable.lock().unwrap() = true;
                                set_stable = true;
                            }
                            if local_latest_validated_ledger < ledger.ledger_index {
                                *latest_validated_ledger.lock().unwrap() = ledger.ledger_index;
                                local_latest_validated_ledger = ledger.ledger_index;
                            }
                        }
                        _ => {}
                    }
                }
                Err(_) => {}
            }
        }
    }

    /// Update the current round if a message is received with a higher ledger sequence number
    fn update_current_round(rmo: RippleMessageObject, current_round: Arc<Mutex<u32>>) {
        let round = match rmo {
            RippleMessageObject::TMGetLedger(get_ledger) => get_ledger.get_ledgerSeq(),
            RippleMessageObject::TMLedgerData(ledger_data) => ledger_data.get_ledgerSeq(),
            // RippleMessageObject::TMProposeSet(propose_set) => propose_set.get TODO: Use previous ledger hash to find ledgerSeq
            RippleMessageObject::TMStatusChange(status_change) => status_change.get_ledgerSeq(),
            //RippleMessageObject::TMValidation(_) => {} TODO: after deserialization can find ledgerSeq
            _ => 0
        };
        if round > 0 {
            let mut locked_round = current_round.lock().unwrap();
            if round > *locked_round {
                *locked_round = round;
            }
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

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use log::*;
use crate::protos::ripple::TMStatusChange;
use crate::message_handler::{parse_message};
use byteorder::{BigEndian, ByteOrder};
use protobuf::ProtobufEnum;

pub struct Scheduler {
    peers: HashMap<usize, PeerChannel>,
    events: Arc<Mutex<Vec<Event>>>,
    latest_status_changes: HashMap<usize, TMStatusChange>
}

impl Scheduler {
    pub fn new(peers: HashMap<usize, PeerChannel>) -> Self {
        let mut latest_status_changes = HashMap::new();
        for peer in &peers {
            latest_status_changes.insert(peer.0.clone(), TMStatusChange::new());
        }
        Scheduler { peers, events: Arc::new(Mutex::new(vec![])), latest_status_changes }
    }

    pub fn start(mut self, receivers: Vec<Receiver<Vec<u8>>>) {
        let n = receivers.len();
        for (i, receiver) in receivers.into_iter().enumerate() {
            let events_clone = Arc::clone(&self.events);
            PeerChannel::receive_messages(receiver, n, i.clone(), events_clone);
        }
        loop {
            let mut events = self.events.lock().unwrap();
            while !events.is_empty() {
                let event = events.remove(0);
                let status_change = self.latest_status_changes.get(&event.to).unwrap().clone();
                self.latest_status_changes.insert(event.to, self.execute_event(event, status_change));
            }
        }
    }

    fn execute_event(&self, event: Event, latest_status_change: TMStatusChange) -> TMStatusChange {
        let (last_status_change, should_execute) = self.execute_status_event(&event, latest_status_change);
        if should_execute {
            match self.peers.get(&event.to).unwrap().sender.send(event.message) {
                Ok(_) => { }//debug!("Sent message from {} to {}", event.from+1, event.to+1) }
                Err(_err) => { }//println!("Failed to send message from {} to {}, because {}", event.from, event.to, err)}
            }
        }
        last_status_change
    }

    fn execute_status_event(&self, event: &Event, latest_status_change: TMStatusChange) -> (TMStatusChange, bool) {
        let latest_status_change = latest_status_change;
        let message_type = BigEndian::read_u16(&event.message[4..6]);
        if message_type == 34 {
            let message = Box::<TMStatusChange>::new(parse_message(&event.message[6..]));
            return if message.has_newEvent() {
                let node_event = message.get_newEvent();
                let seq = message.get_ledgerSeq();
                if seq < latest_status_change.get_ledgerSeq() || (seq == latest_status_change.get_ledgerSeq() && (node_event as u8) >= (latest_status_change.get_newEvent() as u8)) {
                    (latest_status_change, false)
                } else {
                    (*message, true)
                }
            } else {
                (latest_status_change, true)
            }
        }
        (latest_status_change, true)
    }

    fn create_events(n: usize, peer_sender: usize, message: Vec<u8>) -> Vec<Event> {
        let mut events = vec![];
        let message_type = BigEndian::read_u16(&message[4..6]);
        for i in 0..n {
            // Broadcast message to all other peers for now or to all peers for status change
            if peer_sender != i || message_type == 34 {
                let event = Event { from: peer_sender, to: i, message: message.clone() };
                events.push(event);
            }
        }
        return events;
    }
}

pub struct PeerChannel {
    sender: Sender<Vec<u8>>,
}

impl PeerChannel {
    pub fn new(sender: Sender<Vec<u8>>) -> Self {
        PeerChannel { sender }
    }

    fn receive_messages(receiver: Receiver<Vec<u8>>, n: usize, i: usize, events: Arc<Mutex<Vec<Event>>>) {
        thread::spawn(move || {
            loop {
                match receiver.try_recv() {
                    Ok(message) => {
                        events.lock().unwrap().extend(Scheduler::create_events(n, i, message));
                    }
                    Err(_) => {}
                }
            }
        });
    }
}

struct Event {
    from: usize,
    to: usize,
    message: Vec<u8>
}

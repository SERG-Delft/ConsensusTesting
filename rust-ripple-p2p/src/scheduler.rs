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
    p2p_connections: HashMap<usize, HashMap<usize, PeerChannel>>,
    events: Arc<Mutex<Vec<Event>>>,
}

impl Scheduler {
    pub fn new(p2p_connections: HashMap<usize, HashMap<usize, PeerChannel>>) -> Self {
        Scheduler { p2p_connections, events: Arc::new(Mutex::new(vec![])) }
    }

    pub fn start(mut self, mut receivers: HashMap<usize, HashMap<usize, Receiver<Vec<u8>>>>) {
        for (i, mut connections) in receivers.drain() {
            for (j, receiver) in connections.drain() {
                let events_clone = Arc::clone(&self.events);
                PeerChannel::receive_messages(receiver, i.clone(), j.clone(), events_clone);
            }
        }
        loop {
            let mut events = self.events.lock().unwrap();
            while !events.is_empty() {
                let event = events.remove(0);
                self.execute_event(event);
            }
        }
    }

    fn execute_event(&self, event: Event) {
        match self.p2p_connections.get(&event.to).unwrap().get(&event.from).unwrap().sender.send(event.message) {
            Ok(_) => { }//debug!("Sent message from {} to {}", event.from+1, event.to+1) }
            Err(_err) => { }//println!("Failed to send message from {} to {}, because {}", event.from, event.to, err)}
        }
    }

    fn create_event(from: usize, to: usize, message: Vec<u8>) -> Event {
        Event { from, to, message }
    }
}

pub struct PeerChannel {
    sender: Sender<Vec<u8>>,
}

impl PeerChannel {
    pub fn new(sender: Sender<Vec<u8>>) -> Self {
        PeerChannel { sender }
    }

    fn receive_messages(receiver: Receiver<Vec<u8>>, from: usize, to: usize, events: Arc<Mutex<Vec<Event>>>) {
        thread::spawn(move || {
            loop {
                match receiver.try_recv() {
                    Ok(message) => {
                        events.lock().unwrap().push(Scheduler::create_event(from, to, message));
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

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use log::*;

pub struct Scheduler {
    peers: HashMap<usize, PeerChannel>,
    events: Arc<Mutex<Vec<Event>>>
}

impl Scheduler {
    pub fn new(peers: HashMap<usize, PeerChannel>) -> Self {
        Scheduler { peers, events: Arc::new(Mutex::new(vec![])) }
    }

    pub fn start(self, receivers: Vec<Receiver<Vec<u8>>>) {
        let n = receivers.len();
        for (i, receiver) in receivers.into_iter().enumerate() {
            let events_clone = Arc::clone(&self.events);
            PeerChannel::receive_messages(receiver, n, i.clone(), events_clone);
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
        match self.peers.get(&event.to).unwrap().sender.send(event.message) {
            Ok(_) => { debug!("Sent message from {} to {}", event.from+1, event.to+1) }
            Err(_err) => { }//println!("Failed to send message from {} to {}, because {}", event.from, event.to, err)}
        }
    }

    fn create_events(n: usize, peer_sender: usize, message: Vec<u8>) -> Vec<Event> {
        let mut events = vec![];
        for i in 0..n {
            // Broadcast message to all other peers for now
            if peer_sender != i {
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

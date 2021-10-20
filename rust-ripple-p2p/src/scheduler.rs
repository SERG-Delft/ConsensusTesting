use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};

pub struct Scheduler {
    peers: HashMap<usize, PeerChannel>,
    events: Vec<Event>
}

impl Scheduler {
    pub fn new(peers: HashMap<usize, PeerChannel>) -> Self {
        Scheduler { peers, events: vec![] }
    }

    pub fn start(mut self) {
        loop {
            let mut events = vec![];
            for (i, peer) in &self.peers {
                match peer.receiver.try_recv() {
                    Ok(message) => {
                        events.extend(Scheduler::create_events(self.peers.len(), i.clone(), message));
                    },
                    Err(_) => {}
                }
            }
            self.events.extend(events);
            if !self.events.is_empty() {
                let event = self.events.remove(0);
                self.execute_event(event);
            }
        }
    }

    fn execute_event(&self, event: Event) {
        match self.peers.get(&event.to).unwrap().sender.send(event.message) {
            Ok(_) => { println!("Sent message from {} to {}", event.from, event.to)}
            Err(err) => { println!("Failed to send message from {} to {}, because {}", event.from, event.to, err)}
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
    receiver: Receiver<Vec<u8>>
}

impl PeerChannel {
    pub fn new(sender: Sender<Vec<u8>>, receiver: Receiver<Vec<u8>>) -> Self {
        PeerChannel { sender, receiver }
    }
}

struct Event {
    from: usize,
    to: usize,
    message: Vec<u8>
}

use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender, SendError};

pub struct Scheduler {
    peers: HashMap<usize, PeerBridge>,
    events: Vec<Event>
}

impl Scheduler {
    pub fn new(peers: HashMap<usize, PeerBridge>, events: Vec<Event>) -> Self {
        Scheduler { peers, events }
    }

    pub fn start(mut self) {
        loop {
            for (i, peer) in peers {
                match peer.receiver.try_recv() {
                    Ok(message) => {
                        self.add_event(i, message);
                    },
                    Err(_) => {}
                }
            }
            if !self.events.is_empty() {
                self.execute_event(self.events.remove(0));
            }
        }
    }

    fn execute_event(self, event: Event) {
        match self.peers.get(&event.to).unwrap().sender.send(event.message) {
            Ok(_) => { println!("Sent message from {} to {}", event.from, event.to)}
            Err(err) => { println!("Failed to send message from {} to {}, because {}", event.from, event.to, err)}
        }
    }

    fn add_event(self, peer_sender: usize, message: Vec<u8>) {
        for j in self.peers.len() {
            // Broadcast message to all other peers for now
            if peer_sender != j {
                let event = Event { from: peer_sender, to: j, message: message.clone() };
                events.push(event);
            }
        }
    }
}

pub struct PeerBridge {
    sender: Sender<Vec<u8>>,
    receiver: Receiver<Vec<u8>>
}

struct Event {
    from: usize,
    to: usize,
    message: Vec<u8>
}

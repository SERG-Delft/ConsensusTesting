use std::collections::HashMap;
use chrono::Utc;
use tokio::sync::mpsc::{Sender, Receiver};
use byteorder::{BigEndian, ByteOrder};
use crate::collector::RippleMessage;
use crate::message_handler::{invoke_protocol_message, RippleMessageObject};

pub struct Scheduler {
    p2p_connections: HashMap<usize, HashMap<usize, PeerChannel>>,
    collector_sender: std::sync::mpsc::Sender<Box<RippleMessage>>
}

impl Scheduler {
    pub fn new(p2p_connections: HashMap<usize, HashMap<usize, PeerChannel>>, collector_sender: std::sync::mpsc::Sender<Box<RippleMessage>>) -> Self {
        Scheduler { p2p_connections, collector_sender }
    }

    pub fn start(self, mut receiver: Receiver<Event>) {
        loop {
            match receiver.blocking_recv() {
                Some(event) => self.execute_event(event),
                None => panic!("Peer failed")
            }
        }
    }

    fn execute_event(&self, event: Event) {
        let proto_obj: RippleMessageObject = invoke_protocol_message(BigEndian::read_u16(&event.message[4..6]), &event.message[6..]);
        match self.collector_sender.send(RippleMessage::new(format!("Ripple{}", event.from+1), format!("Ripple{}", event.to+1), Utc::now(), proto_obj)) {
            Ok(_) => { }//println!("Sent to collector") }
            Err(_) => { }//println!("Error sending to collector") }
        }
        match self.p2p_connections.get(&event.to).unwrap().get(&event.from).unwrap().sender.blocking_send(event.message) {
            Ok(_) => { }//debug!("Sent message from {} to {}", event.from+1, event.to+1) }
            Err(_err) => {println!("Failed to send message from {} to {}, because {}", event.from, event.to, _err)}
        }
    }

    #[allow(unused)]
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
}

pub struct Event {
    pub from: usize,
    pub to: usize,
    pub message: Vec<u8>
}

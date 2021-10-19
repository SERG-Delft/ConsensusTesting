use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::mpsc::{Receiver};
use std::time::Instant;
use protobuf::Message;

pub struct Collector {
    ripple_message_receiver: Receiver<Box<RippleMessage>>,
    control_receiver: Receiver<String>,
    file: File,
    start: Instant
}

impl Collector {
    pub fn new(ripple_message_receiver: Receiver<Box<RippleMessage>>, control_receiver: Receiver<String>) -> Self {
        let mut file = File::create(Path::new("execution.txt")).expect("Opening execution file failed");
        Collector {
            ripple_message_receiver,
            control_receiver,
            file,
            start: Instant::now()
        }
    }

    pub fn start(&mut self) {
        loop {
            // Stop writing to file if any control message is received
            // Can be extended to start writing to file later
            match self.control_receiver.try_recv() {
                Ok(_) => {
                    break;
                }
                _ => {}
            }
            match self.ripple_message_receiver.try_recv() {
                Ok(mut message) => {
                    self.write_to_file(&mut message);
                }
                _ => {}
            }
        }
    }

    fn write_to_file(&mut self, ripple_message: &mut RippleMessage) {
        ripple_message.set_start(self.start);
        self.file.write_all(ripple_message.to_string().as_bytes());
    }
}

pub struct RippleMessage {
    from_node: String,
    timestamp: Instant,
    message: Box<dyn Message>,
    start: Option<Instant>
}

impl RippleMessage {
    pub fn new(from_node: String, timestamp: Instant, message: Box<dyn Message>) -> Box<Self> {
        Box::from(RippleMessage { from_node, timestamp, message, start: None })
    }

    fn set_start(&mut self, start: Instant) {
        self.start = Option::from(start);
    }
}

impl Display for RippleMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let from_node_buf = &self.from_node;
        let time_since = if self.start.is_some() {
            self.timestamp.duration_since(self.start.unwrap()).as_millis()
        } else {
            0
        };
        let message_buf = self.message.descriptor().name();
        write!(f, "{} {} sent {}\n", time_since, from_node_buf, message_buf)
    }
}
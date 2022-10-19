use tokio::sync::broadcast::Sender;

use super::Flags;

pub const MESSAGE_TIMEOUT: usize = 10_000;

pub(super) struct TimeoutChecker {
    message_count: usize,
    sender: Sender<Flags>,
}

impl TimeoutChecker {
    pub fn new(sender: Sender<Flags>) -> Self {
        Self {
            message_count: 0,
            sender,
        }
    }

    pub fn check(&mut self) {
        self.message_count += 1;
        if self.message_count > MESSAGE_TIMEOUT {
            self.sender.send(Flags::Timeout).unwrap();
        }
    }
}

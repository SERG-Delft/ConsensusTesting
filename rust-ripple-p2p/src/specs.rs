use tokio::sync::broadcast::Sender;

use crate::message_handler::RippleMessageObject;
use std::collections::HashMap;
use std::fmt::Display;

use self::incompatible_ledger::IncompatibleLedgerChecker;
use self::insufficient_support::InsufficientSupportChecker;
use self::timeout::TimeoutChecker;

mod incompatible_ledger;
mod insufficient_support;
mod timeout;

#[derive(Clone, Debug)]
pub enum Flags {
    Timeout,
    InsufficientSupport(usize),
    IncompatibleLedger(String),
}

impl Display for Flags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Flags::Timeout => write!(f, "Timeout after {} messages", timeout::MESSAGE_TIMEOUT),
            Flags::InsufficientSupport(sequence) => {
                write!(
                    f,
                    "Ledger {} diverged and has insufficient support",
                    sequence
                )
            }
            Flags::IncompatibleLedger(log) => {
                write!(f, "Incompatible ledger <{}>", log)
            }
        }
    }
}

pub struct SpecChecker {
    timeout: TimeoutChecker,
    insufficient_support: InsufficientSupportChecker,
    #[allow(dead_code)] // contains tasks that need to be dropped with specchecker
    incompatible_ledger: IncompatibleLedgerChecker,
}

impl SpecChecker {
    pub async fn new(public_key_to_index: HashMap<String, usize>, sender: Sender<Flags>) -> Self {
        let mut incompatible_ledger = IncompatibleLedgerChecker::new(sender.clone());
        incompatible_ledger.attach().await;

        Self {
            timeout: TimeoutChecker::new(sender.clone()),
            insufficient_support: InsufficientSupportChecker::new(
                public_key_to_index,
                sender.clone(),
            ),
            incompatible_ledger,
        }
    }

    pub fn check(&mut self, sender: usize, message: RippleMessageObject) -> () {
        self.timeout.check();
        self.insufficient_support.check(sender, message);
    }
}

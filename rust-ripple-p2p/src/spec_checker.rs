use crate::utils::public_key_to_b58;
use crate::{deserialization::parse2, message_handler::RippleMessageObject};
use std::collections::HashMap;
use std::fmt::Display;

const MESSAGE_TIMEOUT: usize = 100_000;

type Result = core::result::Result<(), Status>;

pub enum Status {
    Timeout,
    Liveness,
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::Timeout => write!(f, "Timeout after {} messages", MESSAGE_TIMEOUT),
            Status::Liveness => write!(f, "Liveness"),
        }
    }
}

pub struct SpecChecker {
    message_count: usize,
    validation_history: HashMap<usize, [Option<String>; 7]>,
    public_key_to_index: HashMap<String, usize>,
}

impl SpecChecker {
    pub fn new(public_key_to_index: HashMap<String, usize>) -> Self {
        Self {
            message_count: 0,
            validation_history: HashMap::new(),
            public_key_to_index,
        }
    }

    pub fn check(&mut self, sender: usize, message: RippleMessageObject) -> Result {
        self.check_insufficient_support_after_fork(sender, message)?;
        self.check_timeout()?;
        Ok(())
    }

    fn check_insufficient_support_after_fork(
        &mut self,
        sender: usize,
        message: RippleMessageObject,
    ) -> Result {
        if let RippleMessageObject::TMValidation(ref validation) = message {
            let validation = match parse2(validation.get_validation()) {
                Ok((_, validation)) => validation,
                Err(_) => return Ok(()),
            };

            let sequence = validation["LedgerSequence"].as_usize().unwrap();
            self.validation_history
                .entry(sequence)
                .or_insert([None, None, None, None, None, None, None]);

            let public_key = hex::decode(validation["SigningPubKey"].as_str().unwrap()).unwrap();
            let public_key_b58 = public_key_to_b58(public_key.as_slice());
            let process_index = *self.public_key_to_index.get(&public_key_b58).unwrap();
            let hashes = self.validation_history.get_mut(&sequence).unwrap();

            if process_index != sender || sender == 3 || hashes[process_index].is_some() {
                return Ok(());
            }

            hashes[process_index] = Some(validation["hash"].as_str().unwrap().to_owned());

            if hashes[0].eq(&hashes[1])
                && hashes[0].eq(&hashes[2])
                && hashes[4].eq(&hashes[5])
                && hashes[4].eq(&hashes[6])
                && !hashes[0].eq(&hashes[6])
                && hashes[0].is_some()
                && hashes[6].is_some()
            {
                println!("validation {}", validation);
                println!("history[{}] = {:?}", sequence, hashes);
                return Err(Status::Liveness);
            } else {
                return Ok(());
            }
        };
        Ok(())
    }

    fn check_timeout(&mut self) -> Result {
        self.message_count += 1;
        if self.message_count > MESSAGE_TIMEOUT {
            Err(Status::Timeout)
        } else {
            Ok(())
        }
    }
}

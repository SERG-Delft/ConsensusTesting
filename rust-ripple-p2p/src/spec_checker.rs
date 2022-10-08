use crate::utils::public_key_to_b58;
use crate::{deserialization::parse2, message_handler::RippleMessageObject};
use std::collections::HashMap;

const MESSAGE_TIMEOUT: usize = 10_000;

type Result = core::result::Result<(), Status>;

pub enum Status {
    Timeout,
    Liveness
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

    pub fn check(&mut self, message: RippleMessageObject) -> Result {
        self.check_insufficient_support_after_fork(message)?;
        self.check_timeout()?;
        Ok(())
    }

    fn check_insufficient_support_after_fork(&mut self, message: RippleMessageObject) -> Result {
        match message {
            RippleMessageObject::TMValidation(ref validation) => {
                let validation = match parse2(validation.get_validation()) {
                    Ok((_, validation)) => validation,
                    Err(_) => return Ok(()),
                };

                let sequence = validation["LedgerSequence"].as_usize().unwrap();
                if !self.validation_history.contains_key(&sequence) {
                    self.validation_history
                        .insert(sequence, [None, None, None, None, None, None, None]);
                }

                let public_key =
                    hex::decode(validation["SigningPubKey"].as_str().unwrap()).unwrap();
                let public_key_b58 = public_key_to_b58(public_key.as_slice());
                let process_index = *self.public_key_to_index.get(&public_key_b58).unwrap();
                let hashes = self.validation_history.get_mut(&sequence).unwrap();

                hashes[process_index] = Some(validation["hash"].as_str().unwrap().to_owned());

                // println!("history[{}] = {:?}", sequence, hashes);

                if hashes[0].eq(&hashes[1]) && hashes[0].eq(&hashes[2]) &&
                    hashes[4].eq(&hashes[5]) && hashes[4].eq(&hashes[6]) &&
                    !hashes[0].eq(&hashes[6]) {
                    return Err(Status::Liveness)
                } else {
                    return Ok(())
                }
            }
            _ => {}
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

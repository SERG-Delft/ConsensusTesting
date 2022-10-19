use std::collections::HashMap;

use tokio::sync::broadcast::Sender;

use crate::deserialization::parse2;
use crate::message_handler::RippleMessageObject;
use crate::utils::public_key_to_b58;

use super::Flags;

pub(super) struct InsufficientSupportChecker {
    validation_history: HashMap<usize, [Option<String>; 7]>,
    public_key_to_index: HashMap<String, usize>,
    sender: Sender<Flags>,
}

impl InsufficientSupportChecker {
    pub fn new(public_key_to_index: HashMap<String, usize>, sender: Sender<Flags>) -> Self {
        Self {
            validation_history: HashMap::new(),
            public_key_to_index,
            sender,
        }
    }

    pub fn check(&mut self, sender: usize, message: RippleMessageObject) -> () {
        if let RippleMessageObject::TMValidation(ref validation) = message {
            let validation = match parse2(validation.get_validation()) {
                Ok((_, validation)) => validation,
                Err(_) => return (),
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
                return ();
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
                self.sender
                    .send(Flags::InsufficientSupport(sequence))
                    .unwrap();
            }
        };
    }
}

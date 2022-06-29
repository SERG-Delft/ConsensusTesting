use itertools::Itertools;
use std::cmp::max;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

use crate::deserialization::parse2;
use crate::message_handler::{from_bytes, invoke_protocol_message, RippleMessageObject};
use crate::protos::ripple::NodeEvent;
use crate::scheduler::Event;
use crate::toxiproxy::ToxiproxyClient;
use rand::prelude::*;
use set_partitions::{set_partitions, ArrayVecSetPartition, HashSubsets};
use tokio::time::sleep;
use RippleMessageObject::{TMProposeSet, TMStatusChange, TMTransaction, TMValidation};

#[derive(Debug)]
pub struct ByzzFuzz {
    n: usize, // number of processes
    d: usize, // bound on the #rounds with network faults
    r: usize, // bound on the #rounds with faults
    current_index: usize,
    current_round: usize,
    applied_partitions: bool,
    network_faults: HashMap<usize, Vec<HashSet<u8>>>,
    toxiproxy: Arc<ToxiproxyClient>,
}

impl ByzzFuzz {
    pub fn new(n: usize, d: usize, r: usize) -> Self {
        let mut network_faults = HashMap::with_capacity(d);
        (0..d)
            .map(|_| NetworkFault::sample_network_fault(n, r))
            .for_each(|fault| {
                network_faults.insert(fault.round, fault.partition);
            });
        Self {
            n,
            d,
            r,
            current_index: 0,
            current_round: 0,
            applied_partitions: true,
            network_faults,
            toxiproxy: Arc::new(ToxiproxyClient::new("http://localhost:8474/")),
        }
    }

    pub async fn on_message(&mut self, event: Event) -> Event {
        let mut message = from_bytes(&event.message);
        self.update_round(&message).await;
        self.apply_partition().await;
        event
    }

    async fn update_round(&mut self, message: &RippleMessageObject) {
        if let Some(index) = self.get_index(message) {
            if self.current_index < index {
                self.current_index = index;
            }
        }
        if let Some(round) = self.get_round(message) {
            if self.current_round < round {
                println!("round {}", round);
                self.current_round = round;
                self.applied_partitions = false;
            }
        }
    }

    fn get_index(&self, message: &RippleMessageObject) -> Option<usize> {
        match message {
            TMStatusChange(status) => {
                let index = status.get_ledgerSeq() as usize;
                if index < 5 {
                    return None;
                }
                match status.get_newEvent() {
                    NodeEvent::neACCEPTED_LEDGER => Some(index - 5),
                    NodeEvent::neCLOSING_LEDGER => Some(index + 1 - 5),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn get_round(&self, message: &RippleMessageObject) -> Option<usize> {
        match message {
            TMProposeSet(_) => Some(self.current_index * 2),
            TMValidation(validation) => {
                let (_, validation) = parse2(validation.get_validation()).unwrap();
                if validation["LedgerSequence"].as_usize().unwrap() != self.current_index + 5 {
                    return None;
                }
                Some(self.current_index * 2 + 1)
            }
            _ => None,
        }
    }

    async fn apply_partition(&mut self) {
        if !self.applied_partitions && self.network_faults.contains_key(&self.current_round) {
            self.applied_partitions = true;
            self.toxiproxy
                .partition(self.network_faults.get(&self.current_round).unwrap())
                .await;
        } else if !self.applied_partitions {
            self.applied_partitions = true;
            self.toxiproxy.partition(&vec![(0..self.n as u8).collect()]).await;
        }
    }
}

#[derive(Debug)]
struct NetworkFault {
    round: usize,
    partition: Vec<HashSet<u8>>,
}

impl NetworkFault {
    pub fn sample_network_fault(n: usize, r: usize) -> Self {
        assert!(n <= 16);

        let bell_number = set_partitions(n).unwrap();
        let partition = thread_rng().gen_range(0..bell_number);

        let mut partitions: ArrayVecSetPartition<u8, HashSubsets<u8>, 16> =
            ArrayVecSetPartition::try_with_size(n).unwrap();
        for _ in 0..partition {
            partitions.increment();
        }

        Self {
            round: thread_rng().gen_range(1..=r),
            partition: partitions.subsets().subsets().to_vec(),
        }
    }
}

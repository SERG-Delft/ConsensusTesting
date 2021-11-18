use std::collections::HashMap;
use std::ops::{Add, Div, Sub};
use std::sync::mpsc::{Receiver, RecvError, Sender};
use chrono::Duration;
use genevo::encoding::ValueEncoded;
use genevo::ga::genetic_algorithm;
use genevo::genetic::{Fitness, FitnessFunction, Genotype, Phenotype};
use itertools::{chain};
use genevo::prelude;
use genevo::prelude::simulate;

const NUM_NODES: usize = 5;

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
enum MessageType {
    TMProposeSet,
    TMStatusChange,
    TMTransaction,
    TMHaveTransactionSet
}

impl MessageType {
    const VALUES: [Self; 4] = [Self::TMProposeSet, Self::TMStatusChange, Self::TMTransaction, Self::TMHaveTransactionSet];
}

// The phenotype
type DelayMap = HashMap<usize, HashMap<usize, HashMap<MessageType, u32>>>;

#[derive(Clone, Debug)]
pub struct DelayMapPhenotype {
    delay_map: DelayMap,
    delays: DelaysGenotype
}

impl Phenotype<DelaysGenotype> for DelayMapPhenotype {
    fn genes(&self) -> DelaysGenotype {
        self.delays.clone()
    }

    fn derive(&self, new_genes: DelaysGenotype) -> Self {
        let index_factor_1 = MessageType::VALUES.len() * (NUM_NODES-1);
        let index_factor_2 = MessageType::VALUES.len();
        let mut from_node = HashMap::new();
        for i in 0..NUM_NODES {
            let mut to_node = HashMap::new();
            for (j, node) in chain(0..i, i+1..NUM_NODES).enumerate() {
                let mut message_type = HashMap::new();
                for (k, message) in MessageType::VALUES.iter().enumerate() {
                    message_type.insert(*message, self[index_factor_1 * i + index_factor_2 * j + k]);
                }
                to_node.insert(node+1, message_type.clone());
            }
            from_node.insert(i+1, to_node.clone());
        }
        Self {
            delay_map: from_node,
            delays: new_genes
        }
    }
}

// The genotype
type Delays = Vec<u32>;

#[derive(Clone, Debug, PartialEq)]
pub struct DelaysGenotype {
    delays: Delays
}

impl Genotype for DelaysGenotype {
    type Dna = Delays;
}

impl ValueEncoded for DelaysGenotype {}

/// Duration in ms from start of test case to validated ledger
#[derive(Eq, Ord, Clone, Debug, Sized)]
struct FitnessValue {
    time: Duration
}

impl Fitness for FitnessValue {
    fn zero() -> Self {
        FitnessValue { time: Duration::zero() }
    }

    fn abs_diff(&self, other: &Self) -> Self {
        let time: Duration = if self.time.sub(other.time) >= Duration::zero() {
            self.time.sub(other.time)
        } else {
            other.time.sub(self.time)
        };
        FitnessValue { time }
    }
}

#[derive(Clone)]
pub struct FitnessCalculation {
    sender: Sender<DelaysGenotype>,
    receiver: Receiver<Duration>
}

impl FitnessFunction<DelaysGenotype, FitnessValue> for FitnessCalculation {
    fn fitness_of(&self, a: &DelaysGenotype) -> FitnessValue {
        self.sender.send(a.clone()).expect("Schedule delay receiver failed");
        match self.receiver.recv() {
            Ok(time) => FitnessValue{ time },
            Err(_) => panic!("Schedule fitness sender failed")
        }
    }

    fn average(&self, a: &[FitnessValue]) -> FitnessValue {
        let mut sum = Duration::zero();
        for fitness in a {
            sum.add(fitness.time);
        }
        FitnessValue { time: sum.div(a.len() as i32) }
    }

    fn highest_possible_fitness(&self) -> FitnessValue {
        FitnessValue { time: Duration::seconds(60) }
    }

    fn lowest_possible_fitness(&self) -> FitnessValue {
        FitnessValue { time: Duration::seconds(3) }
    }
}

#[cfg(test)]
mod ga_tests {
    use std::collections::HashMap;
    use itertools::Itertools;
    use crate::genetic_algorithm::{AsPhenotype, Delays, MessageType};

    #[test]
    fn check_phenotype() {
        let genotype: Delays = (1..81).collect_vec();
        let phenotype = genotype.as_solution();
        println!("{:?}", phenotype as HashMap<usize, HashMap<usize, HashMap<MessageType, u32>>>);
    }
}

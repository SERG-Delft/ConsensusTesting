use std::collections::HashSet;

use rand::prelude::*;
use set_partitions::{ArrayVecSetPartition, HashSubsets, set_partitions};

#[derive(Debug)]
pub struct ByzzFuzz {
    n: usize, // number of processes
    d: usize, // bound on the #rounds with network faults
    r: usize, // bound on the #rounds with faults
    network_faults: Vec<NetworkFault>,
}

impl ByzzFuzz {
    pub fn new(n: usize, d: usize, r: usize) -> Self {
        let network_faults = (0..d)
            .map(|_| NetworkFault::sample_network_fault(n, r))
            .collect();
        Self {
            n,
            d,
            r,
            network_faults,
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
            round: thread_rng().gen_range(0..r),
            partition: partitions.subsets().subsets().to_vec(),
        }
    }
}

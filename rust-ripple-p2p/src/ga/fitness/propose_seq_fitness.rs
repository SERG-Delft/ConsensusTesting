use std::fmt::{Display, Formatter};
use std::sync::Arc;
use genevo::genetic::{AsScalar, Fitness};
use crate::ga::fitness::ExtendedFitness;
use crate::node_state::MutexNodeStates;
use crate::NUM_NODES;
use crate::test_harness::TestHarness;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ProposeSeqFitness {
    pub value: u32
}

impl ProposeSeqFitness {
    pub fn new(propose_seq: u32) -> Self {
        ProposeSeqFitness { value: propose_seq }
    }
}

impl Fitness for ProposeSeqFitness {
    fn zero() -> Self {
        Self { value: 0 }
    }

    #[allow(unstable_name_collisions)]
    fn abs_diff(&self, other: &Self) -> Self {
        let value = u32::abs_diff(self.value, other.value);
        Self { value }
    }
}

impl Display for ProposeSeqFitness {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ProposeSeqFitness: {}\n", self.value)
    }
}

impl ExtendedFitness for ProposeSeqFitness {
    fn average(a: &[Self]) -> Self {
        let mut sum = 0u32;
        for fitness in a {
            sum += fitness.value;
        }
        Self { value: sum / a.len() as u32 }
    }

    fn highest_possible_fitness() -> Self {
        Self { value: 60 }
    }

    fn lowest_possible_fitness() -> Self {
        Self { value: 0 }
    }

    /// Evenly distribute fitness value between propse sequence and number of bow outs.
    fn run_harness(test_harness: &mut TestHarness<'static>, node_states: Arc<MutexNodeStates>) -> Self {
        node_states.clear_highest_propose_seq();
        let liveness = test_harness.schedule_transactions(node_states.clone());
        if liveness {
            let (propose_sequence, bow_outs) = node_states.get_highest_propose_seq();
            Self::new(propose_sequence * NUM_NODES.clone() as u32 + bow_outs)
        } else {
            Self::zero()
        }
    }
}

impl AsScalar for ProposeSeqFitness {
    fn as_scalar(&self) -> f64 {
        self.value as f64
    }
}
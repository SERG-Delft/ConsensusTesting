use std::fmt::{Display, Formatter};
use std::sync::Arc;
use genevo::genetic::{AsScalar, Fitness};
use crate::ga::fitness::ExtendedFitness;
use crate::node_state::MutexNodeStates;
use crate::test_harness::TestHarness;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FailedConsensusFitness {
    pub value: u32
}

impl FailedConsensusFitness {
    pub fn new(value: u32) -> Self {
        Self { value }
    }
}

impl Fitness for FailedConsensusFitness {
    fn zero() -> Self {
        Self { value: 0 }
    }

    #[allow(unstable_name_collisions)]
    fn abs_diff(&self, other: &Self) -> Self {
        let value = u32::abs_diff(&self.value, &other.value);
        Self { value }
    }
}

impl Display for FailedConsensusFitness {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "FailedConsensusFitness: {}\n", self.value)
    }
}

impl ExtendedFitness for FailedConsensusFitness {
    fn average(a: &[Self]) -> Self {
        let mut sum = 0u32;
        for fitness in a {
            sum += fitness.value;
        }
        Self { value: sum / a.len() as u32 }
    }

    fn highest_possible_fitness() -> Self {
        Self { value: 100 }
    }

    fn lowest_possible_fitness() -> Self {
        Self { value: 0 }
    }

    fn run_harness(test_harness: TestHarness<'static>, node_states: Arc<MutexNodeStates>) -> Self {
        node_states.clear_number_of_failed_consensus_rounds();
        test_harness.schedule_transactions(node_states.clone());
        Self::new(node_states.get_total_number_of_failed_consensus_rounds())
    }
}

impl AsScalar for FailedConsensusFitness {
    fn as_scalar(&self) -> f64 {
        self.value as f64
    }
}
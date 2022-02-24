use std::fmt::{Display, Formatter};
use std::sync::Arc;
use genevo::genetic::{AsScalar, Fitness};
use crate::ga::fitness::ExtendedFitness;
use crate::ga::genetic_algorithm::Parameter;
use crate::node_state::MutexNodeStates;
use crate::test_harness::TestHarness;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct DelayFitness {
    pub value: u32
}

impl DelayFitness {
    pub fn new(value: u32) -> Self {
        Self { value }
    }
}

impl Fitness for DelayFitness {
    fn zero() -> Self {
        Self { value: 0 }
    }

    #[allow(unstable_name_collisions)]
    fn abs_diff(&self, other: &Self) -> Self {
        Self { value: u32::abs_diff(&self.value, &other.value) }
    }
}

impl Display for DelayFitness {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "DelayFitness: {}\n", self.value)
    }
}

impl ExtendedFitness for DelayFitness {
    fn average(a: &[Self]) -> Self {
        Self { value: a.iter().map(|x| x.value).sum::<u32>() / a.len() as u32 }
    }

    fn highest_possible_fitness() -> Self {
        Self { value: 0 }
    }

    fn lowest_possible_fitness() -> Self {
        Self { value: (Parameter::num_genes() * 1000) as u32 }
    }

    fn run_harness(test_harness: TestHarness<'static>, node_states: Arc<MutexNodeStates>) -> Self {
        let result = Self::new(node_states.get_current_delays().iter().sum::<u32>());
        test_harness.schedule_transactions(node_states);
        result
    }
}

impl AsScalar for DelayFitness {
    fn as_scalar(&self) -> f64 {
        self.value as f64
    }
}
use std::fmt::{Display, Formatter};
use std::ops::{Add, Div, Sub};
use std::sync::Arc;
use std::time::{Duration as TimeDuration, Instant};
use genevo::genetic::{AsScalar, Fitness};
use crate::ga::fitness::ExtendedFitness;
use crate::node_state::MutexNodeStates;
use crate::test_harness::TestHarness;

/// Duration in ms from start of test case to validated ledger with all transactions
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TimeFitness {
    pub value: TimeDuration
}

impl TimeFitness {
    pub fn new(duration: TimeDuration) -> Self {
        TimeFitness { value: duration }
    }
}

impl Fitness for TimeFitness {
    fn zero() -> Self {
        TimeFitness { value: TimeDuration::default() }
    }

    fn abs_diff(&self, other: &Self) -> Self {
        let time: TimeDuration = if self.value.sub(other.value) >= TimeDuration::default() {
            self.value.sub(other.value)
        } else {
            other.value.sub(self.value)
        };
        TimeFitness { value: time }
    }
}

impl Display for TimeFitness {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "TimeFitness: {:?}\n", self.value)
    }
}

impl ExtendedFitness for TimeFitness {
    fn average(a: &[Self]) -> Self {
        let mut sum = Self::zero().value;
        for fitness in a {
            sum = sum.add(fitness.value);
        }
        TimeFitness { value: sum.div(a.len() as u32) }
    }

    fn highest_possible_fitness() -> Self {
        TimeFitness { value: TimeDuration::from_secs(60) }
    }

    fn lowest_possible_fitness() -> Self {
        TimeFitness { value: TimeDuration::from_secs(0) }
    }

    fn run_harness(test_harness: &mut TestHarness<'static>, node_states: Arc<MutexNodeStates>) -> Self {
        let start = Instant::now();
        test_harness.schedule_transactions(node_states);
        Self::new(Instant::now().duration_since(start))
    }
}

impl AsScalar for TimeFitness {
    fn as_scalar(&self) -> f64 {
        self.value.as_millis() as f64
    }
}
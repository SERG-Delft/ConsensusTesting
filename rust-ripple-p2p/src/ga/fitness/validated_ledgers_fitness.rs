use std::fmt::{Display, Formatter};
use std::sync::Arc;
use genevo::genetic::{AsScalar, Fitness};
use crate::ga::fitness::ExtendedFitness;
use crate::node_state::MutexNodeStates;
use crate::test_harness::TestHarness;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ValidatedLedgersFitness {
    pub value: u32
}

impl ValidatedLedgersFitness {
    pub fn new(ledgers: u32) -> Self {
        ValidatedLedgersFitness { value: ledgers }
    }
}

impl Fitness for ValidatedLedgersFitness {
    fn zero() -> Self {
        Self { value: 0 }
    }

    #[allow(unstable_name_collisions)]
    fn abs_diff(&self, other: &Self) -> Self {
        let value = u32::abs_diff(self.value, other.value);
        Self { value }
    }
}

impl Display for ValidatedLedgersFitness {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ValidatedLedgersFitness: {}\n", self.value)
    }
}

impl ExtendedFitness for ValidatedLedgersFitness {
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

    fn run_harness(test_harness: TestHarness<'static>, node_states: Arc<MutexNodeStates>) -> Self {
        let start = node_states.min_validated_ledger();
        test_harness.schedule_transactions(node_states.clone());
        Self::new(node_states.min_validated_ledger() - start)
    }
}

impl AsScalar for ValidatedLedgersFitness {
    fn as_scalar(&self) -> f64 {
        self.value as f64
    }
}
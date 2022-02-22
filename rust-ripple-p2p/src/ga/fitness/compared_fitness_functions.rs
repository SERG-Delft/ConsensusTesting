use std::fmt::{Display, Formatter};
use std::sync::Arc;
use std::time::{Duration as TimeDuration, Instant};
use genevo::genetic::{AsScalar, Fitness};
use crate::ga::fitness::delay_fitness::DelayFitness;
use crate::ga::fitness::ExtendedFitness;
use crate::ga::fitness::failed_consensus_fitness::FailedConsensusFitness;
use crate::ga::fitness::time_fitness::TimeFitness;
use crate::ga::fitness::validated_ledgers_fitness::ValidatedLedgersFitness;
use crate::node_state::MutexNodeStates;
use crate::test_harness::TestHarness;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ComparedFitnessFunctions {
    failed_consensus_fitness: FailedConsensusFitness,
    validated_ledgers_fitness: ValidatedLedgersFitness,
    time_fitness: TimeFitness,
    delay_fitness: DelayFitness,
}

impl ComparedFitnessFunctions {
    pub fn new(value1: u32, value2: u32, value3: TimeDuration, value4: u32) -> Self {
        Self {
            failed_consensus_fitness: FailedConsensusFitness::new(value1),
            validated_ledgers_fitness: ValidatedLedgersFitness::new(value2),
            time_fitness: TimeFitness::new(value3),
            delay_fitness: DelayFitness::new(value4),
        }
    }
}

impl Fitness for ComparedFitnessFunctions {
    fn zero() -> Self {
        Self {
            failed_consensus_fitness: FailedConsensusFitness::new(0),
            validated_ledgers_fitness: ValidatedLedgersFitness::new(0),
            time_fitness: TimeFitness::new(TimeDuration::default()),
            delay_fitness: DelayFitness::new(0),
        }
    }

    fn abs_diff(&self, other: &Self) -> Self {
        let failed_consensus_fitness = self.failed_consensus_fitness.abs_diff(&other.failed_consensus_fitness);
        let validated_ledgers_fitness = self.validated_ledgers_fitness.abs_diff(&other.validated_ledgers_fitness);
        let time_fitness = self.time_fitness.abs_diff(&other.time_fitness);
        let delay_fitness = self.delay_fitness.abs_diff(&other.delay_fitness);
        Self { failed_consensus_fitness, validated_ledgers_fitness, time_fitness, delay_fitness }
    }
}

impl Display for ComparedFitnessFunctions {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}{}{}", self.failed_consensus_fitness, self.validated_ledgers_fitness, self.time_fitness, self.delay_fitness)
    }
}

impl ExtendedFitness for ComparedFitnessFunctions {
    fn average(a: &[Self]) -> Self {
        let failed_consensus_fitness = FailedConsensusFitness::average(a.iter().map(|x| x.failed_consensus_fitness.clone()).collect::<Vec<FailedConsensusFitness>>().as_slice());
        let validated_ledgers_fitness = ValidatedLedgersFitness::average(a.iter().map(|x| x.validated_ledgers_fitness.clone()).collect::<Vec<ValidatedLedgersFitness>>().as_slice());
        let time_fitness = TimeFitness::average(a.iter().map(|x| x.time_fitness.clone()).collect::<Vec<TimeFitness>>().as_slice());
        let delay_fitness = DelayFitness::average(a.iter().map(|x| x.delay_fitness.clone()).collect::<Vec<DelayFitness>>().as_slice());
        Self { failed_consensus_fitness, validated_ledgers_fitness, time_fitness, delay_fitness }
    }

    fn highest_possible_fitness() -> Self {
        Self {
            failed_consensus_fitness: FailedConsensusFitness::highest_possible_fitness(),
            validated_ledgers_fitness: ValidatedLedgersFitness::highest_possible_fitness(),
            time_fitness: TimeFitness::highest_possible_fitness(),
            delay_fitness: DelayFitness::highest_possible_fitness(),
        }
    }

    fn lowest_possible_fitness() -> Self {
        Self {
            failed_consensus_fitness: FailedConsensusFitness::lowest_possible_fitness(),
            validated_ledgers_fitness: ValidatedLedgersFitness::lowest_possible_fitness(),
            time_fitness: TimeFitness::lowest_possible_fitness(),
            delay_fitness: DelayFitness::lowest_possible_fitness(),
        }
    }

    fn run_harness(test_harness: TestHarness<'static>, node_states: Arc<MutexNodeStates>) -> Self {
        node_states.clear_number_of_failed_consensus_rounds();
        let start_validated_ledgers = node_states.node_states.lock().min_validated_ledger();
        let start_time = Instant::now();
        test_harness.schedule_transactions(node_states.clone());
        Self::new(
            node_states.get_total_number_of_failed_consensus_rounds(),
            node_states.min_validated_ledger() - start_validated_ledgers,
            Instant::now().duration_since(start_time),
            node_states.get_current_delays().iter().sum::<u32>(),
        )
    }
}

impl AsScalar for ComparedFitnessFunctions {
    fn as_scalar(&self) -> f64 {
        &self.validated_ledgers_fitness.as_scalar() +
            &self.failed_consensus_fitness.as_scalar() +
            &self.time_fitness.as_scalar() +
            &self.delay_fitness.as_scalar() /
                4.0 as f64
    }
}
use std::fmt::{Display, Formatter};
use std::sync::Arc;
use std::time::{Duration as TimeDuration, Instant};
use genevo::genetic::{AsScalar, Fitness};
use crate::ga::fitness::delay_fitness::DelayFitness;
use crate::ga::fitness::{ExtendedFitness};
use crate::ga::fitness::failed_consensus_fitness::FailedConsensusFitness;
use crate::ga::fitness::state_accounting_fitness::StateAccountFitness;
use crate::ga::fitness::time_fitness::TimeFitness;
use crate::ga::fitness::validated_ledgers_fitness::ValidatedLedgersFitness;
use crate::node_state::MutexNodeStates;
use crate::test_harness::TestHarness;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ComparedFitnessFunctions {
    pub failed_consensus_fitness: FailedConsensusFitness,
    pub validated_ledgers_fitness: ValidatedLedgersFitness,
    pub time_fitness: TimeFitness,
    pub delay_fitness: DelayFitness,
    pub state_accounting_fitness: StateAccountFitness,
}

impl ComparedFitnessFunctions {
    pub fn new(value1: u32, value2: u32, value3: TimeDuration, value4: u32, value5: u32, value6: u32) -> Self {
        Self {
            failed_consensus_fitness: FailedConsensusFitness::new(value1),
            validated_ledgers_fitness: ValidatedLedgersFitness::new(value2),
            time_fitness: TimeFitness::new(value3),
            delay_fitness: DelayFitness::new(value4),
            state_accounting_fitness: StateAccountFitness::new(value5, value6),
        }
    }
}

impl Fitness for ComparedFitnessFunctions {
    fn zero() -> Self {
        Self {
            failed_consensus_fitness: FailedConsensusFitness::zero(),
            validated_ledgers_fitness: ValidatedLedgersFitness::zero(),
            time_fitness: TimeFitness::zero(),
            delay_fitness: DelayFitness::zero(),
            state_accounting_fitness: StateAccountFitness::zero(),
        }
    }

    fn abs_diff(&self, other: &Self) -> Self {
        let failed_consensus_fitness = self.failed_consensus_fitness.abs_diff(&other.failed_consensus_fitness);
        let validated_ledgers_fitness = self.validated_ledgers_fitness.abs_diff(&other.validated_ledgers_fitness);
        let time_fitness = self.time_fitness.abs_diff(&other.time_fitness);
        let delay_fitness = self.delay_fitness.abs_diff(&other.delay_fitness);
        let state_accounting_fitness = self.state_accounting_fitness.abs_diff(&other.state_accounting_fitness);
        Self { failed_consensus_fitness, validated_ledgers_fitness, time_fitness, delay_fitness, state_accounting_fitness }
    }
}

impl Display for ComparedFitnessFunctions {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}{}{}{}", self.failed_consensus_fitness, self.validated_ledgers_fitness, self.time_fitness, self.delay_fitness, self.state_accounting_fitness)
    }
}

impl ExtendedFitness for ComparedFitnessFunctions {
    fn average(a: &[Self]) -> Self {
        let failed_consensus_fitness = FailedConsensusFitness::average(a.iter().map(|x| x.failed_consensus_fitness.clone()).collect::<Vec<FailedConsensusFitness>>().as_slice());
        let validated_ledgers_fitness = ValidatedLedgersFitness::average(a.iter().map(|x| x.validated_ledgers_fitness.clone()).collect::<Vec<ValidatedLedgersFitness>>().as_slice());
        let time_fitness = TimeFitness::average(a.iter().map(|x| x.time_fitness.clone()).collect::<Vec<TimeFitness>>().as_slice());
        let delay_fitness = DelayFitness::average(a.iter().map(|x| x.delay_fitness.clone()).collect::<Vec<DelayFitness>>().as_slice());
        let state_accounting_fitness = StateAccountFitness::average(a.iter().map(|x| x.state_accounting_fitness.clone()).collect::<Vec<StateAccountFitness>>().as_slice());
        Self { failed_consensus_fitness, validated_ledgers_fitness, time_fitness, delay_fitness, state_accounting_fitness }
    }

    fn highest_possible_fitness() -> Self {
        Self {
            failed_consensus_fitness: FailedConsensusFitness::highest_possible_fitness(),
            validated_ledgers_fitness: ValidatedLedgersFitness::highest_possible_fitness(),
            time_fitness: TimeFitness::highest_possible_fitness(),
            delay_fitness: DelayFitness::highest_possible_fitness(),
            state_accounting_fitness: StateAccountFitness::highest_possible_fitness(),
        }
    }

    fn lowest_possible_fitness() -> Self {
        Self {
            failed_consensus_fitness: FailedConsensusFitness::lowest_possible_fitness(),
            validated_ledgers_fitness: ValidatedLedgersFitness::lowest_possible_fitness(),
            time_fitness: TimeFitness::lowest_possible_fitness(),
            delay_fitness: DelayFitness::lowest_possible_fitness(),
            state_accounting_fitness: StateAccountFitness::lowest_possible_fitness(),
        }
    }

    fn run_harness(test_harness: &mut TestHarness<'static>, node_states: Arc<MutexNodeStates>) -> Self {
        let before_server_states = StateAccountFitness::update_server_states(node_states.clone(), &test_harness);
        node_states.clear_number_of_failed_consensus_rounds();
        let start_validated_ledgers = node_states.node_states.lock().min_validated_ledger();
        let start_time = Instant::now();
        test_harness.schedule_transactions(node_states.clone());
        let failed_consensus_fitness = node_states.get_total_number_of_failed_consensus_rounds();
        let validated_ledgers_fitness = node_states.min_validated_ledger() - start_validated_ledgers;
        let time_fitness = Instant::now().duration_since(start_time);
        let delay_fitness = node_states.get_current_delays().iter().sum::<u32>();
        let after_server_states = StateAccountFitness::update_server_states(node_states, &test_harness);
        let state_accounting_fitness = StateAccountFitness::calculate_fitness(before_server_states, after_server_states);
        Self::new(
            failed_consensus_fitness,
            validated_ledgers_fitness,
            time_fitness,
            delay_fitness,
            state_accounting_fitness.not_full_duration,
            state_accounting_fitness.not_full_transitions,
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
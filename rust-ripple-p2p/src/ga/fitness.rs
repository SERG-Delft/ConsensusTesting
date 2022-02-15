#![allow(dead_code)]
use std::collections::HashMap;
use std::ops::{Add, Div, Sub};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration as TimeDuration};
use chrono::Duration;
use genevo::genetic::{AsScalar, Fitness, FitnessFunction};
use std::fmt::{Display, Formatter};
use log::debug;
use crate::ga::genetic_algorithm::{DelayMapPhenotype, DelaysGenotype, Parameter};
use crate::node_state::MutexNodeStates;
use crate::test_harness::TestHarness;

pub trait ExtendedFitness: Fitness + AsScalar + Clone + Send + Sync + Display {
    fn average(a: &[Self]) -> Self;

    fn highest_possible_fitness() -> Self;

    fn lowest_possible_fitness() -> Self;
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct ComparedFitnessFunctions {
    failed_consensus_fitness: FailedConsensusFitness,
    validated_ledgers_fitness: ValidatedLedgersFitness,
    time_fitness: TimeFitness,
    delay_fitness: DelayFitness,
}

impl ComparedFitnessFunctions {
    pub fn new(value1: u32, value2: u32, value3: Duration, value4: u32) -> Self {
        Self {
            failed_consensus_fitness: FailedConsensusFitness::new(value1),
            validated_ledgers_fitness: ValidatedLedgersFitness::new(value2),
            time_fitness: TimeFitness::new(value3),
            delay_fitness: DelayFitness::new(value4),
        }
    }

    pub fn run_harness(test_harness: TestHarness<'static>, node_states: Arc<MutexNodeStates>) -> Self {
        node_states.clear_number_of_failed_consensus_rounds();
        let start_validated_ledgers = node_states.node_states.lock().min_validated_ledger();
        let start_time = chrono::Utc::now();
        test_harness.schedule_transactions(node_states.clone());
        Self::new(
            node_states.get_total_number_of_failed_consensus_rounds(),
            node_states.min_validated_ledger() - start_validated_ledgers,
            chrono::Utc::now().signed_duration_since(start_time),
            node_states.get_current_delays().iter().sum::<u32>(),
        )
    }
}

impl Fitness for ComparedFitnessFunctions {
    fn zero() -> Self {
        Self {
            failed_consensus_fitness: FailedConsensusFitness::new(0),
            validated_ledgers_fitness: ValidatedLedgersFitness::new(0),
            time_fitness: TimeFitness::new(Duration::zero()),
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

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct FailedConsensusFitness {
    value: u32
}

impl FailedConsensusFitness {
    pub fn new(value: u32) -> Self {
        Self { value }
    }

    pub fn run_harness(test_harness: TestHarness<'static>, node_states: Arc<MutexNodeStates>) -> Self {
        node_states.clear_number_of_failed_consensus_rounds();
        test_harness.schedule_transactions(node_states.clone());
        Self::new(node_states.get_total_number_of_failed_consensus_rounds())
    }
}

impl Fitness for FailedConsensusFitness {
    fn zero() -> Self {
        Self { value: 0 }
    }

    fn abs_diff(&self, other: &Self) -> Self {
        let value = u32::abs_diff(&self.value, &other.value);
        Self { value }
    }
}

impl Display for FailedConsensusFitness {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Fitness value: {}\n", self.value)
    }
}

impl ExtendedFitness for FailedConsensusFitness {
    fn average(a: &[Self]) -> Self {
        let mut sum = 0u32;
        for fitness in a {
            sum = sum.add(fitness.value);
        }
        Self { value: sum.div(a.len() as u32) }
    }

    fn highest_possible_fitness() -> Self {
        Self { value: 100 }
    }

    fn lowest_possible_fitness() -> Self {
        Self { value: 0 }
    }
}

impl AsScalar for FailedConsensusFitness {
    fn as_scalar(&self) -> f64 {
        self.value as f64
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct ValidatedLedgersFitness {
    value: u32
}

impl ValidatedLedgersFitness {
    pub fn new(ledgers: u32) -> Self {
        ValidatedLedgersFitness { value: ledgers }
    }

    pub fn run_harness(test_harness: TestHarness<'static>, node_states: Arc<MutexNodeStates>) -> Self {
        let start = node_states.min_validated_ledger();
        test_harness.schedule_transactions(node_states.clone());
        Self::new(node_states.min_validated_ledger() - start)
    }
}

impl Fitness for ValidatedLedgersFitness {
    fn zero() -> Self {
        Self { value: 0 }
    }

    fn abs_diff(&self, other: &Self) -> Self {
        let value = u32::abs_diff(&self.value, &other.value);
        Self { value }
    }
}

impl Display for ValidatedLedgersFitness {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Fitness value: {}\n", self.value)
    }
}

impl ExtendedFitness for ValidatedLedgersFitness {
    fn average(a: &[Self]) -> Self {
        let mut sum = 0u32;
        for fitness in a {
            sum = sum.add(fitness.value);
        }
        Self { value: sum.div(a.len() as u32) }
    }

    fn highest_possible_fitness() -> Self {
        Self { value: 60 }
    }

    fn lowest_possible_fitness() -> Self {
        Self { value: 0 }
    }
}

impl AsScalar for ValidatedLedgersFitness {
    fn as_scalar(&self) -> f64 {
        self.value as f64
    }
}

/// Duration in ms from start of test case to validated ledger with all transactions
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct TimeFitness {
    value: Duration
}

impl TimeFitness {
    pub fn new(duration: Duration) -> Self {
        TimeFitness { value: duration }
    }

    pub fn run_harness(test_harness: TestHarness<'static>, node_states: Arc<MutexNodeStates>) -> Self {
        let start = chrono::Utc::now();
        test_harness.schedule_transactions(node_states);
        Self::new(chrono::Utc::now().signed_duration_since(start))
    }
}

impl Fitness for TimeFitness {
    fn zero() -> Self {
        TimeFitness { value: Duration::zero() }
    }

    fn abs_diff(&self, other: &Self) -> Self {
        let time: Duration = if self.value.sub(other.value) >= Duration::zero() {
            self.value.sub(other.value)
        } else {
            other.value.sub(self.value)
        };
        TimeFitness { value: time }
    }
}

impl Display for TimeFitness {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Fitness value: {}\n", self.value)
    }
}

impl ExtendedFitness for TimeFitness {
    fn average(a: &[Self]) -> Self {
        let mut sum = Self::zero().value;
        for fitness in a {
            sum = sum.add(fitness.value);
        }
        TimeFitness { value: sum.div(a.len() as i32) }
    }

    fn highest_possible_fitness() -> Self {
        TimeFitness { value: Duration::seconds(60) }
    }

    fn lowest_possible_fitness() -> Self {
        TimeFitness { value: Duration::seconds(2) }
    }
}

impl AsScalar for TimeFitness {
    fn as_scalar(&self) -> f64 {
        self.value.num_milliseconds() as f64
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct DelayFitness {
    pub value: u32
}

impl DelayFitness {
    pub fn new(value: u32) -> Self {
        Self { value }
    }

    pub fn run_harness(test_harness: TestHarness<'static>, node_states: Arc<MutexNodeStates>) -> Self {
        let result = Self::new(node_states.get_current_delays().iter().sum::<u32>());
        test_harness.schedule_transactions(node_states);
        result
    }
}

impl Fitness for DelayFitness {
    fn zero() -> Self {
        Self { value: 0 }
    }

    fn abs_diff(&self, other: &Self) -> Self {
        Self { value: u32::abs_diff(&self.value, &other.value) }
    }
}

impl Display for DelayFitness {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Fitness value: {}\n", self.value)
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
}

impl AsScalar for DelayFitness {
    fn as_scalar(&self) -> f64 {
        self.value as f64
    }
}

/// Fitness function communicates with scheduler handler for calculating and storing fitness of solutions.
#[derive(Clone, Debug)]
pub struct FitnessCalculation<T> where T: ExtendedFitness + Clone {
    pub(crate) fitness_values: Arc<RwLock<HashMap<DelaysGenotype, T>>>,
    pub(crate) sender: Sender<DelaysGenotype>,
}

impl<T> FitnessFunction<DelaysGenotype, T> for FitnessCalculation<T>
    where T: ExtendedFitness
{
    fn fitness_of(&self, delays_genotype: &DelaysGenotype) -> T {
        let mut sent_to_handler = false;
        loop {
            {
                match self.fitness_values.read().unwrap().get(delays_genotype) {
                    Some(fitness) => {
                        println!("Fitness found: {:?} for genotype: {:?}", fitness, delays_genotype);
                        return fitness.clone();
                    }
                    None => {
                        if !sent_to_handler {
                            println!("Fitness not found for genotype: {:?}", delays_genotype);
                            self.sender.send(delays_genotype.clone()).expect("Fitness calculator receiver failed");
                        }
                        sent_to_handler = true;
                    }
                }
            }
            thread::sleep(TimeDuration::from_millis(100));
        }
    }

    fn average(&self, a: &[T]) -> T {
        T::average(a)
    }

    fn highest_possible_fitness(&self) -> T {
        T::highest_possible_fitness()
    }

    fn lowest_possible_fitness(&self) -> T {
        T::lowest_possible_fitness()
    }
}

/// Scheduler handler is in charge of communicating new schedules to the scheduler
/// Fitness functions send to this handler to request fitness values for untested solutions
/// Calculated fitness values are stored in the fitness_values map and fitness functions will first check there
pub struct SchedulerHandler<T>
    where T: ExtendedFitness
{
    scheduler_sender: Sender<DelayMapPhenotype>,
    scheduler_receiver: Receiver<T>,
    fitness_receiver: Receiver<DelaysGenotype>,
    fitness_values: Arc<RwLock<HashMap<DelaysGenotype, T>>>,
}

impl<T> SchedulerHandler<T>
    where T: ExtendedFitness
{
    pub fn new(
        scheduler_sender: Sender<DelayMapPhenotype>,
        scheduler_receiver: Receiver<T>,
        fitness_receiver: Receiver<DelaysGenotype>,
        fitness_values: Arc<RwLock<HashMap<DelaysGenotype, T>>>,
    ) -> Self {
        SchedulerHandler { scheduler_sender, scheduler_receiver, fitness_receiver, fitness_values }
    }

    pub fn run(self) {
        let mut current_delays_genotype = DelaysGenotype::default();
        loop {
            // Receive a new individual to test from a fitness function
            match self.fitness_receiver.recv() {
                Ok(delays_genotype) => {
                    debug!("Fitness function wants fitness for: {:?}", delays_genotype);
                    if current_delays_genotype != delays_genotype && self.fitness_values.read().unwrap().contains_key(&current_delays_genotype) {
                        current_delays_genotype = delays_genotype;
                    }
                }
                Err(_) => {}
            }
            // Send the requested individual to the scheduler
            debug!("delay genome before send: {:?}", current_delays_genotype);
            self.scheduler_sender.send(DelayMapPhenotype::from(current_delays_genotype.as_ref()))
                .expect("Scheduler receiver failed");
            // Receive fitness from scheduler
            match self.scheduler_receiver.recv() {
                Ok(duration) => {
                    debug!("Received fitness of {:?} for genome: {:?}", duration, current_delays_genotype);
                    self.fitness_values.write().unwrap().insert(current_delays_genotype.clone(), duration);
                }
                Err(_) => {}
            }
        }
    }
}
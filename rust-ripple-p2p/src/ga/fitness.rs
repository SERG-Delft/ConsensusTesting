#![allow(dead_code)]

pub(crate) mod compared_fitness_functions;
pub(crate) mod failed_consensus_fitness;
pub(crate) mod validated_ledgers_fitness;
pub(crate) mod time_fitness;
pub(crate) mod delay_fitness;
pub(crate) mod state_accounting_fitness;
pub(crate) mod propose_seq_fitness;

use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, RwLock};
use std::thread;
use genevo::genetic::{AsScalar, Fitness, FitnessFunction, Genotype};
use std::fmt::{Display};
use std::time::{Duration as TimeDuration};
use log::debug;
use crate::ga::encoding::{ExtendedGenotype, ExtendedPhenotype};
use crate::node_state::MutexNodeStates;
use crate::test_harness::TestHarness;

pub trait ExtendedFitness: Fitness + AsScalar + Clone + Send + Sync + Display + serde::Serialize + 'static {
    fn average(a: &[Self]) -> Self;

    fn highest_possible_fitness() -> Self;

    fn lowest_possible_fitness() -> Self;

    fn run_harness(test_harness: &mut TestHarness<'static>, node_states: Arc<MutexNodeStates>) -> Self;
}

/// Fitness function communicates with scheduler handler for calculating and storing fitness of solutions.
#[derive(Clone, Debug)]
pub struct FitnessCalculation<T, G> where T: ExtendedFitness + Clone, G: Genotype {
    pub(crate) fitness_values: Arc<RwLock<HashMap<G, T>>>,
    pub(crate) sender: Sender<G>,
}

impl<T, G> FitnessFunction<G, T> for FitnessCalculation<T, G>
    where T: ExtendedFitness, G: ExtendedGenotype
{
    fn fitness_of(&self, delays_genotype: &G) -> T {
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

pub trait SchedulerHandlerTrait {
    fn run(self);
}

/// Scheduler handler is in charge of communicating new schedules to the scheduler
/// Fitness functions send to this handler to request fitness values for untested solutions
/// Calculated fitness values are stored in the fitness_values map and fitness functions will first check there
pub struct SchedulerHandler<T, G, P>
    where T: ExtendedFitness, G: ExtendedGenotype, P: ExtendedPhenotype<G>
{
    scheduler_sender: Sender<P>,
    scheduler_receiver: Receiver<T>,
    fitness_receiver: Receiver<G>,
    fitness_values: Arc<RwLock<HashMap<G, T>>>,
}

impl<T, G, P> SchedulerHandler<T, G, P>
    where T: ExtendedFitness, G: ExtendedGenotype, P: ExtendedPhenotype<G>
{
    pub fn new(
        scheduler_sender: Sender<P>,
        scheduler_receiver: Receiver<T>,
        fitness_receiver: Receiver<G>,
        fitness_values: Arc<RwLock<HashMap<G, T>>>,
    ) -> Self {
        SchedulerHandler { scheduler_sender, scheduler_receiver, fitness_receiver, fitness_values }
    }
}

impl<T, G, P> SchedulerHandlerTrait for SchedulerHandler<T, G, P>
    where T: ExtendedFitness, G: ExtendedGenotype, P: ExtendedPhenotype<G>
{
    fn run(self) {
        let mut current_individual = G::default();
        self.fitness_values.write().unwrap().insert(current_individual.clone(), T::zero());
        loop {
            // Receive a new individual to test from a fitness function
            match self.fitness_receiver.recv() {
                Ok(individual) => {
                    debug!("Fitness function wants fitness for: {:?}", individual);
                    if current_individual != individual && self.fitness_values.read().unwrap().contains_key(&current_individual) {
                        current_individual = individual;
                    }
                }
                Err(_) => {}
            }
            let current_phenotype = P::from_genes(&current_individual);
            debug!("{}", current_phenotype.display_genotype_by_message());
            // Send the requested individual to the scheduler
            self.scheduler_sender.send(current_phenotype)
                .expect("Scheduler receiver failed");
            // Receive fitness from scheduler
            match self.scheduler_receiver.recv() {
                Ok(fitness) => {
                    debug!("Received fitness of {:?} for individual: {:?}", fitness, current_individual);
                    self.fitness_values.write().unwrap().insert(current_individual.clone(), fitness);
                }
                Err(_) => {}
            }
        }
    }
}
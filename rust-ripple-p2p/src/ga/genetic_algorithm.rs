use std::collections::HashMap;
use std::ops::{Add, Div, Sub};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, RwLock};
use std::thread;
use chrono::Duration;
use std::time::{Duration as TimeDuration};
use genevo::ga::genetic_algorithm;
use genevo::genetic::{AsScalar, Fitness, FitnessFunction, Phenotype};
use genevo::operator::prelude::{MultiPointCrossBreeder, RouletteWheelSelector};
use genevo::population::ValueEncodedGenomeBuilder;
use itertools::{chain};
use genevo::prelude::{build_population, GenerationLimit, Population, SimResult, simulate, Simulation, SimulationBuilder};
use genevo::reinsertion::elitist::ElitistReinserter;
use genevo::types::fmt::Display;
use log::debug;
use super::mutation::GaussianMutator;

/// Parameters for the GA
#[derive(Debug)]
struct Parameter {
    population_size: usize,
    generation_limit: u64,
    num_individuals_per_parents: usize,
    selection_ratio: f64,
    num_crossover_points: usize,
    mutation_rate: f64,
    reinsertion_ratio: f64,
    min_delay: u32,
    max_delay: u32,
}

impl Parameter {
    fn num_genes() -> usize {
        NUM_NODES * (NUM_NODES-1) * MessageType::VALUES.len()
    }
}

impl Default for Parameter {
    fn default() -> Self {
        Parameter {
            population_size: 8,
            generation_limit: 100,
            num_individuals_per_parents: 2,
            selection_ratio: 0.7,
            num_crossover_points: Self::num_genes() / (NUM_NODES * (NUM_NODES - 1)),
            mutation_rate: 0.05,
            reinsertion_ratio: 0.7,
            min_delay: 0,
            max_delay: 1000,
        }
    }
}

// TODO: Get this info from main (global constant?)
const NUM_NODES: usize = 5;

/// The message types that will be subject to delay
#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub enum MessageType {
    TMProposeSet,
    TMStatusChange,
    TMTransaction,
    TMHaveTransactionSet,
}

impl MessageType {
    const VALUES: [Self; 4] = [Self::TMProposeSet, Self::TMStatusChange, Self::TMTransaction, Self::TMHaveTransactionSet];
}

// The phenotype from -> to -> message_type -> delay (ms)
type DelayMap = HashMap<usize, HashMap<usize, HashMap<MessageType, u32>>>;

/// Contains the delayMap for easy use in the scheduler and delays as genotype (vec)
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DelayMapPhenotype {
    pub delay_map: DelayMap,
    delays: DelaysGenotype
}

impl DelayMapPhenotype {
    fn from(genes: &DelaysGenotype) -> Self {
        let index_factor_1 = MessageType::VALUES.len() * (NUM_NODES-1);
        let index_factor_2 = MessageType::VALUES.len();
        let mut from_node = HashMap::new();
        for i in 0..NUM_NODES {
            let mut to_node = HashMap::new();
            for (j, node) in chain(0..i, i+1..NUM_NODES).enumerate() {
                let mut message_type = HashMap::new();
                for (k, message) in MessageType::VALUES.iter().enumerate() {
                    message_type.insert(*message, genes[index_factor_1 * i + index_factor_2 * j + k]);
                }
                to_node.insert(node, message_type.clone());
            }
            from_node.insert(i, to_node.clone());
        }
        Self {
            delay_map: from_node,
            delays: genes.clone()
        }
    }
}

impl Phenotype<DelaysGenotype> for DelayMapPhenotype {
    fn genes(&self) -> DelaysGenotype {
        self.delays.clone()
    }

    fn derive(&self, new_genes: DelaysGenotype) -> Self {
        DelayMapPhenotype::from(&new_genes)
    }
}

// The genotype
type DelaysGenotype = Vec<u32>;

/// Duration in ms from start of test case to validated ledger with all transactions
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
struct FitnessValue {
    time: Duration
}

impl Fitness for FitnessValue {
    fn zero() -> Self {
        FitnessValue { time: Duration::zero() }
    }

    fn abs_diff(&self, other: &Self) -> Self {
        let time: Duration = if self.time.sub(other.time) >= Duration::zero() {
            self.time.sub(other.time)
        } else {
            other.time.sub(self.time)
        };
        FitnessValue { time }
    }
}

impl AsScalar for FitnessValue {
    fn as_scalar(&self) -> f64 {
        self.time.num_milliseconds() as f64
    }
}

/// Fitness function communicates with scheduler handler for calculating and storing fitness of solutions.
#[derive(Clone, Debug)]
pub struct FitnessCalculation {
    fitness_values: Arc<RwLock<HashMap<DelaysGenotype, Duration>>>,
    sender: Sender<DelaysGenotype>,
}

impl FitnessFunction<DelaysGenotype, FitnessValue> for FitnessCalculation {
    fn fitness_of(&self, delays_genotype: &DelaysGenotype) -> FitnessValue {
        let mut sent_to_handler = false;
        loop {
            {
                match self.fitness_values.read().unwrap().get(delays_genotype) {
                    Some(duration) => {
                        println!("Fitness found: {} for genotype: {:?}", duration, delays_genotype);
                        return FitnessValue { time: *duration }
                    },
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

    fn average(&self, a: &[FitnessValue]) -> FitnessValue {
        let mut sum = Duration::zero();
        for fitness in a {
            sum = sum.add(fitness.time);
        }
        FitnessValue { time: sum.div(a.len() as i32) }
    }

    fn highest_possible_fitness(&self) -> FitnessValue {
        FitnessValue { time: Duration::seconds(60) }
    }

    fn lowest_possible_fitness(&self) -> FitnessValue {
        FitnessValue { time: Duration::seconds(2) }
    }
}

/// Scheduler handler is in charge of communicating new schedules to the scheduler
/// Fitness functions send to this handler to request fitness values for untested solutions
/// Calculated fitness values are stored in the fitness_values map and fitness functions will first check there
pub struct SchedulerHandler {
    scheduler_sender: Sender<DelayMapPhenotype>,
    scheduler_receiver: Receiver<Duration>,
    fitness_receiver: Receiver<DelaysGenotype>,
    fitness_values: Arc<RwLock<HashMap<DelaysGenotype, Duration>>>,
}

impl SchedulerHandler {
    fn new(
        scheduler_sender: Sender<DelayMapPhenotype>,
        scheduler_receiver: Receiver<Duration>,
        fitness_receiver: Receiver<DelaysGenotype>,
        fitness_values: Arc<RwLock<HashMap<DelaysGenotype, Duration>>>,
    ) -> Self {
        SchedulerHandler { scheduler_sender, scheduler_receiver, fitness_receiver, fitness_values }
    }

    fn run(self) {
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
                    debug!("Received fitness of {} for genome: {:?}", duration, current_delays_genotype);
                    self.fitness_values.write().unwrap().insert(current_delays_genotype.clone(), duration);
                }
                Err(_) => {}
            }
        }
    }
}

/// Run the genetic algorithm
pub fn run(scheduler_sender: Sender<DelayMapPhenotype>, scheduler_receiver: Receiver<Duration>) {
    let params = Parameter::default();
    // Create initial population of size 8, uniformly distributed over the range of possible values
    let initial_population: Population<DelaysGenotype> = build_population()
        .with_genome_builder(ValueEncodedGenomeBuilder::new(Parameter::num_genes(), params.min_delay, params.max_delay))
        .of_size(8)
        .uniform_at_random();
    println!("{:?}", initial_population);

    let (fitness_sender, fitness_receiver) = channel();
    let fitness_values: Arc<RwLock<HashMap<DelaysGenotype, Duration>>> = Arc::new(RwLock::new(HashMap::new()));
    let scheduler_handler = SchedulerHandler::new(scheduler_sender, scheduler_receiver, fitness_receiver, fitness_values.clone());
    thread::spawn(||scheduler_handler.run());

    let fitness_calculation = FitnessCalculation { fitness_values: fitness_values.clone(), sender: fitness_sender };

    let ga = genetic_algorithm()
        .with_evaluation(fitness_calculation.clone())
        .with_selection(RouletteWheelSelector::new( // Proportionate selection
            params.selection_ratio,                             // How many tuples of individuals should be selected to be used by recombination?
            params.num_individuals_per_parents                  // How many individuals are used in a single recombination (usually 2)
        ))
        // Multi-point crossover
        .with_crossover(MultiPointCrossBreeder::new(params.num_crossover_points))
        .with_mutation(GaussianMutator::new(params.mutation_rate, 0.1 * (params.max_delay as f64)))
        .with_reinsertion(ElitistReinserter::new(
            fitness_calculation,
            false,
            params.reinsertion_ratio,
        ))
        .with_initial_population(initial_population)
        .build();

    let mut sim = simulate(ga)
        .until(GenerationLimit::new(params.generation_limit))
        .build();

    println!("Starting GA with: {:?}", params);
    fitness_values.write().unwrap().clear();
    fitness_values.write().unwrap().insert(vec![], Duration::zero());
    loop {
        let result = sim.step();
        match result {
            Ok(SimResult::Intermediate(step)) => {
                let evaluated_population = step.result.evaluated_population;
                let best_solution = step.result.best_solution;
                println!(
                    "Step: generation: {}, average_fitness: {:?}, \
                     best fitness: {:?}, duration: {}, processing_time: {}",
                    step.iteration,
                    evaluated_population.average_fitness(),
                    best_solution.solution.fitness,
                    step.duration.fmt(),
                    step.processing_time.fmt()
                );
                println!("      {:?}", best_solution.solution.genome);
                //                println!("| population: [{}]", result.population.iter().map(|g| g.as_text())
                //                    .collect::<Vec<String>>().join("], ["));
            },
            Ok(SimResult::Final(step, processing_time, duration, stop_reason)) => {
                let best_solution = step.result.best_solution;
                println!("{}", stop_reason);
                println!(
                    "Final result after {}: generation: {}, \
                     best solution with fitness {:?} found in generation {}, processing_time: {}",
                    duration.fmt(),
                    step.iteration,
                    best_solution.solution.fitness,
                    best_solution.generation,
                    processing_time.fmt()
                );
                println!("      {:?}", best_solution.solution.genome);
                break;
            },
            Err(error) => {
                println!("{:?}", error);
                break;
            },
        }
    }
    // Todo: Terminate the program
}

#[cfg(test)]
mod ga_tests {
    use itertools::Itertools;
    use crate::ga::genetic_algorithm::{DelayMapPhenotype, DelaysGenotype};

    #[test]
    fn check_phenotype() {
        let genotype: DelaysGenotype = (1..81).collect_vec();
        let phenotype = DelayMapPhenotype::from(&genotype);
        println!("{:?}", phenotype.delay_map);
    }
}

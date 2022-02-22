use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, RwLock};
use std::thread;
use genevo::ga::genetic_algorithm;
use genevo::genetic::{Phenotype};
use genevo::operator::prelude::{MultiPointCrossBreeder, RouletteWheelSelector};
use genevo::population::ValueEncodedGenomeBuilder;
use itertools::{chain};
use genevo::prelude::{build_population, GenerationLimit, Population, SimResult, simulate, Simulation, SimulationBuilder};
use genevo::reinsertion::elitist::ElitistReinserter;
use genevo::types::fmt::Display;
use crate::ga::fitness::compared_fitness_functions::ComparedFitnessFunctions;
use crate::ga::fitness::state_accounting_fitness::StateAccountFitness;
use crate::ga::fitness::{ExtendedFitness, FitnessCalculation, SchedulerHandler};
use super::mutation::GaussianMutator;

pub type CurrentFitness = StateAccountFitness;

/// Parameters for the GA
#[allow(unused)]
#[derive(Debug)]
pub struct Parameter {
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
    pub(crate) fn num_genes() -> usize {
        NUM_NODES * (NUM_NODES-1) * MessageType::VALUES.len()
    }
}

impl Default for Parameter {
    fn default() -> Self {
        Parameter {
            population_size: 8,
            generation_limit: 5,
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
    pub fn from(genes: &DelaysGenotype) -> Self {
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
pub(crate) type DelaysGenotype = Vec<u32>;

/// Run the genetic algorithm
#[allow(unused)]
pub fn run<T>(scheduler_sender: Sender<DelayMapPhenotype>, scheduler_receiver: Receiver<T>)
    where T: ExtendedFitness + 'static
{
    let params = Parameter::default();
    // Create initial population of size 8, uniformly distributed over the range of possible values
    let initial_population: Population<DelaysGenotype> = build_population()
        .with_genome_builder(ValueEncodedGenomeBuilder::new(Parameter::num_genes(), params.min_delay, params.max_delay))
        .of_size(8)
        .uniform_at_random();
    println!("{:?}", initial_population);

    let (fitness_sender, fitness_receiver) = channel();
    let fitness_values: Arc<RwLock<HashMap<DelaysGenotype, T>>> = Arc::new(RwLock::new(HashMap::new()));
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
    fitness_values.write().unwrap().insert(vec![], T::zero());
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
    std::process::exit(0);
}

#[cfg(test)]
mod ga_tests {
    use crate::ga::genetic_algorithm::{DelayMapPhenotype};

    #[test]
    fn check_phenotype() {
        //let genotype: DelaysGenotype = (1..81).collect_vec();
        let genotype = vec![959, 533, 12, 717, 406, 603, 767, 0, 304, 366, 925, 54, 854, 159, 611, 747, 839, 555, 985, 146, 678, 499, 67, 802, 991, 557, 185, 312, 557, 676, 659, 149, 963, 347, 817, 987, 451, 972, 515, 631, 174, 564, 551, 889, 665, 527, 645, 336, 977, 946, 641, 441, 113, 872, 778, 385, 878, 528, 947, 435, 913, 643, 4, 101, 472, 416, 624, 792, 925, 573, 225, 948, 862, 142, 580, 50, 742, 648, 338, 914];
        let phenotype = DelayMapPhenotype::from(&genotype);
        println!("{:?}", phenotype.delay_map);
    }
}

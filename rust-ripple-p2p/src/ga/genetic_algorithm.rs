use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
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
use log::debug;
use petgraph::dot::{Config, Dot};
use rand::distributions::Uniform;
use rand::Rng;
use crate::collector::RippleMessage;
use crate::ga::fitness::{ExtendedFitness, FailedConsensusFitness, FitnessCalculation, SchedulerHandler, TimeFitness, ValidatedLedgersFitness};
use crate::node_state::MutexNodeStates;
use super::mutation::GaussianMutator;

pub type CurrentFitness = TimeFitness;

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

pub struct NonGaSchedulerHandler<T>
    where T: ExtendedFitness
{
    scheduler_sender: Sender<DelayMapPhenotype>,
    scheduler_receiver: Receiver<T>,
    fitness_values: Vec<T>,
    file: BufWriter<File>,
    graph_file: BufWriter<File>,
    executions: Vec<Vec<RippleMessage>>
}

impl<T> NonGaSchedulerHandler<T>
    where T: ExtendedFitness
{
    pub fn new(
        scheduler_sender: Sender<DelayMapPhenotype>,
        scheduler_receiver: Receiver<T>,
        fitness_values: Vec<T>,
    ) -> Self {
        let file = File::create(Path::new("run.txt")).expect("Opening execution file failed");
        let graph_file = File::create(Path::new("trace_graphs.txt")).expect("Creating trace graph file failed");
        NonGaSchedulerHandler { scheduler_sender, scheduler_receiver, fitness_values, file: BufWriter::new(file), graph_file: BufWriter::new(graph_file), executions: vec![] }
    }

    pub fn run(&mut self, node_states: Arc<MutexNodeStates>) {
        let params = Parameter::default();
        let initial_population: Population<DelaysGenotype> = build_population()
            .with_genome_builder(ValueEncodedGenomeBuilder::new(Parameter::num_genes(), params.min_delay, params.max_delay))
            .of_size(50)
            .uniform_at_random();
        println!("{:?}", initial_population);

        let delays_genotype = vec![0u32; Parameter::num_genes()];

        let mut graphs = vec![];
        self.scheduler_sender.send(DelayMapPhenotype::from(delays_genotype.as_ref()))
            .expect("Scheduler receiver failed");
        // Receive fitness from scheduler
        match self.scheduler_receiver.recv() {
            Ok(value) => {},
            Err(_) => {},
        }

        for i in 0..2 {
            // let delays_genotype = initial_population.individuals()[i].clone();
            println!("Starting test {}", i);
            debug!("delay genome before send: {:?}", delays_genotype);
            self.scheduler_sender.send(DelayMapPhenotype::from(delays_genotype.as_ref()))
                .expect("Scheduler receiver failed");
            // Receive fitness from scheduler
            match self.scheduler_receiver.recv() {
                Ok(value) => {
                    debug!("Received fitness of {:?} for genome: {:?}", value, delays_genotype);
                    self.executions.push(node_states.get_executions());
                    node_states.clear_executions();
                    self.fitness_values.push(value);
                    self.file.write_all(self.fitness_values[i].to_string().as_bytes()).unwrap();
                    for message in &self.executions[i] {
                        self.file.write_all(message.clone().simple_str().as_bytes()).unwrap();
                    }
                    graphs.push(node_states.get_dependency_graph());
                    self.file.write("\n".as_bytes()).unwrap();
                    self.file.write(format!("{:?}", Dot::with_config(&node_states.get_dependency_graph(), &[Config::EdgeNoLabel])).as_bytes()).unwrap();
                }
                Err(_) => {}
            }
        }
        // println!("Starting ged calc");
        // let distance = ged::approximate_edit_distance::approximate_hed_graph_edit_distance(graphs[0].clone(), graphs[1].clone());
        // println!("Finished ged calc: {}", distance);
        std::process::exit(0);
    }

    pub fn run_trace_graph_creation(&mut self, node_states: Arc<MutexNodeStates>) {
        let zero_delays = vec![0u32; Parameter::num_genes()];
        let one_delays = vec![1000u32; Parameter::num_genes()];
        let range = Uniform::from(0..1000);
        let random_delays: Vec<u32> = rand::thread_rng().sample_iter(&range).take(Parameter::num_genes()).collect();
        let delays = vec![zero_delays, one_delays, random_delays];

        // Allow five test harnesses to pass to mitigate any startup difficulties in the network
        for i in 0..5 {
            self.scheduler_sender.send(DelayMapPhenotype::from(&delays[0]))
                .expect("Scheduler receiver failed");
            // Receive fitness from scheduler
            match self.scheduler_receiver.recv() {
                Ok(_) => {},
                Err(_) => {},
            }
        }

        // Run three different delays twice and write the resulting graphs to the graph_file
        for i in 0..3 {
            let cur_delays = delays[i].clone();
            for j in 0..2 {
                println!("Starting test {} with delays: {:?}", i*2+j+1, cur_delays);
                self.scheduler_sender.send(DelayMapPhenotype::from(&cur_delays))
                    .expect("Scheduler receiver failed");
                // Receive fitness from scheduler
                match self.scheduler_receiver.recv() {
                    Ok(_) => {
                        self.graph_file.write(format!("{:?}", cur_delays).as_bytes()).unwrap();
                        self.graph_file.write(b"+\n").unwrap();
                        let j = serde_json::to_string(&node_states.get_receive_dependency_graph()).unwrap();
                        self.graph_file.write(j.as_bytes()).unwrap();
                        self.graph_file.write(b"+\n").unwrap();
                    }
                    Err(_) => {}
                }
            }
        }
        self.graph_file.flush().unwrap();
        println!("Finished graph creation, exiting...");
        std::process::exit(0);
    }
}

pub fn run_non_ga<T>(scheduler_sender: Sender<DelayMapPhenotype>, scheduler_receiver: Receiver<T>, node_states: Arc<MutexNodeStates>)
    where T: ExtendedFitness + 'static
{
    let fitness_values: Vec<T> = vec![];
    let mut scheduler_handler = NonGaSchedulerHandler::new(scheduler_sender, scheduler_receiver, fitness_values.clone());
    thread::spawn(move ||scheduler_handler.run_trace_graph_creation(node_states));
}

/// Run the genetic algorithm
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

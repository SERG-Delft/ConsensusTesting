use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::sync::Arc;
use std::sync::mpsc::{Sender, Receiver};
use std::thread;
use chrono::{Duration, Utc};
use itertools::chain;
use log::debug;
use petgraph::Graph;
use rand::distributions::Uniform;
use rand::{Rng, thread_rng};
use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::SeedableRng;
use crate::ga::encoding::delay_encoding::{DelayMapPhenotype, DelayGenotype};
use crate::ga::encoding::{ExtendedPhenotype, num_genes};
use crate::ga::encoding::priority_encoding::{PriorityGenotype, PriorityMapPhenotype};
use crate::ga::fitness::ExtendedFitness;
use crate::ga::genetic_algorithm::{ConsensusMessageType};
use crate::locality::{sample_delays_genotype, sample_priority_genotype};
use crate::node_state::{DependencyEvent, MessageTypeDependencyEvent, MutexNodeStates};
use crate::NUM_NODES;

mod compare;
mod compare_fitness;

#[allow(unused)]
pub struct DelayTraceGraphSchedulerHandler<T>
    where T: ExtendedFitness
{
    scheduler_sender: Sender<DelayMapPhenotype>,
    scheduler_receiver: Receiver<T>,
    graph_file: BufWriter<File>,
}

#[allow(unused)]
impl<T> DelayTraceGraphSchedulerHandler<T>
    where T: ExtendedFitness
{
    pub fn new(
        scheduler_sender: Sender<DelayMapPhenotype>,
        scheduler_receiver: Receiver<T>,
    ) -> Self
    {
        let graph_file = File::create(Path::new("delay_trace_graphs.txt")).expect("Creating trace graph file failed");
        DelayTraceGraphSchedulerHandler {
            scheduler_sender,
            scheduler_receiver,
            graph_file: BufWriter::new(graph_file),
        }
    }

    /// Write trace graphs to file after running a number of test harnesses with certain delays
    pub fn delay_trace_graph_creation(&mut self, node_states: Arc<MutexNodeStates>) {
        let zero_delays = vec![0u32; num_genes()];
        let one_delays = vec![1000u32; num_genes()];
        let range = Uniform::from(0..1000);
        let random_delays_1: Vec<u32> = rand::thread_rng().sample_iter(&range).take(num_genes()).collect();
        let random_delays_2: Vec<u32> = rand::thread_rng().sample_iter(&range).take(num_genes()).collect();
        let delays = vec![zero_delays, one_delays, random_delays_1, random_delays_2];

        // Allow five test harnesses to pass to mitigate any startup difficulties in the network
        for _ in 0..5 {
            self.scheduler_sender.send(DelayMapPhenotype::from_genes(&delays[0])).expect("Scheduler receiver failed");
            self.scheduler_receiver.recv().expect("Scheduler sender failed");
        }

        let number_of_tests_per_chromosome = 5;

        // Run the different delays several times and write the resulting graphs to the graph_file
        for i in 0..delays.len() {
            let cur_delays = delays[i].clone();
            for j in 0..number_of_tests_per_chromosome {
                println!("Starting test {} with delays: {:?}", i*number_of_tests_per_chromosome+j+1, cur_delays);
                self.scheduler_sender.send(DelayMapPhenotype::from_genes(&cur_delays))
                    .expect("Scheduler receiver failed");
                // Receive fitness from scheduler
                match self.scheduler_receiver.recv() {
                    Ok(_) => {
                        self.graph_file.write(format!("{:?}", cur_delays).as_bytes()).unwrap();
                        self.graph_file.write(b"+\n").unwrap();
                        let j = serde_json::to_string(&node_states.get_dependency_graph()).unwrap();
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

#[allow(unused)]
pub struct PriorityTraceGraphSchedulerHandler<T>
    where T: ExtendedFitness
{
    scheduler_sender: Sender<PriorityMapPhenotype>,
    scheduler_receiver: Receiver<T>,
    graph_file: BufWriter<File>,
}

#[allow(unused)]
impl<T> PriorityTraceGraphSchedulerHandler<T>
    where T: ExtendedFitness
{
    pub fn new(
        scheduler_sender: Sender<PriorityMapPhenotype>,
        scheduler_receiver: Receiver<T>,
    ) -> Self
    {
        let graph_file = File::create(Path::new("priority_trace_graphs.txt")).expect("Creating trace graph file failed");
        PriorityTraceGraphSchedulerHandler {
            scheduler_sender,
            scheduler_receiver,
            graph_file: BufWriter::new(graph_file),
        }
    }

    /// Write trace graphs to file after running a number of test harnesses with certain delays
    pub fn priority_trace_graph_creation(&mut self, node_states: Arc<MutexNodeStates>) {
        let no_priorities = vec![0; num_genes()];
        let range = Uniform::from(0f32..1f32);
        let mut rng_1 = ChaCha8Rng::seed_from_u64(1);
        let mut rng_2 = ChaCha8Rng::seed_from_u64(2);
        let random_priorities_1: Vec<usize> = rng_1.sample_iter(&range).take(num_genes()).map(|f| f as usize).collect();
        let random_priorities_2: Vec<usize> = rng_2.sample_iter(&range).take(num_genes()).map(|f| f as usize).collect();
        let priorities = vec![no_priorities, random_priorities_1, random_priorities_2];

        // Allow five test harnesses to pass to mitigate any startup difficulties in the network
        for _ in 0..5 {
            self.scheduler_sender.send(PriorityMapPhenotype::from_genes(&priorities[0])).expect("Scheduler receiver failed");
            self.scheduler_receiver.recv().expect("Scheduler sender failed");
        }

        let number_of_tests_per_chromosome = 5;

        // Run the different delays several times and write the resulting graphs to the graph_file
        for i in 0..priorities.len() {
            let cur_priorities = priorities[i].clone();
            for j in 0..number_of_tests_per_chromosome {
                println!("Starting test {} with priorities: {:?}", i*number_of_tests_per_chromosome+j+1, cur_priorities);
                self.scheduler_sender.send(PriorityMapPhenotype::from_genes(&cur_priorities))
                    .expect("Scheduler receiver failed");
                // Receive fitness from scheduler
                match self.scheduler_receiver.recv() {
                    Ok(_) => {
                        let message_type_graph = transform_to_message_type_graph(&node_states.get_dependency_graph());
                        self.graph_file.write(format!("{:?}", cur_priorities).as_bytes()).unwrap();
                        self.graph_file.write(b"+\n").unwrap();
                        let j = serde_json::to_string(&message_type_graph).unwrap();
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

pub fn transform_to_message_type_graph(graph: &Graph<DependencyEvent, ()>) -> Graph<MessageTypeDependencyEvent, ()> {
    graph.map(|_ix, node| MessageTypeDependencyEvent::from(node), |_ix, edge| *edge)
}

pub struct FitnessComparisonSchedulerHandler<F: ExtendedFitness> {
    scheduler_sender: Sender<DelayMapPhenotype>,
    scheduler_receiver: Receiver<F>,
    fitness_file: BufWriter<File>,
}

impl<F: ExtendedFitness> FitnessComparisonSchedulerHandler<F> {
    pub fn new(
        scheduler_sender: Sender<DelayMapPhenotype>,
        scheduler_receiver: Receiver<F>,
    ) -> Self
    {
        let fitness_file = File::create(Path::new("fitness_values.txt")).expect("Opening execution file failed");
        FitnessComparisonSchedulerHandler {
            scheduler_sender,
            scheduler_receiver,
            fitness_file: BufWriter::new(fitness_file),
        }
    }

    pub fn fitness_comparison(&mut self) {
        let mut delays: Vec<Vec<u32>> = vec![];
        let range: Uniform<u32> = Uniform::from(0..1000);
        let number_of_tests = 100;
        let number_of_tests_per_chromosome = 5;
        for _ in 0..number_of_tests {
            delays.push(rand::thread_rng().sample_iter(&range).take(num_genes()).collect());
        }

        // Allow five test harnesses to pass to mitigate any startup difficulties in the network
        let zero_delays = vec![0u32; num_genes()];
        for _ in 0..5 {
            self.scheduler_sender.send(DelayMapPhenotype::from_genes(&zero_delays)).expect("Scheduler receiver failed");
            self.scheduler_receiver.recv().expect("Scheduler sender failed");
        }

        for i in 0..delays.len() {
            let cur_delays = delays[i].clone();
            for j in 0..number_of_tests_per_chromosome {
                println!("Starting test {} with delays: {:?}", i*number_of_tests_per_chromosome+j+1, cur_delays);
                self.scheduler_sender.send(DelayMapPhenotype::from_genes(&cur_delays)).expect("Scheduler receiver failed");
                // Receive fitness from scheduler
                match self.scheduler_receiver.recv() {
                    Ok(fitness) => {
                        self.fitness_file.write(format!("{:?}", cur_delays).as_bytes()).unwrap();
                        self.fitness_file.write(b"+\n").unwrap();
                        let j = serde_json::to_string(&fitness).unwrap();
                        self.fitness_file.write(j.as_bytes()).unwrap();
                        self.fitness_file.write(b"+\n").unwrap();
                    }
                    Err(_) => {}
                }
            }
        }
        self.fitness_file.flush().unwrap();
        println!("Finished fitness comparison, exiting...");
        std::process::exit(0);
    }
}

pub struct NoDelaySchedulerHandler<F: ExtendedFitness> {
    scheduler_sender: Sender<DelayMapPhenotype>,
    scheduler_receiver: Receiver<F>,
    number_of_tests: usize,
}

impl<F: ExtendedFitness> NoDelaySchedulerHandler<F> {
    pub fn new(
        scheduler_sender: Sender<DelayMapPhenotype>,
        scheduler_receiver: Receiver<F>,
        number_of_tests: usize,
    ) -> Self
    {
        NoDelaySchedulerHandler {
            scheduler_sender,
            scheduler_receiver,
            number_of_tests,
        }
    }

    pub fn run(&mut self, search_budget: Duration) {
        let delays: Vec<u32> = vec![0; num_genes()];
        let start_time = Utc::now();
        while Utc::now() - start_time < search_budget {
            self.scheduler_sender.send(DelayMapPhenotype::from_genes(&delays)).expect("Scheduler receiver failed");
            self.scheduler_receiver.recv().expect("Scheduler sender failed");
        }
        println!("Search budget exceeded. exiting...");
        std::process::exit(0);
    }
}

pub struct PreDeterminedDelaySchedulerHandler<F: ExtendedFitness> {
    scheduler_sender: Sender<DelayMapPhenotype>,
    scheduler_receiver: Receiver<F>,
    number_of_tests: usize,
}

impl<F: ExtendedFitness> PreDeterminedDelaySchedulerHandler<F> {
    pub fn new(
        scheduler_sender: Sender<DelayMapPhenotype>,
        scheduler_receiver: Receiver<F>,
        number_of_tests: usize,
    ) -> Self
    {
        Self {
            scheduler_sender,
            scheduler_receiver,
            number_of_tests,
        }
    }

    pub fn run(&mut self) {
        let delays: Vec<u32> = Self::create_proposal_bug_delays();
        for i in 0..self.number_of_tests {
            println!("Starting test {}", i);
            self.scheduler_sender.send(DelayMapPhenotype::from_genes(&delays)).expect("Scheduler receiver failed");
            self.scheduler_receiver.recv().expect("Scheduler sender failed");
        }
        println!("Finished run. exiting...");
        std::process::exit(0);
    }

    #[allow(unused)]
    fn create_liveness_bug_delays() -> DelayGenotype {
        let index_factor_1 = ConsensusMessageType::VALUES.len() * (*NUM_NODES-1);
        let index_factor_2 = ConsensusMessageType::VALUES.len();
        let mut delays = vec![0u32; num_genes()];
        let ledger_data_index = 12;
        let get_ledger_index = 11;
        let transaction_index = 9;
        let ledger_data_delay = 3000;
        let get_ledger_delay = 3000;
        let transaction_delay = 0;
        for i in 0..*NUM_NODES {
            for (j, _) in chain(0..i, i+1..*NUM_NODES).enumerate() {
                delays[index_factor_1 * i + index_factor_2 * j + ledger_data_index] = ledger_data_delay;
                delays[index_factor_1 * i + index_factor_2 * j + get_ledger_index] = get_ledger_delay;
                delays[index_factor_1 * i + index_factor_2 * j + transaction_index] = transaction_delay;
            }
        }
        println!("{}", DelayMapPhenotype::from_genes(&delays).display_genotype_by_message());
        delays
    }

    #[allow(unused)]
    fn create_proposal_bug_delays() -> DelayGenotype {
        let index_factor_1 = ConsensusMessageType::VALUES.len() * (*NUM_NODES-1);
        let index_factor_2 = ConsensusMessageType::VALUES.len();
        let mut delays = vec![0u32; num_genes()];
        let propose_set_0_index = 0;
        let propose_set_1_index = 1;
        let propose_set_2_index = 2;
        let propose_set_3_index = 3;
        let propose_set_4_index = 4;
        let propose_set_5_index = 5;
        let propose_set_bowout_index = 6;
        let ledger_data_index = 12;
        let propose_set_0_delay = 3000;
        let propose_set_1_delay = 1500;
        let propose_set_2_delay = 0;
        let propose_set_3_delay = 0;
        let propose_set_4_delay = 0;
        let propose_set_5_delay = 0;
        let propose_set_bowout_delay = 0;
        for i in [1, 2, 3, 4] {
            for (j, _) in chain(0..i, i+1..*NUM_NODES).enumerate() {
                delays[index_factor_1 * i + index_factor_2 * j + propose_set_0_index] = propose_set_0_delay;
                delays[index_factor_1 * i + index_factor_2 * j + propose_set_1_index] = propose_set_1_delay;
                delays[index_factor_1 * i + index_factor_2 * j + propose_set_2_index] = propose_set_2_delay;
                delays[index_factor_1 * i + index_factor_2 * j + propose_set_3_index] = propose_set_3_delay;
                delays[index_factor_1 * i + index_factor_2 * j + propose_set_4_index] = propose_set_4_delay;
                delays[index_factor_1 * i + index_factor_2 * j + propose_set_5_index] = propose_set_5_delay;
                delays[index_factor_1 * i + index_factor_2 * j + propose_set_bowout_index] = propose_set_bowout_delay;
                delays[index_factor_1 * i + index_factor_2 * j + ledger_data_index] = 3000;
            }
        }
        println!("{}", DelayMapPhenotype::from_genes(&delays).display_genotype_by_message());
        delays
    }
}

pub struct PreDeterminedPrioritySchedulerHandler<F: ExtendedFitness> {
    scheduler_sender: Sender<PriorityMapPhenotype>,
    scheduler_receiver: Receiver<F>,
    number_of_tests: usize,
}

impl<F: ExtendedFitness> PreDeterminedPrioritySchedulerHandler<F> {
    pub fn new(
        scheduler_sender: Sender<PriorityMapPhenotype>,
        scheduler_receiver: Receiver<F>,
        number_of_tests: usize,
    ) -> Self
    {
        Self {
            scheduler_sender,
            scheduler_receiver,
            number_of_tests,
        }
    }

    pub fn run(&mut self) {
        let priorities = Self::create_priorities();
        for i in 0..self.number_of_tests {
            println!("Starting test {}", i);
            self.scheduler_sender.send(PriorityMapPhenotype::from_genes(&priorities)).expect("Scheduler receiver failed");
            self.scheduler_receiver.recv().expect("Scheduler sender failed");
        }
        println!("Finished run. exiting...");
        std::process::exit(0);
    }

    fn create_priorities() -> PriorityGenotype {
        let index_factor_1 = ConsensusMessageType::VALUES.len() * (*NUM_NODES-1);
        let index_factor_2 = ConsensusMessageType::VALUES.len();
        let mut priorities = vec![20usize; num_genes()];
        let ledger_data_index = 12;
        let get_ledger_index = 11;
        let transaction_index = 9;
        let ledger_data_priority = 1;
        let get_ledger_priority = 2;
        let transaction_priority = 3;
        for i in 0..*NUM_NODES {
            for (j, _) in chain(0..i, i+1..*NUM_NODES).enumerate() {
                priorities[index_factor_1 * i + index_factor_2 * j + ledger_data_index] = ledger_data_priority;
                priorities[index_factor_1 * i + index_factor_2 * j + get_ledger_index] = get_ledger_priority;
                priorities[index_factor_1 * i + index_factor_2 * j + transaction_index] = transaction_priority;
            }
        }
        priorities
    }
}

#[allow(unused)]
pub fn run_delay_trace_graph_creation<F: ExtendedFitness>(scheduler_sender: Sender<DelayMapPhenotype>, scheduler_receiver: Receiver<F>, node_states: Arc<MutexNodeStates>) {
    let mut scheduler_handler = DelayTraceGraphSchedulerHandler::new(scheduler_sender, scheduler_receiver);
    thread::spawn(move || scheduler_handler.delay_trace_graph_creation(node_states));
}

#[allow(unused)]
pub fn run_priority_trace_graph_creation<F: ExtendedFitness>(scheduler_sender: Sender<PriorityMapPhenotype>, scheduler_receiver: Receiver<F>, node_states: Arc<MutexNodeStates>) {
    let mut scheduler_handler = PriorityTraceGraphSchedulerHandler::new(scheduler_sender, scheduler_receiver);
    thread::spawn(move || scheduler_handler.priority_trace_graph_creation(node_states));
}

#[allow(unused)]
pub fn run_fitness_comparison<F: ExtendedFitness>(scheduler_sender: Sender<DelayMapPhenotype>, scheduler_receiver: Receiver<F>) {
    let mut scheduler_handler = FitnessComparisonSchedulerHandler::new(scheduler_sender, scheduler_receiver);
    thread::spawn(move || scheduler_handler.fitness_comparison());
}

#[allow(unused)]
pub fn run_no_delays<F: ExtendedFitness>(scheduler_sender: Sender<DelayMapPhenotype>, scheduler_receiver: Receiver<F>, search_budget: Duration) {
    let mut scheduler_handler = NoDelaySchedulerHandler::new(scheduler_sender, scheduler_receiver, 0);
    thread::spawn(move || scheduler_handler.run(search_budget));
}

#[allow(unused)]
pub fn run_predetermined_delays<F: ExtendedFitness>(scheduler_sender: Sender<DelayMapPhenotype>, scheduler_receiver: Receiver<F>, number_of_tests: usize) {
    let mut scheduler_handler = PreDeterminedDelaySchedulerHandler::new(scheduler_sender, scheduler_receiver, number_of_tests);
    thread::spawn(move || scheduler_handler.run());
}

#[allow(unused)]
pub fn run_predetermined_priorities<F: ExtendedFitness>(scheduler_sender: Sender<PriorityMapPhenotype>, scheduler_receiver: Receiver<F>, number_of_tests: usize) {
    let mut scheduler_handler = PreDeterminedPrioritySchedulerHandler::new(scheduler_sender, scheduler_receiver, number_of_tests);
    thread::spawn(move || scheduler_handler.run());
}

#[allow(unused)]
pub fn run_random_priorities<F: ExtendedFitness>(scheduler_sender: Sender<PriorityMapPhenotype>, scheduler_receiver: Receiver<F>, search_budget: Duration) {
    let start_time = Utc::now();
    while Utc::now() - start_time < search_budget {
        let priorities = sample_priority_genotype(num_genes(), &mut thread_rng());
        debug!("Running random priorities: {:?}", priorities);
        scheduler_sender.send(PriorityMapPhenotype::from_genes(&priorities)).expect("Scheduler receiver failed");
        scheduler_receiver.recv().expect("Scheduler sender failed");
    }
    println!("Exiting: Search budget exceeded.");
    std::process::exit(0);
}

#[allow(unused)]
pub fn run_random_delays<F: ExtendedFitness>(scheduler_sender: Sender<DelayMapPhenotype>, scheduler_receiver: Receiver<F>, search_budget: Duration) {
    let start_time = Utc::now();
    while Utc::now() - start_time < search_budget {
        let delays = sample_delays_genotype(num_genes(), 0, 4000, &mut thread_rng());
        debug!("Running random delays: {:?}", delays);
        scheduler_sender.send(DelayMapPhenotype::from_genes(&delays)).expect("Scheduler receiver failed");
        scheduler_receiver.recv().expect("Scheduler sender failed");
    }
    println!("Exiting: Search budget exceeded.");
    std::process::exit(0);
}
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::sync::Arc;
use std::sync::mpsc::{Sender, Receiver};
use std::thread;
use rand::distributions::Uniform;
use rand::Rng;
use crate::ga::fitness::ExtendedFitness;
use crate::ga::genetic_algorithm::{CurrentFitness, DelayMapPhenotype, num_genes, Parameter};
use crate::node_state::MutexNodeStates;

mod compare;
mod compare_fitness;

#[allow(unused)]
pub struct TraceGraphSchedulerHandler<T>
    where T: ExtendedFitness
{
    scheduler_sender: Sender<DelayMapPhenotype>,
    scheduler_receiver: Receiver<T>,
    graph_file: BufWriter<File>,
}

#[allow(unused)]
impl<T> TraceGraphSchedulerHandler<T>
    where T: ExtendedFitness
{
    pub fn new(
        scheduler_sender: Sender<DelayMapPhenotype>,
        scheduler_receiver: Receiver<T>,
    ) -> Self
    {
        let graph_file = File::create(Path::new("trace_graphs.txt")).expect("Creating trace graph file failed");
        TraceGraphSchedulerHandler {
            scheduler_sender,
            scheduler_receiver,
            graph_file: BufWriter::new(graph_file),
        }
    }

    /// Write trace graphs to file after running a number of test harnesses with certain delays
    pub fn trace_graph_creation(&mut self, node_states: Arc<MutexNodeStates>) {
        let zero_delays = vec![0u32; num_genes()];
        let one_delays = vec![1000u32; num_genes()];
        let range = Uniform::from(0..1000);
        let random_delays_1: Vec<u32> = rand::thread_rng().sample_iter(&range).take(num_genes()).collect();
        let random_delays_2: Vec<u32> = rand::thread_rng().sample_iter(&range).take(num_genes()).collect();
        let delays = vec![zero_delays, one_delays, random_delays_1, random_delays_2];

        // Allow five test harnesses to pass to mitigate any startup difficulties in the network
        for _ in 0..5 {
            self.scheduler_sender.send(DelayMapPhenotype::from(&delays[0])).expect("Scheduler receiver failed");
            self.scheduler_receiver.recv().expect("Scheduler sender failed");
        }

        let number_of_tests_per_chromosome = 5;

        // Run the different delays several times and write the resulting graphs to the graph_file
        for i in 0..delays.len() {
            let cur_delays = delays[i].clone();
            for j in 0..number_of_tests_per_chromosome {
                println!("Starting test {} with delays: {:?}", i*2+j+1, cur_delays);
                self.scheduler_sender.send(DelayMapPhenotype::from(&cur_delays))
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

pub struct FitnessComparisonSchedulerHandler {
    scheduler_sender: Sender<DelayMapPhenotype>,
    scheduler_receiver: Receiver<CurrentFitness>,
    fitness_file: BufWriter<File>,
}

impl FitnessComparisonSchedulerHandler {
    pub fn new(
        scheduler_sender: Sender<DelayMapPhenotype>,
        scheduler_receiver: Receiver<CurrentFitness>,
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
            self.scheduler_sender.send(DelayMapPhenotype::from(&zero_delays)).expect("Scheduler receiver failed");
            self.scheduler_receiver.recv().expect("Scheduler sender failed");
        }

        for i in 0..delays.len() {
            let cur_delays = delays[i].clone();
            for j in 0..number_of_tests_per_chromosome {
                println!("Starting test {} with delays: {:?}", i*number_of_tests_per_chromosome+j+1, cur_delays);
                self.scheduler_sender.send(DelayMapPhenotype::from(&cur_delays)).expect("Scheduler receiver failed");
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

#[allow(unused)]
pub fn run_trace_graph_creation<T>(scheduler_sender: Sender<DelayMapPhenotype>, scheduler_receiver: Receiver<T>, node_states: Arc<MutexNodeStates>)
    where T: ExtendedFitness + 'static
{
    let mut scheduler_handler = TraceGraphSchedulerHandler::new(scheduler_sender, scheduler_receiver);
    thread::spawn(move ||scheduler_handler.trace_graph_creation(node_states));
}

#[allow(unused)]
pub fn run_fitness_comparison(scheduler_sender: Sender<DelayMapPhenotype>, scheduler_receiver: Receiver<CurrentFitness>) {
    let mut scheduler_handler = FitnessComparisonSchedulerHandler::new(scheduler_sender, scheduler_receiver);
    thread::spawn(move ||scheduler_handler.fitness_comparison());
}
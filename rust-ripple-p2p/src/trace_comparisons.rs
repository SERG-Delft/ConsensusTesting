use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::sync::Arc;
use std::sync::mpsc::{Sender, Receiver};
use std::thread;
use itertools::chain;
use petgraph::Graph;
use rand::distributions::Uniform;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::SeedableRng;
use crate::ga::encoding::delay_encoding::{DelayMapPhenotype, DelaysGenotype};
use crate::ga::encoding::{ExtendedPhenotype, num_genes};
use crate::ga::encoding::priority_encoding::{PriorityMapPhenotype};
use crate::ga::fitness::ExtendedFitness;
use crate::ga::genetic_algorithm::{ConsensusMessageType, CurrentFitness};
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

fn transform_to_message_type_graph(graph: &Graph<DependencyEvent, ()>) -> Graph<MessageTypeDependencyEvent, ()> {
    graph.map(|_ix, node| MessageTypeDependencyEvent::from(node), |_ix, edge| *edge)
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

pub struct NoDelaySchedulerHandler {
    scheduler_sender: Sender<DelayMapPhenotype>,
    scheduler_receiver: Receiver<CurrentFitness>,
    number_of_tests: usize,
}

impl NoDelaySchedulerHandler {
    pub fn new(
        scheduler_sender: Sender<DelayMapPhenotype>,
        scheduler_receiver: Receiver<CurrentFitness>,
        number_of_tests: usize,
    ) -> Self
    {
        NoDelaySchedulerHandler {
            scheduler_sender,
            scheduler_receiver,
            number_of_tests,
        }
    }

    pub fn run(&mut self) {
        let delays: Vec<u32> = vec![0; num_genes()];
        for i in 0..self.number_of_tests {
            println!("Starting test {}", i);
            self.scheduler_sender.send(DelayMapPhenotype::from_genes(&delays)).expect("Scheduler receiver failed");
            self.scheduler_receiver.recv().expect("Scheduler sender failed");
        }
        println!("Finished run. exiting...");
        std::process::exit(0);
    }
}

pub struct PreDeterminedDelaySchedulerHandler {
    scheduler_sender: Sender<DelayMapPhenotype>,
    scheduler_receiver: Receiver<CurrentFitness>,
    number_of_tests: usize,
}

impl PreDeterminedDelaySchedulerHandler {
    pub fn new(
        scheduler_sender: Sender<DelayMapPhenotype>,
        scheduler_receiver: Receiver<CurrentFitness>,
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
        let delays: Vec<u32> = Self::create_delays();
        for i in 0..self.number_of_tests {
            println!("Starting test {}", i);
            self.scheduler_sender.send(DelayMapPhenotype::from_genes(&delays)).expect("Scheduler receiver failed");
            self.scheduler_receiver.recv().expect("Scheduler sender failed");
        }
        println!("Finished run. exiting...");
        std::process::exit(0);
    }

    fn create_delays() -> DelaysGenotype {
        let index_factor_1 = ConsensusMessageType::VALUES.len() * (*NUM_NODES-1);
        let index_factor_2 = ConsensusMessageType::VALUES.len();
        let mut delays = vec![0u32; num_genes()];
        let ledger_data_index = 12;
        let get_ledger_index = 11;
        let transaction_index = 9;
        let ledger_data_delay = 6000;
        let get_ledger_delay = 0;
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
}

// pub struct PreDeterminedPrioritySchedulerHandler {
//     scheduler_sender: Sender<PriorityMapPhenotype>,
//     scheduler_receiver: Receiver<CurrentFitness>,
//     number_of_tests: usize,
// }
//
// impl PreDeterminedPrioritySchedulerHandler {
//     pub fn new(
//         scheduler_sender: Sender<PriorityMapPhenotype>,
//         scheduler_receiver: Receiver<CurrentFitness>,
//         number_of_tests: usize,
//     ) -> Self
//     {
//         Self {
//             scheduler_sender,
//             scheduler_receiver,
//             number_of_tests,
//         }
//     }
//
//     pub fn run(&mut self) {
//         let delays: Vec<u32> = Self::create_delays();
//         for i in 0..self.number_of_tests {
//             println!("Starting test {}", i);
//             self.scheduler_sender.send(PriorityMapPhenotype::from_genes(&delays)).expect("Scheduler receiver failed");
//             self.scheduler_receiver.recv().expect("Scheduler sender failed");
//         }
//         println!("Finished run. exiting...");
//         std::process::exit(0);
//     }
//
//     /// From node 1 to 0 ProposeSet0 gets 2000 delay
//     /// From node 2 to 0 ProposeSet0 gets 3000 delay
//     fn create_priorities() -> PriorityGenotype {
//         let index_factor_1 = ConsensusMessageType::VALUES.len() * (*NUM_NODES-1);
//         let index_factor_2 = ConsensusMessageType::VALUES.len();
//         let mut priorities = vec![Priority(0f32); num_genes()];
//         // TMProposeSet0: [(1, [Priority(0.7821766), Priority(0.092473626), Priority(0.48026264), Priority(0.4778285)]), (3, [Priority(0.2593546), Priority(0.12294316), Priority(0.6753402), Priority(0.7828284)]), (2, [Priority(0.690683), Priority(0.65723324), Priority(0.61833704), Priority(0.49876213)]), (4, [Priority(0.875072), Priority(0.5752288), Priority(0.58243775), Priority(0.81654465)]), (0, [Priority(0.65661144), Priority(0.7340294), Priority(0.13750815), Priority(0.31901383)])]
//         // TMProposeSet1: [(1, [Priority(0.43842566), Priority(0.20223701), Priority(0.45049834), Priority(0.195879)]), (3, [Priority(0.22853124), Priority(0.48894036), Priority(0.016237218), Priority(0.547441)]), (2, [Priority(0.6402242), Priority(0.8226489), Priority(0.3690585), Priority(0.9688575)]), (4, [Priority(0.21954392), Priority(0.47771573), Priority(0.24781144), Priority(0.22597027)]), (0, [Priority(0.8950151), Priority(0.6679809), Priority(0.77661204), Priority(0.33151567)])]
//         // TMProposeSet2: [(1, [Priority(0.38044345), Priority(0.22339773), Priority(0.19324124), Priority(0.55113363)]), (3, [Priority(0.9675127), Priority(0.11971688), Priority(0.87724257), Priority(0.6069747)]), (2, [Priority(0.7461163), Priority(0.8518003), Priority(0.34085572), Priority(0.6229204)]), (4, [Priority(0.21366203), Priority(0.30562675), Priority(0.52897024), Priority(0.23124921)]), (0, [Priority(0.26952016), Priority(0.7274964), Priority(0.30521715), Priority(0.7358526)])]
//         // TMProposeSet3: [(1, [Priority(0.6029191), Priority(0.6907802), Priority(0.5567845), Priority(0.84383464)]), (3, [Priority(0.8985449), Priority(0.022029877), Priority(0.6931081), Priority(0.33923125)]), (2, [Priority(0.31752646), Priority(0.5728848), Priority(0.43567693), Priority(0.6688744)]), (4, [Priority(0.66550684), Priority(0.90422404), Priority(0.062274694), Priority(0.90821135)]), (0, [Priority(0.87048864), Priority(0.081334114), Priority(0.7078996), Priority(0.83546054)])]
//         // TMProposeSet4: [(1, [Priority(0.4118451), Priority(0.14793062), Priority(0.17180598), Priority(0.6235647)]), (3, [Priority(0.054819465), Priority(0.94977105), Priority(0.32259202), Priority(0.9472345)]), (2, [Priority(0.08963907), Priority(0.6263664), Priority(0.82619095), Priority(0.31927645)]), (4, [Priority(0.17672324), Priority(0.61523604), Priority(0.10236049), Priority(0.25020695)]), (0, [Priority(0.35751522), Priority(0.7829473), Priority(0.89694524), Priority(0.44885302)])]
//         // TMProposeSet5: [(1, [Priority(0.66629565), Priority(0.8087758), Priority(0.8052678), Priority(0.2654072)]), (3, [Priority(0.087144256), Priority(0.98617744), Priority(0.42974603), Priority(0.7687563)]), (2, [Priority(0.56976545), Priority(0.549078), Priority(0.49572527), Priority(0.56478834)]), (4, [Priority(0.04828632), Priority(0.72194374), Priority(0.07470703), Priority(0.58566886)]), (0, [Priority(0.4854673), Priority(0.7915108), Priority(0.15995121), Priority(0.3351606)])]
//         // TMProposeSetBowOut: [(1, [Priority(0.630779), Priority(0.009342313), Priority(0.5053445), Priority(0.5008931)]), (3, [Priority(0.42792606), Priority(0.8164432), Priority(0.74793696), Priority(0.17093804)]), (2, [Priority(0.6185162), Priority(0.2116512), Priority(0.32827044), Priority(0.05437839)]), (4, [Priority(0.5176934), Priority(0.14106882), Priority(0.31658903), Priority(0.1994152)]), (0, [Priority(0.10672915), Priority(0.5429306), Priority(0.5637684), Priority(0.5221151)])]
//         // TMStatusChange: [(1, [Priority(0.050421596), Priority(0.2633897), Priority(0.6131108), Priority(0.60115075)]), (3, [Priority(0.029076576), Priority(0.7356303), Priority(0.8202771), Priority(0.5402722)]), (2, [Priority(0.6991321), Priority(0.060041666), Priority(0.7084172), Priority(0.58083)]), (4, [Priority(0.81145227), Priority(0.54099905), Priority(0.9015188), Priority(0.8516823)]), (0, [Priority(0.47086847), Priority(0.5945262), Priority(0.16536939), Priority(0.48721367)])]
//         // TMValidation: [(1, [Priority(0.918996), Priority(0.08246899), Priority(0.62088454), Priority(0.08493042)]), (3, [Priority(0.45811725), Priority(0.28948012), Priority(0.509814), Priority(0.96958375)]), (2, [Priority(0.97959566), Priority(0.10885322), Priority(0.058987617), Priority(0.68486863)]), (4, [Priority(0.82071245), Priority(0.84900856), Priority(0.32491088), Priority(0.9100052)]), (0, [Priority(0.07816386), Priority(0.48995686), Priority(0.6721231), Priority(0.2606318)])]
//         // TMTransaction: [(1, [Priority(0.16221309), Priority(0.86350596), Priority(0.97483873), Priority(0.12248433)]), (3, [Priority(0.095977664), Priority(0.25667787), Priority(0.23662245), Priority(0.31017923)]), (2, [Priority(0.51350987), Priority(0.3232484), Priority(0.39501047), Priority(0.00889725)]), (4, [Priority(0.16440034), Priority(0.5924622), Priority(0.8179934), Priority(0.2442745)]), (0, [Priority(0.07581806), Priority(0.5565827), Priority(0.6443323), Priority(0.29505134)])]
//         // TMHaveTransactionSet: [(1, [Priority(0.6468364), Priority(0.46406746), Priority(0.6468277), Priority(0.49094617)]), (3, [Priority(0.13504577), Priority(0.7944745), Priority(0.055434942), Priority(0.193138)]), (2, [Priority(0.64822674), Priority(0.627885), Priority(0.173069), Priority(0.99294543)]), (4, [Priority(0.21697998), Priority(0.21544266), Priority(0.13577616), Priority(0.84328574)]), (0, [Priority(0.4549843), Priority(0.77936137), Priority(0.20335639), Priority(0.24028987)])]
//         // TMGetLedger: [(1, [Priority(0.16872752), Priority(0.99359), Priority(0.61834824), Priority(0.12043285)]), (3, [Priority(0.5526805), Priority(0.7197696), Priority(0.8407577), Priority(0.22996545)]), (2, [Priority(0.89769745), Priority(0.053411126), Priority(0.23527682), Priority(0.22479701)]), (4, [Priority(0.2096014), Priority(0.52428484), Priority(0.30431294), Priority(0.10886669)]), (0, [Priority(0.45502448), Priority(0.46028447), Priority(0.9892378), Priority(0.6525278)])]
//         // TMLedgerData: [(1, [Priority(0.65836406), Priority(0.9801972), Priority(0.4248892), Priority(0.34680486)]), (3, [Priority(0.9703009), Priority(0.5296693), Priority(0.9237437), Priority(0.9692242)]), (2, [Priority(0.536219), Priority(0.5497365), Priority(0.4864967), Priority(0.32873857)]), (4, [Priority(0.15383184), Priority(0.87073255), Priority(0.62224734), Priority(0.44015813)]), (0, [Priority(0.4716047), Priority(0.96685636), Priority(0.27913928), Priority(0.99627495)])]
//         // delays[index_factor_1 * 1 + index_factor_2 * 0 + 0] = 2000;
//         let delay = 3000;
//         delays[index_factor_1 * 2 + index_factor_2 * 0 + 0] = delay;
//         delays[index_factor_1 * 2 + index_factor_2 * 1 + 0] = delay;
//         delays[index_factor_1 * 2 + index_factor_2 * 3 + 0] = delay;
//         delays[index_factor_1 * 2 + index_factor_2 * 4 + 0] = delay;
//         delays[index_factor_1 * 1 + index_factor_2 * 0 + 0] = delay;
//         delays[index_factor_1 * 1 + index_factor_2 * 2 + 0] = delay;
//         delays[index_factor_1 * 1 + index_factor_2 * 3 + 0] = delay;
//         delays[index_factor_1 * 1 + index_factor_2 * 4 + 0] = delay;
//         delays[index_factor_1 * 0 + index_factor_2 * 1 + 0] = delay;
//         delays[index_factor_1 * 0 + index_factor_2 * 2 + 0] = delay;
//         delays[index_factor_1 * 0 + index_factor_2 * 3 + 0] = delay;
//         delays[index_factor_1 * 0 + index_factor_2 * 4 + 0] = delay;
//         delays
//     }
// }

#[allow(unused)]
pub fn run_delay_trace_graph_creation<T>(scheduler_sender: Sender<DelayMapPhenotype>, scheduler_receiver: Receiver<T>, node_states: Arc<MutexNodeStates>)
    where T: ExtendedFitness + 'static
{
    let mut scheduler_handler = DelayTraceGraphSchedulerHandler::new(scheduler_sender, scheduler_receiver);
    thread::spawn(move || scheduler_handler.delay_trace_graph_creation(node_states));
}

#[allow(unused)]
pub fn run_priority_trace_graph_creation<T>(scheduler_sender: Sender<PriorityMapPhenotype>, scheduler_receiver: Receiver<T>, node_states: Arc<MutexNodeStates>)
    where T: ExtendedFitness + 'static
{
    let mut scheduler_handler = PriorityTraceGraphSchedulerHandler::new(scheduler_sender, scheduler_receiver);
    thread::spawn(move || scheduler_handler.priority_trace_graph_creation(node_states));
}

#[allow(unused)]
pub fn run_fitness_comparison(scheduler_sender: Sender<DelayMapPhenotype>, scheduler_receiver: Receiver<CurrentFitness>) {
    let mut scheduler_handler = FitnessComparisonSchedulerHandler::new(scheduler_sender, scheduler_receiver);
    thread::spawn(move || scheduler_handler.fitness_comparison());
}

#[allow(unused)]
pub fn run_no_delays(scheduler_sender: Sender<DelayMapPhenotype>, scheduler_receiver: Receiver<CurrentFitness>, number_of_tests: usize) {
    let mut scheduler_handler = NoDelaySchedulerHandler::new(scheduler_sender, scheduler_receiver, number_of_tests);
    thread::spawn(move || scheduler_handler.run());
}

#[allow(unused)]
pub fn run_predetermined_delays(scheduler_sender: Sender<DelayMapPhenotype>, scheduler_receiver: Receiver<CurrentFitness>, number_of_tests: usize) {
    let mut scheduler_handler = PreDeterminedDelaySchedulerHandler::new(scheduler_sender, scheduler_receiver, number_of_tests);
    thread::spawn(move || scheduler_handler.run());
}

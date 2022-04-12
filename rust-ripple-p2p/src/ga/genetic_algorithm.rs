use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, RwLock};
use std::thread;
use genevo::ga::genetic_algorithm;
use genevo::genetic::{Phenotype};
use genevo::operator::prelude::{MultiPointCrossover};
use genevo::operator::{CrossoverOp, GeneticOperator, SelectionOp};
use itertools::{chain, Itertools};
use genevo::prelude::{GenerationLimit, Population, SimResult, simulate, Simulation, SimulationBuilder};
use genevo::reinsertion::elitist::ElitistReinserter;
#[allow(unused_imports)]
use crate::ga::crossover::NoCrossoverOperator;
#[allow(unused_imports)]
use crate::ga::fitness::state_accounting_fitness::StateAccountFitness;
use crate::ga::fitness::{ExtendedFitness, FitnessCalculation, SchedulerHandler, SchedulerHandlerTrait};
#[allow(unused_imports)]
use crate::ga::fitness::compared_fitness_functions::ComparedFitnessFunctions;
use crate::ga::fitness::time_fitness::TimeFitness;
use crate::ga::mutation::GaussianGenomeMutation;
use crate::ga::parameters::Parameter;
use crate::NUM_NODES;
use super::mutation::GaussianMutator;

pub type CurrentFitness = TimeFitness;
pub const DROP_THRESHOLD: u32 = 1800;

pub(crate) fn num_genes() -> usize {
    *NUM_NODES * (*NUM_NODES-1) * ConsensusMessageType::VALUES.len()
}

/// The message types that will be subject to delay
#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub enum ConsensusMessageType {
    TMProposeSet,
    TMStatusChange,
    TMValidation,
    TMHaveTransactionSet,
}

impl ConsensusMessageType {
    pub const VALUES: [Self; 4] = [Self::TMProposeSet, Self::TMStatusChange, Self::TMValidation, Self::TMHaveTransactionSet];
    pub const RMO_MESSAGE_TYPE: [&'static str; 4] = ["ProposeSet", "StatusChange", "Validation", "HaveTransactionSet"];
}

// The phenotype from -> to -> message_type -> delay (ms)
type DelayMap = HashMap<usize, HashMap<usize, HashMap<ConsensusMessageType, u32>>>;

pub trait ExtendedPhenotype<G>: Phenotype<G> + Send where G: ExtendedGenotype {
    fn from_genes(geno: &G) -> Self;
}

pub trait ExtendedGenotype: MultiPointCrossover + GaussianGenomeMutation + Eq + Hash + Debug + Default {}

impl ExtendedGenotype for DelaysGenotype{}

/// Contains the delayMap for easy use in the scheduler and delays as genotype (vec)
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DelayMapPhenotype {
    pub delay_map: DelayMap,
    delays: DelaysGenotype
}

impl DelayMapPhenotype {
    /// Display delays grouped by message and receiver node
    pub fn message_type_delays(&self, message_type: &ConsensusMessageType) -> Vec<(usize, Vec<u32>)> {
        self.delay_map.iter()
            .map(|(to, from)| (*to, from.values()
                .map(|x| *x.get(message_type).unwrap())
                .collect_vec()))
            .collect::<Vec<(usize, Vec<u32>)>>()
    }

    pub fn display_delays_by_message(&self) {
        for message_type in ConsensusMessageType::VALUES {
            println!("{:?}: {:?}", message_type, self.message_type_delays(&message_type))
        }
    }
}

impl Phenotype<DelaysGenotype> for DelayMapPhenotype {
    fn genes(&self) -> DelaysGenotype {
        self.delays.clone()
    }

    fn derive(&self, new_genes: DelaysGenotype) -> Self {
        DelayMapPhenotype::from_genes(&new_genes)
    }
}

impl ExtendedPhenotype<DelaysGenotype> for DelayMapPhenotype {
    fn from_genes(genes: &DelaysGenotype) -> Self {
        let index_factor_1 = ConsensusMessageType::VALUES.len() * (*NUM_NODES-1);
        let index_factor_2 = ConsensusMessageType::VALUES.len();
        let mut from_node = HashMap::new();
        for i in 0..*NUM_NODES {
            let mut to_node = HashMap::new();
            for (j, node) in chain(0..i, i+1..*NUM_NODES).enumerate() {
                let mut message_type = HashMap::new();
                for (k, message) in ConsensusMessageType::VALUES.iter().enumerate() {
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

// The genotype
pub(crate) type DelaysGenotype = Vec<u32>;

/// Run the genetic algorithm
#[allow(unused)]
pub fn run<S, C, T, G, P>(scheduler_sender: Sender<P>, scheduler_receiver: Receiver<T>, params: Parameter<S, C, T, G>, initial_population: Population<G>)
    where S: SelectionOp<G, T> + Debug, C: CrossoverOp<G> + Debug + Sync, T: ExtendedFitness + 'static, G: ExtendedGenotype + 'static, P: ExtendedPhenotype<G> + 'static
{
    let (fitness_sender, fitness_receiver) = channel();
    let fitness_values: Arc<RwLock<HashMap<G, T>>> = Arc::new(RwLock::new(HashMap::new()));
    let scheduler_handler = SchedulerHandler::new(scheduler_sender, scheduler_receiver, fitness_receiver, fitness_values.clone());
    let fitness_calculation = FitnessCalculation { fitness_values: fitness_values.clone(), sender: fitness_sender };

    run_ga(fitness_values, scheduler_handler, fitness_calculation, params, initial_population);
}

/// Run a standard mu lambda GA
#[allow(unused)]
pub fn run_mu_lambda<T, S, C, G, P>(mu: usize, lambda: usize, scheduler_sender: Sender<P>, scheduler_receiver: Receiver<T>, crossover_operator: C, params: Parameter<S, C, T, G>, initial_population: Population<G>)
    where T: ExtendedFitness + Debug + 'static, S: SelectionOp<G, T> + Debug, C: CrossoverOp<G> + GeneticOperator + Sync + Debug, G: ExtendedGenotype + 'static, P: ExtendedPhenotype<G> + 'static
{
    match C::name().as_str() {
        "No Crossover Operator" => {}
        _ => {
            if mu == 1 {
                panic!("Cannot apply binary variation operator to single parent GA");
            }
        }
    }
    // let params = mu_lambda(mu, lambda, None, 2000, 50, 0.1, crossover_operator);
    // Create initial population of size lambda, uniformly distributed over the range of possible values

    let (fitness_sender, fitness_receiver) = channel();
    let fitness_values: Arc<RwLock<HashMap<G, T>>> = Arc::new(RwLock::new(HashMap::new()));
    let scheduler_handler = SchedulerHandler::new(scheduler_sender, scheduler_receiver, fitness_receiver, fitness_values.clone());
    let fitness_calculation = FitnessCalculation { fitness_values: fitness_values.clone(), sender: fitness_sender };

    run_ga(fitness_values, scheduler_handler, fitness_calculation, params, initial_population);
}

#[allow(unused)]
pub fn run_no_ga(number_of_tests: usize) {
    let zero_delay = vec![0; num_genes()];
    for i in 0..number_of_tests {

    }
}

pub fn run_ga<S, C, T, H, G>(fitness_values: Arc<RwLock<HashMap<G, T>>>, scheduler_handler: H, fitness_calculation: FitnessCalculation<T, G>, params: Parameter<S, C, T, G>, initial_population: Population<G>)
    where S: SelectionOp<G, T> + Debug, C: CrossoverOp<G> + Debug + Sync, T: ExtendedFitness + 'static, H: SchedulerHandlerTrait + Send + 'static, G: ExtendedGenotype
{

    println!("{:?}", initial_population);

    thread::spawn(move || scheduler_handler.run());

    let ga = genetic_algorithm()
        .with_evaluation(fitness_calculation.clone())
        .with_selection(params.selection_operator.clone())
        .with_crossover(params.crossover_operator.clone())
        .with_mutation(GaussianMutator::new(params.mutation_rate, params.mutation_std))
        // reinsertion_ratio is only used if offspring_has_precedence to determine the number of offspring to choose
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
    // fitness_values.write().unwrap().clear();
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
                    step.duration,
                    step.processing_time
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
                    duration,
                    step.iteration,
                    best_solution.solution.fitness,
                    best_solution.generation,
                    processing_time
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
    use std::collections::HashMap;
    use std::sync::{Arc, RwLock};
    use std::sync::mpsc::{Receiver};
    use crate::ga::crossover::NoCrossoverOperator;
    use crate::ga::fitness::{FitnessCalculation, SchedulerHandlerTrait};
    use crate::ga::genetic_algorithm::{DelayMapPhenotype, DelaysGenotype, ConsensusMessageType, mu_lambda, run_ga, ExtendedPhenotype};
    use crate::ga::fitness::validated_ledgers_fitness::ValidatedLedgersFitness;

    #[test]
    fn test_phenotype() {
        //let genotype: DelaysGenotype = (1..81).collect_vec();
        let genotype: DelaysGenotype = vec![959, 533, 12, 717, 406, 603, 767, 0, 304, 366, 925, 54, 854, 159, 611, 747, 839, 555, 985, 146, 678, 499, 67, 802, 991, 557, 185, 312, 557, 676, 659, 149, 963, 347, 817, 987, 451, 972, 515, 631, 174, 564, 551, 889, 665, 527, 645, 336, 977, 946, 641, 441, 113, 872, 778, 385, 878, 528, 947, 435, 913, 643, 4, 101, 472, 416, 624, 792, 925, 573, 225, 948, 862, 142, 580, 50, 742, 648, 338, 914];
        let phenotype = DelayMapPhenotype::from_genes(&genotype);
        println!("{:?}", phenotype.delay_map);
        println!("{:?}", phenotype.message_type_delays(&ConsensusMessageType::TMValidation));
        phenotype.display_delays_by_message();
    }

    #[test]
    fn test_mu_lambda() {
        let params = mu_lambda(1, 2, Some(1), 2000, 5, 1.0, NoCrossoverOperator{});

        let (fitness_sender, fitness_receiver) = std::sync::mpsc::channel();
        let fitness_values: Arc<RwLock<HashMap<DelaysGenotype, ValidatedLedgersFitness>>> = Arc::new(RwLock::new(HashMap::new()));
        let scheduler_handler = TestSchedulerHandler::new(fitness_receiver, fitness_values.clone());
        let fitness_calculation = FitnessCalculation { fitness_values: fitness_values.clone(), sender: fitness_sender };

        run_ga(fitness_values, scheduler_handler, fitness_calculation, params);
    }

    struct TestSchedulerHandler {
        fitness_receiver: Receiver<DelaysGenotype>,
        fitness_values: Arc<RwLock<HashMap<DelaysGenotype, ValidatedLedgersFitness>>>
    }

    impl TestSchedulerHandler {
        pub fn new(
            fitness_receiver: Receiver<DelaysGenotype>,
            fitness_values: Arc<RwLock<HashMap<DelaysGenotype, ValidatedLedgersFitness>>>,
        ) -> Self {
            TestSchedulerHandler { fitness_receiver, fitness_values }
        }
    }

    impl SchedulerHandlerTrait for TestSchedulerHandler {
        fn run(self) {
            loop {
                // Receive a new individual to test from a fitness function
                match self.fitness_receiver.recv() {
                    Ok(delays_genotype) => match &delays_genotype[..] {
                        x => {
                            println!("Received {:?} from fitness calculation", x);
                            self.fitness_values.write().unwrap().insert(delays_genotype.clone(), ValidatedLedgersFitness::new(x[0]));
                        }
                    },
                    Err(_) => {}
                }
            }
        }
    }
}

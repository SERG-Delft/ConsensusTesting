use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::File;
use std::hash::Hash;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, RwLock};
use std::thread;
use serde_with::{serde_as, DurationSeconds};
use chrono::Duration;
use genevo::ga::genetic_algorithm;
use genevo::mutation::order::SwapOrderMutator;
use genevo::operator::{CrossoverOp, SelectionOp};
use genevo::operator::prelude::{MaximizeSelector, PartiallyMappedCrossover};
use genevo::prelude::{Population, SimResult, simulate, Simulation, SimulationBuilder, TimeLimit};
use log::error;
use crate::{CONFIG, LOG_FOLDER};
use crate::ga::crossover::SimulatedBinaryCrossBreeder;
use crate::ga::encoding::delay_encoding::{DelayMapPhenotype, DelayGenotype};
use crate::ga::encoding::{ExtendedGenotype, ExtendedPhenotype, num_genes, SuperExtendedGenotype};
#[allow(unused_imports)]
use crate::ga::fitness::state_accounting_fitness::StateAccountFitness;
use crate::ga::fitness::{ExtendedFitness, FitnessCalculation, SchedulerHandler, SchedulerHandlerTrait};
#[allow(unused_imports)]
use crate::ga::fitness::compared_fitness_functions::ComparedFitnessFunctions;
#[allow(unused_imports)]
use crate::ga::fitness::time_fitness::TimeFitness;
use crate::ga::parameters::{default_mu_lambda_delays, default_mu_lambda_priorities, Parameter, PermutationParameters};
use crate::ga::population_builder::{build_delays_population, build_priorities_population};
use crate::ga::encoding::priority_encoding::{PriorityGenotype, PriorityMapPhenotype};
#[allow(unused_imports)]
use crate::ga::fitness::propose_seq_fitness::ProposeSeqFitness;
use crate::ga::selection::MuLambdaSelector;
use crate::ga::reinsertion::MuLambdaReinserter;
use crate::message_handler::RippleMessageObject;
use super::mutation::GaussianMutator;

/// The message types that will be subject to delay
#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub enum ConsensusMessageType {
    TMProposeSet0,
    TMProposeSet1,
    TMProposeSet2,
    TMProposeSet3,
    TMProposeSet4,
    TMProposeSet5,
    TMProposeSetBowOut,
    TMStatusChange,
    TMValidation,
    TMTransaction,
    TMHaveTransactionSet,
    TMGetLedger,
    TMLedgerData
}

impl ConsensusMessageType {
    pub const VALUES: [Self; 13] = [Self::TMProposeSet0, Self::TMProposeSet1, Self::TMProposeSet2, Self::TMProposeSet3, Self::TMProposeSet4, Self::TMProposeSet5, Self::TMProposeSetBowOut, Self::TMStatusChange, Self::TMValidation, Self::TMTransaction, Self::TMHaveTransactionSet, Self::TMGetLedger, Self::TMLedgerData];
    pub const RMO_MESSAGE_TYPE: [&'static str; 7] = ["ProposeSet", "StatusChange", "Validation", "Transaction", "HaveTransactionSet", "GetLedger", "LedgerData"];

    pub fn create_consensus_message_type(rmo: &RippleMessageObject) -> Option<Self> {
        match rmo {
            RippleMessageObject::TMValidation(_) => Some(ConsensusMessageType::TMValidation),
            RippleMessageObject::TMProposeSet(proposal) => {
                match proposal.get_proposeSeq() {
                    0 => Some(ConsensusMessageType::TMProposeSet0),
                    1 => Some(ConsensusMessageType::TMProposeSet1),
                    2 => Some(ConsensusMessageType::TMProposeSet2),
                    3 => Some(ConsensusMessageType::TMProposeSet3),
                    4 => Some(ConsensusMessageType::TMProposeSet4),
                    5 => Some(ConsensusMessageType::TMProposeSet5),
                    4294967295 => Some(ConsensusMessageType::TMProposeSetBowOut),
                    _ => Some(ConsensusMessageType::TMProposeSet0),
                }
            },
            RippleMessageObject::TMStatusChange(_) => Some(ConsensusMessageType::TMStatusChange),
            RippleMessageObject::TMHaveTransactionSet(_) => Some(ConsensusMessageType::TMHaveTransactionSet),
            RippleMessageObject::TMTransaction(_) => Some(ConsensusMessageType::TMTransaction),
            RippleMessageObject::TMLedgerData(_) => Some(ConsensusMessageType::TMLedgerData),
            RippleMessageObject::TMGetLedger(_) => Some(ConsensusMessageType::TMGetLedger),
            _ => None
        }
    }
}

/// Run a standard mu lambda GA with delay encoding
#[allow(unused)]
pub fn run_default_mu_lambda_delays<F: ExtendedFitness>(mu: usize, lambda: usize, scheduler_sender: Sender<DelayMapPhenotype>, scheduler_receiver: Receiver<F>) {
    let params = default_mu_lambda_delays(mu, lambda);
    let population = build_delays_population(params.num_genes, params.min_value, params.max_value, lambda);

    let (fitness_sender, fitness_receiver) = channel();
    let fitness_values: Arc<RwLock<HashMap<DelayGenotype, F>>> = Arc::new(RwLock::new(HashMap::new()));
    let scheduler_handler = SchedulerHandler::new(scheduler_sender, scheduler_receiver, fitness_receiver, fitness_values.clone());
    let fitness_calculation = FitnessCalculation { fitness_values: fitness_values.clone(), sender: fitness_sender };

    run_ga::<MaximizeSelector, SimulatedBinaryCrossBreeder, F, SchedulerHandler<F, DelayGenotype, DelayMapPhenotype>, DelayGenotype, DelayMapPhenotype>(scheduler_handler, fitness_calculation, params, population);
}

/// Run a standard mu lambda GA with priority encoding
#[allow(unused)]
pub fn run_default_mu_lambda_priorities<F: ExtendedFitness>(mu: usize, lambda: usize, scheduler_sender: Sender<PriorityMapPhenotype>, scheduler_receiver: Receiver<F>) {
    let params = default_mu_lambda_priorities::<F>(mu, lambda);
    let population = build_priorities_population(params.num_genes, params.population_size);

    let (fitness_sender, fitness_receiver) = channel();
    let fitness_values: Arc<RwLock<HashMap<PriorityGenotype, F>>> = Arc::new(RwLock::new(HashMap::new()));
    let scheduler_handler = SchedulerHandler::new(scheduler_sender, scheduler_receiver, fitness_receiver, fitness_values.clone());
    let fitness_calculation = FitnessCalculation { fitness_values: fitness_values.clone(), sender: fitness_sender };

    run_permutation_ga::<MaximizeSelector, F, SchedulerHandler<F, PriorityGenotype, PriorityMapPhenotype>>(scheduler_handler, fitness_calculation, params, population);
}

#[allow(unused)]
pub fn run_no_ga(number_of_tests: usize) {
    let zero_delay = vec![0; num_genes()];
    for i in 0..number_of_tests {

    }
}

pub fn run_ga<S, C, T, H, G, P>(scheduler_handler: H, fitness_calculation: FitnessCalculation<T, G>, params: Parameter<MuLambdaSelector, SimulatedBinaryCrossBreeder, T, DelayGenotype>, initial_population: Population<G>)
    where S: SelectionOp<G, T> + Debug, C: CrossoverOp<G> + Debug + Sync, T: ExtendedFitness + serde::Serialize + 'static, H: SchedulerHandlerTrait + Send + 'static, G: SuperExtendedGenotype + serde::Serialize, P: ExtendedPhenotype<G>
{
    println!("{:?}", initial_population);

    thread::spawn(move || scheduler_handler.run());

    let ga = genetic_algorithm()
        .with_evaluation(fitness_calculation.clone())
        .with_selection(params.selection_operator.clone())
        .with_crossover(params.crossover_operator.clone())
        .with_mutation(GaussianMutator::new(params.mutation_rate, params.mutation_std))
        .with_reinsertion(MuLambdaReinserter::new(
            fitness_calculation,
            params.population_size,
        ))
        .with_initial_population(initial_population)
        .build();

    let mut sim = simulate(ga)
        .until(TimeLimit::new(CONFIG.search_budget))
        .build();

    let mut ga_writer = create_ga_writer();
    match serde_json::to_writer_pretty(&mut ga_writer, &params) {
        Ok(_) => {}
        Err(err) => error!("Failed writing to ga file: {}", err)
    };
    ga_writer.flush().expect("GA writer flush failed");

    println!("Starting GA with: {:?}", params);
    loop {
        let result = sim.step();
        match result {
            Ok(SimResult::Intermediate(step)) => {
                let evaluated_population = step.result.evaluated_population;
                let best_solution = step.result.best_solution;
                let generation_info: GaStepInfo<T, G> = GaStepInfo::new(
                    step.iteration,
                    evaluated_population.average_fitness().clone(),
                    best_solution.solution.fitness,
                    best_solution.solution.genome,
                    step.duration,
                );
                println!("{}", serde_json::to_string(&generation_info).unwrap());
                match serde_json::to_writer_pretty(&mut ga_writer, &generation_info) {
                    Ok(_) => {}
                    Err(err) => error!("Failed writing to ga file: {}", err)
                };
                ga_writer.flush().expect("GA writer flush failed");
                //                println!("| population: [{}]", result.population.iter().map(|g| g.as_text())
                //                    .collect::<Vec<String>>().join("], ["));
            },
            Ok(SimResult::Final(step, _processing_time, duration, stop_reason)) => {
                let evaluated_population = step.result.evaluated_population;
                let best_solution = step.result.best_solution;
                let generation_info: GaStepInfo<T, G> = GaStepInfo::new(
                    step.iteration,
                    evaluated_population.average_fitness().clone(),
                    best_solution.solution.fitness,
                    best_solution.solution.genome,
                    duration,
                );
                match serde_json::to_writer_pretty(&mut ga_writer, &generation_info) {
                    Ok(_) => {}
                    Err(err) => error!("Failed writing to ga file: {}", err)
                };
                ga_writer.flush().expect("GA writer flush failed");
                println!("{}", stop_reason);
                println!("{}", serde_json::to_string(&generation_info).unwrap());
                print!("      ");
                println!("{}", P::from_genes(&generation_info.best_individual).display_genotype_by_message());
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

pub fn run_permutation_ga<S, T, H>(scheduler_handler: H, fitness_calculation: FitnessCalculation<T, PriorityGenotype>, params: PermutationParameters<MuLambdaSelector, T, PriorityGenotype>, initial_population: Population<PriorityGenotype>)
    where S: SelectionOp<PriorityGenotype, T> + Debug, T: ExtendedFitness + serde::Serialize + 'static, H: SchedulerHandlerTrait + Send + 'static
{
    println!("{:?}", initial_population);

    thread::spawn(move || scheduler_handler.run());

    let ga = genetic_algorithm()
        .with_evaluation(fitness_calculation.clone())
        .with_selection(params.selection_operator.clone())
        .with_crossover(PartiallyMappedCrossover::new())
        .with_mutation(SwapOrderMutator::new(params.mutation_rate))
        .with_reinsertion(MuLambdaReinserter::new(
            fitness_calculation,
            params.population_size,
        ))
        .with_initial_population(initial_population)
        .build();

    let mut sim = simulate(ga)
        .until(TimeLimit::new(CONFIG.search_budget))
        .build();

    let mut ga_writer = create_ga_writer();
    match serde_json::to_writer_pretty(&mut ga_writer, &params) {
        Ok(_) => {}
        Err(err) => error!("Failed writing to ga file: {}", err)
    };
    ga_writer.flush().expect("GA writer flush failed");

    println!("Starting GA with: {:?}", params);
    loop {
        let result = sim.step();
        match result {
            Ok(SimResult::Intermediate(step)) => {
                let evaluated_population = step.result.evaluated_population;
                let best_solution = step.result.best_solution;
                let generation_info: GaStepInfo<T, PriorityGenotype> = GaStepInfo::new(
                    step.iteration,
                    evaluated_population.average_fitness().clone(),
                    best_solution.solution.fitness,
                    best_solution.solution.genome,
                    step.duration,
                );
                println!("{}", serde_json::to_string(&generation_info).unwrap());
                match serde_json::to_writer_pretty(&mut ga_writer, &generation_info) {
                    Ok(_) => {}
                    Err(err) => error!("Failed writing to ga file: {}", err)
                };
                ga_writer.flush().expect("GA writer flush failed");
                // println!("Best individual:");
                // println!("{}", PriorityMapPhenotype::from_genes(&best_solution.solution.genome).display_genotype_by_message());
                //                println!("| population: [{}]", result.population.iter().map(|g| g.as_text())
                //                    .collect::<Vec<String>>().join("], ["));
            },
            Ok(SimResult::Final(step, _processing_time, duration, stop_reason)) => {
                let evaluated_population = step.result.evaluated_population;
                let best_solution = step.result.best_solution;
                let generation_info: GaStepInfo<T, PriorityGenotype> = GaStepInfo::new(
                    step.iteration,
                    evaluated_population.average_fitness().clone(),
                    best_solution.solution.fitness,
                    best_solution.solution.genome,
                    duration,
                );
                match serde_json::to_writer_pretty(&mut ga_writer, &generation_info) {
                    Ok(_) => {}
                    Err(err) => error!("Failed writing to ga file: {}", err)
                };
                ga_writer.flush().expect("GA writer flush failed");
                println!("{}", stop_reason);
                println!("{}", serde_json::to_string(&generation_info).unwrap());
                print!("      ");
                println!("{}", PriorityMapPhenotype::from_genes(&generation_info.best_individual).display_genotype_by_message());
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

pub fn create_ga_writer() -> BufWriter<File> {
    BufWriter::new(
        File::create(
            Path::new(format!("{}\\ga.txt", *LOG_FOLDER).as_str())
        ).expect("Creating ga file failed")
    )
}

#[serde_as]
#[derive(serde::Serialize)]
struct GaStepInfo<F: ExtendedFitness, G: ExtendedGenotype> {
    iteration: u64,
    average_fitness: F,
    best_fitness: F,
    best_individual: G,
    #[serde_as(as = "DurationSeconds<i64>")]
    duration: Duration,
}

impl<F: ExtendedFitness, G: ExtendedGenotype> GaStepInfo<F, G> {
    pub fn new(iteration: u64, average_fitness: F, best_fitness: F, best_individual: G, duration: Duration) -> Self {
        Self {
            iteration,
            average_fitness,
            best_fitness,
            best_individual,
            duration
        }
    }
}

#[cfg(test)]
mod ga_tests {
    use std::collections::HashMap;
    use std::sync::{Arc, RwLock};
    use std::sync::mpsc::{Receiver};
    use std::thread;
    use std::time::Duration;
    use genevo::ga::genetic_algorithm;
    use genevo::prelude::{GenerationLimit, SimResult, simulate, Simulation, SimulationBuilder};
    use crate::ga::encoding::delay_encoding::{DelayMapPhenotype, DelayGenotype};
    use crate::ga::encoding::num_genes;
    use crate::ga::fitness::{FitnessCalculation, SchedulerHandlerTrait};
    use crate::ga::genetic_algorithm::{ConsensusMessageType, ExtendedPhenotype, CurrentFitness};
    use crate::ga::mutation::{NoMutation};
    use crate::ga::parameters::default_mu_lambda_delays;
    use crate::ga::population_builder::build_delays_population;
    use crate::ga::reinsertion::MuLambdaReinserter;

    #[test]
    #[ignore]
    fn test_phenotype() {
        //let genotype: DelaysGenotype = (1..81).collect_vec();
        let genotype: DelayGenotype = vec![100u32; num_genes()];
        let phenotype = DelayMapPhenotype::from_genes(&genotype);
        println!("{:?}", phenotype.delay_map);
        println!("{:?}", phenotype.message_type_delays(&ConsensusMessageType::TMValidation));
        println!("{}", phenotype.display_genotype_by_message());
    }

    #[test]
    #[ignore]
    fn test_mu_lambda() {
        let mut params = default_mu_lambda_delays(2, 4);
        params.num_genes = 4;
        params.crossover_operator.set_crossover_probability(1.0);
        let population = build_delays_population(params.num_genes, params.min_value, params.max_value, params.population_size);

        let (fitness_sender, fitness_receiver) = std::sync::mpsc::channel();
        let fitness_values: Arc<RwLock<HashMap<DelayGenotype, CurrentFitness>>> = Arc::new(RwLock::new(HashMap::new()));
        let scheduler_handler = TestSchedulerHandler::new(fitness_receiver, fitness_values.clone());
        let fitness_calculation = FitnessCalculation { fitness_values: fitness_values.clone(), sender: fitness_sender };

        thread::spawn(move || scheduler_handler.run());

        let ga = genetic_algorithm()
            .with_evaluation(fitness_calculation.clone())
            .with_selection(params.selection_operator.clone())
            .with_crossover(params.crossover_operator.clone())
            .with_mutation(NoMutation{})
            .with_reinsertion(MuLambdaReinserter::new(
                fitness_calculation,
                params.population_size,
            ))
            .with_initial_population(population)
            .build();

        let mut sim = simulate(ga)
            .until(GenerationLimit::new(params.generation_limit))
            .build();

        loop {
            let result = sim.step();
            match result {
                Ok(SimResult::Intermediate(step)) => {
                    let evaluated_population = step.result.evaluated_population;
                    assert_eq!(evaluated_population.individuals().len(), 2);
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
                    println!("Individuals evaluated: {:?}", evaluated_population);
                    println!("Best individual: {:?}", &best_solution.solution.genome);
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
                    let evaluated_population = step.result.evaluated_population;
                    println!("Individuals evaluated: {:?}", evaluated_population);
                    println!("Best individual: {:?}", &best_solution.solution.genome);
                    print!("      ");
                    break;
                },
                Err(error) => {
                    println!("{:?}", error);
                    break;
                },
            }
        }
    }

    struct TestSchedulerHandler {
        fitness_receiver: Receiver<DelayGenotype>,
        fitness_values: Arc<RwLock<HashMap<DelayGenotype, CurrentFitness>>>
    }

    impl TestSchedulerHandler {
        pub fn new(
            fitness_receiver: Receiver<DelayGenotype>,
            fitness_values: Arc<RwLock<HashMap<DelayGenotype, CurrentFitness>>>,
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
                            self.fitness_values.write().unwrap().insert(delays_genotype.clone(), CurrentFitness::new(Duration::from_secs(x[0] as u64)));
                            // println!("Received {:?} from fitness calculation with fitness: {}", x, x[0]);
                            // self.fitness_values.write().unwrap().insert(delays_genotype.clone(), CurrentFitness::new(x[0]));
                        }
                    },
                    Err(_) => {}
                }
            }
        }
    }
}

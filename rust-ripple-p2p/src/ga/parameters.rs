use std::marker::PhantomData;
use genevo::genetic::Genotype;
use genevo::operator::{CrossoverOp, SelectionOp};
use genevo::operator::prelude::{MaximizeSelector, RouletteWheelSelector};
use crate::ga::crossover::{SimulatedBinaryCrossBreeder};
use crate::ga::encoding::delay_encoding::DelaysGenotype;
use crate::ga::encoding::{num_genes, SuperExtendedGenotype};
use crate::ga::fitness::ExtendedFitness;
use crate::ga::genetic_algorithm::{ConsensusMessageType, CurrentFitness};
use crate::ga::encoding::priority_encoding::{PriorityGenotype};

/// Parameters for the GA
#[allow(unused)]
#[derive(Debug)]
pub struct Parameter<S, C, T, G>
    where S: SelectionOp<G, T>, C: CrossoverOp<G>, T: ExtendedFitness, G: SuperExtendedGenotype
{
    pub(crate) population_size: usize,
    pub(crate) generation_limit: u64,
    num_individuals_per_parents: usize,
    num_crossover_points: usize,
    pub(crate) mutation_rate: f64,
    pub(crate) mutation_std: f64,
    pub(crate) reinsertion_ratio: f64,
    pub(crate) min_value: u32,
    pub(crate) max_value: u32,
    pub(crate) num_genes: usize,
    pub(crate) selection_operator: S,
    pub(crate) crossover_operator: C,
    stupid_type_system: PhantomData<T>,
    stupid_type_system_2: PhantomData<G>,
}

#[allow(unused)]
impl<S, C, G> Parameter<S, C, CurrentFitness, G> where S: SelectionOp<G, CurrentFitness>, C: CrossoverOp<G>, G: SuperExtendedGenotype {
    pub fn new(population_size: usize,
               generation_limit: u64,
               num_individuals_per_parents: usize,
               num_crossover_points: usize,
               mutation_rate: f64,
               mutation_std: f64,
               reinsertion_ratio: f64,
               min_value: u32,
               max_value: u32,
               num_genes: usize,
               selection_operator: S,
               crossover_operator: C,
    ) -> Self {
        Self {
            population_size,
            generation_limit,
            num_individuals_per_parents,
            num_crossover_points,
            mutation_rate,
            mutation_std,
            reinsertion_ratio,
            min_value,
            max_value,
            selection_operator,
            num_genes,
            crossover_operator,
            stupid_type_system: PhantomData,
            stupid_type_system_2: PhantomData
        }
    }

    pub fn default_delays() -> Parameter<RouletteWheelSelector, SimulatedBinaryCrossBreeder, CurrentFitness, DelaysGenotype> {
        Parameter {
            population_size: 8,
            generation_limit: 5,
            num_individuals_per_parents: 2,
            num_crossover_points: ConsensusMessageType::VALUES.len(),
            mutation_rate: 0.05,
            mutation_std: 50f64,
            reinsertion_ratio: 0.7,
            min_value: 0,
            max_value: 1000,
            num_genes: num_genes(),
            selection_operator: RouletteWheelSelector::new(0.7, 2),
            crossover_operator: SimulatedBinaryCrossBreeder::new(0.5),
            stupid_type_system: PhantomData,
            stupid_type_system_2: PhantomData
        }
    }

    // pub fn default_priorities() -> Parameter<RouletteWheelSelector, MultiPointCrossBreeder, CurrentFitness, PriorityGenotype> {
    //     Parameter {
    //         population_size: 8,
    //         generation_limit: 5,
    //         num_individuals_per_parents: 2,
    //         num_crossover_points: ConsensusMessageType::VALUES.len(),
    //         mutation_rate: 0.05,
    //         mutation_std: 50f64,
    //         reinsertion_ratio: 0.7,
    //         min_value: Priority(0u32),
    //         max_value: Priority(1000u32),
    //         num_genes: num_genes(),
    //         selection_operator: RouletteWheelSelector::new(0.7, 2),
    //         crossover_operator: MultiPointCrossBreeder::new(ConsensusMessageType::VALUES.len()),
    //         stupid_type_system: PhantomData,
    //         stupid_type_system_2: PhantomData
    //     }
    // }
}

pub fn default_mu_lambda_delays(mu: usize, lambda: usize) -> Parameter<MaximizeSelector, SimulatedBinaryCrossBreeder, CurrentFitness, DelaysGenotype> {
    let reinsertion_ratio = mu as f64 / lambda as f64;
    Parameter {
        population_size: lambda,
        generation_limit: 5,
        num_individuals_per_parents: 2,
        num_crossover_points: ConsensusMessageType::VALUES.len(),
        mutation_rate: 0.05,
        mutation_std: 50f64,
        reinsertion_ratio,
        min_value: 0,
        max_value: 2000,
        num_genes: num_genes(),
        selection_operator: MaximizeSelector::new(reinsertion_ratio, 2),
        crossover_operator: SimulatedBinaryCrossBreeder::new(0.5),
        stupid_type_system: PhantomData,
        stupid_type_system_2: PhantomData
    }
}

pub fn default_mu_lambda_priorities(mu: usize, lambda: usize) -> PermutationParameters<MaximizeSelector, CurrentFitness, PriorityGenotype> {
    let reinsertion_ratio = mu as f64 / lambda as f64;
    PermutationParameters {
        population_size: lambda,
        generation_limit: 100,
        num_individuals_per_parents: 2,
        mutation_rate: 0.05,
        mutation_std: 0.1f64,
        reinsertion_ratio,
        num_genes: num_genes(),
        selection_operator: MaximizeSelector::new(reinsertion_ratio, 2),
        stupid_type_system: PhantomData,
        stupid_type_system_2: PhantomData
    }
}

#[allow(unused)]
#[derive(Debug)]
pub struct PermutationParameters<S, T, G>
    where S: SelectionOp<G, T>, T: ExtendedFitness, G: Genotype
{
    pub(crate) population_size: usize,
    pub(crate) generation_limit: u64,
    num_individuals_per_parents: usize,
    pub(crate) mutation_rate: f64,
    pub(crate) mutation_std: f64,
    pub(crate) reinsertion_ratio: f64,
    pub(crate) num_genes: usize,
    pub(crate) selection_operator: S,
    stupid_type_system: PhantomData<T>,
    stupid_type_system_2: PhantomData<G>,
}
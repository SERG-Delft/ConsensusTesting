//! The provided `SelectionOp` implementations are:
//! * `MuLambdaSelector`

use genevo::algorithm::EvaluatedPopulation;
use genevo::genetic::{Fitness, Genotype, Parents};
use genevo::operator::{GeneticOperator, MultiObjective, SelectionOp, SingleObjective};
use itertools::Itertools;
use rand::prelude::SliceRandom;
use rand::Rng;

/// The `MuLambdaSelector` selects the best performing `genetic::Genotype`s
/// from the population.
///
/// This `MuLambdaSelector` can be used for single-objective fitness values
/// as well as multi-objective fitness values.
#[allow(missing_copy_implementations)]
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct MuLambdaSelector {
    /// Mu: the number of individuals to select as parents
    /// lambda: The number of parents tuples to create
    mu: usize,
    lambda: usize,
    /// The number of individuals per parents.
    num_individuals_per_parents: usize,
    // phantom types
}

impl MuLambdaSelector {
    /// Constructs a new instance of the `MaximizeSelector`.
    pub fn new(mu: usize, lambda: usize, num_individuals_per_parents: usize) -> Self {
        MuLambdaSelector {
            mu,
            lambda,
            num_individuals_per_parents,
        }
    }
}

/// Can be used for single-objective optimization
impl SingleObjective for MuLambdaSelector {}
/// Can be used for multi-objective optimization
impl MultiObjective for MuLambdaSelector {}

impl GeneticOperator for MuLambdaSelector {
    fn name() -> String {
        "MuLambda-Selection".to_string()
    }
}

impl<G, F> SelectionOp<G, F> for MuLambdaSelector
    where
        G: Genotype,
        F: Fitness,
{
    fn select_from<R>(&self, evaluated: &EvaluatedPopulation<G, F>, rng: &mut R) -> Vec<Parents<G>>
        where
            R: Rng + Sized,
    {
        let individuals = evaluated.individuals();
        let fitness_values = evaluated.fitness_values();

        // mating pool holds indices to the individuals and fitness_values slices
        let mut mating_pool: Vec<usize> = (0..fitness_values.len()).collect();
        // sort mating pool from best performing to worst performing index
        mating_pool.sort_by(|x, y| fitness_values[*y].cmp(&fitness_values[*x]));
        let mut mating_pool = mating_pool.into_iter().take(self.mu).collect_vec();
        mating_pool.shuffle(rng);

        let pool_size = mating_pool.len();
        let mut selected: Vec<Parents<G>> = Vec::with_capacity(self.mu);

        let mut index_m = 0;
        for _ in 0..self.lambda/2 {
            let mut tuple = Vec::with_capacity(self.num_individuals_per_parents);
            for _ in 0..self.num_individuals_per_parents {
                // index into mating pool
                index_m %= pool_size;
                // index into individuals slice
                let index_i = mating_pool[index_m];
                tuple.push(individuals[index_i].clone());
                index_m += 1;
            }
            selected.push(tuple);
        }
        selected
    }
}

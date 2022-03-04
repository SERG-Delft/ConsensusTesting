use genevo::genetic::{Children, Genotype, Parents};
use genevo::operator::{CrossoverOp, GeneticOperator};
use genevo::prelude::Rng;

#[derive(Clone, Debug, PartialEq)]
pub struct NoCrossoverOperator {}

impl GeneticOperator for NoCrossoverOperator {
    fn name() -> String {
        "No Crossover Operator".to_string()
    }
}

impl<G> CrossoverOp<G> for NoCrossoverOperator
    where G: Genotype
{
    fn crossover<R>(&self, parents: Parents<G>, rng: &mut R) -> Children<G> where R: Rng + Sized {
        parents
    }
}
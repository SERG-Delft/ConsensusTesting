use std::marker::PhantomData;
use genevo::algorithm::EvaluatedPopulation;
use genevo::genetic::Offspring;
use genevo::operator::{GeneticOperator, MultiObjective, ReinsertionOp, SingleObjective};
use genevo::prelude::*;
use itertools::Itertools;

/// This reinsertion operator takes the best mu individuals
/// from the mu parents + lambda offspring population
#[derive(Clone, Debug, PartialEq)]
pub struct MuLambdaReinserter<G, F, E>
    where
        G: Genotype,
        F: Fitness,
        E: FitnessFunction<G, F>,
{
    /// The `FitnessFunction` to be used to calculate fitness values of
    /// individuals of the offspring.
    fitness_evaluator: Box<E>,
    /// The `replace_ratio` defines the fraction of the population size that
    /// is going to be replaced by individuals from the offspring.
    mu: usize,
    // phantom types
    _g: PhantomData<G>,
    _f: PhantomData<F>,
}

impl<G, F, E> MuLambdaReinserter<G, F, E>
    where
        G: Genotype,
        F: Fitness,
        E: FitnessFunction<G, F>,
{
    /// Constructs a new instance of the `ElitistReinserter`.
    pub fn new(fitness_evaluator: E, mu: usize) -> Self {
        MuLambdaReinserter {
            fitness_evaluator: Box::new(fitness_evaluator),
            mu,
            _g: PhantomData,
            _f: PhantomData,
        }
    }
}

impl<G, F, E> GeneticOperator for MuLambdaReinserter<G, F, E>
    where
        G: Genotype,
        F: Fitness,
        E: FitnessFunction<G, F>,
{
    fn name() -> String {
        "MuLambda-Reinserter".to_string()
    }
}

/// Can be used for single-objective optimization
impl<G, F, E> SingleObjective for MuLambdaReinserter<G, F, E>
    where
        G: Genotype,
        F: Fitness,
        E: FitnessFunction<G, F>,
{
}
/// Can be used for multi-objective optimization
impl<G, F, E> MultiObjective for MuLambdaReinserter<G, F, E>
    where
        G: Genotype,
        F: Fitness,
        E: FitnessFunction<G, F>,
{
}

impl<G, F, E> ReinsertionOp<G, F> for MuLambdaReinserter<G, F, E>
    where
        G: Genotype,
        F: Fitness,
        E: FitnessFunction<G, F>,
{
    fn combine<R>(
        &self,
        offspring: &mut Offspring<G>,
        evaluated: &EvaluatedPopulation<G, F>,
        _: &mut R,
    ) -> Vec<G>
        where
            R: Rng + Sized,
    {
        let mut combined_individuals: Vec<(G, F)> = vec![];

        let old_individuals = evaluated.individuals();
        let old_fitness_values = evaluated.fitness_values();

        for i in 0..old_individuals.len() {
            combined_individuals.push((old_individuals[i].clone(), old_fitness_values[i].clone()));
        }

        // evaluate fitness of the offspring individuals
        while let Some(child) = offspring.pop() {
            let fitness = self.fitness_evaluator.fitness_of(&child);
            combined_individuals.push((child, fitness));
        }
        // sort offspring from worst to best performing performing
        combined_individuals.sort_by(|x, y| y.1.cmp(&x.1));

        combined_individuals.into_iter().take(old_individuals.len()).map(|(g, _)| g).collect_vec()
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;
    use genevo::algorithm::EvaluatedPopulation;
    use genevo::genetic::FitnessFunction;
    use genevo::operator::ReinsertionOp;
    use rand::thread_rng;
    use crate::ga::reinsertion::MuLambdaReinserter;

    #[test]
    fn test_reinsertion() {
        let mock_fitness = MockFitnessFunction{};
        let reinserter: MuLambdaReinserter<Vec<usize>, usize, MockFitnessFunction> = MuLambdaReinserter::new(mock_fitness, 2);
        let mut offspring = vec![vec![1], vec![3]];
        let evaluated_population = EvaluatedPopulation::new(Rc::new(vec![vec![0], vec![2]]), vec![0, 2], 2, 0, 1);
        let result = reinserter.combine(&mut offspring, &evaluated_population, &mut thread_rng());
        assert_eq!(result, vec![vec![3], vec![2]]);
    }

    #[derive(Clone)]
    struct MockFitnessFunction {}

    impl FitnessFunction<Vec<usize>, usize> for MockFitnessFunction {
        fn fitness_of(&self, a: &Vec<usize>) -> usize {
            a[0]
        }

        fn average(&self, a: &[usize]) -> usize {
            let count = a.len();
            a.iter().sum::<usize>() / count
        }

        fn highest_possible_fitness(&self) -> usize {
            usize::MAX
        }

        fn lowest_possible_fitness(&self) -> usize {
            usize::MIN
        }
    }
}

use std::fmt::Debug;
use rand_distr::{Normal, Distribution};
use genevo::{
    random::{random_index, Rng},
    operator::{GeneticOperator, MutationOp},
    prelude::Genotype,
};

#[derive(Clone, Debug, PartialEq)]
pub struct NoMutation {}

impl GeneticOperator for NoMutation {
    fn name() -> String {
        "No-Mutation".to_string()
    }
}

impl<G> MutationOp<G> for NoMutation
    where
        G: Genotype,
{
    fn mutate<R>(&self, genome: G, _rng: &mut R) -> G where R: Rng + Sized {
        genome
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GaussianMutator {
    mutation_rate: f64,
    standard_deviation: f64,
}

impl GaussianMutator {
    pub fn new(
        mutation_rate: f64,
        standard_deviation: f64,
    ) -> Self {
        GaussianMutator {
            mutation_rate,
            standard_deviation,
        }
    }
}

impl GeneticOperator for GaussianMutator {
    fn name() -> String {
        "Gaussian-Mutator".to_string()
    }
}

impl<G> MutationOp<G> for GaussianMutator
    where
        G: Genotype + GaussianGenomeMutation,
{
    fn mutate<R>(&self, genome: G, rng: &mut R) -> G
        where
            R: Rng + Sized,
    {
        GaussianGenomeMutation::mutate_genome(
            genome,
            self.mutation_rate,
            self.standard_deviation,
            rng,
        )
    }
}

pub trait GaussianGenomeMutation: Genotype {
    type Dna: Clone;

    fn mutate_genome<R>(
        genome: Self,
        mutation_rate: f64,
        standard_deviation: f64,
        rng: &mut R,
    ) -> Self
        where
            R: Rng + Sized;
}

impl<V> GaussianGenomeMutation for Vec<V>
    where
        V: Clone + Debug + PartialEq + Send + Sync + GaussianMutation,
{
    type Dna = V;

    fn mutate_genome<R>(
        genome: Self,
        mutation_rate: f64,
        standard_deviation: f64,
        rng: &mut R,
    ) -> Self
        where
            R: Rng + Sized,
    {
        let genome_length = genome.len();
        let num_mutations =
            ((genome_length as f64 * mutation_rate) + rng.gen::<f64>()).floor() as usize;
        let mut mutated = genome;
        for _ in 0..num_mutations {
            let index = random_index(rng, genome_length);
            mutated[index] = GaussianMutation::random_mutated(
                mutated[index].clone(),
                standard_deviation,
                rng,
            );
        }
        mutated
    }
}

pub trait GaussianMutation {
    fn random_mutated<R>(value: Self, standard_deviation: f64, rng: &mut R) -> Self
        where
            R: Rng + Sized;
}

macro_rules! impl_gaussian_mutation {
    ($($t:ty),*) => {
        $(
            impl GaussianMutation for $t {
                #[inline]
                fn random_mutated<R>(gene: $t, standard_deviation: f64, rng: &mut R) -> $t
                    where R: Rng + Sized
                {
                    let normal = Normal::new(gene, standard_deviation as $t).unwrap();
                    normal.sample(rng)
                }
            }
        )*
    }
}

impl_gaussian_mutation!(f32, f64);

macro_rules! impl_gaussian_mutation_integer {
    ($($t:ty),*) => {
        $(
            impl GaussianMutation for $t {
                #[inline]
                fn random_mutated<R>(gene: $t, standard_deviation: f64, rng: &mut R) -> $t
                    where R: Rng + Sized
                {
                    let normal = Normal::new(gene as f64, standard_deviation).unwrap();
                    normal.sample(rng).round() as $t
                }
            }
        )*
    }
}

impl_gaussian_mutation_integer!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize);

#[cfg(test)]
mod tests {
    use genevo::operator::MutationOp;
    use genevo::prelude::Seed;
    use genevo::random::get_rng;
    use crate::ga::mutation::GaussianMutator;

    #[test]
    fn gaussian_mutation_int() {
        let mutator = GaussianMutator::new(1f64, 10f64);
        let chromosome = vec![500, 500, 500, 500];
        let seed: Seed = Seed::default();
        let mut rng = get_rng(seed);
        assert_eq!(vec![515, 495, 500, 496], mutator.mutate(chromosome, &mut rng));
    }

    #[test]
    fn gaussian_mutation_float() {
        let mutator = GaussianMutator::new(1f64, 10f64);
        let chromosome = vec![500f64, 500f64, 500f64, 500f64];
        let seed: Seed = Seed::default();
        let mut rng = get_rng(seed);
        assert_eq!(vec![514.6929837294464, 494.54409703529524, 500.0, 496.14762940270674], mutator.mutate(chromosome, &mut rng));
    }
}
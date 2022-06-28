use genevo::population::{build_population, GenomeBuilder, Population, ValueEncodedGenomeBuilder};
use genevo::prelude::Rng;
use rand::prelude::SliceRandom;
use crate::ga::encoding::delay_encoding::DelaysGenotype;
use crate::ga::encoding::priority_encoding::{PriorityGenotype};

pub fn build_delays_population(num_genes: usize, min_delay: u32, max_delay: u32, population_size: usize) -> Population<DelaysGenotype> {
    build_population()
        .with_genome_builder(ValueEncodedGenomeBuilder::new(num_genes, min_delay, max_delay))
        .of_size(population_size)
        .uniform_at_random()
}

pub fn build_priorities_population(num_genes: usize, population_size: usize) -> Population<PriorityGenotype> {
    build_population()
        .with_genome_builder(PermutationEncodedGenomeBuilder::new(num_genes))
        .of_size(population_size)
        .uniform_at_random()
}

struct PermutationEncodedGenomeBuilder {
    genome_length: usize,
}

impl PermutationEncodedGenomeBuilder {
    pub fn new(genome_length: usize) -> Self {
        PermutationEncodedGenomeBuilder {
            genome_length,
        }
    }
}

impl GenomeBuilder<PriorityGenotype> for PermutationEncodedGenomeBuilder {
    fn build_genome<R>(&self, _: usize, rng: &mut R) -> PriorityGenotype where R: Rng + Sized {
        let mut vec: Vec<usize> = (0..self.genome_length).collect();
        vec.shuffle(rng);
        vec
    }
}

#[cfg(test)]
mod population_builder_tests {
    use genevo::population::build_population;
    use itertools::Itertools;
    use crate::ga::population_builder::{PermutationEncodedGenomeBuilder};

    #[test]
    fn test_permutation_encoding_population_builder() {
        let population = build_population()
            .with_genome_builder(PermutationEncodedGenomeBuilder::new(10))
            .of_size(4)
            .uniform_at_random();
        assert_eq!(population.size(), 4);
        // All individuals are different
        assert!(population.individuals().iter().all_unique());
        // All genes are unique, i.e. permutation without repetition
        assert!(population.individuals().iter().map(|individual| individual.iter().all_unique()).all(|unique| unique));
        // All genes are within the sequence (0 ... genotype length-1)
        assert!(population.individuals().iter().map(|individual| individual.iter().all(|x| x >= &0 && x <= &9)).all(|in_range| in_range));
    }
}
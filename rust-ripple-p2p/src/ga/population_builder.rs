use genevo::population::{build_population, Population, ValueEncodedGenomeBuilder};
use crate::ga::encoding::delay_encoding::DelaysGenotype;
use crate::ga::encoding::priority_encoding::{Priority, PriorityGenotype};

pub fn build_delays_population(num_genes: usize, min_delay: u32, max_delay: u32, population_size: usize) -> Population<DelaysGenotype> {
    build_population()
        .with_genome_builder(ValueEncodedGenomeBuilder::new(num_genes, min_delay, max_delay))
        .of_size(population_size)
        .uniform_at_random()
}

pub fn build_priorities_population(num_genes: usize, min_priority: Priority, max_priority: Priority, population_size: usize) -> Population<PriorityGenotype> {
    build_population()
        .with_genome_builder(ValueEncodedGenomeBuilder::new(num_genes, min_priority, max_priority))
        .of_size(population_size)
        .uniform_at_random()
}
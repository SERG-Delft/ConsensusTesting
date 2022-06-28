use std::fmt::Debug;
use std::hash::Hash;
use genevo::genetic::{Genotype, Phenotype};
use genevo::operator::prelude::MultiPointCrossover;
use crate::ga::genetic_algorithm::ConsensusMessageType;
use crate::ga::mutation::GaussianGenomeMutation;
use crate::NUM_NODES;

pub mod priority_encoding;
pub mod delay_encoding;

pub(crate) fn num_genes() -> usize {
    *NUM_NODES * (*NUM_NODES-1) * ConsensusMessageType::VALUES.len()
}

pub trait ExtendedPhenotype<G>: Phenotype<G> + Send where G: ExtendedGenotype {
    fn from_genes(geno: &G) -> Self;

    fn display_genotype_by_message(&self) -> String;
}

pub trait ExtendedGenotype: Genotype + Eq + Hash + Debug + Default {}

pub trait SuperExtendedGenotype: ExtendedGenotype + MultiPointCrossover + GaussianGenomeMutation {}
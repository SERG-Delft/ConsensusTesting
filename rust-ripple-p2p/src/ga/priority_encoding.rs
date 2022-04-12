use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use genevo::genetic::Phenotype;
use genevo::prelude::Rng;
use itertools::{chain, Itertools};
use rand::distributions::uniform::{SampleBorrow, SampleUniform, UniformFloat, UniformSampler};
use rand_distr::{Normal, Distribution};
use crate::ga::genetic_algorithm::{ConsensusMessageType, ExtendedGenotype, ExtendedPhenotype};
use crate::ga::mutation::{GaussianMutation};
use crate::NUM_NODES;

#[derive(Debug, Copy, Clone, Default, PartialOrd)]
pub struct Priority(pub f32);

impl Priority {
    fn key(&self) -> u32 {
        self.0.to_bits()
    }
}

// The phenotype from -> to -> message_type -> delay (ms)
type PriorityMap = HashMap<usize, HashMap<usize, HashMap<ConsensusMessageType, Priority>>>;

impl Hash for Priority {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.key().hash(state);
    }
}

impl PartialEq for Priority {
    fn eq(&self, other: &Self) -> bool {
        self.key() == other.key()
    }
}

impl Eq for Priority{}

impl GaussianMutation for Priority {
    fn random_mutated<R>(value: Self, standard_deviation: f64, rng: &mut R) -> Self where R: Rng + Sized {
        let normal = Normal::new(value.0, standard_deviation as f32).unwrap();
        Self(normal.sample(rng))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct UniformPriority(UniformFloat<f32>);
impl UniformSampler for UniformPriority {
    type X = Priority;

    fn new<B1, B2>(low: B1, high: B2) -> Self
        where
            B1: SampleBorrow<Self::X> + Sized,
            B2: SampleBorrow<Self::X> + Sized,
    {
        UniformPriority(UniformFloat::<f32>::new(low.borrow().0, high.borrow().0))
    }

    fn new_inclusive<B1, B2>(low: B1, high: B2) -> Self
        where
            B1: SampleBorrow<Self::X> + Sized,
            B2: SampleBorrow<Self::X> + Sized,
    {
        UniformSampler::new(low, high)
    }

    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Self::X {
        Priority(self.0.sample(rng))
    }
}

impl SampleUniform for Priority {
    type Sampler = UniformPriority;
}

// The genotype
pub(crate) type PriorityGenotype = Vec<Priority>;

impl ExtendedGenotype for PriorityGenotype{}

/// Contains the delayMap for easy use in the scheduler and delays as genotype (vec)
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PriorityMapPhenotype {
    pub priority_map: PriorityMap,
    priorities: PriorityGenotype
}

impl PriorityMapPhenotype {
    pub fn from(genes: &PriorityGenotype) -> Self {
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
            priority_map: from_node,
            priorities: genes.clone()
        }
    }

    /// Display delays grouped by message and receiver node
    pub fn message_type_priorities(&self, message_type: &ConsensusMessageType) -> Vec<(usize, Vec<Priority>)> {
        self.priority_map.iter()
            .map(|(to, from)| (*to, from.values()
                .map(|x| *x.get(message_type).unwrap())
                .collect_vec()))
            .collect::<Vec<(usize, Vec<Priority>)>>()
    }

    pub fn display_priorities_by_message(&self) {
        for message_type in ConsensusMessageType::VALUES {
            println!("{:?}: {:?}", message_type, self.message_type_priorities(&message_type))
        }
    }
}

impl Phenotype<PriorityGenotype> for PriorityMapPhenotype {
    fn genes(&self) -> PriorityGenotype {
        self.priorities.clone()
    }

    fn derive(&self, new_genes: PriorityGenotype) -> Self {
        PriorityMapPhenotype::from_genes(&new_genes)
    }
}

impl ExtendedPhenotype<PriorityGenotype> for PriorityMapPhenotype {
    fn from_genes(genes: &PriorityGenotype) -> Self {
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
            priority_map: from_node,
            priorities: genes.clone()
        }
    }
}

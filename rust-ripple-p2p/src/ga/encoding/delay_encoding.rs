use std::collections::HashMap;
use genevo::genetic::Phenotype;
use itertools::{chain, Itertools};
use crate::ga::encoding::{ExtendedGenotype, ExtendedPhenotype};
use crate::ga::genetic_algorithm::{ConsensusMessageType};
use crate::NUM_NODES;

pub const DROP_THRESHOLD: u32 = 1800;

// The genotype
pub(crate) type DelaysGenotype = Vec<u32>;

impl ExtendedGenotype for DelaysGenotype{}

// The phenotype from -> to -> message_type -> delay (ms)
type DelayMap = HashMap<usize, HashMap<usize, HashMap<ConsensusMessageType, u32>>>;

/// Contains the delayMap for easy use in the scheduler and delays as genotype (vec)
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DelayMapPhenotype {
    pub delay_map: DelayMap,
    delays: DelaysGenotype
}

impl DelayMapPhenotype {
    /// Display delays grouped by message and receiver node
    #[allow(unused)]
    pub fn message_type_delays(&self, message_type: &ConsensusMessageType) -> Vec<(usize, Vec<u32>)> {
        self.delay_map.iter()
            .map(|(to, from)| (*to, from.values()
                .map(|x| *x.get(message_type).unwrap())
                .collect_vec()))
            .collect::<Vec<(usize, Vec<u32>)>>()
    }
}

impl Phenotype<DelaysGenotype> for DelayMapPhenotype {
    fn genes(&self) -> DelaysGenotype {
        self.delays.clone()
    }

    fn derive(&self, new_genes: DelaysGenotype) -> Self {
        DelayMapPhenotype::from_genes(&new_genes)
    }
}

impl ExtendedPhenotype<DelaysGenotype> for DelayMapPhenotype {
    fn from_genes(genes: &DelaysGenotype) -> Self {
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
            delay_map: from_node,
            delays: genes.clone()
        }
    }

    #[allow(unused)]
    fn display_genotype_by_message(&self) {
        for message_type in ConsensusMessageType::VALUES {
            println!("{:?}: {:?}", message_type, self.message_type_delays(&message_type))
        }
    }
}

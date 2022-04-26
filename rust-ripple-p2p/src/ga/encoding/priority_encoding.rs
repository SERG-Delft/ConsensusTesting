use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use genevo::genetic::Phenotype;
use genevo::prelude::Rng;
use itertools::{chain, Itertools};
use rand::distributions::uniform::{SampleBorrow, SampleUniform, UniformFloat, UniformSampler};
use rand_distr::{Normal, Distribution};
use crate::ga::encoding::{ExtendedGenotype, ExtendedPhenotype};
use crate::ga::genetic_algorithm::{ConsensusMessageType};
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
    /// Display delays grouped by message and receiver node
    #[allow(unused)]
    pub fn message_type_priorities(&self, message_type: &ConsensusMessageType) -> Vec<(usize, Vec<Priority>)> {
        self.priority_map.iter()
            .map(|(to, from)| (*to, from.values()
                .map(|x| *x.get(message_type).unwrap())
                .collect_vec()))
            .collect::<Vec<(usize, Vec<Priority>)>>()
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

    #[allow(unused)]
    fn display_genotype_by_message(&self) -> String {
        let mut res = String::new();
        for message_type in ConsensusMessageType::VALUES {
            res += format!("{:?}: {:?}\n", message_type, self.message_type_priorities(&message_type)).as_str();
        }
        res
    }
}

#[cfg(test)]
mod test_priority_encoding {
    use std::collections::BinaryHeap;
    use crate::ga::encoding::priority_encoding::Priority;
    use crate::message_handler::RippleMessageObject;
    use crate::protos::ripple::TMStatusChange;
    use crate::scheduler::priority_scheduler::OrderedRMOEvent;
    use crate::scheduler::RMOEvent;

    #[test]
    fn test_priority_queue() {
        let mut inbox = BinaryHeap::new();
        let mut rmo_event = RMOEvent::default();
        rmo_event.message = RippleMessageObject::TMStatusChange(TMStatusChange::new());
        let rmo_event_1 = OrderedRMOEvent::new(rmo_event.clone(), Priority(1.01));
        let rmo_event_2 = OrderedRMOEvent::new(RMOEvent::default(), Priority(1f32));
        let rmo_event_3 = OrderedRMOEvent::new(RMOEvent::default(), Priority(1.02));
        inbox.push(rmo_event_1.clone());
        inbox.push(rmo_event_2.clone());
        assert_eq!(&rmo_event_1, inbox.peek().unwrap());
        assert_eq!(rmo_event_1, inbox.pop().unwrap());
        assert_eq!(&rmo_event_2, inbox.peek().unwrap());
        inbox.push(rmo_event_3.clone());
        assert_eq!(&rmo_event_3, inbox.peek().unwrap());
        assert_eq!(rmo_event_3, inbox.pop().unwrap());
        assert_eq!(&rmo_event_2, inbox.peek().unwrap());
        assert_eq!(rmo_event_2, inbox.pop().unwrap());
        assert_eq!(None, inbox.peek());
        assert_eq!(None, inbox.pop());
    }
}

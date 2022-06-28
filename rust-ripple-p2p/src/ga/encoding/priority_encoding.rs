use std::collections::HashMap;
use genevo::genetic::Phenotype;
use itertools::{chain, Itertools};
use crate::ga::encoding::{ExtendedGenotype, ExtendedPhenotype};
use crate::ga::genetic_algorithm::{ConsensusMessageType};
use crate::NUM_NODES;

// The phenotype from -> to -> message_type -> priority
type PriorityMap = HashMap<usize, HashMap<usize, HashMap<ConsensusMessageType, usize>>>;

// The genotype
pub(crate) type PriorityGenotype = Vec<usize>;

impl ExtendedGenotype for PriorityGenotype {}

/// Contains the delayMap for easy use in the scheduler and delays as genotype (vec)
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PriorityMapPhenotype {
    pub priority_map: PriorityMap,
    priorities: PriorityGenotype
}

impl PriorityMapPhenotype {
    /// Display delays grouped by message and receiver node
    #[allow(unused)]
    pub fn message_type_priorities(&self, message_type: &ConsensusMessageType) -> Vec<(usize, Vec<String>)> {
        self.priority_map.iter()
            .sorted_by(|x, y| x.0.cmp(y.0))
            .map(|(to, from)| (*to, from.iter()
                .sorted_by(|x, y| x.0.cmp(y.0))
                .map(|x| format!("{}: {}", x.0, *x.1.get(message_type).unwrap()))
                .collect_vec()))
            .collect::<Vec<(usize, Vec<String>)>>()
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
    use std::collections::binary_heap::BinaryHeap;
    use crate::ga::encoding::{ExtendedPhenotype};
    use crate::ga::encoding::priority_encoding::{PriorityMapPhenotype};
    use crate::message_handler::RippleMessageObject;
    use crate::protos::ripple::TMStatusChange;
    use crate::scheduler::priority_scheduler::OrderedRMOEvent;
    use crate::scheduler::RMOEvent;

    #[test]
    fn test_priority_queue() {
        let mut inbox = BinaryHeap::new();
        let mut rmo_event = RMOEvent::default();
        rmo_event.message = RippleMessageObject::TMStatusChange(TMStatusChange::new());
        let rmo_event_1 = OrderedRMOEvent::new(rmo_event.clone(), 2);
        let rmo_event_2 = OrderedRMOEvent::new(RMOEvent::default(), 1);
        let rmo_event_3 = OrderedRMOEvent::new(RMOEvent::default(), 1);
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

    #[test]
    fn test_arbitrary_hashmap_order() {
        // Priorities that cause the liveness bug
        let string = [193, 177, 94, 204, 12, 83, 63, 38, 58, 65, 55, 158, 138, 85, 223, 30, 201, 73, 188, 210, 171, 89, 250, 149, 230, 24, 183, 166, 106, 39, 22, 245, 213, 11, 71, 76, 1, 102, 42, 200, 164, 51, 16, 113, 187, 110, 40, 127, 21, 150, 6, 17, 62, 135, 26, 105, 236, 255, 189, 175, 178, 96, 37, 191, 254, 129, 157, 118, 61, 100, 139, 93, 229, 124, 134, 152, 221, 173, 155, 66, 60, 253, 19, 251, 180, 101, 114, 181, 2, 56, 36, 34, 195, 172, 192, 70, 228, 163, 131, 212, 28, 185, 219, 226, 227, 211, 198, 167, 220, 144, 143, 233, 232, 86, 50, 122, 128, 136, 117, 115, 130, 35, 78, 8, 238, 159, 3, 74, 207, 215, 186, 165, 156, 120, 77, 247, 97, 145, 182, 246, 153, 88, 179, 95, 194, 82, 141, 140, 133, 258, 104, 46, 123, 44, 160, 84, 75, 32, 52, 216, 33, 148, 43, 90, 20, 80, 205, 49, 146, 244, 47, 197, 241, 10, 121, 18, 99, 59, 235, 67, 249, 243, 87, 256, 196, 69, 91, 7, 111, 202, 147, 108, 14, 116, 169, 64, 41, 81, 203, 161, 162, 9, 248, 237, 239, 252, 142, 217, 151, 92, 119, 176, 48, 137, 132, 154, 27, 23, 199, 184, 45, 206, 242, 103, 54, 259, 126, 240, 231, 112, 222, 125, 209, 107, 79, 218, 13, 72, 31, 0, 57, 225, 109, 168, 68, 29, 53, 15, 234, 25, 98, 224, 5, 4, 257, 190, 170, 214, 208, 174];
        let priority_genotype: Vec<usize> = string.to_vec();
        let priority_map = PriorityMapPhenotype::from_genes(&priority_genotype);
        println!("{}", priority_map.display_genotype_by_message());
    }
}

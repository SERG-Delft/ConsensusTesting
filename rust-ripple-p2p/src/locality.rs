use std::fs::File;
use std::io::{BufWriter, Write, Result};
use std::path::Path;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;
use log::error;
use petgraph::Graph;
use rand::prelude::SliceRandom;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Serialize, Deserialize};
use crate::ga::encoding::delay_encoding::{DelayMapPhenotype, DelaysGenotype};
use crate::ga::encoding::{ExtendedPhenotype, num_genes};
use crate::ga::encoding::priority_encoding::{PriorityGenotype, PriorityMapPhenotype};
use crate::ga::genetic_algorithm::CurrentFitness;
use crate::node_state::{MessageTypeDependencyEvent, MutexNodeStates};
use crate::trace_comparisons::transform_to_message_type_graph;

pub struct PriorityLocalityExperiment {
    priority_result_file: BufWriter<File>,
    scheduler_sender: Sender<PriorityMapPhenotype>,
    scheduler_receiver: Receiver<CurrentFitness>,
    node_states: Arc<MutexNodeStates>
}

impl PriorityLocalityExperiment {
    pub fn new(scheduler_sender: Sender<PriorityMapPhenotype>, scheduler_receiver: Receiver<CurrentFitness>, node_states: Arc<MutexNodeStates>) -> Self {
        let priority_file = File::create(Path::new("priority_locality.txt")).expect("Creating result file failed");
        Self {
            priority_result_file: BufWriter::new(priority_file),
            scheduler_sender,
            scheduler_receiver,
            node_states
        }
    }

    pub fn run_locality_experiment_priorities(&mut self) {
        let mut rng = ChaCha8Rng::seed_from_u64(1);
        // Maximum kendall tau distance is 0.5n(n-1), take fourth of that:
        let min_distance = 0.5 * 260.0 * 259.0 / 4.0;
        println!("Starting priority locality experiment");
        let distant_priority_genotypes: Vec<PriorityGenotype> = sample_n_distant_priority_genotypes(10, num_genes(), min_distance as u64, &mut rng);
        println!("Done creating genotypes");
        let mut neighbors_list: Vec<Vec<PriorityGenotype>> = vec![];
        for genotype in distant_priority_genotypes.iter() {
            let neighbors = sample_n_neighbors_priority_genotypes(10, &genotype, &mut rng);
            if neighbors.iter().map(|neighbor| priority_genotype_kendall_tau_distance(&neighbor, &genotype)).any(|x| x != 1) {
                println!("Wrong neighboring priority genotypes!!!");
            }
            neighbors_list.push(neighbors);
        }
        println!("Done creating neighbors");

        let mut genotype_phenotype_pairs = vec![];
        // Get the phenotypes / schedules / trace graphs for these (10 genotypes + 10 neighbors for each genotype = 110 trace graphs)
        for i in 0..distant_priority_genotypes.len() {
            println!("Starting evaluation for new genotype: {}", i);
            self.execute_priority_schedule(&distant_priority_genotypes[i]);
            let genotype = distant_priority_genotypes[i].clone();
            let phenotype = transform_to_message_type_graph(&self.node_states.get_dependency_graph());
            genotype_phenotype_pairs.push(PriorityGenotypePhenotypePair((genotype, phenotype, EvaluationType::BaseGenotype)));
            for j in 0..neighbors_list[i].len() {
                println!("Starting evaluation for neighbor {} of {}", j, i);
                self.execute_priority_schedule(&neighbors_list[i][j]);
                let genotype = neighbors_list[i][j].clone();
                let phenotype = transform_to_message_type_graph(&self.node_states.get_dependency_graph());
                genotype_phenotype_pairs.push(PriorityGenotypePhenotypePair((genotype, phenotype, EvaluationType::NeighborGenotype)));
            }
        }
        self.write_priority_genotype_phenotype_pairs_to_file(&genotype_phenotype_pairs).unwrap();
        println!("Finished run. exiting...");
        std::process::exit(0);
    }

    fn execute_priority_schedule(&self, priority_genotype: &PriorityGenotype) {
        self.scheduler_sender.send(PriorityMapPhenotype::from_genes(&priority_genotype)).expect("Scheduler receiver failed");
        // If the event cap is exceeded, something went wrong and we need to run again
        match self.scheduler_receiver.recv() {
            Ok(fit) => if fit.value == Duration::default() {
                self.execute_priority_schedule(&priority_genotype);
            }
            Err(_) => {}
        }
    }

    fn write_priority_genotype_phenotype_pairs_to_file(&mut self, genotype_phenotype_pairs: &Vec<PriorityGenotypePhenotypePair>) -> Result<()> {
        let buf = serde_json::to_vec(genotype_phenotype_pairs)?;
        self.priority_result_file.write_all(&buf[..])?;
        Ok(())
    }
}

pub struct DelayLocalityExperiment {
    delay_result_file: BufWriter<File>,
    scheduler_sender: Sender<DelayMapPhenotype>,
    scheduler_receiver: Receiver<CurrentFitness>,
    node_states: Arc<MutexNodeStates>
}

impl DelayLocalityExperiment {
    pub fn new(scheduler_sender: Sender<DelayMapPhenotype>, scheduler_receiver: Receiver<CurrentFitness>, node_states: Arc<MutexNodeStates>) -> Self {
        let delay_file = File::create(Path::new("delay_locality.txt")).expect("Creating result file failed");
        Self {
            delay_result_file: BufWriter::new(delay_file),
            scheduler_sender,
            scheduler_receiver,
            node_states
        }
    }

    pub fn run_locality_experiment_delays(&mut self) {
        let mut rng = ChaCha8Rng::seed_from_u64(2);
        // Maximum euclidean distance is n.sqrt()*(max - min) 260.sqrt() * 4000, take half that:
        let max_distance = 260f64.sqrt() * 4000f64 / 4.0;
        println!("Starting delay locality experiment");
        let distant_delay_genotypes: Vec<DelaysGenotype> = sample_n_distant_delay_genotypes(10, num_genes(), max_distance, &mut rng);
        println!("Done creating genotypes");
        let mut neighbors_list: Vec<Vec<DelaysGenotype>> = vec![];
        for genotype in distant_delay_genotypes.iter() {
            let neighbors = sample_n_neighbors_delay_genotypes(10, &genotype, &mut rng);
            neighbors_list.push(neighbors);
        }
        println!("Done creating neighbors");

        let mut genotype_phenotype_pairs = vec![];
        // Get the phenotypes / schedules / trace graphs for these (10 genotypes + 10 neighbors for each genotype = 110 trace graphs)
        for i in 0..distant_delay_genotypes.len() {
            println!("Starting evaluation for new genotype: {}", i);
            self.execute_delay_schedule(&distant_delay_genotypes[i]);
            let genotype = distant_delay_genotypes[i].clone();
            let phenotype = transform_to_message_type_graph(&self.node_states.get_dependency_graph());
            genotype_phenotype_pairs.push(DelayGenotypePhenotypePair((genotype, phenotype, EvaluationType::BaseGenotype)));
            for j in 0..neighbors_list[i].len() {
                println!("Starting evaluation for neighbor {} of {}", j, i);
                self.execute_delay_schedule(&neighbors_list[i][j]);
                let genotype = neighbors_list[i][j].clone();
                let phenotype = transform_to_message_type_graph(&self.node_states.get_dependency_graph());
                genotype_phenotype_pairs.push(DelayGenotypePhenotypePair((genotype, phenotype, EvaluationType::NeighborGenotype)));
            }
        }
        self.write_delay_genotype_phenotype_pairs_to_file(&genotype_phenotype_pairs).unwrap();
        println!("Finished run. exiting...");
        std::process::exit(0);
    }

    fn execute_delay_schedule(&self, delay_genotype: &DelaysGenotype) {
        self.scheduler_sender.send(DelayMapPhenotype::from_genes(&delay_genotype)).expect("Scheduler receiver failed");
        // If the event cap is exceeded, something went wrong and we need to run again
        match self.scheduler_receiver.recv() {
            Ok(fit) => if fit.value == Duration::default() {
                self.execute_delay_schedule(&delay_genotype);
            }
            Err(_) => {}
        }
    }

    fn write_delay_genotype_phenotype_pairs_to_file(&mut self, genotype_phenotype_pairs: &Vec<DelayGenotypePhenotypePair>) -> Result<()> {
        let buf = serde_json::to_vec(genotype_phenotype_pairs)?;
        self.delay_result_file.write_all(&buf[..])?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct DelayGenotypePhenotypePair((DelaysGenotype, Graph<MessageTypeDependencyEvent, ()>, EvaluationType));

#[derive(Serialize, Deserialize, Debug)]
struct PriorityGenotypePhenotypePair((PriorityGenotype, Graph<MessageTypeDependencyEvent, ()>, EvaluationType));

#[derive(Serialize, Deserialize, Debug, PartialEq)]
enum EvaluationType {
    BaseGenotype,
    NeighborGenotype,
}

pub fn run_locality_experiment_priorities(scheduler_sender: Sender<PriorityMapPhenotype>, scheduler_receiver: Receiver<CurrentFitness>, node_states: Arc<MutexNodeStates>) {
    let mut loc_exp = PriorityLocalityExperiment::new(scheduler_sender, scheduler_receiver, node_states);
    loc_exp.run_locality_experiment_priorities();
}

pub fn run_locality_experiment_delays(scheduler_sender: Sender<DelayMapPhenotype>, scheduler_receiver: Receiver<CurrentFitness>, node_states: Arc<MutexNodeStates>) {
    let mut loc_exp = DelayLocalityExperiment::new(scheduler_sender, scheduler_receiver, node_states);
    loc_exp.run_locality_experiment_delays();
}

/// Sample n closest neighbors of delay genotype (+1 / -1 for one gene)
pub fn sample_n_neighbors_delay_genotypes(n: usize, genotype: &DelaysGenotype, rng: &mut impl Rng) -> Vec<DelaysGenotype> {
    // Technically we can sample 2*num_genes distinct neighbors (up and down)
    if n > genotype.len() {
        error!("Cannot sample more than num_genes distinct neighbors");
        return vec![];
    }
    let mut res = vec![];
    // Create n distinct gene indices where a new neighbor will be created from
    for x in rand::seq::index::sample(rng, genotype.len(), n) {
        let mut new_neighbor = genotype.clone();
        // Either add 1 or subtract 1
        new_neighbor[x] = new_neighbor[x] -1 + (rng.gen_bool(0.5) as u32 * 2);
        res.push(new_neighbor);
    };
    res
}

/// Sample n closest neighbors of priority genotype (two adjacent genes swapped)
pub fn sample_n_neighbors_priority_genotypes(n: usize, genotype: &PriorityGenotype, rng: &mut impl Rng) -> Vec<PriorityGenotype> {
    if n > genotype.len()-1 {
        error!("Cannot sample more than num_genes - 1 distinct neighbors");
        return vec![];
    }
    let mut res = vec![];
    // Create n distinct indices where genes will be swapped to create a new neighbor
    for i in rand::seq::index::sample(rng, genotype.len()-1, n) {
        let mut new_neighbor = genotype.clone();
        // Swap the genes at the i and i+1
        new_neighbor[i] = new_neighbor[i+1];
        new_neighbor[i+1] = genotype[i];
        res.push(new_neighbor);
    };
    res
}

pub fn sample_n_distant_delay_genotypes(n: usize, num_genes: usize, min_distance: f64, rng: &mut impl Rng) -> Vec<DelaysGenotype> {
    let mut res = vec![];
    for _ in 0..n {
        loop {
            let genotype = sample_delays_genotype(num_genes.clone(), 0, 4000, rng);
            if res.is_empty() || res.iter().all(|x| delay_genotype_euclidean_distance(&genotype, x) > min_distance) {
                res.push(genotype);
                break;
            }
        }
    }
    res
}

pub fn sample_n_distant_priority_genotypes(n: usize, num_genes: usize, min_distance: u64, rng: &mut impl Rng) -> Vec<PriorityGenotype> {
    let mut res = vec![];
    for _ in 0..n {
        loop {
            let genotype = sample_priority_genotype(num_genes.clone(), rng);
            if res.is_empty() || res.iter().all(|x| priority_genotype_kendall_tau_distance(&genotype, x) > min_distance) {
                res.push(genotype);
                break;
            }
        }
    }
    res
}
/// 4 * 4^2 .sqrt() = 8
/// 2 * 4 = 8
/// Maximum euclidean distance is n*((max - min)^2).sqrt()
#[allow(unused)]
pub fn delay_genotype_euclidean_distance(genotype_1: &DelaysGenotype, genotype_2: &DelaysGenotype) -> f64 {
    if genotype_1.len() != genotype_2.len() {
        return f64::NAN
    }
    let mut res = 0;
    for (gene_1, gene_2) in genotype_1.iter().zip(genotype_2.iter()) {
        res += gene_2.abs_diff(*gene_1).pow(2);
    }
    (res as f64).sqrt()
}

/// Maximum kendall tau distance is 0.5n(n-1)
#[allow(unused)]
pub fn priority_genotype_kendall_tau_distance(genotype_1: &PriorityGenotype, genotype_2: &PriorityGenotype) -> u64 {
    if genotype_1.len() != genotype_2.len() {
        error!("Genotypes not of equal length");
        return 0;
    }
    let mut index_1 = vec![0; genotype_1.len()];
    let mut index_2 = vec![0; genotype_2.len()];
    for i in 0..genotype_1.len() {
        index_1[genotype_1[i]] = i as i32;
        index_2[genotype_2[i]] = i as i32;
    }
    let mut res = 0;
    for i in 0..genotype_1.len()-1 {
        for j in i+1..genotype_2.len() {
            // let a = genotype_1[i] < genotype_1[j] && genotype_2[i] > genotype_2[j];
            // let b = genotype_1[i] > genotype_1[j] && genotype_2[i] < genotype_2[j];
            // if a || b {
            //     res += 1;
            // }
            let a: i32 = index_1[i] - index_1[j];
            let b: i32 = index_2[i] - index_2[j];
            if a * b < 0 {
                res += 1;
            }
        }
    }
    res
}

#[allow(unused)]
pub fn sample_delays_genotype(n: usize, min: u32, max: u32, rng: &mut impl Rng) -> DelaysGenotype {
    let mut genotype = vec![];
    for i in 0..n {
        let gene = rng.gen_range(min..=max);
        genotype.push(gene)
    }
    genotype
}

#[allow(unused)]
pub fn sample_priority_genotype(n: usize, rng: &mut impl Rng) -> PriorityGenotype {
    let mut genotype: Vec<usize> = (0..n).collect();
    genotype.shuffle(rng);
    genotype
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::BufReader;
    use chrono::Utc;
    use itertools::{Itertools};
    use rand::prelude::SliceRandom;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    use ged::approximate_edit_distance::{approximate_hed_graph_edit_distance, DistanceScoring};
    use crate::locality::{delay_genotype_euclidean_distance, DelayGenotypePhenotypePair, priority_genotype_kendall_tau_distance, PriorityGenotypePhenotypePair, sample_delays_genotype, sample_n_distant_delay_genotypes, sample_n_distant_priority_genotypes, sample_n_neighbors_delay_genotypes, sample_n_neighbors_priority_genotypes, sample_priority_genotype};
    use crate::locality::EvaluationType::{BaseGenotype, NeighborGenotype};

    #[test]
    fn test_euclidean_distance() {
        let genotype_1 = vec![1, 1, 1, 1];
        let genotype_2 = vec![2, 2, 2, 2];
        assert_eq!(delay_genotype_euclidean_distance(&genotype_1, &genotype_2), 2f64);
        let genotype_2 = genotype_1.clone();
        assert_eq!(delay_genotype_euclidean_distance(&genotype_1, &genotype_2), 0f64);
        let genotype_2 = vec![5, 5, 5, 5];
        assert_eq!(delay_genotype_euclidean_distance(&genotype_1, &genotype_2), 8f64);
        let genotype_2 = vec![5, 5, 5, 4];
        assert_eq!(delay_genotype_euclidean_distance(&genotype_1, &genotype_2), 57f64.sqrt());
        let genotype_1 = vec![];
        let genotype_2 = vec![];
        assert_eq!(delay_genotype_euclidean_distance(&genotype_1, &genotype_2), 0f64);
    }

    #[test]
    fn test_kendall_tau_distance() {
        let genotype_1 = vec![2, 0, 1];
        let genotype_2 = vec![1, 0, 2];
        assert_eq!(priority_genotype_kendall_tau_distance(&genotype_1, &genotype_2), 3);
        let genotype_1 = vec![0, 1, 2, 3, 4];
        let genotype_2 = vec![2, 3, 0, 1, 4];
        assert_eq!(priority_genotype_kendall_tau_distance(&genotype_1, &genotype_2), 4);
        let genotype_2 = vec![0, 1, 2, 4, 3];
        assert_eq!(priority_genotype_kendall_tau_distance(&genotype_1, &genotype_2), 1);
        let genotype_2 = vec![0, 1, 3, 2, 4];
        assert_eq!(priority_genotype_kendall_tau_distance(&genotype_1, &genotype_2), 1);
        let genotype_2 = vec![0, 1, 4, 3, 2];
        assert_eq!(priority_genotype_kendall_tau_distance(&genotype_1, &genotype_2), 3);
        let genotype_1 = vec![0, 4, 1, 3, 2];
        assert_eq!(priority_genotype_kendall_tau_distance(&genotype_1, &genotype_2), 1);
    }

    #[test]
    fn test_delays_sample() {
        let mut rng = ChaCha8Rng::seed_from_u64(2);
        let genotype = sample_delays_genotype(10, 0, 1000, &mut rng);
        let expected_genotype = vec![193, 882, 522, 32, 790, 230, 858, 222, 815, 676];
        assert_eq!(genotype, expected_genotype);
    }

    #[test]
    fn test_priorities_sample() {
        let mut rng = ChaCha8Rng::seed_from_u64(2);
        let genotype = sample_priority_genotype(10, &mut rng);
        let expected_genotype = vec![4, 8, 2, 3, 7, 9, 1, 6, 0, 5];
        assert_eq!(genotype, expected_genotype);
    }

    #[test]
    fn test_n_distance_delay_genotypes() {
        let mut rng = ChaCha8Rng::seed_from_u64(2);
        let genotypes = sample_n_distant_delay_genotypes(4, 16, 2000f64, &mut rng);
        for (genotype_1, genotype_2) in genotypes.iter().tuple_combinations() {
            assert!(delay_genotype_euclidean_distance(genotype_1, genotype_2) > 2000f64);
        }
    }

    ///0.5n(n-1): 8 * 15 = 120
    #[test]
    fn test_n_distance_priority_genotypes() {
        let mut rng = ChaCha8Rng::seed_from_u64(2);
        let genotypes = sample_n_distant_priority_genotypes(4, 16, 30, &mut rng);
        for (genotype_1, genotype_2) in genotypes.iter().tuple_combinations() {
            assert!(priority_genotype_kendall_tau_distance(genotype_1, genotype_2) > 30);
        }
    }

    #[test]
    fn test_n_neighbors_delay_genotypes() {
        let mut rng = ChaCha8Rng::seed_from_u64(2);
        let genotype = vec![300, 300, 300, 300];
        let neighbors = sample_n_neighbors_delay_genotypes(4, &genotype, &mut rng);
        assert!(neighbors.iter().all_unique());
        for neighbor in neighbors {
            assert_eq!(delay_genotype_euclidean_distance(&neighbor, &genotype), 1f64);
        }
    }

    #[test]
    fn test_n_neighbors_priority_genotypes() {
        let mut rng = ChaCha8Rng::seed_from_u64(2);
        // let min_distance = 0.5 * 260.0 * 259.0 / 4.0;
        // let genotype = sample_n_distant_priority_genotypes(1, 260, min_distance as u64, &mut rng)[0].clone();
        // dbg!(genotype.clone());
        let mut genotype: Vec<usize> = (0..260).collect_vec();
        genotype.shuffle(&mut rng);
        let neighbors = sample_n_neighbors_priority_genotypes(10, &genotype, &mut rng);
        assert!(neighbors.iter().all_unique());
        assert_eq!(neighbors.iter().map(|neighbor| priority_genotype_kendall_tau_distance(&neighbor, &genotype)).any(|x| x != 1), false);
    }

    fn import_delay_genotype_phenotype_pairs(filename: &str) -> Vec<DelayGenotypePhenotypePair> {
        let file = fs::File::open(filename)
            .expect("Something went wrong opening the file");
        let mut reader = BufReader::new(file);
        serde_json::from_reader(&mut reader).unwrap()
    }

    fn import_priority_genotype_phenotype_pairs(filename: &str) -> Vec<PriorityGenotypePhenotypePair> {
        let file = fs::File::open(filename)
            .expect("Something went wrong opening the file");
        let mut reader = BufReader::new(file);
        serde_json::from_reader(&mut reader).unwrap()
    }

    #[test]
    fn test_import_pair() {
        let delay_file = "delay_locality.txt";
        let pairs = import_delay_genotype_phenotype_pairs(delay_file);
        assert_eq!(pairs.len(), 110);
    }

    #[test]
    fn hed_performance_test() {
        let pairs = import_delay_genotype_phenotype_pairs("delay_locality.txt");
        let time_before = Utc::now();
        approximate_hed_graph_edit_distance(&pairs[0].0.1, &pairs[1].0.1, DistanceScoring::Absolute);
        let duration = Utc::now() - time_before;
        dbg!(duration);
    }

    #[test]
    fn locality_delay_distance_experiment() {
        let delay_file = "delay_locality.txt";
        let pairs = import_delay_genotype_phenotype_pairs(delay_file);
        let mut distances_list = vec![];
        for i in 0..10 {
            let base_genotype = &pairs[i*11];
            let mut distances = vec![];
            if base_genotype.0.2 != BaseGenotype {
                panic!("Code is wrong");
            }
            for j in 1..=10 {
                let neighbor_genotype = &pairs[i*11+j];
                if neighbor_genotype.0.2 != NeighborGenotype {
                    panic!("Code is wrong");
                }
                let genotype_distance = delay_genotype_euclidean_distance(&base_genotype.0.0, &neighbor_genotype.0.0);
                let phenotype_distance = approximate_hed_graph_edit_distance(&base_genotype.0.1, &neighbor_genotype.0.1, DistanceScoring::Normalized);
                distances.push((genotype_distance, phenotype_distance));
            }
            distances_list.push(distances);
        }
        dbg!(distances_list.clone());
        let mut average_distances: Vec<f32> = vec![];
        distances_list.iter().for_each(|x| average_distances.push(x.iter().map(|d| d.1).sum::<f32>() / 10.0));
        dbg!(average_distances);
    }

    #[test]
    fn locality_priority_distance_experiment() {
        let delay_file = "priority_locality.txt";
        let pairs = import_priority_genotype_phenotype_pairs(delay_file);
        let mut distances_list = vec![];
        for i in 0..10 {
            let base_genotype = &pairs[i*11];
            let mut distances = vec![];
            if base_genotype.0.2 != BaseGenotype {
                panic!("Code is wrong");
            }
            for j in 1..=10 {
                let neighbor_genotype = &pairs[i*11+j];
                if neighbor_genotype.0.2 != NeighborGenotype {
                    panic!("Code is wrong");
                }
                let genotype_distance = priority_genotype_kendall_tau_distance(&base_genotype.0.0, &neighbor_genotype.0.0);
                let phenotype_distance = approximate_hed_graph_edit_distance(&base_genotype.0.1, &neighbor_genotype.0.1, DistanceScoring::Normalized);
                distances.push((genotype_distance, phenotype_distance));
            }
            distances_list.push(distances);
        }
        dbg!(distances_list.clone());
        let mut average_distances: Vec<f32> = vec![];
        distances_list.iter().for_each(|x| average_distances.push(x.iter().map(|d| d.1).sum::<f32>() / 10.0));
        dbg!(average_distances);
    }

    #[test]
    fn calc_result() {
        let priority_average_distances = [
            0.7356258,
            0.80348337,
            0.80436885,
            0.80398685,
            0.78607714,
            0.8235253,
            0.79328054,
            0.74935853,
            0.84722316,
            0.84871304,
        ];
        let priority_average_distances_2 = [
            0.79741,
            0.7319101,
            0.8317283,
            0.7769042,
            0.8216816,
            0.82218504,
            0.815512,
            0.81250083,
            0.82108,
            0.81171054,
        ];
        let delay_average_distances = [
            0.8176409,
            0.82219493,
            0.8119997,
            0.79026824,
            0.8077215,
            0.8063756,
            0.81142294,
            0.80532753,
            0.8126539,
            0.81739885,
        ];
        println!("Delay: {}, Priority: {}, Priority_2: {}", delay_average_distances.iter().sum::<f64>() / 10.0, priority_average_distances.iter().sum::<f64>() / 10.0, priority_average_distances_2.iter().sum::<f64>() / 10.0);
    }
}

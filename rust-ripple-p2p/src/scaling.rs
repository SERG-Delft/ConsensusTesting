use std::{io::BufWriter, fs::File, sync::{mpsc::{Sender, Receiver}, Arc}, path::Path, time::Duration};
use std::io::Write;

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use crate::{ga::{encoding::{delay_encoding::{DelayMapPhenotype, DelayGenotype}, ExtendedPhenotype, num_genes}, genetic_algorithm::CurrentFitness}, node_state::MutexNodeStates, locality::sample_delays_genotype};
use crate::ga::encoding::priority_encoding::{PriorityGenotype, PriorityMapPhenotype};
use crate::locality::sample_priority_genotype;
use crate::trace_comparisons::transform_to_message_type_graph;

#[allow(unused)]
pub struct ScalingExperiment {
	scaling_file: BufWriter<File>,
	scheduler_sender: Sender<DelayMapPhenotype>,
	scheduler_receiver: Receiver<CurrentFitness>,
	node_states: Arc<MutexNodeStates>
}

impl ScalingExperiment {
	#[allow(unused)]
	pub fn new(scheduler_sender: Sender<DelayMapPhenotype>, scheduler_receiver: Receiver<CurrentFitness>, node_states: Arc<MutexNodeStates>) -> Self {
		let scaling_file = File::create(Path::new("scaling.txt")).expect("Creating scaling file failed");
		Self {
			scaling_file: BufWriter::new(scaling_file),
			scheduler_sender,
			scheduler_receiver,
			node_states
		}
	}

	#[allow(unused)]
	pub fn create_random_schedules(&mut self) {
		let mut rng = ChaCha8Rng::seed_from_u64(8);
		let mut schedules = vec![];
		let mut genotypes = vec![];
		let num_schedules = 500;
		for _ in 0..num_schedules {
			genotypes.push(sample_delays_genotype(num_genes(), 0, 4000, &mut rng));
		}
		for i in 0..num_schedules {
			self.execute_schedule(&genotypes[i]);
			schedules.push(transform_to_message_type_graph(&self.node_states.get_dependency_graph()));
		}
		let buf = serde_json::to_vec(&schedules).unwrap();
        self.scaling_file.write_all(&buf[..]).unwrap();
	}

	#[allow(unused)]
	fn execute_schedule(&self, genotype: &DelayGenotype) {
        self.scheduler_sender.send(DelayMapPhenotype::from_genes(&genotype)).expect("Scheduler receiver failed");
        // If the event cap is exceeded, something went wrong and we need to run again
        match self.scheduler_receiver.recv() {
            Ok(fit) => if fit.value == Duration::default() {
                self.execute_schedule(&genotype);
            }
            Err(_) => {}
        }
    }
}

#[allow(unused)]
pub struct PriorityScalingExperiment {
	scaling_file: BufWriter<File>,
	scheduler_sender: Sender<PriorityMapPhenotype>,
	scheduler_receiver: Receiver<CurrentFitness>,
	node_states: Arc<MutexNodeStates>
}

impl PriorityScalingExperiment {
	#[allow(unused)]
	pub fn new(scheduler_sender: Sender<PriorityMapPhenotype>, scheduler_receiver: Receiver<CurrentFitness>, node_states: Arc<MutexNodeStates>) -> Self {
		let scaling_file = File::create(Path::new("scaling.txt")).expect("Creating scaling file failed");
		Self {
			scaling_file: BufWriter::new(scaling_file),
			scheduler_sender,
			scheduler_receiver,
			node_states
		}
	}

	#[allow(unused)]
	pub fn create_random_schedules(&mut self) {
		let mut rng = ChaCha8Rng::seed_from_u64(8);
		let mut schedules = vec![];
		let mut genotypes = vec![];
		let num_schedules = 500;
		for _ in 0..num_schedules {
			genotypes.push(sample_priority_genotype(num_genes(), &mut rng));
		}
		for i in 0..num_schedules {
			self.execute_schedule(&genotypes[i]);
			schedules.push(transform_to_message_type_graph(&self.node_states.get_dependency_graph()));
		}
		let buf = serde_json::to_vec(&schedules).unwrap();
		self.scaling_file.write_all(&buf[..]).unwrap();
	}

	#[allow(unused)]
	fn execute_schedule(&self, genotype: &PriorityGenotype) {
		self.scheduler_sender.send(PriorityMapPhenotype::from_genes(&genotype)).expect("Scheduler receiver failed");
		// If the event cap is exceeded, something went wrong and we need to run again
		match self.scheduler_receiver.recv() {
			Ok(fit) => if fit.value == Duration::default() {
				self.execute_schedule(&genotype);
			}
			Err(_) => {}
		}
	}
}

#[allow(unused)]
pub fn run_scaling_experiment(scheduler_sender: Sender<DelayMapPhenotype>, scheduler_receiver: Receiver<CurrentFitness>, node_states: Arc<MutexNodeStates>) {
    let mut exp = ScalingExperiment::new(scheduler_sender, scheduler_receiver, node_states);
    exp.create_random_schedules();
}

#[allow(unused)]
pub fn run_priority_scaling_experiment(scheduler_sender: Sender<PriorityMapPhenotype>, scheduler_receiver: Receiver<CurrentFitness>, node_states: Arc<MutexNodeStates>) {
	let mut exp = PriorityScalingExperiment::new(scheduler_sender, scheduler_receiver, node_states);
	exp.create_random_schedules();
}

#[cfg(test)]
mod tests {
	use std::collections::HashMap;
	use std::fs;
	use std::io::BufReader;
	use itertools::Itertools;
	use petgraph::Graph;
	use crate::node_state::MessageTypeDependencyEvent;

	fn import_schedules() -> Vec<Graph<MessageTypeDependencyEvent, ()>> {
		let file = fs::File::open("scaling.txt")
			.expect("Something went wrong opening the file");
		let mut reader = BufReader::new(file);
		serde_json::from_reader(&mut reader).unwrap()
	}

	fn mean(data: &[f64]) -> Option<f64> {
		let sum = data.iter().sum::<f64>();
		let count = data.len();

		match count {
			positive if positive > 0 => Some(sum / count as f64),
			_ => None,
		}
	}

	fn std_deviation(data: &[f64]) -> Option<f64> {
		match (mean(data), data.len()) {
			(Some(data_mean), count) if count > 0 => {
				let variance = data.iter().map(|value| {
					let diff = data_mean - (*value as f64);

					diff * diff
				}).sum::<f64>() / count as f64;

				Some(variance.sqrt())
			},
			_ => None
		}
	}

	#[test]
	fn scaling_experiment() {
		let schedules = import_schedules();
		assert_eq!(schedules.len(), 500);
		let mut frequencies_map: HashMap<String, Vec<f64>> = HashMap::new();
		for schedule in schedules {
			let size = schedule.node_count() as f64;
			schedule.node_weights()
				.counts_by(|node_weight| node_weight.message_type.clone())
				.iter()
				.for_each(|(message_type, count)| match frequencies_map.get_mut(message_type) {
					Some(list) => list.push(*count as f64 / size),
					None => {
						frequencies_map.insert(message_type.clone(), vec![*count as f64 / size]);
					}
				});
		}
		for (message_type, frequency_list) in frequencies_map.into_iter() {
			println!("{} -> mean: {}, std: {}", message_type, mean(frequency_list.as_slice()).unwrap(), std_deviation(frequency_list.as_slice()).unwrap());
		}
	}
}
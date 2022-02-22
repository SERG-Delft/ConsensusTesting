use std::fmt::{Display, Formatter};
use std::sync::Arc;
use genevo::genetic::{AsScalar, Fitness};
use crate::client::{Client, ServerStateObject, State, StateAccounting};
use crate::ga::fitness::ExtendedFitness;
use crate::ga::genetic_algorithm::Parameter;
use crate::node_state::MutexNodeStates;
use crate::test_harness::TestHarness;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct StateAccountFitness {
    pub not_full_duration: u32,
    pub not_full_transitions: u32,
}

impl StateAccountFitness {
    pub fn new(not_full_duration: u32, not_full_transitions: u32) -> Self {
        Self { not_full_duration, not_full_transitions }
    }

    pub fn update_server_states(node_states: Arc<MutexNodeStates>, test_harness: &TestHarness) -> Vec<ServerStateObject> {
        (0..node_states.number_of_nodes).into_iter().for_each(|i| Client::server_state(&test_harness.client_senders[i]));
        println!("Waiting on server states");
        {
            node_states.server_state_cvar.wait(&mut node_states.node_states.lock());
        }
        println!("Getting server states");
        (0..node_states.number_of_nodes).into_iter().map(|i| node_states.get_server_state(i)).collect::<Vec<ServerStateObject>>()
    }

    pub fn calculate_fitness(before_server_states: Vec<ServerStateObject>, after_server_states: Vec<ServerStateObject>) -> Self {
        let disconnected = Self::collect_durations_and_transitions(State::Disconnected, &before_server_states, &after_server_states);
        let connected = Self::collect_durations_and_transitions(State::Connected, &before_server_states, &after_server_states);
        let syncing = Self::collect_durations_and_transitions(State::Syncing, &before_server_states, &after_server_states);
        let tracking = Self::collect_durations_and_transitions(State::Tracking, &before_server_states, &after_server_states);
        Self { not_full_duration: disconnected.0 + connected.0 + syncing.0 + tracking.0, not_full_transitions: disconnected.1 + connected.1 + syncing.1 + tracking.1 }
    }

    fn collect_durations_and_transitions(state: State, before_server_states: &Vec<ServerStateObject>, after_server_states: &Vec<ServerStateObject>) -> (u32, u32) {
        let durations_transitions_list = before_server_states.iter().map(|x| &x.state_accounting)
            .zip(after_server_states.iter().map(|x| &x.state_accounting))
            .map(|x| StateAccounting::diff(&state, x.0, x.1))
            .collect::<Vec<(u32, u32)>>();
        (durations_transitions_list.iter().map(|x| x.0).sum(), durations_transitions_list.iter().map(|x| x.1).sum())
    }
}

impl Fitness for StateAccountFitness {
    fn zero() -> Self {
        Self { not_full_duration: 0, not_full_transitions: 0 }
    }

    #[allow(unstable_name_collisions)]
    fn abs_diff(&self, other: &Self) -> Self {
        Self {
            not_full_duration: u32::abs_diff(&self.not_full_duration, &other.not_full_duration),
            not_full_transitions: u32::abs_diff(&self.not_full_transitions, &other.not_full_transitions),
        }
    }
}

impl Display for StateAccountFitness {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "StateAccountFitness: duration {}, transitions: {}\n", self.not_full_duration, self.not_full_transitions)
    }
}

impl ExtendedFitness for StateAccountFitness {
    fn average(a: &[Self]) -> Self {
        Self {
            not_full_duration: a.iter().map(|x| x.not_full_duration).sum::<u32>() / a.len() as u32,
            not_full_transitions: a.iter().map(|x| x.not_full_transitions).sum::<u32>() / a.len() as u32,
        }
    }

    fn highest_possible_fitness() -> Self {
        Self { not_full_duration: 100000000, not_full_transitions: 50 }
    }

    fn lowest_possible_fitness() -> Self {
        Self { not_full_duration: 0, not_full_transitions: 0 }
    }

    fn run_harness(test_harness: TestHarness<'static>, node_states: Arc<MutexNodeStates>) -> Self {
        let before_server_states = Self::update_server_states(node_states.clone(), &test_harness);
        test_harness.schedule_transactions(node_states.clone());
        let after_server_states = Self::update_server_states(node_states, &test_harness);
        Self::calculate_fitness(before_server_states, after_server_states)
    }
}

impl AsScalar for StateAccountFitness {
    fn as_scalar(&self) -> f64 {
        self.not_full_duration as f64
    }
}
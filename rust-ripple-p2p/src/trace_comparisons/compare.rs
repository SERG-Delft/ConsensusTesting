#[cfg(test)]
mod graph_comparisons {
    use std::collections::HashMap;
    use std::str;
    use std::fs;
    use std::fs::File;
    use std::io::{BufRead, BufReader, Write};
    use std::ops::Range;
    use std::path::Path;
    use chrono::{Duration, MAX_DATETIME};
    use itertools::Itertools;
    use petgraph::dot::{Config, Dot};
    use petgraph::Graph;
    use rand_distr::num_traits::Pow;
    use ged::approximate_edit_distance::DistanceScoring;
    use crate::collector::RippleMessage;
    use crate::message_handler::RippleMessageObject;
    use crate::node_state::{DependencyEvent};
    use crate::protos::ripple::{TMProposeSet, TMTransaction};

    #[test]
    fn trace_comparison_test1() {
        let mut graph1: Graph<DependencyEvent, ()> = Graph::new();
        let event_1 = DependencyEvent{ ripple_message: *RippleMessage::new("0".to_string(), "1".to_string(), Duration::zero(), MAX_DATETIME, RippleMessageObject::TMTransaction(TMTransaction::default())) };
        let event_2 = DependencyEvent{ ripple_message: *RippleMessage::new("0".to_string(), "2".to_string(), Duration::zero(), MAX_DATETIME, RippleMessageObject::TMProposeSet(TMProposeSet::default())) };
        let event_3 = DependencyEvent{ ripple_message: *RippleMessage::new("2".to_string(), "1".to_string(), Duration::zero(), MAX_DATETIME, RippleMessageObject::TMTransaction(TMTransaction::default())) };
        let event_4 = DependencyEvent{ ripple_message: *RippleMessage::new("3".to_string(), "1".to_string(), Duration::zero(), MAX_DATETIME, RippleMessageObject::TMProposeSet(TMProposeSet::default())) };
        let message1 = graph1.add_node(event_1.clone());
        let message2 = graph1.add_node(event_2.clone());
        let message3 = graph1.add_node(event_3.clone());
        let message4 = graph1.add_node(event_4.clone());
        graph1.extend_with_edges(&[(message1, message3), (message2, message4)]);
        let mut graph2: Graph<DependencyEvent, ()> = Graph::new();
        let message1 = graph2.add_node(event_1);
        let message2 = graph2.add_node(event_2);
        let message3 = graph2.add_node(event_3);
        let message4 = graph2.add_node(event_4);
        graph2.extend_with_edges(&[(message1, message2), (message3, message4)]);
        let actual_similarity = ged::approximate_edit_distance::approximate_hed_graph_edit_distance(&graph1, &graph2, DistanceScoring::Normalized);
        let expected_similarity = 1.0 - (2.0 / 8.0);
        assert_eq!(actual_similarity, expected_similarity);
    }

    #[test]
    fn trace_comparison_test2() {
        let mut graph1: Graph<DependencyEvent, ()> = Graph::new();
        let event_1 = DependencyEvent{ ripple_message: *RippleMessage::new("0".to_string(), "1".to_string(), Duration::zero(), MAX_DATETIME, RippleMessageObject::TMTransaction(TMTransaction::default())) };
        let _ = graph1.add_node(event_1.clone());
        let mut graph2: Graph<DependencyEvent, ()> = Graph::new();
        let _ = graph2.add_node(event_1);
        let actual_similarity = ged::approximate_edit_distance::approximate_hed_graph_edit_distance(&graph1, &graph2, DistanceScoring::Normalized);
        let expected_similarity = 1.0;
        assert_eq!(actual_similarity, expected_similarity);
    }

    #[test]
    fn trace_comparison_test3() {
        let mut graph1: Graph<DependencyEvent, ()> = Graph::new();
        let event_1 = DependencyEvent{ ripple_message: *RippleMessage::new("0".to_string(), "1".to_string(), Duration::zero(), MAX_DATETIME, RippleMessageObject::TMTransaction(TMTransaction::default())) };
        let event_2 = DependencyEvent{ ripple_message: *RippleMessage::new("0".to_string(), "2".to_string(), Duration::zero(), MAX_DATETIME, RippleMessageObject::TMProposeSet(TMProposeSet::default())) };
        let message1 = graph1.add_node(event_1.clone());
        let message2 = graph1.add_node(event_2.clone());
        graph1.extend_with_edges(&[(message1, message2)]);
        let mut graph2: Graph<DependencyEvent, ()> = Graph::new();
        let message1 = graph2.add_node(event_1);
        let message2 = graph2.add_node(event_2);
        graph2.extend_with_edges(&[(message1, message2)]);
        let actual_similarity = ged::approximate_edit_distance::approximate_hed_graph_edit_distance(&graph1, &graph2, DistanceScoring::Normalized);
        let expected_similarity = 1.0;
        assert_eq!(actual_similarity, expected_similarity);
    }

    #[test]
    fn trace_comparison_test4() {
        let mut graph1: Graph<DependencyEvent, ()> = Graph::new();
        let event_1 = DependencyEvent{ ripple_message: *RippleMessage::new("0".to_string(), "1".to_string(), Duration::zero(), MAX_DATETIME, RippleMessageObject::TMTransaction(TMTransaction::default())) };
        let event_2 = DependencyEvent{ ripple_message: *RippleMessage::new("0".to_string(), "2".to_string(), Duration::zero(), MAX_DATETIME, RippleMessageObject::TMProposeSet(TMProposeSet::default())) };
        let message1 = graph1.add_node(event_1.clone());
        let message2 = graph1.add_node(event_2.clone());
        graph1.extend_with_edges(&[(message1, message2)]);
        let mut graph2: Graph<DependencyEvent, ()> = Graph::new();
        let message1 = graph2.add_node(event_1);
        let message2 = graph2.add_node(event_2);
        graph2.extend_with_edges(&[(message2, message1)]);
        let actual_similarity = ged::approximate_edit_distance::approximate_hed_graph_edit_distance(&graph1, &graph2, DistanceScoring::Normalized);
        let expected_similarity = 1.0 - (1.0 / 4.0);
        assert_eq!(actual_similarity, expected_similarity);
    }

    #[test]
    fn get_dot_file() {
        let graphs = import_graphs("trace_graphs.txt", 1);
        let mut file = File::create(Path::new("run.txt")).expect("Opening dot file failed");
        file.write(format!("{:?}", Dot::with_config(&graphs[0].1, &[Config::EdgeNoLabel])).as_bytes()).unwrap();
    }

    #[test]
    #[ignore]
    fn delay_trace_comparison() {
        let graphs = import_graphs("trace_graphs.txt", 20);
        let number_of_different_delays = 4;
        let number_of_runs_per_delay = 5;
        println!("{}", number_of_different_delays);
        let graph_pairs = (0..graphs.len()).into_iter().combinations(2);
        let mut distances = HashMap::new();
        for graph_pair in graph_pairs.into_iter() {
            let graph1 = graphs[graph_pair[0] as usize].clone();
            let graph2 = graphs[graph_pair[1] as usize].clone();
            let distance = ged::approximate_edit_distance::approximate_hed_graph_edit_distance(&graph1.1, &graph2.1, DistanceScoring::Normalized);
            distances.insert(graph_pair, distance);
        }
        println!("{:?}", distances);

        let zero_traces = 0..number_of_runs_per_delay;
        let one_traces = 1*number_of_runs_per_delay..2*number_of_runs_per_delay;
        let rand_1_traces = 2*number_of_runs_per_delay..3*number_of_runs_per_delay;
        let rand_2_traces = 3*number_of_runs_per_delay..4*number_of_runs_per_delay;

        println!("{:?}, {:?}, {:?}, {:?}, {:?}", number_of_runs_per_delay, zero_traces, one_traces, rand_1_traces, rand_2_traces);

        let zero_to_zero_sims = get_similar_similarities(&distances, &zero_traces);
        let one_to_one_sims = get_similar_similarities(&distances, &one_traces);
        let rand1_to_rand1_sims = get_similar_similarities(&distances, &rand_1_traces);
        let rand2_to_rand2_sims = get_similar_similarities(&distances, &rand_2_traces);
        let zero_to_one_sims = get_different_similarities(&distances, &zero_traces, &one_traces);
        let zero_to_rand1_sims = get_different_similarities(&distances, &zero_traces, &rand_1_traces);
        let zero_to_rand2_sims = get_different_similarities(&distances, &zero_traces, &rand_2_traces);
        let one_to_rand1_sims = get_different_similarities(&distances, &one_traces, &rand_1_traces);
        let one_to_rand2_sims = get_different_similarities(&distances, &one_traces, &rand_2_traces);
        let rand1_to_rand2_sims = get_different_similarities(&distances, &rand_1_traces, &rand_2_traces);

        let mut distances_list = distances.iter().map(|x| ((x.0[0], x.0[1]), *x.1)).collect::<Vec<((usize, usize), f32)>>();
        distances_list.sort_unstable_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        for distance in distances_list.iter() {
            println!("{:?}", distance);
        }
        println!("0 delay -> 0 delay = {} +- {}", zero_to_zero_sims.iter().map(|x| x.1).sum::<f32>() / zero_to_zero_sims.len() as f32, calculate_std(&zero_to_zero_sims));
        println!("0 delay -> 1 delay = {} +- {}", zero_to_one_sims.iter().map(|x| x.1).sum::<f32>() / zero_to_one_sims.len() as f32, calculate_std(&zero_to_one_sims));
        println!("0 delay -> rand1 delay = {} +- {}", zero_to_rand1_sims.iter().map(|x| x.1).sum::<f32>() / zero_to_rand1_sims.len() as f32, calculate_std(&zero_to_rand1_sims));
        println!("0 delay -> rand2 delay = {} +- {}", zero_to_rand2_sims.iter().map(|x| x.1).sum::<f32>() / zero_to_rand2_sims.len() as f32, calculate_std(&zero_to_rand2_sims));
        println!("1 delay -> 1 delay = {} +- {}", one_to_one_sims.iter().map(|x| x.1).sum::<f32>() / one_to_one_sims.len() as f32, calculate_std(&one_to_one_sims));
        println!("1 delay -> rand1 delay = {} +- {}", one_to_rand1_sims.iter().map(|x| x.1).sum::<f32>() / one_to_rand1_sims.len() as f32, calculate_std(&one_to_rand1_sims));
        println!("1 delay -> rand2 delay = {} +- {}", one_to_rand2_sims.iter().map(|x| x.1).sum::<f32>() / one_to_rand2_sims.len() as f32, calculate_std(&one_to_rand2_sims));
        println!("rand1 delay -> rand1 delay = {} +- {}", rand1_to_rand1_sims.iter().map(|x| x.1).sum::<f32>() / rand1_to_rand1_sims.len() as f32, calculate_std(&rand1_to_rand1_sims));
        println!("rand1 delay -> rand2 delay = {} +- {}", rand1_to_rand2_sims.iter().map(|x| x.1).sum::<f32>() / rand1_to_rand2_sims.len() as f32, calculate_std(&rand1_to_rand2_sims));
        println!("rand2 delay -> rand2 delay = {} +- {}", rand2_to_rand2_sims.iter().map(|x| x.1).sum::<f32>() / rand2_to_rand2_sims.len() as f32, calculate_std(&rand2_to_rand2_sims));
    }

    #[test]
    #[ignore]
    fn priority_trace_comparison() {
        let graphs = import_graphs("priority_trace_graphs.txt", 15);
        let number_of_different_delays = 3;
        let number_of_runs_per_delay = 5;
        println!("{}", number_of_different_delays);
        let graph_pairs = (0..graphs.len()).into_iter().combinations(2);
        let mut distances = HashMap::new();
        for graph_pair in graph_pairs.into_iter() {
            let graph1 = graphs[graph_pair[0] as usize].clone();
            let graph2 = graphs[graph_pair[1] as usize].clone();
            let distance = ged::approximate_edit_distance::approximate_hed_graph_edit_distance(&graph1.1, &graph2.1, DistanceScoring::Normalized);
            distances.insert(graph_pair, distance);
        }
        println!("{:?}", distances);

        let zero_traces = 0..number_of_runs_per_delay;
        let rand_1_traces = 1*number_of_runs_per_delay..2*number_of_runs_per_delay;
        let rand_2_traces = 2*number_of_runs_per_delay..3*number_of_runs_per_delay;

        println!("{:?}, {:?}, {:?}, {:?}", number_of_runs_per_delay, zero_traces, rand_1_traces, rand_2_traces);

        let zero_to_zero_sims = get_similar_similarities(&distances, &zero_traces);
        let rand1_to_rand1_sims = get_similar_similarities(&distances, &rand_1_traces);
        let rand2_to_rand2_sims = get_similar_similarities(&distances, &rand_2_traces);
        let zero_to_rand1_sims = get_different_similarities(&distances, &zero_traces, &rand_1_traces);
        let zero_to_rand2_sims = get_different_similarities(&distances, &zero_traces, &rand_2_traces);
        let rand1_to_rand2_sims = get_different_similarities(&distances, &rand_1_traces, &rand_2_traces);

        let mut distances_list = distances.iter().map(|x| ((x.0[0], x.0[1]), *x.1)).collect::<Vec<((usize, usize), f32)>>();
        distances_list.sort_unstable_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        for distance in distances_list.iter() {
            println!("{:?}", distance);
        }
        println!("0 priority -> 0 priority = {} +- {}", zero_to_zero_sims.iter().map(|x| x.1).sum::<f32>() / zero_to_zero_sims.len() as f32, calculate_std(&zero_to_zero_sims));
        println!("0 priority -> rand1 priority = {} +- {}", zero_to_rand1_sims.iter().map(|x| x.1).sum::<f32>() / zero_to_rand1_sims.len() as f32, calculate_std(&zero_to_rand1_sims));
        println!("0 priority -> rand2 priority = {} +- {}", zero_to_rand2_sims.iter().map(|x| x.1).sum::<f32>() / zero_to_rand2_sims.len() as f32, calculate_std(&zero_to_rand2_sims));
        println!("rand1 priority -> rand1 priority = {} +- {}", rand1_to_rand1_sims.iter().map(|x| x.1).sum::<f32>() / rand1_to_rand1_sims.len() as f32, calculate_std(&rand1_to_rand1_sims));
        println!("rand1 priority -> rand2 priority = {} +- {}", rand1_to_rand2_sims.iter().map(|x| x.1).sum::<f32>() / rand1_to_rand2_sims.len() as f32, calculate_std(&rand1_to_rand2_sims));
        println!("rand2 priority -> rand2 priority = {} +- {}", rand2_to_rand2_sims.iter().map(|x| x.1).sum::<f32>() / rand2_to_rand2_sims.len() as f32, calculate_std(&rand2_to_rand2_sims));
    }

    fn import_graphs(filename: &str, num_graphs: usize) -> Vec<(String, Graph<DependencyEvent, ()>)> {
        let file = fs::File::open(filename)
            .expect("Something went wrong opening the file");
        let mut reader = BufReader::new(file);
        let mut graphs = vec![];
        for i in 0..num_graphs {
            let mut delay_buf = vec![];
            reader.read_until(b'+', &mut delay_buf).expect("Reading until delimiter failed");
            let delay_string = match str::from_utf8(&delay_buf) {
                Ok(v) => v,
                Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
            };
            let mut graph_buf = vec![];
            reader.read_until(b'+', &mut graph_buf).expect("Reading until delimiter failed");
            println!("{}", i);
            // println!("{}", str::from_utf8(&graph_buf[1..graph_buf.len()-1]).unwrap());
            let graph_string = match str::from_utf8(&graph_buf[1..graph_buf.len()-1]) {
                Ok(v) => v,
                Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
            };
            let graph: Graph<DependencyEvent, ()> = serde_json::from_str(&graph_string).unwrap();
            graphs.push((delay_string.to_string(), graph));
        }
        graphs
    }

    fn get_similar_similarities(distances: &HashMap<Vec<usize>, f32>, trace_indices: &Range<usize>) -> Vec<(Vec<usize>, f32)> {
        distances.clone().into_iter()
            .filter(|x| trace_indices.contains(&x.0[0]) && trace_indices.contains(&x.0[1]))
            .collect::<Vec<(Vec<usize>, f32)>>()
    }

    fn get_different_similarities(distances: &HashMap<Vec<usize>, f32>, trace_indices1: &Range<usize>, trace_indices2: &Range<usize>) -> Vec<(Vec<usize>, f32)> {
        distances.clone().into_iter()
            .filter(|x| (trace_indices1.contains(&x.0[0]) && trace_indices2.contains(&x.0[1]))
                || (trace_indices2.contains(&x.0[0]) && trace_indices1.contains(&x.0[1])))
            .collect::<Vec<(Vec<usize>, f32)>>()
    }

    fn calculate_std(list: &Vec<(Vec<usize>, f32)>) -> f32 {
        let mean = list.iter().map(|x| x.1).sum::<f32>() / list.len() as f32;
        let mut variance: f32 = 0.0;
        for i in 0..list.len() {
            variance += (list[i].1 - mean).pow(2);
        }
        (variance / list.len() as f32).sqrt()
    }

    #[test]
    fn calculate_std_test() {
        let list = vec![(vec![], -2f32), (vec![], 2f32), (vec![], 2f32), (vec![], -2f32)];
        assert_eq!(4f32, calculate_std(&list) * calculate_std(&list));
    }
}
#[cfg(test)]
mod graph_comparisons {
    use std::collections::HashMap;
    use std::str;
    use std::fs;
    use std::io::{BufRead, BufReader};
    use chrono::MAX_DATETIME;
    use itertools::Itertools;
    use petgraph::Graph;
    use crate::collector::RippleMessage;
    use crate::message_handler::RippleMessageObject;
    use crate::node_state::{DependencyEvent};
    use crate::protos::ripple::{TMProposeSet, TMTransaction};

    #[test]
    fn trace_comparison_test1() {
        let mut graph1: Graph<DependencyEvent, ()> = Graph::new();
        let event_1 = DependencyEvent{ ripple_message: *RippleMessage::new("0".to_string(), "1".to_string(), MAX_DATETIME, RippleMessageObject::TMTransaction(TMTransaction::default())) };
        let event_2 = DependencyEvent{ ripple_message: *RippleMessage::new("0".to_string(), "2".to_string(), MAX_DATETIME, RippleMessageObject::TMProposeSet(TMProposeSet::default())) };
        let event_3 = DependencyEvent{ ripple_message: *RippleMessage::new("2".to_string(), "1".to_string(), MAX_DATETIME, RippleMessageObject::TMTransaction(TMTransaction::default())) };
        let event_4 = DependencyEvent{ ripple_message: *RippleMessage::new("3".to_string(), "1".to_string(), MAX_DATETIME, RippleMessageObject::TMProposeSet(TMProposeSet::default())) };
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
        let actual_similarity = ged::approximate_edit_distance::approximate_hed_graph_edit_distance(graph1, graph2);
        let expected_similarity = 1.0 - (2.0 / 12.0);
        assert_eq!(actual_similarity, expected_similarity);
    }

    #[test]
    fn trace_comparison_test2() {
        let mut graph1: Graph<DependencyEvent, ()> = Graph::new();
        let event_1 = DependencyEvent{ ripple_message: *RippleMessage::new("0".to_string(), "1".to_string(), MAX_DATETIME, RippleMessageObject::TMTransaction(TMTransaction::default())) };
        let message1 = graph1.add_node(event_1.clone());
        let mut graph2: Graph<DependencyEvent, ()> = Graph::new();
        let message1 = graph2.add_node(event_1);
        let actual_similarity = ged::approximate_edit_distance::approximate_hed_graph_edit_distance(graph1, graph2);
        let expected_similarity = 1.0;
        assert_eq!(actual_similarity, expected_similarity);
    }

    #[test]
    fn trace_comparison_test3() {
        let mut graph1: Graph<DependencyEvent, ()> = Graph::new();
        let event_1 = DependencyEvent{ ripple_message: *RippleMessage::new("0".to_string(), "1".to_string(), MAX_DATETIME, RippleMessageObject::TMTransaction(TMTransaction::default())) };
        let event_2 = DependencyEvent{ ripple_message: *RippleMessage::new("0".to_string(), "2".to_string(), MAX_DATETIME, RippleMessageObject::TMProposeSet(TMProposeSet::default())) };
        let message1 = graph1.add_node(event_1.clone());
        let message2 = graph1.add_node(event_2.clone());
        graph1.extend_with_edges(&[(message1, message2)]);
        let mut graph2: Graph<DependencyEvent, ()> = Graph::new();
        let message1 = graph2.add_node(event_1);
        let message2 = graph2.add_node(event_2);
        graph2.extend_with_edges(&[(message1, message2)]);
        let actual_similarity = ged::approximate_edit_distance::approximate_hed_graph_edit_distance(graph1, graph2);
        let expected_similarity = 1.0;
        assert_eq!(actual_similarity, expected_similarity);
    }

    #[test]
    fn trace_comparison_test4() {
        let mut graph1: Graph<DependencyEvent, ()> = Graph::new();
        let event_1 = DependencyEvent{ ripple_message: *RippleMessage::new("0".to_string(), "1".to_string(), MAX_DATETIME, RippleMessageObject::TMTransaction(TMTransaction::default())) };
        let event_2 = DependencyEvent{ ripple_message: *RippleMessage::new("0".to_string(), "2".to_string(), MAX_DATETIME, RippleMessageObject::TMProposeSet(TMProposeSet::default())) };
        let message1 = graph1.add_node(event_1.clone());
        let message2 = graph1.add_node(event_2.clone());
        graph1.extend_with_edges(&[(message1, message2)]);
        let mut graph2: Graph<DependencyEvent, ()> = Graph::new();
        let message1 = graph2.add_node(event_1);
        let message2 = graph2.add_node(event_2);
        graph2.extend_with_edges(&[(message2, message1)]);
        let actual_similarity = ged::approximate_edit_distance::approximate_hed_graph_edit_distance(graph1, graph2);
        let expected_similarity = 1.0 - (1.0 / 6.0);
        assert_eq!(actual_similarity, expected_similarity);
    }

    #[test]
    fn trace_comparison() {
        let graphs = import_graphs("trace_graphs.txt");
        let graph_pairs = (0..6).into_iter().combinations(2);
        let mut distances = HashMap::new();
        for graph_pair in graph_pairs.into_iter() {
            let graph1 = graphs[graph_pair[0] as usize].clone();
            let graph2 = graphs[graph_pair[1] as usize].clone();
            let distance = ged::approximate_edit_distance::approximate_hed_graph_edit_distance(graph1.1, graph2.1);
            distances.insert(graph_pair, distance);
        }
        println!("{:?}", distances);
        let mut distances_list = distances.iter().map(|x| ((x.0[0], x.0[1]), *x.1)).collect::<Vec<((i32, i32), f32)>>();
        distances_list.sort_unstable_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        for distance in distances_list.iter() {
            println!("{:?}", distance);
        }
        let zero_to_zero_sim = distances_list[0].1;
        let zero_to_one_sim = (distances_list[1].1 + distances_list[2].1 + distances_list[5].1 + distances_list[6].1) / 4.0;
        let zero_to_rand_sim = (distances_list[3].1 + distances_list[4].1 + distances_list[7].1 + distances_list[8].1) / 4.0;
        let one_to_one_sim = distances_list[9].1;
        let one_to_rand_sim = (distances_list[10].1 + distances_list[11].1 + distances_list[12].1 + distances_list[13].1) / 4.0;
        let rand_to_rand_sim = distances_list[14].1;
        println!("0 delay -> 0 delay = {}", zero_to_zero_sim);
        println!("0 delay -> 1 delay = {}", zero_to_one_sim);
        println!("0 delay -> rand delay = {}", zero_to_rand_sim);
        println!("1 delay -> 1 delay = {}", one_to_one_sim);
        println!("1 delay -> rand delay = {}", one_to_rand_sim);
        println!("rand delay -> rand delay = {}", rand_to_rand_sim);

    }

    fn import_graphs(filename: &str) -> Vec<(String, Graph<DependencyEvent, ()>)> {
        let file = fs::File::open(filename)
            .expect("Something went wrong opening the file");
        let mut reader = BufReader::new(file);
        let mut graphs = vec![];
        for i in 0..6 {
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
}
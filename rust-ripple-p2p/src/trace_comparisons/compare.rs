#[cfg(test)]
mod graph_comparisons {
    use std::collections::HashMap;
    use std::str;
    use std::fmt::Debug;
    use std::fs;
    use std::hash::Hash;
    use std::io::{BufRead, BufReader};
    use itertools::Itertools;
    use petgraph::Graph;
    use crate::node_state::DependencyEvent;

    #[test]
    fn it_works() {
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
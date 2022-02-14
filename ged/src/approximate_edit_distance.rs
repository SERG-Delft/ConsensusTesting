use std::fmt::Debug;
use std::hash::Hash;
use ndarray::{Array2};
use petgraph::Graph;
use crate::graph_edit_distance::{calculate_cost_matrix, create_indexed_graph, IndexNodePair, munkres_min_cost};

pub fn approximate_aed_graph_edit_distance<N, E>(graph1: Graph<N, E, petgraph::Directed>, graph2: Graph<N, E, petgraph::Directed>) -> i32
    where N: PartialEq + Eq + Clone + Debug + Hash, E: PartialEq + Eq + Clone
{
    let (indexed_nodes_1, indexed_nodes_2, _indexed_edges_1, _indexed_edges_2) = create_indexed_graph(&graph1, &graph2);
    println!("Graphs indexed");
    let mut node_matrix_cost = calculate_cost_matrix(&indexed_nodes_1, &indexed_nodes_2);
    println!("node cost matrix calced: {:?}", node_matrix_cost.dim());
    add_edge_cost(&mut node_matrix_cost, &indexed_nodes_1, &indexed_nodes_2);
    println!("edge cost matrix calced");
    let star_matrix = munkres_min_cost(&mut node_matrix_cost.clone());
    println!("star matrix cost calced");

    calculate_min_cost(&node_matrix_cost, &star_matrix)
}

pub fn approximate_hed_graph_edit_distance<N, E>(graph1: Graph<N, E, petgraph::Directed>, graph2: Graph<N, E, petgraph::Directed>, scoring: DistanceScoring) -> f32
    where N: PartialEq + Eq + Clone + Debug + Hash, E: PartialEq + Eq + Clone
{
    let (indexed_nodes_1, indexed_nodes_2, _indexed_edges_1, _indexed_edges_2) = create_indexed_graph(&graph1, &graph2);
    println!("Nodes_1: {}, Edges_1: {}, Nodes_2: {}, Edges_2: {}", indexed_nodes_1.len(),
             indexed_nodes_1.iter().map(|x| x.number_of_edges).sum::<i32>() / 2,
             indexed_nodes_2.len(), indexed_nodes_2.iter().map(|x| x.number_of_edges).sum::<i32>() / 2);
    let hed = hausdorff_edit_distance(&indexed_nodes_1, &indexed_nodes_2);
    match scoring {
        DistanceScoring::Absolute => hed,
        DistanceScoring::Normalized => {
            let max_nodes = indexed_nodes_1.len().max(indexed_nodes_2.len()) as i32;
            let max_hed: f32 = (max_nodes + indexed_nodes_1.iter().map(|x| x.number_of_edges).sum::<i32>() / 2
                + indexed_nodes_2.iter().map(|x| x.number_of_edges).sum::<i32>() / 2) as f32;
            println!("{}, {}", hed, max_hed);
            1.0 - (hed / max_hed)
        }
    }

}

fn hausdorff_edit_distance<N: PartialEq + Eq + Clone + Debug + Hash>(nodes_1: &Vec<IndexNodePair<N>>, nodes_2: &Vec<IndexNodePair<N>>) -> f32
{
    let mut distance_1: Vec<f32> = vec![1f32; nodes_1.len()];
    let mut distance_2: Vec<f32> = vec![1f32; nodes_2.len()];
    for i in 0..nodes_1.len() {
        distance_1[i] += nodes_1[i].number_of_edges as f32 / 2f32;
    }
    for j in 0..nodes_2.len() {
        distance_2[j] += nodes_2[j].number_of_edges as f32 / 2f32;
    }
    for i in 0..nodes_1.len() {
        for j in 0..nodes_2.len() {
            let mut cost_edge: f32 = hausdorff_edit_cost(
                &nodes_1[i].edges,
                &nodes_2[j].edges,
            );
            cost_edge = ((nodes_1[i].number_of_edges - nodes_2[j].number_of_edges).abs() as f32).max(cost_edge);
            let sub_cost = match nodes_1[i] == nodes_2[j] {
                true => 0.0,
                false => 1.0
            };
            distance_1[i] = (((sub_cost + cost_edge / 2.0) / 2.0) as f32).min(distance_1[i]);
            distance_2[j] = (((sub_cost + cost_edge / 2.0) / 2.0) as f32).min(distance_2[j]);
        }
    }
    let distance = distance_1.iter().sum::<f32>() + distance_2.iter().sum::<f32>();
    println!("Lower graph bound: {}, distance: {}", (nodes_1.len() as i32 - nodes_2.len() as i32).abs(), distance);
    (nodes_1.len() as f32 - nodes_2.len() as f32).abs().max(distance)
}

fn hausdorff_edit_cost<N: PartialEq + Eq + Clone + Debug + Hash>(
    edges_1: &Vec<(N, N)>,
    edges_2: &Vec<(N, N)>,
) -> f32
{
    let mut cost_1: Vec<f32> = vec![1.0; edges_1.len()];
    let mut cost_2: Vec<f32> = vec![1.0; edges_2.len()];
    for i in 0..edges_1.len() {
        for j in 0..edges_2.len() {
            let sub_cost = match edges_1[i] == edges_2[j] {
                true => 0.0,
                false => 1.0 / 2.0
            };
            cost_1[i] = f32::min(sub_cost, cost_1[i]);
            cost_2[j] = f32::min(sub_cost, cost_2[j]);
        }
    }
    cost_1.iter().sum::<f32>() + cost_2.iter().sum::<f32>()
}

fn add_edge_cost<N: PartialEq + Eq + Clone + Debug + Hash>(cost_matrix: &mut Array2<i32>, nodes_1: &Vec<IndexNodePair<N>>, nodes_2: &Vec<IndexNodePair<N>>) {
    // Substitution
    for i in 0..nodes_1.len() {
        for j in 0..nodes_2.len() {
            println!("{}, {}", i, j);
            cost_matrix[(i, j)] += calculate_edge_substitution_cost(&nodes_1[i], &nodes_2[j]);
        }
    }

    // Bottom left node insertion
    for j in 0..nodes_2.len() {
        cost_matrix[(j + nodes_1.len(), j)] += nodes_2[j].number_of_edges;
    }

    // Top right node deletion
    for i in 0..nodes_1.len() {
        cost_matrix[(i, i + nodes_2.len())] += nodes_1[i].number_of_edges;
    }
}

fn calculate_edge_substitution_cost<N: PartialEq + Eq + Clone + Debug + Hash>(node_1: &IndexNodePair<N>, node_2: &IndexNodePair<N>) -> i32 {
    let edge_cost_matrix = calculate_cost_matrix(&node_1.index_edges(), &node_2.index_edges());
    let star_matrix = munkres_min_cost(&mut edge_cost_matrix.clone());
    calculate_min_cost(&edge_cost_matrix, &star_matrix)
}

fn calculate_min_cost(cost_matrix: &Array2<i32>, star_matrix: &Array2<bool>) -> i32 {
    let mut min_cost = 0;
    for i in 0..star_matrix.nrows() {
        for j in 0..star_matrix.ncols() {
            if star_matrix[(i, j)] {
                min_cost += cost_matrix[(i, j)];
            }
        }
    }
    min_cost
}

pub enum DistanceScoring {
    Absolute,
    Normalized,
}

#[cfg(test)]
mod tests {
    use ndarray::{arr2, Array2};
    use crate::approximate_edit_distance::{add_edge_cost, approximate_hed_graph_edit_distance, calculate_edge_substitution_cost, calculate_min_cost, DistanceScoring};
    use crate::graph_edit_distance::{calculate_cost_matrix, create_indexed_graph, munkres_min_cost};
    use crate::graph_edit_distance::tests::{setup_graph, setup_graph_2};

    #[test]
    fn approximate_hed_test_1() {
        let (graph1, graph2) = setup_graph();
        let actual_similarity = approximate_hed_graph_edit_distance(graph1, graph2, DistanceScoring::Normalized);
        let expected_similarity: f32 = 1.0 - (4.0/8.0);
        assert!(actual_similarity >= expected_similarity);
    }

    #[test]
    fn approximate_hed_test_2() {
        let (graph1, graph2) = setup_graph_2();
        let actual_similarity = approximate_hed_graph_edit_distance(graph1, graph2, DistanceScoring::Normalized);
        let expected_similarity: f32 = 1.0 - (5.0/8.0);
        assert!(actual_similarity > expected_similarity);
    }

    #[test]
    fn approximate_ged_test() {
        let (graph1, graph2) = setup_graph();
        let (indexed_nodes_1, indexed_nodes_2, indexed_edges_1, indexed_edges_2) = create_indexed_graph(&graph1, &graph2);
        let mut cost_matrix = calculate_cost_matrix(&indexed_nodes_1, &indexed_nodes_2);
        add_edge_cost(&mut cost_matrix, &indexed_nodes_1, &indexed_nodes_2);
        let star_matrix= munkres_min_cost(&mut cost_matrix.clone());
        let actual_cost = calculate_min_cost(&cost_matrix, &star_matrix);
        assert_eq!(actual_cost, 6);
    }

    #[test]
    fn calculate_edge_substitution_cost_test() {
        let (graph1, graph2) = setup_graph();
        let (indexed_nodes_1, indexed_nodes_2, indexed_edges_1, indexed_edges_2) = create_indexed_graph(&graph1, &graph2);
        let mut cost_matrix = calculate_cost_matrix(&indexed_nodes_1, &indexed_nodes_2);
        let expected_edge_substitution_added_cost_matrix = arr2(&[
            [1, 2, 1, 2, 2, i32::MAX, i32::MAX, i32::MAX],
            [2, 2, 2, 2, i32::MAX, 2, i32::MAX, i32::MAX],
            [1, 2, 1, 2, i32::MAX, i32::MAX, 2, i32::MAX],
            [2, 2, 2, 2, i32::MAX, i32::MAX, i32::MAX, 2],
            [2, i32::MAX, i32::MAX, i32::MAX, 0, 0, 0, 0],
            [i32::MAX, 2, i32::MAX, i32::MAX, 0, 0, 0, 0],
            [i32::MAX, i32::MAX, 2, i32::MAX, 0, 0, 0, 0],
            [i32::MAX, i32::MAX, i32::MAX, 2, 0, 0, 0, 0],
        ]);
        add_edge_cost(&mut cost_matrix, &indexed_nodes_1, &indexed_nodes_2);
        assert_eq!(cost_matrix, expected_edge_substitution_added_cost_matrix);
    }

    #[test]
    fn calculate_min_cost_test() {
        let (cost_matrix, star_matrix) = setup_cost_star_matrix();
        assert_eq!(1 + 1 + 0, calculate_min_cost(&cost_matrix, &star_matrix));
    }

    fn setup_cost_star_matrix() -> (Array2<i32>, Array2<bool>) {
        let cost_matrix: Array2<i32> = arr2(&[
            [1, 1, 1, 1, i32::MAX, i32::MAX],
            [1, 1, 1, i32::MAX, 1, i32::MAX],
            [0, 1, 1, i32::MAX, i32::MAX, 1],
            [1, i32::MAX, i32::MAX, 0, 0, 0],
            [i32::MAX, 1, i32::MAX, 0, 0, 0],
            [i32::MAX, i32::MAX, 1, 0, 0, 0],
        ]);

        let mut star_matrix = Array2::from_elem((8, 8), false);
        star_matrix[(0, 3)] = true;
        star_matrix[(1, 2)] = true;
        star_matrix[(2, 0)] = true;

        (cost_matrix, star_matrix)
    }
}
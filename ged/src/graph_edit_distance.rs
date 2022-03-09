use std::fmt::{Debug, Display};
use std::hash::Hash;
use petgraph::{Direction, Graph};
use ndarray::{Array2};
use petgraph::prelude::EdgeRef;

#[allow(unused_variables)]
#[allow(dead_code)]
pub fn calculate_graph_edit_distance<N: Clone, E>(graph1: Graph<N, E, petgraph::Directed>, graph2: Graph<N, E, petgraph::Directed>) -> (i32, AStarNode<N>)
    where N: PartialEq + Eq + Clone + Debug + Hash, E: PartialEq + Eq + Clone + Display
{
    let (indexed_nodes_1, indexed_nodes_2, indexed_edges_1, indexed_edges_2) = create_indexed_graph(&graph1, &graph2);

    let node_matrix_cost = calculate_cost_matrix(&indexed_nodes_1, &indexed_nodes_2);
    let edge_matrix_cost = calculate_cost_matrix(&indexed_edges_1, &indexed_edges_2);
    let munkres_nodes = munkres_min_cost(&mut node_matrix_cost.clone());
    let munkres_edges = munkres_min_cost(&mut edge_matrix_cost.clone());

    let sorted_v1: Vec<IndexNodePair<N>> = sort_nodes(&indexed_nodes_1, &node_matrix_cost, &munkres_nodes);
    println!("{:?}", sorted_v1);

    let mut upper_bound = calculate_upper_bound(&node_matrix_cost, &munkres_nodes) + 1;
    upper_bound += calculate_upper_bound(&edge_matrix_cost, &munkres_edges) + 1;

    let mut open: Vec<AStarNode<N>> = vec![];
    let mut best_edit_path: Vec<AStarNode<N>> = vec![];

    let root: AStarNode<N> = AStarNode::new(vec![], vec![], indexed_nodes_1.clone(), indexed_nodes_2.clone(), indexed_edges_1, indexed_edges_2, None, 0);
    let mut parent_temp: AStarNode<N> = root.clone();
    let children = parent_temp.children(&sorted_v1[0]);
    children.iter().for_each(|child| print!("g: {}, lb: {}; ", child.g, child.lb));
    println!("UB: {}", upper_bound);
    open.extend(children);
    let mut k = 0;
    loop {
        let mut p_min= if k < sorted_v1.len() {
            parent_temp.best_child(&sorted_v1[k], &open)
        } else {
            vec![]
        };
        while p_min.is_empty() && parent_temp != root {
            k -= 1;
            parent_temp = *parent_temp.parent.unwrap();
            p_min = parent_temp.best_child(&sorted_v1[k], &open);
        }
        if p_min.is_empty() && parent_temp == root {
            return (upper_bound, best_edit_path[0].clone());
        }
        open.remove(open.iter().position(|v| *v == p_min[0]).unwrap());
        println!("new p_min: matched_nodes {:?}, matched_edges {:?}", p_min[0].matched_nodes, p_min[0].matched_edges);
        k += 1;
        if p_min[0].g + p_min[0].lb < upper_bound {
            if !p_min[0].pending_nodes_1.is_empty() {
                let children = p_min[0].children(&sorted_v1[k]);
                children.iter().for_each(|child| print!("g: {}, lb: {}; ", child.g, child.lb));
                println!("UB: {}", upper_bound);

                for i in 0..children.len() {
                    if children[i].g + children[i].lb < upper_bound {
                        open.push(children[i].clone());
                    }
                }
            } else {
                let p = p_min[0].clone().generate_complete_solution();
                println!("g: {}", p.g);
                if p.g < upper_bound {
                    upper_bound = p.g;
                    best_edit_path.push(p.clone());
                }
            }
        }
        parent_temp = p_min[0].clone();
    }



}

fn sort_nodes<N: Clone + Eq + Debug + Hash>(nodes: &Vec<IndexNodePair<N>>, node_cost_matrix: &Array2<i32>, munkres_nodes: &Array2<bool>) -> Vec<IndexNodePair<N>> {
    let mut index_node_value_pairs = vec![];
    for i in 0..nodes.len() {
        index_node_value_pairs.push((nodes[i].clone(), node_cost_matrix[(i, munkres_nodes.row(i).iter().position(|v| *v).unwrap())]));
    }
    index_node_value_pairs.sort_unstable_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    index_node_value_pairs.iter().map(|x| x.0.clone()).collect()
}

fn calculate_upper_bound(cost_matrix: &Array2<i32>, munkres_matrix: &Array2<bool>) -> i32 {
    let mut upper_bound = 0;
    for i in 0..munkres_matrix.nrows() {
        for j in 0..munkres_matrix.ncols() {
            if munkres_matrix[(i, j)] {
                upper_bound += cost_matrix[(i, j)];
            }
        }
    }
    upper_bound
}

pub fn create_indexed_graph<'a, N: Clone + Eq + Debug + Hash, E>(graph1: &'a Graph<N, E>, graph2: &'a Graph<N, E>) -> (Vec<IndexNodePair<N>>, Vec<IndexNodePair<N>>, Vec<IndexEdgePair<N>>, Vec<IndexEdgePair<N>>) {
    let mut indexed_nodes_1: Vec<IndexNodePair<N>> = vec![];
    let mut indexed_nodes_2: Vec<IndexNodePair<N>> = vec![];
    let indexed_edges_1 = fill_nodes_edges(graph1, &mut indexed_nodes_1);
    let indexed_edges_2 = fill_nodes_edges(graph2, &mut indexed_nodes_2);

    (indexed_nodes_1, indexed_nodes_2, indexed_edges_1, indexed_edges_2)
}

pub fn fill_nodes_edges<N: Clone + Eq + Debug + Hash, E>(graph: &Graph<N, E>, indexed_nodes: &mut Vec<IndexNodePair<N>>) -> Vec<IndexEdgePair<N>> {
    let mut edges = vec![];
    for (i, index) in graph.node_indices().enumerate() {
        let mut outgoing_edges = vec![];
        let mut incoming_edges = vec![];
        for edge in graph.edges_directed(index, Direction::Outgoing) {
            let index_edge = graph.node_weight(edge.target()).unwrap().clone();
            outgoing_edges.push(index_edge.clone());
            edges.push(IndexEdgePair::new(graph.node_weight(index).unwrap().clone(), index_edge));
        }
        for edge in graph.edges_directed(index, Direction::Incoming) {
            let index_edge = graph.node_weight(edge.source()).unwrap().clone();
            incoming_edges.push(index_edge.clone());
        }
        indexed_nodes.push(IndexNodePair::new(graph.node_weight(index).unwrap().clone(), incoming_edges, outgoing_edges, i));
    }
    edges
}

pub fn calculate_cost_matrix<T: GraphComponent>(elements1: &Vec<T>, elements2: &Vec<T>) -> Array2<i32> {
    let k = elements1.len() + elements2.len();
    let mut matrix = Array2::from_elem((k, k), 0);

    // Substitution cost
    for i in 0..elements1.len() {
        for j in 0..elements2.len() {
            if elements1[i].value() == elements2[j].value() {
                matrix[(i, j)] = 0
            } else {
                matrix[(i, j)] = 1
            }
        }
    }

    // Insertion cost
    for i in elements1.len()..k {
        for j in 0..elements2.len() {
            if i-elements1.len() == j {
                matrix[(i, j)] = 1;
            } else {
                matrix[(i, j)] = i32::MAX
            }
        }
    }

    // Deletion cost
    for i in 0..elements1.len() {
        for j in elements2.len()..k {
            if i == j-elements2.len() {
                matrix[(i, j)] = 1;
            } else {
                matrix[(i, j)] = i32::MAX
            }
        }
    }

    matrix
}

pub fn munkres_min_cost(cost_matrix: &mut Array2<i32>) -> Array2<bool> {
    let mut star_matrix: Array2<bool> = Array2::from_elem((cost_matrix.nrows(), cost_matrix.ncols()), false);
    let mut prime_matrix: Array2<bool> = Array2::from_elem((cost_matrix.nrows(), cost_matrix.ncols()), false);
    let mut covered_columns = vec![false; cost_matrix.ncols()];
    let mut covered_rows = vec![false; cost_matrix.nrows()];

    for mut row in cost_matrix.rows_mut().into_iter() {
        let min = *row.iter().min().unwrap();
        row -= min
    }

    for mut col in cost_matrix.columns_mut().into_iter() {
        let min = *col.iter().min().unwrap();
        col -= min;
    }

    for i in 0..cost_matrix.nrows() {
        for j in 0..cost_matrix.ncols() {
            if cost_matrix.get((i, j)).unwrap() == &0 && star_row_col_check(&star_matrix, i, j) {
                star_matrix[(i, j)] = true;
            }
        }
    }

    //Step 1
    loop {
        for (i, column) in star_matrix.columns().into_iter().enumerate() {
            if column.iter().any(|v| *v) {
                covered_columns[i] = true;
            }
        }
        if covered_columns.iter().filter(|x| **x).count() != cost_matrix.ncols() {
            //step 2
            loop {
                if let Some(z) = find_uncovered_zero(cost_matrix, &covered_columns, &covered_rows) {
                    let (x, y) = z;
                    prime_matrix[(x, y)] = true;
                    if !star_matrix.row(x).iter().any(|v| *v) {
                        //Step 3
                        let mut z0 = (x, y);
                        let mut s = vec![z0];
                        while let Some(z1) = find_z1(&star_matrix, &z0) {
                            s.push(z1);
                            let new_y = prime_matrix.row(z1.0).iter().position(|v| *v).unwrap();
                            z0 = (z1.0, new_y);
                            s.push(z0);
                        }
                        for (x, y) in s {
                            star_matrix[(x, y)] = false;
                            if prime_matrix[(x, y)] {
                                star_matrix[(x, y)] = true;
                            }
                        }
                        prime_matrix = Array2::from_elem((cost_matrix.nrows(), cost_matrix.ncols()), false);
                        covered_columns = vec![false; cost_matrix.ncols()];
                        covered_rows = vec![false; cost_matrix.nrows()];
                        break;
                        //// 3
                    } else {
                        covered_rows[x] = true;
                        let y = star_matrix.row(x).iter().position(|v| *v).unwrap();
                        covered_columns[y] = false;
                    }
                } else {
                    let e_min = find_smallest_uncovered_element(cost_matrix, &covered_columns, &covered_rows);
                    // Step 4
                    for i in 0..cost_matrix.nrows() {
                        for j in 0..cost_matrix.ncols() {
                            if covered_rows[i] { cost_matrix[(i, j)] += e_min }
                            if !covered_columns[j] { cost_matrix[(i, j)] -= e_min }
                        }
                    }
                }
            }
        } else {
            break;
        }
    }
    star_matrix
}

fn find_z1(star_matrix: &Array2<bool>, z0: &(usize, usize)) -> Option<(usize, usize)> {
    for (i, v) in star_matrix.column(z0.1).iter().enumerate() {
        if *v { return Some((i, z0.1)); }
    }
    None
}

fn find_smallest_uncovered_element(cost_matrix: &mut Array2<i32>, covered_columns: &Vec<bool>, covered_rows: &Vec<bool>) -> i32 {
    let mut smallest_e = i32::MAX;
    for i in 0..cost_matrix.nrows() {
        for j in 0..cost_matrix.ncols() {
            if cost_matrix[(i, j)] < smallest_e && !covered_columns[j] && !covered_rows[i] {
                smallest_e = cost_matrix[(i, j)];
            }
        }
    }
    smallest_e
}

fn find_uncovered_zero(cost_matrix: &mut Array2<i32>, covered_columns: &Vec<bool>, covered_rows: &Vec<bool>) -> Option<(usize, usize)> {
    for i in 0..cost_matrix.nrows() {
        for j in 0..cost_matrix.ncols() {
            if cost_matrix[(i, j)] == 0 && !covered_columns[j] && !covered_rows[i] {
                return Some((i, j));
            }
        }
    }
    None
}

/// Return true if no other value in col and row is true
fn star_row_col_check(star_matrix: &Array2<bool>, row: usize, col: usize) -> bool {
    !star_matrix.row(row).iter().any(|x| *x) && !star_matrix.column(col).iter().any(|x| *x)
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AStarNode<N>
    where N: PartialEq + Eq + Clone + Debug + Hash
{
    matched_nodes: Vec<IndexNodePair<N>>,
    matched_edges: Vec<IndexEdgePair<N>>,
    pending_nodes_1: Vec<IndexNodePair<N>>,
    pending_nodes_2: Vec<IndexNodePair<N>>,
    pending_edges_1: Vec<IndexEdgePair<N>>,
    pending_edges_2: Vec<IndexEdgePair<N>>,
    parent: Option<Box<AStarNode<N>>>,
    g: i32,
    lb: i32,
    node_cost_matrix: Array2<i32>,
    edge_cost_matrix: Array2<i32>,
    children: Option<Vec<AStarNode<N>>>
}

impl<N> AStarNode<N>
    where N: PartialEq + Eq + Clone + Debug + Hash
{
    fn new(
        matched_nodes: Vec<IndexNodePair<N>>,
        matched_edges: Vec<IndexEdgePair<N>>,
        pending_nodes_1: Vec<IndexNodePair<N>>,
        pending_nodes_2: Vec<IndexNodePair<N>>,
        pending_edges_1: Vec<IndexEdgePair<N>>,
        pending_edges_2: Vec<IndexEdgePair<N>>,
        parent: Option<AStarNode<N>>,
        g: i32
    ) -> Self {
        let node_cost_matrix = calculate_cost_matrix(&pending_nodes_1, &pending_nodes_2);
        let edge_cost_matrix = calculate_cost_matrix(&pending_edges_1, &pending_edges_2);

        if parent == None {
            return Self { matched_nodes, matched_edges, pending_nodes_1, pending_nodes_2, pending_edges_1, pending_edges_2, parent: None, g, lb: i32::MAX, node_cost_matrix, edge_cost_matrix, children: None };
        }
        let node_munkres_matrix = munkres_min_cost(&mut node_cost_matrix.clone());
        let mut lb = 0;
        for i in 0..node_munkres_matrix.nrows() {
            for j in 0..node_munkres_matrix.ncols() {
                if node_munkres_matrix[(i, j)] { lb += node_cost_matrix[(i, j)] }
            }
        }
        let edge_munkres_matrix = munkres_min_cost(&mut edge_cost_matrix.clone());
        for i in 0..edge_munkres_matrix.nrows() {
            for j in 0..edge_munkres_matrix.ncols() {
                if edge_munkres_matrix[(i, j)] { lb += edge_cost_matrix[(i, j)] }
            }
        }
        return match parent {
            None => Self { matched_nodes, matched_edges, pending_nodes_1, pending_nodes_2, pending_edges_1, pending_edges_2, parent: None, g, lb, node_cost_matrix, edge_cost_matrix, children: None },
            Some(par) => Self { matched_nodes, matched_edges, pending_nodes_1, pending_nodes_2, pending_edges_1, pending_edges_2, parent: Some(Box::new(par)), g, lb, node_cost_matrix, edge_cost_matrix, children: None }
        }
    }

    fn children(&mut self, v1: &IndexNodePair<N>) -> Vec<AStarNode<N>> where N: Clone + Hash {
        if self.pending_nodes_1.is_empty() { return vec![]; }

        let mut children = vec![];
        let mut new_pending_nodes_1 = self.pending_nodes_1.clone();
        let new_node_1_index = new_pending_nodes_1.iter().position(|node| *node == *v1).unwrap();
        new_pending_nodes_1.remove(new_node_1_index);

        let mut new_pending_edges_1 = self.pending_edges_1.clone();
        let matched_edges = self.matched_edges.clone();
        let mut new_matched_edges = vec![];
        for index in &v1.check_edges(&new_pending_edges_1) {
            new_matched_edges.push(new_pending_edges_1.remove(*index));
        }

        let mut new_matched_nodes = self.matched_nodes.clone();
        new_matched_nodes.push(v1.clone());

        for i in 0..self.pending_nodes_2.len() {
            let mut new_pending_nodes_2 = self.pending_nodes_2.clone();
            let removed_node = new_pending_nodes_2.remove(i);
            let mut new_pending_edges_2 = self.pending_edges_2.clone();
            let mut new_matched_edges_2 = vec![];
            for index in &removed_node.check_edges(&new_pending_edges_2) {
                new_matched_edges_2.push(new_pending_edges_2.remove(*index));
            }
            let child_node = Self::new(
                [new_matched_nodes.as_slice(), &[self.pending_nodes_2[i].clone()]].concat().to_vec(),
                [matched_edges.as_slice(), new_matched_edges.clone().as_slice(), new_matched_edges_2.as_slice()].concat().to_vec(),
                new_pending_nodes_1.clone(),
                new_pending_nodes_2,
                new_pending_edges_1.clone(),
                new_pending_edges_2,
                Some(self.clone()),
                self.g + self.node_cost_matrix[(new_node_1_index, i)].clone() + self.calculate_edge_cost_substitution(&v1, &removed_node)
                // self.g + self.node_cost_matrix[(new_node_1_index, i)].clone() + self.calculate_edge_cost(&new_matched_edges, &new_matched_edges_2)
            );
            children.push(child_node);
        }
        children.push(Self::new(
            new_matched_nodes.clone(),
            [matched_edges.as_slice(), new_matched_edges.as_slice()].concat().to_vec(),
            new_pending_nodes_1.clone(),
            self.pending_nodes_2.clone(),
            new_pending_edges_1.clone(),
            self.pending_edges_2.clone(),
            Some(self.clone()),
            self.g + self.node_cost_matrix[(new_node_1_index, self.pending_nodes_2.len() + new_node_1_index)].clone() + self.calculate_edge_cost_deletion(&new_matched_edges)
            // self.g + self.node_cost_matrix[(new_node_1_index, self.pending_nodes_2.len() + new_node_1_index)].clone() + self.calculate_edge_cost(&new_matched_edges, &vec![])
        ));

        children
    }

    fn best_child(&mut self, v1: &IndexNodePair<N>, open: &Vec<AStarNode<N>>) -> Vec<AStarNode<N>> {
        let mut children = self.children(v1);
        let (mut best_child, mut min_cost) = (-1i32, i32::MAX);
        for i in 0..children.len() {
            if children[i].g + children[i].lb < min_cost && open.contains(&children[i]) {
                best_child = i as i32;
                min_cost = children[i].g + children[i].lb }
        }
        if best_child == -1 { return vec![] }
        let best_child_node = children.remove(best_child as usize);
        vec![best_child_node]
    }

    fn generate_complete_solution(self) -> AStarNode<N> {
        let mut g = self.g;
        for i in 0..self.pending_nodes_2.len() {
            g += self.node_cost_matrix[(i, i)];
        }
        if !self.pending_edges_1.is_empty() {
            println!("Ohjeeeee edges niet leeg, maar nodes wel");
        }
        g += self.calculate_edge_cost_insertion(&self.pending_edges_2);

        Self::new([self.matched_nodes.as_slice(), self.pending_nodes_2.as_slice()].concat().to_vec(), self.matched_edges.clone(), vec![], vec![], vec![], vec![], Some(self.clone()), g)
    }

    fn calculate_edge_cost_substitution(&self, node_1: &IndexNodePair<N>, node_2: &IndexNodePair<N>) -> i32 {
        // let edge_cost_matrix = calculate_cost_matrix(&edges_1, &edges_2);
        // let edge_cost_munkres = munkres_min_cost(&mut edge_cost_matrix.clone());
        ((node_2.outgoing_edges.len() as i32 - node_1.outgoing_edges.len() as i32).abs() + (node_2.incoming_edges.len() as i32 - node_1.incoming_edges.len() as i32).abs()) as i32
    }

    fn calculate_edge_cost_deletion(&self, edges: &Vec<IndexEdgePair<N>>) -> i32 {
        edges.len() as i32
    }

    fn calculate_edge_cost_insertion(&self, edges: &Vec<IndexEdgePair<N>>) -> i32 {
        edges.len() as i32
    }
}

#[derive(Clone, Debug)]
pub struct IndexNodePair<N>
    where N: PartialEq + Eq + Clone + Debug + Hash
{
    pub node: N,
    pub incoming_edges: Vec<N>,
    pub outgoing_edges: Vec<N>,
    pub edges: Vec<(N, N)>,
    pub number_of_edges: i32,
    _index: usize,
}

#[allow(unused)]
impl<N> IndexNodePair<N> where N: PartialEq + Eq + Clone + Debug + Hash {
    pub fn new(node: N, incoming_edges: Vec<N>, outgoing_edges: Vec<N>, index: usize) -> Self {
        let edges = [incoming_edges.iter().map(|x| (x.clone(), node.clone())).collect::<Vec<(N, N)>>().as_slice(), outgoing_edges.iter().map(|x| (node.clone(), x.clone())).collect::<Vec<(N, N)>>().as_slice()].concat().to_vec();
        let number_of_edges = edges.len() as i32;
        Self { node, incoming_edges, outgoing_edges, edges, number_of_edges, _index: index }
    }

    pub fn edges(&self) -> Vec<(N, N)> {
        [self.incoming_edges.iter().map(|x| (x.clone(), self.node.clone())).collect::<Vec<(N, N)>>().as_slice(), self.outgoing_edges.iter().map(|x| (self.node.clone(), x.clone())).collect::<Vec<(N, N)>>().as_slice()].concat().to_vec()
    }

    pub fn index_edges(&self) -> Vec<IndexEdgePair<N>> {
        [self.incoming_edges.iter().map(|x| IndexEdgePair::new(x.clone(), self.node.clone())).collect::<Vec<IndexEdgePair<N>>>().as_slice(),
            self.outgoing_edges.iter().map(|x| IndexEdgePair::new(self.node.clone(), x.clone())).collect::<Vec<IndexEdgePair<N>>>().as_slice()].concat().to_vec()
    }

    fn check_edges(&self, edges_to_match: &Vec<IndexEdgePair<N>>) -> Vec<usize> {
        let mut edge_indices = vec![];
        for edge in &self.outgoing_edges {
            match edges_to_match.iter().position(|edg| *edg == IndexEdgePair::new(self.node.clone(), edge.clone())) {
                Some(index) => edge_indices.push(index),
                None => ()
            }
        }
        for edge in &self.incoming_edges {
            match edges_to_match.iter().position(|edg| *edg == IndexEdgePair::new(edge.clone(), self.node.clone())) {
                Some(index) => edge_indices.push(index),
                None => ()
            }
        }
        edge_indices
    }
}

impl<N> GraphComponent for IndexNodePair<N> where N: PartialEq + Eq + Clone + Debug + Hash {
    type ValueType = N;

    fn value(&self) -> Self::ValueType {
        self.node.clone()
    }
}

impl<N> PartialEq for IndexNodePair<N> where N: PartialEq + Eq + Clone + Debug + Hash {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node
    }
}

impl<N> Eq for IndexNodePair<N> where N: PartialEq + Eq + Clone + Debug + Hash {}

#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub struct IndexEdgePair<N>
    where N: PartialEq + Eq + Clone + Debug + Hash
{
    source: N,
    target: N,
    edge: (N, N),
}

impl<N> IndexEdgePair<N> where N: PartialEq + Eq + Clone + Debug + Hash {
    pub fn new(source: N, target: N) -> Self {
        Self { source: source.clone(), target: target.clone(), edge: (source, target) }
    }
}

impl<N> GraphComponent for IndexEdgePair<N> where N: PartialEq + Eq + Clone + Debug + Hash {
    type ValueType = (N, N);

    fn value(&self) -> Self::ValueType {
        self.edge.clone()
    }
}

pub trait GraphComponent {
    type ValueType: PartialEq + Eq + Clone + Hash;

    fn value(&self) -> Self::ValueType;
}

#[cfg(test)]
pub mod tests {
    use ndarray::{arr2, Array2};
    use petgraph::Graph;
    use crate::graph_edit_distance::{AStarNode, calculate_cost_matrix, calculate_graph_edit_distance, create_indexed_graph, IndexEdgePair, IndexNodePair, munkres_min_cost, sort_nodes, star_row_col_check};

    #[test]
    fn star_row_col_check_test() {
        let mut star_matrix: Array2<bool> = Array2::from_elem((4, 4), false);
        star_matrix[(2, 2)] = true;
        assert_eq!(star_row_col_check(&star_matrix, 1, 2), false);
        assert_eq!(star_row_col_check(&star_matrix, 1, 3), true);
        star_matrix[(0, 1)] = true;
        assert_eq!(star_row_col_check(&star_matrix, 3, 1), false);
    }

    #[test]
    fn calculate_cost_matrix_test() {
        let n1 = vec![IndexNodePair::new(1, vec![],vec![],0), IndexNodePair::new(2, vec![], vec![],1), IndexNodePair::new(3, vec![], vec![], 2)];
        let n2 = vec![IndexNodePair::new(3, vec![], vec![], 0), IndexNodePair::new(4, vec![], vec![], 1), IndexNodePair::new(5, vec![], vec![], 2)];
        let expected_cost_matrix: Array2<i32> = arr2(&[
            [1, 1, 1, 1, i32::MAX, i32::MAX],
            [1, 1, 1, i32::MAX, 1, i32::MAX],
            [0, 1, 1, i32::MAX, i32::MAX, 1],
            [1, i32::MAX, i32::MAX, 0, 0, 0],
            [i32::MAX, 1, i32::MAX, 0, 0, 0],
            [i32::MAX, i32::MAX, 1, 0, 0, 0],
        ]);
        let actual_cost_matrix = calculate_cost_matrix(&n1, &n2);
        assert_eq!(actual_cost_matrix, expected_cost_matrix);
    }

    #[test]
    fn munkres_algorithm_test() {
        let mut cost_matrix = arr2(
            &[[2, 1, 3],
            [3, 2, 3],
            [3, 3, 2]]);
        let result = munkres_min_cost(&mut cost_matrix);
        println!("{:?}", result);
    }

    #[test]
    fn test_ndarray() {
        let cost_matrix = arr2(&[[2, 1, 3], [3, 2, 3], [3, 3, 2]]);
        let mut star_matrix: Array2<bool> = Array2::from_elem((cost_matrix.nrows(), cost_matrix.ncols()), false);
        assert_eq!(star_matrix[(0, 0)], false);
        star_matrix[(0, 0)] = true;
        assert_eq!(star_matrix[(0, 0)], true);
    }

    #[test]
    fn test_a_star_node() {
        let matched_nodes = vec![];
        let matched_edges = vec![];
        let edge1 = IndexEdgePair::new("node1", "node2");
        let edge2 = IndexEdgePair::new("node1", "node3");
        let node1 = IndexNodePair::new("node1", vec![], vec!["node2"], 0);
        let node2 = IndexNodePair::new("node1", vec![], vec!["node3"], 0);
        let node3 = IndexNodePair::new("node2", vec!["node1"], vec![], 1);
        let node4 = IndexNodePair::new("node3", vec!["node1"], vec![], 1);
        let pending_nodes_1 = vec![node1, node3];
        let pending_nodes_2 = vec![node2, node4];
        let mut star_node = AStarNode::new(matched_nodes,
                                           matched_edges,
                                           pending_nodes_1.clone(),
                                           pending_nodes_2,
                                           vec![edge1],
                                           vec![edge2],
                                           None,
                                           1);
        assert_eq!(star_node.lb, i32::MAX);
        let expected_node_cost_matrix = arr2(
            &[[0, 1, 1, i32::MAX],
                [1, 1, i32::MAX, 1],
                [1, i32::MAX, 0, 0],
                [i32::MAX, 1, 0, 0]]);
        assert_eq!(star_node.node_cost_matrix, expected_node_cost_matrix);
        let expected_edge_cost_matrix = arr2(
            &[[1, 1],
                [1, 0]]);
        assert_eq!(star_node.edge_cost_matrix, expected_edge_cost_matrix);
        let children = star_node.children(&pending_nodes_1[0]);
        assert_eq!(children.len(), 3);
        assert_eq!(children[0].g, 2);
        assert_eq!(children[1].g, 3);
        assert_eq!(children[2].g, 3);
        assert_eq!(star_node.best_child(&pending_nodes_1[0], &children)[0], children[0]);
    }

    #[test]
    fn test_sort_v1() {
        let (graph1, graph2) = setup_graph();
        let (indexed_nodes_1, indexed_nodes_2, _indexed_edges_1, _indexed_edges_2) = create_indexed_graph(&graph1, &graph2);

        println!("{:?}", indexed_nodes_1);
        println!("{:?}", indexed_nodes_2);

        let node_matrix_cost = calculate_cost_matrix(&indexed_nodes_1, &indexed_nodes_2);
        let munkres_nodes = munkres_min_cost(&mut node_matrix_cost.clone());

        println!("{}", node_matrix_cost);
        println!("{}", munkres_nodes);

        let sorted_v1 = sort_nodes(&indexed_nodes_1, &node_matrix_cost, &munkres_nodes);

        assert_eq!(sorted_v1, vec![indexed_nodes_1[0].clone(), indexed_nodes_1[2].clone(), indexed_nodes_1[1].clone(), indexed_nodes_1[3].clone()]);
    }

    #[test]
    fn test_children_twice_equality() {
        let (graph1, graph2) = setup_graph();
        let (indexed_nodes_1, indexed_nodes_2, indexed_edges_1, indexed_edges_2) = create_indexed_graph(&graph1, &graph2);

        let node_matrix_cost = calculate_cost_matrix(&indexed_nodes_1, &indexed_nodes_2);
        let munkres_nodes = munkres_min_cost(&mut node_matrix_cost.clone());

        let sorted_v1 = sort_nodes(&indexed_nodes_1, &node_matrix_cost, &munkres_nodes);

        let mut root = AStarNode::new(vec![], vec![], indexed_nodes_1.clone(), indexed_nodes_2.clone(), indexed_edges_1.clone(), indexed_edges_2.clone(), None, 0);
        assert_eq!(root.children(&sorted_v1[0]), root.children(&sorted_v1[0]))
    }

    #[test]
    fn test_graph_edit_distance() {
        let (graph1, graph2) = setup_graph();
        let (upper_bound, edit_path) = calculate_graph_edit_distance(graph1, graph2);
        assert_eq!(upper_bound, 3);
    }

    #[test]
    fn test_graph_edit_distance_2() {
        let (graph1, graph2) = setup_graph_2();
        let (upper_bound, edit_path) = calculate_graph_edit_distance(graph1, graph2);
        assert_eq!(upper_bound, 4);
    }

    pub fn setup_graph() -> (Graph<&'static str, &'static str>, Graph<&'static str, &'static str>) {
        let mut graph1 = Graph::<&str, &str>::new();
        let message1 = graph1.add_node("Proposal");
        let message2 = graph1.add_node("Validation");
        let message3 = graph1.add_node("Proposal");
        let message4 = graph1.add_node("Validation");
        graph1.extend_with_edges(&[(message1, message3), (message2, message4)]);
        let mut graph2 = Graph::<&str, &str>::new();
        let message1 = graph2.add_node("Proposal");
        let message2 = graph2.add_node("Transaction");
        let message3 = graph2.add_node("Proposal");
        let message4 = graph2.add_node("Transaction");
        graph2.extend_with_edges(&[(message1, message2), (message3, message4)]);
        (graph1, graph2)
    }

    pub fn setup_graph_2() -> (Graph<&'static str, &'static str>, Graph<&'static str, &'static str>) {
        let mut graph1 = Graph::<&str, &str>::new();
        let message1 = graph1.add_node("Proposal");
        let message2 = graph1.add_node("Validation");
        let message3 = graph1.add_node("Proposal");
        let message4 = graph1.add_node("Validation");
        graph1.extend_with_edges(&[(message1, message3), (message2, message4), (message2, message3)]);
        let mut graph2 = Graph::<&str, &str>::new();
        let message1 = graph2.add_node("Proposal");
        let message2 = graph2.add_node("Transaction");
        let message3 = graph2.add_node("Proposal");
        let message4 = graph2.add_node("Transaction");
        graph2.extend_with_edges(&[(message1, message2), (message3, message4), (message1, message4)]);
        (graph1, graph2)
    }
}
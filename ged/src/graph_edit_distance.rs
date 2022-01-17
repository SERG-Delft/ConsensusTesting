use petgraph::Graph;

pub fn calculate_graph_edit_distance<N: Clone, E>(graph1: Graph<N, E>, graph2: Graph<N, E>) -> f64
    where N: PartialEq
{
    let node_matrix_cost = calculate_cost_matrix(graph1.raw_nodes().iter().map(|n| n.weight.clone()).collect(), graph2.raw_nodes().iter().map(|n| n.weight.clone()).collect());
    let edge_matrix_cost = calculate_cost_matrix(graph1.raw_edges().iter().map(|e| (e.source(), e.target())).collect(), graph2.raw_edges().iter().map(|e| (e.source(), e.target())).collect());
    0f64
}

pub fn calculate_cost_matrix<N>(elements1: Vec<N>, elements2: Vec<N>) -> Vec<Vec<i32>>
    where N: PartialEq
{
    let k = elements1.len() + elements2.len();
    let mut matrix = vec![vec![0; k]; k];

    // Substitution cost
    for i in 0..elements1.len() {
        for j in 0..elements2.len() {
            if elements1[i] == elements2[j] {
                matrix[i][j] = 0
            } else {
                matrix[i][j] = 1
            }
        }
    }

    // Insertion cost
    for i in elements1.len()..k {
        for j in 0..elements2.len() {
            if i-elements1.len() == j {
                matrix[i][j] = 1;
            } else {
                matrix[i][j] = i32::MAX
            }
        }
    }

    // Deletion cost
    for i in 0..elements1.len() {
        for j in elements2.len()..k {
            if i == j-elements2.len() {
                matrix[i][j] = 1;
            } else {
                matrix[i][j] = i32::MAX
            }
        }
    }

    matrix
}

pub fn munkres_min_cost(cost_matrix: &mut Vec<Vec<i32>>) {
    let mut star_matrix = vec![vec![false; cost_matrix[0].len()]; cost_matrix.len()];
    let mut covered_columns = vec![false; cost_matrix[0].len()];
    let mut covered_rows = vec![false; cost_matrix.len()];

    for i in 0..cost_matrix.len() {
        let min = cost_matrix[i].iter().min().unwrap();
        cost_matrix[i] = cost_matrix[i].iter().map(|x| x - min).collect();
    }

    for i in 0..cost_matrix.len() {
        for j in 0..cost_matrix[0].len() {
            if cost_matrix[i][j] == 0 && star_row_col_check(&star_matrix, i, j) {
                star_matrix[i][j] = true;
            }
        }
    }

    //Step 1
    for j in start_matrix.iter().any(|row| row.iter().any(|x| x))
}

/// Return true if no other value in col and row is true
fn star_row_col_check(star_matrix: &Vec<Vec<bool>>, row: usize, col: usize) -> bool {
    !star_matrix[row].iter().any(|x| *x) && !star_matrix.iter().map(|x| x[col]).any(|x| x)
}

#[cfg(test)]
mod tests {
    use crate::graph_edit_distance::{calculate_cost_matrix, star_row_col_check};

    #[test]
    fn star_row_col_check_test() {
        let mut star_matrix = vec![vec![false; 4]; 4];
        star_matrix[2][2] = true;
        assert_eq!(star_row_col_check(&star_matrix, 1, 2), false);
        assert_eq!(star_row_col_check(&star_matrix, 1, 3), true);
        star_matrix[0][1] = true;
        assert_eq!(star_row_col_check(&star_matrix, 3, 1), false);
    }

    #[test]
    fn calculate_cost_matrix_test() {
        let n1 = vec![1, 2, 3];
        let n2 = vec![3, 4, 5];
        let expected_cost_matrix = vec![
            vec![1, 1, 1, 1, i32::MAX, i32::MAX],
            vec![1, 1, 1, i32::MAX, 1, i32::MAX],
            vec![0, 1, 1, i32::MAX, i32::MAX, 1],
            vec![1, i32::MAX, i32::MAX, 0, 0, 0],
            vec![i32::MAX, 1, i32::MAX, 0, 0, 0],
            vec![i32::MAX, i32::MAX, 1, 0, 0, 0],
        ];
        let actual_cost_matrix = calculate_cost_matrix(n1, n2);
        assert_eq!(actual_cost_matrix, expected_cost_matrix);
    }
}
#[cfg(test)]
mod fitness_comparisons {
    use std::fs;
    use std::io::{BufRead, BufReader};
    use genevo::genetic::AsScalar;
    use ndarray::{Array, ShapeBuilder};
    use ndarray_stats::CorrelationExt;
    use crate::ga::fitness::compared_fitness_functions::ComparedFitnessFunctions;

    #[test]
    fn calculate_correlation() {
        let fitness_values = import_fitness_values("fitness_values.txt");

        let delays_fitness = fitness_values.iter().map(|x| x.1.delay_fitness.as_scalar()).collect::<Vec<f64>>();
        let time_fitness = fitness_values.iter().map(|x| x.1.time_fitness.as_scalar()).collect::<Vec<f64>>();
        let validated_ledgers_fitness = fitness_values.iter().map(|x| x.1.validated_ledgers_fitness.as_scalar()).collect::<Vec<f64>>();
        let failed_consensus_fitness = fitness_values.iter().map(|x| x.1.failed_consensus_fitness.as_scalar()).collect::<Vec<f64>>();
        let state_accounting_fitness = fitness_values.iter().map(|x| x.1.state_accounting_fitness.as_scalar()).collect::<Vec<f64>>();

        println!("{:?}", delays_fitness);
        println!("{:?}", time_fitness);
        println!("{:?}", validated_ledgers_fitness);
        println!("{:?}", failed_consensus_fitness);
        println!("{:?}", state_accounting_fitness);

        let a = Array::from_shape_vec((5, 500).strides((500, 1)), [delays_fitness, time_fitness, validated_ledgers_fitness, failed_consensus_fitness, state_accounting_fitness].concat()).unwrap();
        println!("{:?}, {:?}", a.ncols(), a.nrows());
        println!("{:?}, {:?}", a.columns().to_string(), a.rows().to_string());
        let correlation = a.pearson_correlation().unwrap();
        assert_eq!(fitness_values.len(), 500);
        println!("{:?}", correlation);
    }

    fn import_fitness_values(filename: &str) -> Vec<(String, ComparedFitnessFunctions)> {
        let file = fs::File::open(filename)
            .expect("Something went wrong opening the file");
        let mut reader = BufReader::new(file);
        let mut fitness_values = vec![];
        let number_of_different_delays = 100;
        let number_of_tests_per_delay = 5;
        for _ in 0..number_of_different_delays*number_of_tests_per_delay {
            let mut delay_buf = vec![];
            reader.read_until(b'+', &mut delay_buf).expect("Reading until delimiter failed");
            let delay_string = match std::str::from_utf8(&delay_buf[1..delay_buf.len()-1]) {
                Ok(v) => v,
                Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
            };
            let mut fitness_buf = vec![];
            reader.read_until(b'+', &mut fitness_buf).expect("Reading until delimiter failed");
            let fitness_string = match std::str::from_utf8(&fitness_buf[1..fitness_buf.len()-1]) {
                Ok(v) => v,
                Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
            };
            let compared_fitness_functions: ComparedFitnessFunctions = serde_json::from_str(&fitness_string).unwrap();
            fitness_values.push((delay_string.to_string(), compared_fitness_functions));
        }
        fitness_values
    }
}
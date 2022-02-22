#[cfg(test)]
mod fitness_comparisons {
    use std::fs;
    use std::io::{BufRead, BufReader};
    use crate::ga::fitness::compared_fitness_functions::ComparedFitnessFunctions;

    #[test]
    fn calculate_correlation() {
        let fitness_values = import_fitness_values("fitness_values.txt");
        println!("{:?}", fitness_values);
        assert_eq!(fitness_values.len(), 500);
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
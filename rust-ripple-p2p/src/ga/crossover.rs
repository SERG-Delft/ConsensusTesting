use std::fmt::Debug;
use genevo::genetic::{Children, Genotype, Parents};
use genevo::operator::{CrossoverOp, GeneticOperator};
use genevo::prelude::Rng;

#[derive(Clone, Debug, PartialEq)]
pub struct NoCrossoverOperator {}

impl GeneticOperator for NoCrossoverOperator {
    fn name() -> String {
        "No Crossover Operator".to_string()
    }
}

impl<G> CrossoverOp<G> for NoCrossoverOperator
    where G: Genotype
{
    fn crossover<R>(&self, parents: Parents<G>, _rng: &mut R) -> Children<G> where R: Rng + Sized {
        parents
    }
}


#[allow(missing_copy_implementations)]
#[derive(Clone, Debug, PartialEq)]
pub struct SimulatedBinaryCrossBreeder {
    crossover_probability: f32,
    min_value: u32,
    max_value: u32,
}

impl SimulatedBinaryCrossBreeder {
    pub fn new(crossover_probability: f32, min_value: u32, max_value: u32) -> Self {
        SimulatedBinaryCrossBreeder { crossover_probability, min_value, max_value }
    }

    pub fn set_crossover_probability(&mut self, value: f32) {
        self.crossover_probability = value;
    }
}

impl GeneticOperator for SimulatedBinaryCrossBreeder {
    fn name() -> String {
        "Simulated-Binary-Cross-Breeder".to_string()
    }
}

impl<G> CrossoverOp<G> for SimulatedBinaryCrossBreeder
    where
        G: Genotype + SimulatedBinaryCrossover,
{
    fn crossover<R>(&self, parents: Parents<G>, rng: &mut R) -> Children<G>
        where
            R: Rng + Sized,
    {
        SimulatedBinaryCrossover::crossover(parents, self.crossover_probability, self.min_value, self.max_value, rng)
    }
}

pub trait SimulatedBinaryCrossover: Genotype {
    type Dna: Debug;

    fn crossover<R>(parents: Parents<Self>, crossover_probability: f32, min_value: u32, max_value: u32, rng: &mut R) -> Children<Self>
        where
            R: Rng + Sized;
}

impl SimulatedBinaryCrossover for Vec<u32> {
    type Dna = u32;

    fn crossover<R>(parents: Parents<Self>, crossover_probability: f32, min_value: u32, max_value: u32, rng: &mut R) -> Children<Self>
        where
            R: Rng + Sized,
    {
        let n = 3.0;
        let genome_length = parents[0].len();
        let num_parents = parents.len();
        if num_parents != 2 {
            panic!("SBX requires exactly two parents")
        }
        // breed one child for each partner in parents
        let mut child_1 = Vec::with_capacity(genome_length);
        let mut child_2 = Vec::with_capacity(genome_length);
        let parent_1 = &parents[0];
        let parent_2 = &parents[1];
        for (gene_1, gene_2) in parent_1.iter().zip(parent_2) {
            if rng.gen::<f32>() > crossover_probability || gene_1 == gene_2 {
                child_1.push(gene_1.clone());
                child_2.push(gene_2.clone());
            } else {
                let (y1, y2) = (*gene_1.min(gene_2) as f32, *gene_1.max(gene_2) as f32);
                let beta = if y1 - min_value as f32 > max_value as f32 - y2 {
                    1.0 + (2.0 * (max_value as f32 - y2) / (y2 - y1))
                } else {
                    1.0 + (2.0 * (y1 - min_value as f32) / (y2 - y1))
                };
                let mut alpha = 2.0 - (beta.powf(-(n+1.0)));
                let u = rng.gen::<f32>();
                let betaq = if u <= 1.0 / alpha {
                    alpha *= u;
                    alpha.powf(1.0 / (n + 1.0))
                } else {
                    alpha = 1.0 / (2.0 - (alpha * u));
                    alpha.powf(1.0 / (n + 1.0))
                };
                let child_gene_1 = 0.5 * ((y1 + y2) - betaq * (y2 - y1));
                let child_gene_2 = 0.5 * ((y1 + y2) + betaq * (y2 - y1));
                child_1.push((child_gene_1.round() as u32).clamp(min_value, max_value));
                child_2.push((child_gene_2.round() as u32).clamp(min_value, max_value));
            }
        }
        vec![child_1, child_2]
    }
}

#[cfg(test)]
mod tests {
    use rand::{thread_rng};
    use crate::ga::crossover::SimulatedBinaryCrossover;

    #[test]
    fn test_sbx() {
        let p1 = vec![10, 50, 40, 30, 20];
        let p2 = vec![30, 20, 10, 40, 50];

        let mut rng = thread_rng();
        let min_value = 0;
        let max_value = 60;
        let children = SimulatedBinaryCrossover::crossover(vec![p1.clone(), p2.clone()], 0.5, 0, 60, &mut rng);
        assert_eq!(children[0].len(), p1.len());
        for (i, (gene_1, gene_2)) in children[0].iter().zip(children[1].iter()).enumerate() {
            assert_eq!((gene_1 + gene_2) / 2, (p1[i] + p2[i]) / 2);
            assert!(*gene_1 >= min_value && *gene_1 <= max_value && *gene_2 >= min_value && *gene_2 <= max_value);
        }
        dbg!(children);
    }
}
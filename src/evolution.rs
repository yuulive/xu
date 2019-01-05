use super::population::Population;
use super::operations::{
    Operation,
    cull_lowest_agents
};
use std::thread;
use rand::{
    distributions::{Distribution, Standard}
};
use std::hash::Hash;
use super::agent::Agent;



pub fn population_from_multilevel_sub_populations<Gene, Data>(
    levels: u32,
    sub_populations_per_level: usize,
    data: Data,
    number_of_genes: usize,
    initial_population_size: usize,
    iterations_on_each_population: usize,
    get_score_index: fn(&Agent<Gene>, &Data) -> isize,
    operations: Vec<Operation<Gene, Data>>
) -> Population<Gene> 
where
Gene: Clone + Hash + Send + 'static,
Standard: Distribution<Gene>,
Data: Clone + Send + 'static
{
    let number_of_initial_populations = sub_populations_per_level.pow(levels);
    let mut populations = Vec::new();
    for _ in 0..number_of_initial_populations {
        populations.push(
            run_iterations(
                Population::new(initial_population_size, number_of_genes, false, &data, get_score_index),
                iterations_on_each_population,
                &data,
                &operations,
                get_score_index
            )
        );
    }

    populations_from_existing_multillevel(populations, levels, sub_populations_per_level, &data, iterations_on_each_population, &operations, get_score_index)
}

pub fn threaded_population_from_multilevel_sub_populations<Gene, Data>(
    levels: u32,
    sub_populations_per_level: usize,
    data: &Data,
    number_of_genes: usize,
    initial_population_size: usize,
    iterations_on_each_population: usize,
    get_score_index: fn(&Agent<Gene>, &Data) -> isize,
    operations: &Vec<Operation<Gene, Data>>
) -> Population<Gene> 
where
Gene: Clone + Send + Sync + Hash + 'static,
Standard: Distribution<Gene>,
Data: Clone + Send + Sync + 'static
{
    let mut populations = Vec::new();
    let mut handles = Vec::new();
    for _ in 0..sub_populations_per_level {
        let data_copy = data.clone();
        let operations_copy = operations.clone();
        handles.push(thread::spawn(move || population_from_multilevel_sub_populations(levels - 1, sub_populations_per_level, data_copy, number_of_genes, initial_population_size, iterations_on_each_population, get_score_index, operations_copy)));
    }

    for handle in handles {
        populations.push(handle.join().unwrap());
    }

    populations_from_existing_multillevel(populations, 1, sub_populations_per_level, data, iterations_on_each_population, operations, get_score_index)
}

fn populations_from_existing_multillevel<Gene, Data>(
    mut populations: Vec<Population<Gene>>,
    levels: u32,
    sub_populations_per_level: usize,
    data: &Data,
    iterations_on_each_population: usize,
    operations: &Vec<Operation<Gene, Data>>,
    get_score_index: fn(&Agent<Gene>, &Data) -> isize,
) -> Population<Gene>
where 
Gene: Clone + Hash + Send + 'static,
Standard: Distribution<Gene>,
Data: Clone + Send + 'static
{
    let cull_proportion = 1.0 - 1.0 / sub_populations_per_level as f64;
    for level in (0..levels).rev() {
        let number_of_new_populations = sub_populations_per_level.pow(level);
        let mut new_populations = Vec::new();
        for _ in 0..number_of_new_populations {
            let mut population = Population::new_empty(false);
            for _ in 0..sub_populations_per_level {
                let agents = populations.pop().unwrap().get_agents().clone();
                for (score, agent) in agents {
                    population.insert(score, agent);
                }
            }
            new_populations.push(
                cull_lowest_agents(
                    run_iterations(population, iterations_on_each_population, data, operations, get_score_index),
                    cull_proportion,
                    1
                )
            );
        }

        populations = new_populations;
    }

    populations.pop().unwrap()
}

fn run_iterations<Gene, Data>(
    mut population: Population<Gene>,
    iterations: usize,
    data: &Data,
    operations: &Vec<Operation<Gene, Data>>,
    get_score_index: fn(&Agent<Gene>, &Data) -> isize
) -> Population<Gene>
where
Standard: Distribution<Gene>,
Gene: Clone + Hash + Send + 'static,
Data: Clone + Send + 'static
{
    for _ in 0..iterations {
        for operation in operations.iter() {
            population = operation.run(population, data, get_score_index);
        }
    }

    population
}

use std::time::Instant;

use markov::{MarkovAllNodes, MarkovNode, MarkovToken};

use rayon::{iter::ParallelIterator, str::ParallelString};

mod markov;

fn main() {
    // chain creation

    let content = std::fs::read_to_string("./data.txt").unwrap();

    let all_nodes = MarkovAllNodes::new();
    let root_node = MarkovNode::new(MarkovToken::Root);
    let end_node = MarkovNode::new(MarkovToken::End);

    let start = Instant::now();
    content.par_lines().for_each(|line| {
        let mut nodes = line
            .split_whitespace()
            .filter(|s| s.chars().all(|c| c.is_ascii_alphanumeric()))
            .map(|t| all_nodes.node(t));

        let mut node = root_node.clone();
        while !node.value.is_end() {
            let next_node = nodes.next().unwrap_or_else(|| end_node.clone());

            // it will be an empty chain so skip
            if node.value.is_root() && next_node.value.is_end() {
                break;
            }

            node.conns.connect(next_node.clone());

            node = next_node;
        }
    });
    println!("took {:?} to create chain", start.elapsed());

    // cache index maps

    let start = Instant::now();
    root_node.conns.index_map();
    all_nodes.cache_index_maps();
    println!("took {:?} to cache index maps", start.elapsed());

    // generation

    let mut rng = rand::thread_rng();
    loop {
        let mut picked_start_word = String::new();
        std::io::stdin().read_line(&mut picked_start_word).unwrap();

        let start = Instant::now();

        let mut result = String::new();
        let mut node = all_nodes
            .try_node(picked_start_word.trim())
            .unwrap_or_else(|| root_node.clone());

        loop {
            let next_node = node.conns.random_weighted(&mut rng);

            if let MarkovToken::Value(value) = &node.value {
                result += value;
                result.push(' ');
            }

            if next_node.value.is_end() {
                break;
            }

            node = next_node;
        }

        let elapsed = start.elapsed();
        println!("output: {result}");
        println!("gen took {elapsed:?}");
    }
}

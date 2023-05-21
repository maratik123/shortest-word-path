use clap::Parser;
use dict::{Dict, Index, Neighbours};
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::iter::successors;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version)]
#[command(
about = "Find shortest path",
long_about = None
)]
struct Cli {
    /// Sets a custom dict file
    #[arg(short, long, value_name = "FILE")]
    dict: Option<PathBuf>,
    /// Begin word
    word_begin: Option<String>,
    /// End word
    word_end: Option<String>,
}

/// A* finds a path from start to goal.
fn a_star(neighbours: &Neighbours, dict: &Dict, start: u32, goal: u32) -> Result<Vec<u32>, String> {
    let end = &dict[goal];

    // The set of discovered nodes that may need to be (re-)expanded.
    // Initially, only the start node is known.
    // This is usually implemented as a min-heap or priority queue rather than a hash-set.
    let mut open_set = BinaryHeap::new();
    // Sorting by our current best guess as to how cheap a path could be from start to finish
    // if it goes through n.
    open_set.push((Reverse(dict.heuristic(end, start)), &dict[start], start));

    // Backing min-heap with hash-map due to min-heap can not find element in O(1)
    let mut open_set_hash = HashSet::new();
    open_set_hash.insert(start);

    // For node n, cameFrom[n] is the node immediately preceding it on the cheapest path from
    // the start to n currently known.
    let mut came_from = HashMap::new();

    // For node n, gScore[n] is the cost of the cheapest path from start to n currently known.
    let mut g_score = HashMap::new();
    g_score.insert(start, 0);

    while let Some(&(_, _, current)) = open_set.peek() {
        if current == goal {
            return Ok(successors(Some(current), |prev| came_from.get(prev).copied()).collect());
        }
        // This operation can occur in O(Log(N)) time if openSet is a min-heap or a priority
        // queue
        open_set.pop();
        open_set_hash.remove(&current);
        for &neighbour in neighbours
            .edges()
            .get(&current)
            .iter()
            .flat_map(|neighbours| neighbours.iter())
        {
            // dict.word_len is the weight of the edge from current to neighbor
            // tentative_g_score is the distance from start to the neighbor through current
            let tentative_g_score = g_score[&current] + dict.word_len();
            if g_score
                .get(&neighbour)
                .filter(|neighbour_score| &tentative_g_score >= neighbour_score)
                .is_none()
            {
                // This path to neighbor is better than any previous one. Record it!
                came_from.insert(neighbour, current);
                g_score.insert(neighbour, tentative_g_score);
                if open_set_hash.insert(neighbour) {
                    // For node n, gScore[n] + h(n) represents our current best guess
                    // as to how cheap a path could be from start to finish if it goes
                    // through n.
                    open_set.push((
                        Reverse(tentative_g_score + dict.heuristic(end, neighbour)),
                        &dict[neighbour],
                        neighbour,
                    ));
                }
            }
        }
    }

    // Open set is empty but goal was never reached
    Err("Path not found".to_string())
}

fn main() {
    let cli = Cli::parse();
    let (begin, end) = (cli.word_begin, cli.word_end);

    let dict = if let Some(dict) = cli.dict {
        Dict::create_from_file(dict)
    } else {
        Dict::default()
    };
    let index = Index::from(&dict);
    let neighbours = Neighbours::from(&dict);

    if begin.is_none() && end.is_none() {
        for (wi, word) in dict.words().iter().enumerate() {
            let wi = wi as u32;
            print!("{word}: ");
            if let Some(neighbours_i) = neighbours.edges().get(&wi) {
                for &neighbour_i in neighbours_i {
                    print!("{} ", &dict[neighbour_i]);
                }
            }
            println!();
        }
        return;
    }

    let (begin, end) = (
        begin.expect("Begin word not defined"),
        end.expect("End word not defined"),
    );
    let &begin_i = index
        .index()
        .get(&begin[..])
        .ok_or_else(|| format!("Can not found begin word: {begin}"))
        .unwrap();
    let &end_i = index
        .index()
        .get(&end[..])
        .ok_or_else(|| format!("Can not found end word: {end}"))
        .unwrap();

    for word in a_star(&neighbours, &dict, begin_i, end_i)
        .unwrap()
        .into_iter()
        .rev()
        .map(|i| &dict[i])
    {
        print!("{word} ");
    }
    println!()
}

#[cfg(test)]
mod tests {
    use crate::{Dict, Index, Neighbours};

    #[test]
    fn a_star() {
        let dict = Dict::default();
        let index = Index::from(&dict);
        let neighbours = Neighbours::from(&dict);

        let way: Vec<_> = super::a_star(
            &neighbours,
            &dict,
            index.index()["рожа"],
            index.index()["учет"],
        )
        .unwrap()
        .into_iter()
        .rev()
        .map(|i| &dict[i])
        .collect();
        assert_eq!(
            way,
            [
                "рожа", "роза", "поза", "пора", "пара", "парс", "паюс", "плюс", "плес", "плед",
                "след", "слет", "счет", "учет"
            ]
        );
    }
}

use clap::Parser;
use roaring::{MultiOps, RoaringBitmap};
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::iter::zip;
use std::path::{Path, PathBuf};

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

struct Dict {
    words: Vec<String>,
    word_len: usize,
}

impl Dict {
    fn heuristic(&self, end: impl AsRef<str>, n: u32) -> usize {
        zip(end.as_ref().chars(), self.words[n as usize].chars())
            .filter(|(ch1, ch2)| ch1 != ch2)
            .count()
    }

    fn create_from_file(dict: impl AsRef<Path>) -> Self {
        Self::create(std::fs::read_to_string(dict).unwrap())
    }

    fn create(word_list: impl AsRef<str>) -> Self {
        let word_list = word_list.as_ref();

        let word_len = word_list.lines().next().unwrap().chars().count();

        let words = word_list
            .lines()
            .inspect(|s| {
                let cnt = s.chars().count();
                if cnt != word_len {
                    panic!("Word length mismatch: expected {word_len}, found {cnt}");
                }
            })
            .map(|s| s.to_string())
            .collect();

        Dict { words, word_len }
    }
}

impl Default for Dict {
    fn default() -> Self {
        Self::create(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/dict.txt"
        )))
    }
}

struct Index<'a> {
    index: HashMap<&'a str, u32>,
}

impl<'a> From<&'a Dict> for Index<'a> {
    fn from(dict: &'a Dict) -> Self {
        Index {
            index: dict
                .words
                .iter()
                .enumerate()
                .map(|(i, s)| (s.as_str(), i as u32))
                .collect(),
        }
    }
}

struct Neighbours {
    edges: HashMap<u32, HashSet<u32>>,
}

impl Neighbours {
    fn reconstruct_path(&self, came_from: HashMap<u32, u32>, mut current: u32) -> Vec<u32> {
        let mut total_path = vec![current];
        while let Some(&next) = came_from.get(&current) {
            total_path.push(next);
            current = next;
        }
        total_path
    }

    /// A* finds a path from start to goal.
    fn a_star(&self, dict: &Dict, start: u32, goal: u32) -> Result<Vec<u32>, String> {
        let end = dict.words[goal as usize].as_str();

        // The set of discovered nodes that may need to be (re-)expanded.
        // Initially, only the start node is known.
        // This is usually implemented as a min-heap or priority queue rather than a hash-set.
        let mut open_set = BinaryHeap::new();
        // Sorting by our current best guess as to how cheap a path could be from start to finish
        // if it goes through n.
        open_set.push((
            Reverse(dict.heuristic(end, start)),
            &dict.words[start as usize][..],
            start,
        ));

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
                return Ok(self.reconstruct_path(came_from, current));
            }
            // This operation can occur in O(Log(N)) time if openSet is a min-heap or a priority
            // queue
            open_set.pop();
            open_set_hash.remove(&current);
            for &neighbour in self
                .edges
                .get(&current)
                .iter()
                .flat_map(|neighbours| neighbours.iter())
            {
                // dict.word_len is the weight of the edge from current to neighbor
                // tentative_g_score is the distance from start to the neighbor through current
                if let Some(tentative_g_score) =
                    g_score.get(&current).map(|score| score + dict.word_len)
                {
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
                                &dict.words[neighbour as usize][..],
                                neighbour,
                            ));
                        }
                    }
                }
            }
        }

        // Open set is empty but goal was never reached
        Err("Path not found".to_string())
    }
}

impl From<&Dict> for Neighbours {
    fn from(dict: &Dict) -> Self {
        let mut bitmaps = vec![];
        for (wi, word) in dict.words.iter().enumerate() {
            let wi = wi as u32;
            for (ci, ch) in word.chars().enumerate() {
                if ci >= bitmaps.len() {
                    bitmaps.resize_with(ci + 1, || HashMap::with_capacity(32));
                }
                bitmaps[ci]
                    .entry(ch)
                    .or_insert_with(RoaringBitmap::new)
                    .insert(wi);
            }
        }

        let empty_bitmap = RoaringBitmap::new();
        let mut edges = HashMap::new();
        for (wi, word) in dict.words.iter().enumerate() {
            let wi = wi as u32;
            let chars: Vec<_> = word.chars().collect();
            for exclude_ci in 0..chars.len() {
                for neighbour in (chars
                    .iter()
                    .enumerate()
                    .filter(|(ci, _)| ci != &exclude_ci)
                    .map(|(ci, ch)| bitmaps[ci].get(ch).unwrap_or(&empty_bitmap))
                    .intersection()
                    - bitmaps[exclude_ci]
                        .get(&chars[exclude_ci])
                        .unwrap_or(&empty_bitmap))
                .iter()
                {
                    edges
                        .entry(wi)
                        .or_insert_with(HashSet::new)
                        .insert(neighbour);
                    edges
                        .entry(neighbour)
                        .or_insert_with(HashSet::new)
                        .insert(wi);
                }
            }
        }

        Self { edges }
    }
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

    if begin.is_none() || end.is_none() {
        for (wi, word) in dict.words.iter().enumerate() {
            let wi = wi as u32;
            print!("{word}: ");
            if let Some(neighbours_i) = neighbours.edges.get(&wi) {
                for &neighbour_i in neighbours_i {
                    print!("{} ", dict.words[neighbour_i as usize]);
                }
            }
            println!();
        }
        return;
    }

    let (begin, end) = (
        begin.ok_or("Begin word not defined").unwrap(),
        end.ok_or("End word not defined").unwrap(),
    );
    let &begin_i = index
        .index
        .get(&begin[..])
        .ok_or_else(|| format!("Can not found begin word: {begin}"))
        .unwrap();
    let &end_i = index
        .index
        .get(&end[..])
        .ok_or_else(|| format!("Can not found end word: {end}"))
        .unwrap();

    for word in neighbours
        .a_star(&dict, begin_i, end_i)
        .unwrap()
        .iter()
        .rev()
        .map(|&i| &dict.words[i as usize][..])
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

        let way: Vec<_> = neighbours
            .a_star(&dict, index.index["рожа"], index.index["учет"])
            .unwrap()
            .iter()
            .rev()
            .map(|&i| &dict.words[i as usize][..])
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

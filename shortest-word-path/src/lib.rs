use anyhow::{bail, Result};
use dict_lib::{Dict, Neighbours};
use log::Level::Debug;
use log::{debug, error, info, log_enabled, trace};
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::iter::{successors, zip};

fn heuristic(dict: &Dict, end: impl AsRef<str>, n: u32) -> usize {
    zip(end.as_ref().chars(), dict[n].chars())
        .filter(|(ch1, ch2)| ch1 != ch2)
        .count()
}

struct CameFromWithGScoreEntry {
    came_from: Option<u32>,
    g_score: usize,
}

/// A* finds a path from start to goal.
pub fn a_star(neighbours: &Neighbours, dict: &Dict, start: u32, goal: u32) -> Result<Vec<u32>> {
    let end = &dict[goal];

    // The set of discovered nodes that may need to be (re-)expanded.
    // Initially, only the start node is known.
    // This is usually implemented as a min-heap or priority queue rather than a hash-set.
    // Sorting by our current best guess as to how cheap a path could be from start to finish
    // if it goes through n.
    let score = heuristic(dict, end, start);
    debug!("Saving start node to open set with score = {score}");
    let mut open_set = BinaryHeap::from([(Reverse((score, score)), &dict[start], start)]);

    // Backing min-heap with hash-map due to min-heap can not find element in O(1)
    let mut open_set_hash = HashSet::from([start]);

    // For node n, cameFrom[n] is the node immediately preceding it on the cheapest path from
    // the start to n currently known.
    // For node n, gScore[n] is the cost of the cheapest path from start to n currently known.
    let mut came_from_with_g_score = HashMap::from([(
        start,
        CameFromWithGScoreEntry {
            came_from: None,
            g_score: 0,
        },
    )]);

    while let Some(&(_, _, current)) = open_set.peek() {
        debug!("Trying node {current}: '{}'", &dict[current]);
        if current == goal {
            info!("Found!");
            return Ok(successors(Some(current), |prev| {
                came_from_with_g_score
                    .get(prev)
                    .and_then(|came_from_with_g_score_entry| came_from_with_g_score_entry.came_from)
            })
            .collect());
        }
        // This operation can occur in O(Log(N)) time if openSet is a min-heap or a priority
        // queue
        open_set.pop();
        open_set_hash.remove(&current);
        if let Some(neighbours) = neighbours.get(current) {
            for &neighbour in neighbours {
                // dict.word_len is the weight of the edge from current to neighbor
                // tentative_g_score is the distance from start to the neighbor through current
                let tentative_g_score = came_from_with_g_score[&current].g_score + 1;
                let stored_g_score = came_from_with_g_score
                    .get(&neighbour)
                    .map(|came_from_with_g_score_entry| came_from_with_g_score_entry.g_score);
                trace!("Neighbour {neighbour}: '{}' has tentative g_score = {tentative_g_score}, stored g_score = {stored_g_score:?}", &dict[neighbour]);
                if stored_g_score
                    .filter(|neighbour_score| &tentative_g_score >= neighbour_score)
                    .is_none()
                {
                    if log_enabled!(Debug) {
                        if let Some(stored_g_score) = stored_g_score {
                            debug!(
                                "({}). Neighbour '{}': Tentative g_score {tentative_g_score} is better than stored one {stored_g_score}",
                                came_from_with_g_score.len(),
                                &dict[neighbour]
                            );
                        } else {
                            debug!(
                                "({}). Neighbour '{}': Store tentative g_score {tentative_g_score}",
                                came_from_with_g_score.len(),
                                &dict[neighbour]
                            );
                        }
                    }
                    // This path to neighbor is better than any previous one. Record it!
                    came_from_with_g_score.insert(
                        neighbour,
                        CameFromWithGScoreEntry {
                            came_from: Some(current),
                            g_score: tentative_g_score,
                        },
                    );
                    if open_set_hash.insert(neighbour) {
                        // For node n, gScore[n] + h(n) represents our current best guess
                        // as to how cheap a path could be from start to finish if it goes
                        // through n.
                        let heuristic = heuristic(dict, end, neighbour);
                        let score = tentative_g_score + heuristic;
                        debug!(
                            "({}). Saving neighbour node to open set with score = ({score}, {heuristic})",
                            open_set.len()
                        );
                        open_set.push((Reverse((score, heuristic)), &dict[neighbour], neighbour));
                    }
                }
            }
        } else {
            debug!("Node {current}: '{}' has none neighbours", &dict[current]);
        }
    }

    error!("Not found!");
    // Open set is empty but goal was never reached
    bail!("Path not found");
}

#[cfg(test)]
mod tests {
    use super::*;
    use dict_lib::Index;

    #[test]
    fn a_star() {
        let dict = Dict::create_default().unwrap();
        let index = Index::from(&dict);
        let neighbours = Neighbours::try_from(&dict).unwrap();

        let way: Vec<_> = super::a_star(&neighbours, &dict, index["рожа"], index["учет"])
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

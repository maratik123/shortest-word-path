use crate::{maratik, Dict};
use roaring::{MultiOps, RoaringBitmap};
use std::collections::hash_map::Iter;
use std::collections::{HashMap, HashSet};

pub struct Neighbours {
    edges: HashMap<u32, HashSet<u32>>,
}

impl Neighbours {
    #[inline]
    pub fn get(&self, key: u32) -> Option<&HashSet<u32>> {
        self.edges.get(&key)
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, u32, HashSet<u32>> {
        self.edges.iter()
    }

    pub(crate) fn create_from_proto(neighbours: maratik::shortest_word_path::Neighbours) -> Self {
        Self {
            edges: neighbours
                .edges
                .into_iter()
                .map(|(k, v)| (k, v.edges.into_iter().collect()))
                .collect(),
        }
    }
}

impl From<&Dict> for Neighbours {
    fn from(dict: &Dict) -> Self {
        let mut bitmaps = vec![HashMap::with_capacity(32); dict.word_len()];
        for (wi, word) in dict.iter().enumerate() {
            let wi = wi as u32;
            for (ci, ch) in word.chars().enumerate() {
                bitmaps[ci]
                    .entry(ch)
                    .or_insert_with(RoaringBitmap::new)
                    .insert(wi);
            }
        }

        let empty_bitmap = RoaringBitmap::new();
        let mut edges = HashMap::new();
        for (wi, word) in dict.iter().enumerate() {
            let wi = wi as u32;
            let chars: Vec<_> = word.chars().collect();
            for (exclude_ci, exclude_ch) in chars.iter().enumerate() {
                let mut neighbours = chars
                    .iter()
                    .enumerate()
                    .filter(|(ci, _)| ci != &exclude_ci)
                    .map(|(ci, ch)| bitmaps[ci].get(ch).unwrap_or(&empty_bitmap))
                    .intersection();
                neighbours -= &bitmaps[exclude_ci][exclude_ch];
                for neighbour in neighbours {
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

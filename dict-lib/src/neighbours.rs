use crate::Dict;
use anyhow::{Context, Error, Result};
use roaring::{MultiOps, RoaringBitmap};
use std::collections::hash_map::Iter;
use std::collections::{HashMap, HashSet};

pub struct Neighbours {
    edges: HashMap<u32, HashSet<u32>>,
}

impl Neighbours {
    #[inline]
    pub(crate) fn create(edges: HashMap<u32, HashSet<u32>>) -> Self {
        Self { edges }
    }

    #[inline]
    pub(crate) fn destructure(self) -> HashMap<u32, HashSet<u32>> {
        self.edges
    }

    #[inline]
    pub fn get(&self, key: u32) -> Option<&HashSet<u32>> {
        self.edges.get(&key)
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, u32, HashSet<u32>> {
        self.edges.iter()
    }
}

impl TryFrom<&Dict> for Neighbours {
    type Error = Error;

    fn try_from(dict: &Dict) -> Result<Self> {
        let mut bitmaps = vec![
            HashMap::with_capacity(32);
            dict.iter().next().context("Empty dict")?.chars().count()
        ];
        for (wi, word) in dict.iter().enumerate() {
            let wi = wi as _;
            for (ci, ch) in word.chars().enumerate() {
                bitmaps[ci]
                    .entry(ch)
                    .or_insert_with(RoaringBitmap::new)
                    .insert(wi);
            }
        }

        let mut edges = HashMap::new();
        for (wi, word) in dict.iter().enumerate() {
            let wi = wi as _;
            let chars: Vec<_> = word.chars().collect();
            for (exclude_ci, exclude_ch) in chars.iter().enumerate() {
                if let Ok(mut neighbours) = chars
                    .iter()
                    .enumerate()
                    .filter(|(ci, _)| ci != &exclude_ci)
                    .map(|(ci, ch)| bitmaps[ci].get(ch).ok_or(()))
                    .intersection()
                {
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
        }

        Ok(Self { edges })
    }
}

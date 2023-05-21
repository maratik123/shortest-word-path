use crate::Dict;
use std::collections::HashMap;

pub struct Index<'a> {
    index: HashMap<&'a str, u32>,
}

impl<'a> Index<'a> {
    #[inline]
    pub fn get(&self, key: &str) -> Option<u32> {
        self.index.get(key).copied()
    }
}

impl<'a> std::ops::Index<&str> for Index<'a> {
    type Output = u32;

    #[inline]
    fn index(&self, index: &str) -> &Self::Output {
        &self.index[index]
    }
}

impl<'a> From<&'a Dict> for Index<'a> {
    fn from(dict: &'a Dict) -> Self {
        Index {
            index: dict
                .iter()
                .enumerate()
                .map(|(i, s)| (s.as_str(), i as u32))
                .collect(),
        }
    }
}

use crate::Dict;
use std::collections::HashMap;

pub struct Index<'a> {
    index: HashMap<&'a str, u32>,
}

impl<'a> Index<'a> {
    pub fn index(&self) -> &HashMap<&'a str, u32> {
        &self.index
    }
}

impl<'a> From<&'a Dict> for Index<'a> {
    fn from(dict: &'a Dict) -> Self {
        Index {
            index: dict
                .words()
                .iter()
                .enumerate()
                .map(|(i, s)| (s.as_str(), i as u32))
                .collect(),
        }
    }
}

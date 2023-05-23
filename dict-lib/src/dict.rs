use crate::maratik;
use anyhow::{bail, Context, Result};
use itertools::Itertools;
use std::ops::Index;
use std::path::Path;
use std::slice::Iter;

pub struct Dict {
    words: Vec<String>,
    word_len: usize,
}

impl Dict {
    #[inline]
    pub fn word_len(&self) -> usize {
        self.word_len
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, String> {
        self.words.iter()
    }

    #[inline]
    pub fn create_default() -> Result<Self> {
        Self::create(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../dict/dict.txt"
        )))
    }

    #[inline]
    pub fn create_from_file(dict: impl AsRef<Path>) -> Result<Self> {
        let dict = dict.as_ref();
        Self::create(
            std::fs::read_to_string(dict)
                .with_context(|| format!("Failed to read file '{}'", dict.display()))?,
        )
    }

    #[inline]
    pub(crate) fn create_from_proto(dict: maratik::shortest_word_path::Dict) -> Self {
        Self {
            words: dict.words,
            word_len: dict.word_len as usize,
        }
    }

    pub fn create(word_list: impl AsRef<str>) -> Result<Self> {
        let word_list = word_list.as_ref();

        let word_len = word_list
            .lines()
            .next()
            .context("Empty word list")?
            .chars()
            .count();

        let words = word_list
            .lines()
            .map(|s| {
                let cnt = s.chars().count();
                if cnt != word_len {
                    bail!("Word length mismatch: expected {word_len}, found {cnt} in word '{s}'");
                }
                Ok(s.to_string())
            })
            .try_collect()
            .context("Can not collect word list")?;

        Ok(Dict { words, word_len })
    }
}

impl Index<u32> for Dict {
    type Output = str;

    #[inline]
    fn index(&self, index: u32) -> &Self::Output {
        self.words[index as usize].as_str()
    }
}

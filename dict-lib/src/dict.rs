use anyhow::{bail, Context, Error, Result};
use itertools::Itertools;
use std::ops::Index;
use std::path::Path;
use std::slice::Iter;

pub struct Dict {
    words: Vec<String>,
}

impl Dict {
    #[inline]
    pub(crate) fn create(words: Vec<String>) -> Self {
        Self { words }
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, String> {
        self.words.iter()
    }

    #[inline]
    pub(crate) fn desctructure(self) -> Vec<String> {
        self.words
    }

    #[inline]
    pub fn create_default() -> Result<Self> {
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../dict/dict.txt"))
            .try_into()
            .context("Can not create default dict")
    }

    #[inline]
    pub fn create_from_file(dict: impl AsRef<Path>) -> Result<Self> {
        let dict = dict.as_ref();
        std::fs::read_to_string(dict)
            .with_context(|| format!("Failed to read file '{}'", dict.display()))?
            .try_into()
            .with_context(|| format!("Can not create dict from file '{}'", dict.display()))
    }
}

impl TryFrom<String> for Dict {
    type Error = Error;

    #[inline]
    fn try_from(word_list: String) -> Result<Self> {
        (&word_list)
            .try_into()
            .context("Can not create dict from string")
    }
}

impl TryFrom<&String> for Dict {
    type Error = Error;

    #[inline]
    fn try_from(word_list: &String) -> Result<Self> {
        word_list
            .as_str()
            .try_into()
            .context("Can not create dict from &String")
    }
}

impl TryFrom<&str> for Dict {
    type Error = Error;

    fn try_from(word_list: &str) -> Result<Self> {
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

        Ok(Dict { words })
    }
}

impl Index<u32> for Dict {
    type Output = str;

    #[inline]
    fn index(&self, index: u32) -> &Self::Output {
        self.words[index as usize].as_str()
    }
}

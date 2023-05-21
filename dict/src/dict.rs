use std::iter::zip;
use std::ops::Index;
use std::path::Path;

pub struct Dict {
    words: Vec<String>,
    word_len: usize,
}

impl Dict {
    pub fn words(&self) -> &Vec<String> {
        &self.words
    }

    pub fn word_len(&self) -> usize {
        self.word_len
    }

    pub fn heuristic(&self, end: impl AsRef<str>, n: u32) -> usize {
        zip(end.as_ref().chars(), self.words[n as usize].chars())
            .filter(|(ch1, ch2)| ch1 != ch2)
            .count()
    }

    pub fn create_from_file(dict: impl AsRef<Path>) -> Self {
        Self::create(std::fs::read_to_string(dict).unwrap())
    }

    pub fn create(word_list: impl AsRef<str>) -> Self {
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

impl Index<u32> for &Dict {
    type Output = str;

    fn index(&self, index: u32) -> &Self::Output {
        self.words()[index as usize].as_str()
    }
}

impl Index<u32> for Dict {
    type Output = str;

    fn index(&self, index: u32) -> &Self::Output {
        self.words()[index as usize].as_str()
    }
}

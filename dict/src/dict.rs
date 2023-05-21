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
    #[inline]
    fn default() -> Self {
        Self::create(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/dict.txt"
        )))
    }
}

impl Index<u32> for Dict {
    type Output = str;

    #[inline]
    fn index(&self, index: u32) -> &Self::Output {
        self.words[index as usize].as_str()
    }
}

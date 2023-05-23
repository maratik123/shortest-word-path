use crate::{maratik, Dict, Neighbours};
use anyhow::{Context, Result};
use prost::bytes::Bytes;
use prost::Message;
use std::io::Write;
use std::path::Path;

const MAGIC: &[u8; 4] = b"swpd";
const VERSION: u8 = 1;

pub fn to_file(file: impl AsRef<Path>, dict: &Dict, neighbours: &Neighbours) -> Result<()> {
    let dict = Some(maratik::shortest_word_path::Dict {
        words: dict.iter().map(|s| s.into()).collect(),
        word_len: dict.word_len() as u64,
    });
    let neighbours = Some(maratik::shortest_word_path::Neighbours {
        edges: neighbours
            .iter()
            .map(|(&k, v)| {
                (
                    k,
                    maratik::shortest_word_path::neighbours::Edges {
                        edges: v.iter().copied().collect(),
                    },
                )
            })
            .collect(),
    });
    let all = maratik::shortest_word_path::All { dict, neighbours };
    let mut buf = Vec::with_capacity(MAGIC.len() + 1 + all.encoded_len());
    buf.write_all(MAGIC).context("Can not write magic")?;
    buf.write_all(&[VERSION]).context("Can not write version")?;
    all.encode(&mut buf).context("Can not encode data")?;
    let file = file.as_ref();
    std::fs::write(file, buf)
        .with_context(|| format!("Can not write to file '{}'", file.display()))?;
    Ok(())
}

pub fn from_file(file: impl AsRef<Path>) -> Result<(Dict, Neighbours)> {
    let file = file.as_ref();
    let all = maratik::shortest_word_path::All::decode(Bytes::from(
        std::fs::read(file)
            .with_context(|| format!("Can not read from file '{}'", file.display()))?,
    ))
    .with_context(|| format!("Can not decode data from file '{}'", file.display()))?;
    let dict = all.dict.context("Dict not found")?;
    let neighbours = all.neighbours.context("Neighbours not found")?;
    let dict = Dict::create_from_proto(dict);
    let neighbours = Neighbours::create_from_proto(neighbours);
    Ok((dict, neighbours))
}

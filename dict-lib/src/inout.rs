use crate::{proto, Dict, Neighbours};
use anyhow::{Context, Result};
use prost::bytes::Bytes;
use prost::Message;
use std::io::Write;
use std::path::Path;

const MAGIC: &[u8; 4] = b"swpd";
const VERSION: u8 = 1;

pub fn to_file(file: impl AsRef<Path>, dict: Dict, neighbours: Neighbours) -> Result<()> {
    let all = proto::All::try_from((dict, neighbours))
        .context("Dict with index can not be serialized")?;
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
    proto::All::decode(Bytes::from(
        std::fs::read(file)
            .with_context(|| format!("Can not read from file '{}'", file.display()))?,
    ))
    .with_context(|| format!("Can not decode data from file '{}'", file.display()))?
    .try_into()
    .with_context(|| {
        format!(
            "Can not create dict with index from file '{}'",
            file.display()
        )
    })
}

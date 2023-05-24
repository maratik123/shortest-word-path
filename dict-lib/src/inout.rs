use crate::{proto, Dict, Neighbours};
use anyhow::{bail, Context, Result};
use prost::Message;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::mem::size_of;
use std::path::Path;

const MAGIC: &[u8; 4] = b"swpd";
const VERSION: u8 = 1;

pub fn to_file(file: impl AsRef<Path>, dict: Dict, neighbours: Neighbours) -> Result<()> {
    let file = file.as_ref();
    let write = BufWriter::new(
        File::create(file).with_context(|| format!("Can not create file {}", file.display()))?,
    );
    to_write(write, dict, neighbours)
        .with_context(|| format!("Can not write to file '{}'", file.display()))
}

pub fn to_write(mut write: impl Write, dict: Dict, neighbours: Neighbours) -> Result<()> {
    let all = proto::All::try_from((dict, neighbours))
        .context("Dict with index can not be serialized")?;
    let all_encoded_len = all.encoded_len();
    write.write_all(MAGIC).context("Can not write magic")?;
    write
        .write_all(&[VERSION])
        .context("Can not write version")?;
    write
        .write_all(
            &u32::try_from(all_encoded_len)
                .with_context(|| {
                    format!(
                        "Encoded size {all_encoded_len} does not fit to {:?}",
                        u32::MIN..=u32::MAX
                    )
                })?
                .to_le_bytes(),
        )
        .context("Can not write encoded len")?;
    let mut buf = vec![0; all_encoded_len];
    all.encode(&mut buf)
        .context("Can not encode data to protobuf")?;
    write.write_all(&buf).context("Can not write buf to file")
}

pub fn from_file(file: impl AsRef<Path>) -> Result<(Dict, Neighbours)> {
    let file_path = file.as_ref();
    let file = BufReader::new(
        File::open(file_path)
            .with_context(|| format!("Can not open file '{}'", file_path.display()))?,
    );
    from_read(file).with_context(|| format!("Can not read from file '{}'", file_path.display()))
}

pub fn from_read(mut read: impl Read) -> Result<(Dict, Neighbours)> {
    let mut maybe_magic = [0; MAGIC.len()];
    read.read_exact(&mut maybe_magic)
        .context("Can not read magic")?;
    if &maybe_magic != MAGIC {
        bail!("Invalid magic: {maybe_magic:?}");
    }
    let mut maybe_version = [0; 1];
    read.read_exact(&mut maybe_version)
        .context("Can not read version")?;
    if maybe_version[0] != VERSION {
        bail!("Invalid version: {}", maybe_version[0]);
    }
    let mut size_le = [0; size_of::<u32>()];
    read.read_exact(&mut size_le).context("Can not read size")?;
    let all_size = u32::from_le_bytes(size_le);
    let mut data = vec![0; all_size as usize];
    read.read_exact(&mut data).context("Can not read data")?;
    proto::All::decode(&data[..])
        .context("Can not decode data")?
        .try_into()
        .context("Can not create dict with index")
}

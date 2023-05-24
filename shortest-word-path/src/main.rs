use anyhow::{Context, Result};
use clap::Parser;
use dict_lib::{Dict, Index, Neighbours};
use log::{debug, LevelFilter};
use shortest_word_path::a_star;
use simple_logger::SimpleLogger;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version)]
#[command(
about = "Find shortest path",
long_about = None
)]
struct Cli {
    /// Sets a custom dict file
    #[arg(short, long, value_name = "FILE")]
    dict: Option<PathBuf>,
    /// Begin word
    word_begin: Option<String>,
    /// End word
    word_end: Option<String>,
}

fn main() {
    if let Err(err) = try_main() {
        eprintln!("ERROR: {}", err);
        err.chain()
            .skip(1)
            .for_each(|cause| eprintln!("because: {}", cause));
        std::process::exit(1);
    }
}

fn try_main() -> Result<()> {
    SimpleLogger::new()
        .with_level(LevelFilter::Error)
        .with_colors(true)
        .env()
        .init()
        .context("Failed to init logger")?;

    let cli = match Cli::try_parse() {
        Err(err) if !err.use_stderr() => {
            err.print().context("Can not print help")?;
            return Ok(());
        }
        cli => cli.context("Error on parsing params")?,
    };
    let (begin, end) = (cli.word_begin, cli.word_end);

    let dict = if let Some(dict) = cli.dict {
        Dict::create_from_file(&dict)
            .with_context(|| format!("Can not create dict from file '{}'", dict.display()))
    } else {
        Dict::create_default().context("Can not create default dict")
    }?;
    let index = Index::from(&dict);
    let neighbours = Neighbours::try_from(&dict).context("Can not create neighbours map")?;

    if begin.is_none() && end.is_none() {
        debug!("No words defined, dump dict");
        for (wi, word) in dict.iter().enumerate() {
            print!("{word}: ");
            if let Some(neighbours_i) = neighbours.get(wi as _) {
                for &neighbour_i in neighbours_i {
                    print!("{} ", &dict[neighbour_i]);
                }
            }
            println!();
        }
        return Ok(());
    }

    let (begin, end) = (
        begin.context("Begin word not defined")?,
        end.context("End word not defined")?,
    );
    let begin_i = index
        .get(&begin[..])
        .with_context(|| format!("Can not found begin word: {begin}"))?;
    let end_i = index
        .get(&end[..])
        .with_context(|| format!("Can not found end word: {end}"))?;

    for word in a_star(&neighbours, &dict, begin_i, end_i)
        .with_context(|| format!("Path from '{begin}' to '{end}' does not exist"))?
        .into_iter()
        .rev()
        .map(|i| &dict[i])
    {
        print!("{word} ");
    }
    println!();
    Ok(())
}

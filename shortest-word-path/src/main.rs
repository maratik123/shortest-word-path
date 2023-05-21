use clap::Parser;
use dict::{Dict, Index, Neighbours};
use shortest_word_path::a_star;
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
    let cli = Cli::parse();
    let (begin, end) = (cli.word_begin, cli.word_end);

    let dict = if let Some(dict) = cli.dict {
        Dict::create_from_file(dict)
    } else {
        Dict::default()
    };
    let index = Index::from(&dict);
    let neighbours = Neighbours::from(&dict);

    if begin.is_none() && end.is_none() {
        for (wi, word) in dict.iter().enumerate() {
            let wi = wi as u32;
            print!("{word}: ");
            if let Some(neighbours_i) = neighbours.get(wi) {
                for &neighbour_i in neighbours_i {
                    print!("{} ", &dict[neighbour_i]);
                }
            }
            println!();
        }
        return;
    }

    let (begin, end) = (
        begin.expect("Begin word not defined"),
        end.expect("End word not defined"),
    );
    let begin_i = index
        .get(&begin[..])
        .ok_or_else(|| format!("Can not found begin word: {begin}"))
        .unwrap();
    let end_i = index
        .get(&end[..])
        .ok_or_else(|| format!("Can not found end word: {end}"))
        .unwrap();

    for word in a_star(&neighbours, &dict, begin_i, end_i)
        .unwrap()
        .into_iter()
        .rev()
        .map(|i| &dict[i])
    {
        print!("{word} ");
    }
    println!()
}

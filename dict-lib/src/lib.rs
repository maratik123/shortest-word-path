mod dict;
mod index;
mod neighbours;
mod inout;

pub use dict::Dict;
pub use index::Index;
pub use neighbours::Neighbours;

pub(crate) mod maratik {
    pub(crate) mod shortest_word_path {
        include!(concat!(env!("OUT_DIR"), "/maratik.shortest_word_path.rs"));
    }
}

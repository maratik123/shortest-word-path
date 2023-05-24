use anyhow::{Context, Error, Result};

include!(concat!(env!("OUT_DIR"), "/maratik.shortest_word_path.rs"));

impl From<Dict> for crate::Dict {
    #[inline]
    fn from(dict: Dict) -> Self {
        Self::create(dict.words)
    }
}

impl TryFrom<crate::Dict> for Dict {
    type Error = Error;

    #[inline]
    fn try_from(dict: crate::Dict) -> Result<Self> {
        Ok(Self {
            words: dict.desctructure(),
        })
    }
}

impl From<Neighbours> for crate::Neighbours {
    fn from(neighbours: Neighbours) -> Self {
        Self::create(
            neighbours
                .edges
                .into_iter()
                .map(|(k, v)| (k, v.edges.into_iter().collect()))
                .collect(),
        )
    }
}

impl From<crate::Neighbours> for Neighbours {
    fn from(neighbours: crate::Neighbours) -> Self {
        Self {
            edges: neighbours
                .destructure()
                .into_iter()
                .map(|(k, v)| {
                    (
                        k,
                        neighbours::Edges {
                            edges: v.into_iter().collect(),
                        },
                    )
                })
                .collect(),
        }
    }
}

impl TryFrom<(crate::Dict, crate::Neighbours)> for All {
    type Error = Error;

    #[inline]
    fn try_from((dict, neighbours): (crate::Dict, crate::Neighbours)) -> Result<Self> {
        Ok(Self {
            dict: Some(dict.try_into().context("Can not create dict")?),
            neighbours: Some(neighbours.into()),
        })
    }
}

impl TryFrom<All> for (crate::Dict, crate::Neighbours) {
    type Error = Error;

    #[inline]
    fn try_from(all: All) -> Result<Self> {
        Ok((
            all.dict.context("Dict not found")?.into(),
            all.neighbours.context("Neighbours not found")?.into(),
        ))
    }
}

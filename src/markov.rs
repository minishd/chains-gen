use std::{
    hash::Hash,
    sync::{Arc, OnceLock},
};

use rand::{distributions::WeightedIndex, prelude::Distribution, rngs::ThreadRng};

type FnvDashMap<K, V> = dashmap::DashMap<K, V, fnv::FnvBuildHasher>;
type FnvIndexMap<K, V> = indexmap::IndexMap<K, V, fnv::FnvBuildHasher>;

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum MarkovToken {
    Root,
    Value(String),
    End,
}

impl MarkovToken {
    pub fn is_root(&self) -> bool {
        matches!(self, Self::Root)
    }

    pub fn is_end(&self) -> bool {
        matches!(self, Self::End)
    }
}

pub struct MarkovNode {
    pub value: MarkovToken,
    pub conns: MarkovConns,
}

impl std::fmt::Debug for MarkovNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.value.fmt(f)
    }
}
impl PartialEq for MarkovNode {
    fn eq(&self, other: &Self) -> bool {
        self.value.eq(&other.value)
    }
}
impl Eq for MarkovNode {}
impl Hash for MarkovNode {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl MarkovNode {
    pub fn new(token: MarkovToken) -> Arc<Self> {
        let conns = MarkovConns {
            conns: FnvDashMap::default(),
            im_cached: OnceLock::new(),
        };

        Arc::new(Self {
            value: token,
            conns,
        })
    }
}

pub struct MarkovConns {
    conns: FnvDashMap<Arc<MarkovNode>, u32>,
    im_cached: OnceLock<FnvIndexMap<Arc<MarkovNode>, u32>>,
}

impl MarkovConns {
    pub fn connect(&self, n: Arc<MarkovNode>) {
        self.conns.entry(n).and_modify(|c| *c += 1).or_insert(1);
    }

    pub fn index_map(&self) -> &FnvIndexMap<Arc<MarkovNode>, u32> {
        self.im_cached.get_or_init(|| {
            self.conns
                .iter()
                .map(|r| (r.key().clone(), *r.value()))
                .collect::<FnvIndexMap<_, _>>()
        })
    }

    pub fn random_weighted(&self, rng: &mut ThreadRng) -> Arc<MarkovNode> {
        let im = self.index_map();

        let wi = WeightedIndex::new(im.iter().map(|(_, c)| c).copied()).unwrap(); // SAFETY: there should always be at least an end token
        let i = wi.sample(rng);

        im.get_index(i).map(|(n, _)| n.clone()).unwrap()
    }
}

pub struct MarkovAllNodes(FnvDashMap<String, Arc<MarkovNode>>);

impl MarkovAllNodes {
    pub fn new() -> Self {
        Self(FnvDashMap::default())
    }

    pub fn node(&self, word: &str) -> Arc<MarkovNode> {
        self.0
            .entry(word.to_string())
            .or_insert_with(|| MarkovNode::new(MarkovToken::Value(word.to_string())))
            .value()
            .clone()
    }

    pub fn try_node(&self, word: &str) -> Option<Arc<MarkovNode>> {
        self.0.get(word).map(|r| r.clone())
    }

    pub fn cache_index_maps(&self) {
        self.0.iter().for_each(|e| {
            e.value().conns.index_map();
        });
    }
}

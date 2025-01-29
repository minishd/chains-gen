use std::{cell::RefCell, collections::HashMap, hash::Hash, rc::Rc};

use indexmap::IndexMap;
use rand::{distributions::WeightedIndex, prelude::Distribution, rngs::ThreadRng};

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
    pub conns: RefCell<MarkovConns>,
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
    pub fn new(token: MarkovToken) -> Rc<Self> {
        Rc::new(Self {
            value: token,
            conns: RefCell::new(MarkovConns(IndexMap::new())),
        })
    }
}

pub struct MarkovConns(pub IndexMap<Rc<MarkovNode>, u32>);

impl MarkovConns {
    pub fn connect(&mut self, word: Rc<MarkovNode>) {
        if let Some(count) = self.0.get_mut(&word) {
            *count += 1;
        } else {
            self.0.insert(word, 1);
        }
    }

    fn index(&self, i: usize) -> Rc<MarkovNode> {
        self.0.get_index(i).map(|(n, _)| n.clone()).unwrap()
    }

    pub fn random_weighted(&self, rng: &mut ThreadRng) -> Rc<MarkovNode> {
        let wi = WeightedIndex::new(self.0.iter().map(|(_, c)| c)).unwrap(); // SAFETY: there should always be at least an end token
        let i = wi.sample(rng);

        self.index(i)
    }
}

pub struct MarkovAllNodes(HashMap<String, Rc<MarkovNode>>);

impl MarkovAllNodes {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn node(&mut self, word: &str) -> Rc<MarkovNode> {
        if let Some(node) = self.0.get(word) {
            node.clone()
        } else {
            let node = MarkovNode::new(MarkovToken::Value(word.to_string()));
            self.0.insert(word.to_string(), node.clone());
            node
        }
    }

    pub fn try_node(&self, word: &str) -> Option<Rc<MarkovNode>> {
        self.0.get(word).cloned()
    }
}

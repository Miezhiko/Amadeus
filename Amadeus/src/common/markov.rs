//! original source: https://raw.githubusercontent.com/aatxe/markov/stable/src/lib.rs

use std::borrow::ToOwned;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;
use std::fs::File;
use std::hash::Hash;
use std::io::prelude::*;
use std::io::Result;
use std::io::{Error, ErrorKind};
use std::path::Path;

use rand::{thread_rng, Rng};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_yaml as yaml;

/// The definition of all types that can be used in a `Chain`.
pub trait Chainable: Eq + Hash + Clone {}
impl<T> Chainable for T where T: Eq + Hash + Clone {}

type Token<T> = Option<T>;

/// A generic [Markov chain](https://en.wikipedia.org/wiki/Markov_chain) for almost any type.
/// In particular, elements of the chain must be `Eq`, `Hash`, and `Clone`.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Chain<T>
where
  T: Chainable,
{
  map: HashMap<Vec<Token<T>>, HashMap<Token<T>, usize>>,
  order: usize,
}

impl<T> Default for Chain<T>
where
  T: Chainable,
{
  fn default() -> Self {
    Self::new()
  }
}

impl<T> Chain<T>
where
  T: Chainable,
{
  /// Constructs a new Markov chain.
  pub fn new() -> Chain<T> {
    Self::of_order(1)
  }

  /// Creates a new Markov chain of the specified order. The order is the number of previous
  /// tokens to use for each mapping in the chain. Higher orders mean that the generated text
  /// will more closely resemble the training set. Increasing the order can yield more realistic
  /// output, but typically at the cost of requiring more training data.
  pub fn of_order(order: usize) -> Chain<T> {
    assert!(order != 0);
    Chain {
      map: {
        let mut map = HashMap::new();
        map.insert(vec![None; order], HashMap::new());
        map
      },
      order,
    }
  }

  /// Determines whether or not the chain is empty. A chain is considered empty if nothing has
  /// been fed into it.
  pub fn is_empty(&self) -> bool {
    self.map[&vec![None; self.order]].is_empty()
  }

  /// Feeds the chain a collection of tokens. This operation is `O(n)` where `n` is the number of
  /// tokens to be fed into the chain.
  pub fn feed<S: AsRef<[T]>>(&mut self, tokens: S) -> &mut Chain<T> {
    let tokens = tokens.as_ref();
    if tokens.is_empty() {
      return self;
    }
    let mut toks = vec![None; self.order];
    toks.extend(tokens.iter().map(|token| Some(token.clone())));
    toks.push(None);
    for p in toks.windows(self.order + 1) {
      self.map
        .entry(p[0..self.order].to_vec())
        .or_default();
      self.map
        .get_mut(&p[0..self.order])
        .unwrap()
        .add(p[self.order].clone(), 1);
    }
    self
  }

  pub fn generate(&self) -> Vec<T> {
    let mut ret = Vec::new();
    let mut curs = vec![None; self.order];
    loop {
      let next = self.map[&curs].next();
      curs = curs[1..self.order].to_vec();
      curs.push(next.clone());
      if let Some(next) = next {
        ret.push(next)
      };
      if curs[self.order - 1].is_none() {
        break;
      }
    }
    ret
  }
}

impl<T> Chain<T>
where
  T: Chainable + Serialize,
{
  /// Saves the current chain to the specified path.
  pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
    let mut file = File::create(&path)?;
    let data = yaml::to_string(self).map_err(|e| Error::new(ErrorKind::InvalidData, e))?;
    file.write_all(data.as_bytes())?;
    Ok(())
  }
}

impl<T> Chain<T>
where
  T: Chainable + DeserializeOwned,
{
  /// Loads a chain from the specified path.
  pub fn load<P: AsRef<Path>>(path: P) -> Result<Chain<T>> {
    let mut file = File::open(&path)?;
    let mut data = String::new();
    file.read_to_string(&mut data)?;

    yaml::from_str(&data).map_err(|e| Error::new(ErrorKind::InvalidInput, e))
  }
}

impl Chain<String> {
  /// Feeds a string of text into the chain.
  pub fn feed_str(&mut self, string: &str) -> &mut Chain<String> {
    self.feed(string.split(' ').map(|s| s.to_owned()).collect::<Vec<_>>())
  }

  /// Converts the output of `generate(...)` on a String chain to a single String.
  fn vec_to_string(vec: Vec<String>) -> String {
    let mut ret = String::new();
    for s in &vec {
      ret.push_str(s);
      ret.push(' ');
    }
    let len = ret.len();
    if len > 0 {
      ret.truncate(len - 1);
    }
    ret
  }

  /// Generates a random string of text.
  pub fn generate_str(&self) -> String {
    Chain::vec_to_string(self.generate())
  }
}

/// A sized iterator over a Markov chain.
pub struct SizedChainIterator<'a, T: Chainable + 'a> {
  chain: &'a Chain<T>,
  size: usize,
}

impl<'a, T> Iterator for SizedChainIterator<'a, T>
where
  T: Chainable + 'a,
{
  type Item = Vec<T>;
  fn next(&mut self) -> Option<Vec<T>> {
    if self.size > 0 {
      self.size -= 1;
      Some(self.chain.generate())
    } else {
      None
    }
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    (self.size, Some(self.size))
  }
}

/// An infinite iterator over a Markov chain.
pub struct InfiniteChainIterator<'a, T: Chainable + 'a> {
  chain: &'a Chain<T>,
}

impl<'a, T> Iterator for InfiniteChainIterator<'a, T>
where
  T: Chainable + 'a,
{
  type Item = Vec<T>;
  fn next(&mut self) -> Option<Vec<T>> {
    Some(self.chain.generate())
  }
}

/// A collection of states for the Markov chain.
trait States<T: PartialEq> {
  /// Adds a state to this states collection.
  fn add(&mut self, token: Token<T>, count: usize);
  /// Gets the next state from this collection of states.
  fn next(&self) -> Token<T>;
}

impl<T> States<T> for HashMap<Token<T>, usize>
where
  T: Chainable,
{
  fn add(&mut self, token: Token<T>, count: usize) {
    match self.entry(token) {
      Occupied(mut e) => *e.get_mut() += count,
      Vacant(e) => {
        e.insert(count);
      }
    }
  }

  fn next(&self) -> Token<T> {
    let mut sum = 0;
    for &value in self.values() {
      sum += value;
    }
    let mut rng = thread_rng();
    let cap = rng.gen_range(0..sum);
    sum = 0;
    for (key, &value) in self.iter() {
      sum += value;
      if sum > cap {
        return key.clone();
      }
    }
    unreachable!("The random number generator failed.")
  }
}

use std::collections::{BTreeMap, HashMap};
use std::hash::BuildHasherDefault;
use std::mem::replace;

use daglight_protocol_block::{Block, BlockId, Work};
use daglight_protocol_topology::IdHasher;

/// The received blocks waiting to enter a node: for parents that have not arrived yet, or for the
/// node's clock to reach their stamps. A block waiting for the clock is missing to whatever builds
/// on it, so everything enters in causal order.
#[derive(Clone, Debug)]
pub(crate) struct Inbox<Id, W> {
    /// The waiting blocks, by id, with how many parents each still misses.
    held: HashMap<Id, (Block<Id, W>, usize), BuildHasherDefault<IdHasher>>,
    /// The blocks waiting for parents, by the parent they wait on.
    waiting_on: HashMap<Id, Vec<Id>, BuildHasherDefault<IdHasher>>,
    /// The blocks waiting for the node's clock, by the time it must reach.
    early: BTreeMap<u64, Vec<Id>>,
}

/// An empty inbox.
impl<Id, W> Default for Inbox<Id, W> {
    fn default() -> Self {
        Self {
            held: HashMap::default(),
            waiting_on: HashMap::default(),
            early: BTreeMap::new(),
        }
    }
}

impl<Id: BlockId, W: Work> Inbox<Id, W> {
    /// Returns whether the block waits here.
    pub fn holds(&self, id: Id) -> bool {
        self.held.contains_key(&id)
    }

    /// Makes `block` wait until the node's clock reaches its stamp.
    pub fn park(&mut self, block: Block<Id, W>) {
        self.early.entry(block.time).or_default().push(block.id);
        self.held.insert(block.id, (block, 0));
    }

    /// Makes `block` wait on its `absent` parents, or hands it back if none is absent.
    pub fn wait(&mut self, block: Block<Id, W>, absent: Vec<Id>) -> Option<Block<Id, W>> {
        // Nothing to wait for.
        if absent.is_empty() {
            return Some(block);
        }

        // The block is listed under every absent parent, with their count.
        for &p in &absent {
            self.waiting_on.entry(p).or_default().push(block.id);
        }
        self.held.insert(block.id, (block, absent.len()));
        None
    }

    /// Notes that block `id` arrived; returns the waiting blocks whose last missing parent it was.
    pub fn arrived(&mut self, id: Id) -> Vec<Block<Id, W>> {
        // Every block waiting on it misses one parent less; those missing none are released.
        let mut ready = Vec::new();
        for w in self.waiting_on.remove(&id).unwrap_or_default() {
            let (_, missing) = self.held.get_mut(&w).expect("a waiting block");
            *missing -= 1;
            if *missing == 0 {
                ready.push(self.held.remove(&w).expect("a waiting block").0);
            }
        }
        ready
    }

    /// Returns the blocks whose stamps the node's clock has reached at `now`, earliest first.
    pub fn due(&mut self, now: u64) -> Vec<Block<Id, W>> {
        self.take_stamped_until(now)
            .into_iter()
            .map(|id| self.held.remove(&id).expect("an early block").0)
            .collect()
    }

    /// Removes from the blocks waiting for the clock those stamped up to `now` and returns their
    /// ids, earliest first.
    fn take_stamped_until(&mut self, now: u64) -> Vec<Id> {
        // Split at the first stamp after `now`; the later part stays.
        let later = match now.checked_add(1) {
            Some(after) => self.early.split_off(&after),
            None => BTreeMap::new(),
        };
        replace(&mut self.early, later)
            .into_values()
            .flatten()
            .collect()
    }

    /// Iterates over the parents the waiting blocks miss.
    pub fn missing(&self) -> impl Iterator<Item = Id> + '_ {
        self.waiting_on.keys().copied()
    }

    /// Returns the earliest time a block waits for, if any does.
    pub fn next(&self) -> Option<u64> {
        self.early.keys().next().copied()
    }
}

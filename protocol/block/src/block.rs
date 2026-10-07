use std::iter::once;

use crate::{BlockId, Work};

/// A block as it travels: an id, the parent it extends, the parents it folds besides, its work and
/// its time.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block<Id, W> {
    /// The block's hash.
    pub id: Id,
    /// The parent it extends: the heaviest it lists.
    pub selected_parent: Id,
    /// The other parents, whose chains it folds.
    pub folded: Vec<Id>,
    /// The proof of work the block carries.
    pub work: W,
    /// The time its miner stamped on it, in milliseconds.
    pub time: u64,
}

impl<Id: BlockId, W: Work> Block<Id, W> {
    /// Creates a block from its parts.
    pub fn new(id: Id, selected_parent: Id, folded: Vec<Id>, work: W, time: u64) -> Self {
        Self {
            id,
            selected_parent,
            folded,
            work,
            time,
        }
    }

    /// Iterates over all its parents, the selected one first.
    pub fn parents(&self) -> impl Iterator<Item = Id> + '_ {
        once(self.selected_parent).chain(self.folded.iter().copied())
    }
}

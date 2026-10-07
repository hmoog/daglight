use std::sync::Arc;

use daglight_protocol_block::{BlockId, Work};
use daglight_protocol_block_perception::BlockPerception;

use crate::SequencedBlock;

/// A change of sides: the order moved off the chain it followed, and what that chain had sequenced
/// above the fork is no longer sequenced.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reorg<Id, W, N> {
    /// The highest block both chains share: the fork the perception changed sides at.
    pub common: Arc<BlockPerception<Id, W, N>>,
    /// Every block no longer sequenced, as it was sequenced, the last first.
    pub reverted: Vec<SequencedBlock<Id, W, N>>,
}

impl<Id: BlockId, W: Work, N> Reorg<Id, W, N> {
    /// Iterates over the chain blocks reverted, the highest first.
    pub fn chain(&self) -> impl Iterator<Item = &Arc<BlockPerception<Id, W, N>>> {
        self.reverted.iter().filter_map(|s| match s {
            SequencedBlock::Chain(c) => Some(c),
            SequencedBlock::Blue(_) | SequencedBlock::Red(_) => None,
        })
    }
}

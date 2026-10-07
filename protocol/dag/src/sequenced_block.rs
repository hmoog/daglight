use std::sync::Arc;

use daglight_protocol_block::{BlockId, Work};
use daglight_protocol_block_perception::BlockPerception;

/// A block in the order the DAG sequences it along a chain, as its chain block counts it, with the
/// perception every node derives for it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SequencedBlock<Id, W, N> {
    /// A block of the mergeset its chain block credits.
    Blue(Arc<BlockPerception<Id, W, N>>),
    /// A block of the mergeset that joins the chain below the folding threshold of its chain
    /// block's selected parent: past work only.
    Red(Arc<BlockPerception<Id, W, N>>),
    /// The chain block itself, after the blues and reds it sequences.
    Chain(Arc<BlockPerception<Id, W, N>>),
}

impl<Id: BlockId, W: Work, N> SequencedBlock<Id, W, N> {
    /// Returns the block's perception.
    pub fn perception(&self) -> &Arc<BlockPerception<Id, W, N>> {
        match self {
            Self::Blue(m) | Self::Red(m) | Self::Chain(m) => m,
        }
    }

    /// Returns the block's id.
    pub fn id(&self) -> Id {
        self.perception().id
    }
}

use crate::{Reorg, SequencedBlock};

/// A change of the DAG's order as the chosen chain moves from one tip to another, in the order it
/// happens: a followed node reports them as its heaviest tip moves.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Update<Id, W, N> {
    /// The chain changed sides: what the old one had sequenced above the fork is no longer
    /// sequenced.
    Reorg(Reorg<Id, W, N>),
    /// The blocks newly sequenced on the new chain, in order: each chain block's credited
    /// mergeset, then its red one, then the chain block.
    Advance(Vec<SequencedBlock<Id, W, N>>),
}

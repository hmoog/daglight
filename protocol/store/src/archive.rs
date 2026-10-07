use daglight_protocol_block_perception::{BlockPerception, LearnedNetworkPerception};
use daglight_protocol_topology::BlockAddress;

/// Where the perception of blocks that left the recent window is kept.
pub trait Archive<Id, W, N = LearnedNetworkPerception<W>> {
    /// Stores the perception of the block at `address`.
    fn store(&mut self, address: BlockAddress, perception: BlockPerception<Id, W, N>);

    /// Loads the perception of an archived block; panics if it was never stored.
    fn load(&self, address: BlockAddress) -> BlockPerception<Id, W, N>;

    /// Forgets the perception of a pruned block, if stored.
    fn forget(&mut self, address: BlockAddress);
}

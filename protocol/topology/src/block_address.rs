use crate::Height;

/// Where a block lives in a topology, as opposed to its id: its height and its offset among the
/// blocks there. It is internal to the topology and means nothing outside it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlockAddress {
    /// The block's height.
    pub height: Height,
    /// The block's offset among the blocks at its height, in insertion order.
    pub offset: u32,
}

impl BlockAddress {
    /// The genesis's address.
    pub const GENESIS: BlockAddress = BlockAddress {
        height: 0,
        offset: 0,
    };
}

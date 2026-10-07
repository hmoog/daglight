use daglight_protocol_topology::BlockAddress;

/// What a chain block sequences before itself: the blocks of its mergeset it credits, then the red
/// ones, which joined its chain below its selected parent's folding threshold; each part by past
/// work, then by id, so that every block comes after its past.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Sequence {
    /// The credited blocks, in order.
    pub blues: Vec<BlockAddress>,
    /// The red blocks, in order.
    pub reds: Vec<BlockAddress>,
}

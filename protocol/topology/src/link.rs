use crate::BlockAddress;

/// A block's two ways down its selected chain, kept apart from its vertex for fast walks.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Link {
    /// The selected parent; the genesis points to itself.
    pub parent: BlockAddress,
    /// An ancestor spaced so that any chain walk takes logarithmic steps.
    pub jump: BlockAddress,
}

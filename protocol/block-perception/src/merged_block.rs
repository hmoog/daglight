use daglight_protocol_topology::Height;

/// One block of a block's mergeset, as the block merges it: where that block's chain joins the
/// selected parent's chain, the rival it backs there, its work, the work it had missed of the
/// selected parent's past, and whether its own lineage had already closed that fork.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MergedBlock<Id, W> {
    /// The block.
    pub id: Id,
    /// The height where its chain joins the selected parent's chain.
    pub join: Height,
    /// The block on its chain right above the join: the rival it backs.
    pub rival: Id,
    /// Its own work.
    pub work: W,
    /// The work of its chain above the join, itself included: how deep the rival's cone reaches.
    pub depth: W,
    /// The work of the selected parent's past it had not seen.
    pub missed: W,
    /// Whether its own lineage had closed the fork at the join already: its work is settled there.
    pub settled: bool,
}

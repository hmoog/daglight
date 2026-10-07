use crate::Height;

/// The part of a block's perception the topology keeps for it: what deep blocks are read for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PerceptionSummary<W> {
    /// The block's own work.
    pub work: W,
    /// The block's time, in milliseconds.
    pub time: u64,
    /// The work of its selected chain, the block included.
    pub chain_work: W,
    /// All work in its past, the block included.
    pub past_work: W,
    /// Its folding threshold: the lowest height on its chain whose fork is not closed for good.
    pub folding_threshold: Height,
    /// The finality point on its chain: nothing its children fold may fork off below it.
    pub final_height: Height,
    /// Its blue work, the weight tips are chosen by: acknowledged work plus the counted chain.
    pub blue_work: W,
}

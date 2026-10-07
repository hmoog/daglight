/// What a run records of a block when it is mined.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MinedBlock {
    /// When it was mined, in seconds.
    pub time: f64,
    /// The miner who mined it.
    pub miner: usize,
    /// Its work.
    pub work: u64,
}

use std::fmt::Debug;
use std::hash::Hash;

/// A block hash, ordered for tie-breaks (the lower hash wins).
pub trait BlockId: Copy + Eq + Ord + Hash + Debug {
    /// The id of the genesis block.
    const GENESIS: Self;
}

/// Ids counted in a `u32`, for tests and small simulations.
impl BlockId for u32 {
    const GENESIS: Self = 0;
}

/// Ids counted in a `u64`, as the simulator draws them.
impl BlockId for u64 {
    const GENESIS: Self = 0;
}

/// Ids as 256-bit hashes.
impl BlockId for [u8; 32] {
    const GENESIS: Self = [0; 32];
}

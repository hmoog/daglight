use crate::BlockAddress;

/// A block in the topology: its structure, its place in the lane cover and a payload.
#[derive(Clone, Debug)]
pub struct Vertex<Id, T> {
    /// The block's id.
    pub id: Id,
    /// The parents, the selected parent first; none for the genesis.
    pub parents: Vec<BlockAddress>,
    /// The lane the block extends: a sequence of blocks, each in the past of the next.
    pub lane: u32,
    /// The block's position in its lane.
    pub position: u32,
    /// For each lane, how many of its blocks are the block or in its past; a prefix of the lane.
    pub reach: Box<[u32]>,
    /// What the topology keeps about the block besides its structure.
    pub data: T,
}

impl<Id, T> Vertex<Id, T> {
    /// Returns how many blocks of `lane` are the block or in its past.
    pub fn reach(&self, lane: u32) -> u32 {
        self.reach.get(lane as usize).copied().unwrap_or(0)
    }

    /// Returns whether `other` is this block or in its past.
    pub fn sees(&self, other: &Self) -> bool {
        other.position < self.reach(other.lane)
    }
}

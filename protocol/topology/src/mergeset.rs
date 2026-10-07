use crate::{BlockAddress, Vertex};

/// The blocks some folded blocks add to the past of a selected parent: in every lane a run of
/// consecutive blocks, since the past of a block meets every lane in a prefix.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Mergeset {
    /// Per lane it meets: the lane, the position of its first block there, and its blocks there in
    /// lane order.
    lanes: Vec<(u32, u32, Vec<BlockAddress>)>,
    /// Its blocks in address order.
    addresses: Vec<BlockAddress>,
}

impl Mergeset {
    /// Returns the mergeset of the given runs: per lane, the lane, the first block's position and
    /// the blocks.
    pub(crate) fn of(lanes: Vec<(u32, u32, Vec<BlockAddress>)>) -> Self {
        let mut addresses: Vec<BlockAddress> =
            lanes.iter().flat_map(|(_, _, run)| run).copied().collect();
        addresses.sort_unstable();
        Self { lanes, addresses }
    }

    /// Returns its blocks in address order.
    pub fn addresses(&self) -> &[BlockAddress] {
        &self.addresses
    }

    /// Iterates over its blocks lane by lane, each lane's in lane order.
    pub fn lanes(&self) -> impl Iterator<Item = &[BlockAddress]> {
        self.lanes.iter().map(|(_, _, run)| run.as_slice())
    }

    /// Iterates, lane by lane as `lanes` does, over how many of its blocks there are `vertex` or
    /// lie in its past: those below the vertex's reach.
    pub fn within<'a, Id, T>(
        &'a self,
        vertex: &'a Vertex<Id, T>,
    ) -> impl Iterator<Item = usize> + 'a {
        // The vertex's reach into a lane, clamped to the run, counts the run's blocks it sees.
        self.lanes.iter().map(move |&(lane, first, ref run)| {
            let reach = vertex.reach(lane).clamp(first, first + run.len() as u32);
            (reach - first) as usize
        })
    }
}

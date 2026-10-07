use std::iter::successors;

use daglight_protocol_block::BlockId;

use crate::{BlockAddress, Height, Topology, Vertex};

/// A block's selected chain, from the genesis up to the block, read by height or walked down.
pub struct Chain<'a, Id, T> {
    /// The topology the chain lives in.
    topology: &'a Topology<Id, T>,
    /// The chain's top block.
    top: BlockAddress,
}

impl<'a, Id: BlockId, T> Chain<'a, Id, T> {
    /// Returns the chain of `top`, `top` included.
    pub(crate) fn new(topology: &'a Topology<Id, T>, top: BlockAddress) -> Self {
        Self { topology, top }
    }

    /// Returns the height of the top.
    pub fn height(&self) -> Height {
        self.top.height
    }

    /// Returns the chain block at height `h`.
    pub fn vertex(&self, h: Height) -> &'a Vertex<Id, T> {
        self.topology.vertex(self.topology.ancestor(self.top, h))
    }

    /// Returns what the topology keeps about the chain block at height `h`.
    pub fn at(&self, h: Height) -> &'a T {
        &self.vertex(h).data
    }

    /// Iterates over the chain's blocks, from the top down to the genesis or the lowest one kept.
    pub fn addresses(&self) -> impl Iterator<Item = BlockAddress> + 'a {
        // One selected parent at a time, stopping at the first pruned one.
        let topology = self.topology;
        successors(Some(self.top), move |&s| {
            topology
                .selected_parent(s)
                .filter(|&p| topology.kept(p).is_some())
        })
    }
}

/// A view: copying it copies two references, whatever `T` is.
impl<Id, T> Clone for Chain<'_, Id, T> {
    fn clone(&self) -> Self {
        *self
    }
}

/// A view: copying it copies two references, whatever `T` is.
impl<Id, T> Copy for Chain<'_, Id, T> {}

use std::sync::Arc;

use daglight_protocol_block::{Block, BlockId, Work};
use daglight_protocol_block_perception::{BlockPerception, NetworkPerception};
use daglight_protocol_topology::{BlockAddress, PerceptionSummary, Topology, Vertex};

/// What a DAG is held in: the topology of its blocks and every block's perception, on one kind of
/// block and network, where the rules place the blocks they admit and from which what no one needs
/// any more is pruned. An implementation decides where it keeps them, in memory or on disk.
pub trait DagStore {
    /// The blocks' ids.
    type Id: BlockId;
    /// The blocks' work.
    type Work: Work;
    /// The network perception the blocks carry.
    type Network: NetworkPerception<Self::Work>;

    /// Returns the topology of the DAG's blocks.
    fn topology(&self) -> &Topology<Self::Id, PerceptionSummary<Self::Work>>;

    /// Returns the perception of the block at an address.
    fn perception_at(
        &self,
        address: BlockAddress,
    ) -> Arc<BlockPerception<Self::Id, Self::Work, Self::Network>>;

    /// Places a block admitted on `parents`, the selected one first, with its `perception`, and
    /// returns its address.
    fn place(
        &mut self,
        parents: Vec<BlockAddress>,
        perception: BlockPerception<Self::Id, Self::Work, Self::Network>,
    ) -> BlockAddress;

    /// Prunes every block that is neither one of `points` nor has one in its past: its structure
    /// and its perception. Nothing a node building on its points can need is lost.
    fn prune(&mut self, points: &[BlockAddress]);

    /// Returns the address of a known block; panics otherwise.
    #[track_caller]
    fn address(&self, id: Self::Id) -> BlockAddress {
        self.topology()
            .address(id)
            .unwrap_or_else(|| panic!("unknown block {id:?}"))
    }

    /// Returns the vertex of a known block.
    #[track_caller]
    fn vertex(&self, id: Self::Id) -> &Vertex<Self::Id, PerceptionSummary<Self::Work>> {
        self.topology().vertex(self.address(id))
    }

    /// Returns the perception of a known block.
    #[track_caller]
    fn perception(
        &self,
        id: Self::Id,
    ) -> Arc<BlockPerception<Self::Id, Self::Work, Self::Network>> {
        self.perception_at(self.address(id))
    }

    /// Returns a known block other than the genesis as it is relayed, without its redundant
    /// parents.
    fn block(&self, id: Self::Id) -> Block<Self::Id, Self::Work> {
        // The parents are stored as the rules saw them: the selected one first, the rest by id.
        let topology = self.topology();
        let vertex = self.vertex(id);
        let id_of = |&p: &BlockAddress| topology.vertex(p).id;
        let (s, folded) = vertex.parents.split_first().expect("a selected parent");
        Block::new(
            id,
            id_of(s),
            folded.iter().map(id_of).collect(),
            vertex.data.work,
            vertex.data.time,
        )
    }
}

use std::sync::Arc;

use daglight_protocol_block::{BlockId, Work};
use daglight_protocol_block_perception::{
    BlockPerception, LearnedNetworkPerception, NetworkPerception,
};
use daglight_protocol_dag::DagStore;
use daglight_protocol_parameters::ProtocolParameters;
use daglight_protocol_topology::{BlockAddress, Height, PerceptionSummary, Topology};

use crate::address_map::AddressMap;
use crate::{Archive, MemoryArchive};

/// A DAG's blocks in memory: every block placed once, the perception of the heights above recent
/// blocks' folding thresholds at hand, older perception archived.
#[derive(Clone, Debug)]
pub struct MemoryStore<Id, W, N = LearnedNetworkPerception<W>, A = MemoryArchive<Id, W, N>> {
    /// Every block's structure and topology perception.
    topology: Topology<Id, PerceptionSummary<W>>,
    /// The perception of the heights not yet evicted.
    recent: AddressMap<Arc<BlockPerception<Id, W, N>>>,
    /// The perception of everything older.
    archive: A,
    /// The height below which perception is archived.
    settled: Height,
    /// The lowest folding threshold among the blocks placed since the last eviction.
    lowest: Height,
    /// The blocks placed since the last eviction.
    placed: usize,
}

impl<Id: BlockId, W: Work, N: NetworkPerception<W>> MemoryStore<Id, W, N> {
    /// Creates a store holding only the genesis of `protocol_parameters` and `network`.
    pub fn new(protocol_parameters: ProtocolParameters, network: N) -> Self {
        Self::with_archive(protocol_parameters, network, MemoryArchive::default())
    }
}

impl<Id: BlockId, W: Work, N: NetworkPerception<W>, A: Archive<Id, W, N>> MemoryStore<Id, W, N, A> {
    /// How many placed blocks pass between two evictions to the archive.
    const EVICT_EVERY: usize = 16;

    /// Creates a store holding only the genesis of `protocol_parameters` and `network`, archiving
    /// into `archive`.
    pub fn with_archive(protocol_parameters: ProtocolParameters, network: N, archive: A) -> Self {
        // The genesis is the first block of the topology and the first recent perception.
        let genesis = BlockPerception::genesis(protocol_parameters, network);
        let topology = Topology::new(genesis.id, genesis.summary());
        let mut recent = AddressMap::default();
        recent
            .insert(BlockAddress::GENESIS, Arc::new(genesis))
            .expect("the genesis is recent");
        Self {
            topology,
            recent,
            archive,
            settled: 0,
            lowest: Height::MAX,
            placed: 0,
        }
    }

    /// Returns the archive.
    pub fn archive(&self) -> &A {
        &self.archive
    }

    /// Notes a placed block's `folding_threshold` and, every so many blocks, archives what every
    /// recent block has folded below.
    fn evict(&mut self, folding_threshold: Height) {
        // Note the block; most calls end here.
        self.lowest = self.lowest.min(folding_threshold);
        self.placed += 1;
        if self.placed < Self::EVICT_EVERY {
            return;
        }

        // Full perception is read only for the blocks new ones build on, so what all recent blocks
        // have folded below moves to the archive; a rare deep read loads it back. Heights of
        // different chains do not compare, so the lowest of them counts.
        if self.lowest > self.settled {
            self.settled = self.lowest;
            for (address, perception) in self.recent.evict_below(self.settled) {
                self.archive.store(address, Self::unshare(perception));
            }
        }
        (self.lowest, self.placed) = (Height::MAX, 0);
    }

    /// Returns the perception out of its `Arc`, copying it only if it is still shared.
    fn unshare(perception: Arc<BlockPerception<Id, W, N>>) -> BlockPerception<Id, W, N> {
        Arc::try_unwrap(perception).unwrap_or_else(|m| (*m).clone())
    }
}

/// The DAG's blocks in memory, older perception in the archive; unknown addresses panic.
impl<Id: BlockId, W: Work, N: NetworkPerception<W>, A: Archive<Id, W, N>> DagStore
    for MemoryStore<Id, W, N, A>
{
    type Id = Id;
    type Work = W;
    type Network = N;

    fn topology(&self) -> &Topology<Id, PerceptionSummary<W>> {
        &self.topology
    }

    /// Loads the perception from the archive if it is no longer recent.
    fn perception_at(&self, address: BlockAddress) -> Arc<BlockPerception<Id, W, N>> {
        match self.recent.get(address) {
            Some(m) => Arc::clone(m),
            None => Arc::new(self.archive.load(address)),
        }
    }

    fn place(
        &mut self,
        parents: Vec<BlockAddress>,
        perception: BlockPerception<Id, W, N>,
    ) -> BlockAddress {
        // The topology takes the block's structure and summary; its full perception stays recent
        // unless its height was evicted already, as a late block's can be.
        let folding_threshold = perception.forks.folding_threshold();
        let address = self
            .topology
            .insert(perception.id, parents, perception.summary());
        if let Err(old) = self.recent.insert(address, Arc::new(perception)) {
            self.archive.store(address, Self::unshare(old));
        }
        self.evict(folding_threshold);
        address
    }

    /// Drops the pruned blocks' perception too, recent or archived.
    fn prune(&mut self, points: &[BlockAddress]) {
        for address in self.topology.prune(points) {
            self.recent.remove(address);
            self.archive.forget(address);
        }
    }
}

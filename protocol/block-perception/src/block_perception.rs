use std::cmp::Ordering;

use daglight_protocol_block::{BlockId, Work};
use daglight_protocol_parameters::ProtocolParameters;
use daglight_protocol_topology::{Height, PerceptionSummary};

use crate::{ForksPerception, LearnedNetworkPerception, MergedPerception, NetworkPerception};

/// What a node derives about a block, on network perception `N`, learned unless given; ordered by
/// weight, ties to the lower hash.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockPerception<Id, W, N = LearnedNetworkPerception<W>> {
    /// The block's id.
    pub id: Id,
    /// The block's own work.
    pub work: W,
    /// The block's time, in milliseconds.
    pub time: u64,
    /// The protocol's parameters, inherited from the selected parent.
    pub protocol_parameters: ProtocolParameters,
    /// All work in the block's past, this block included.
    pub past_work: W,
    /// The work of its chain, the block's included.
    pub chain_work: W,
    /// Its blue work, the weight tips are chosen by: its chain with what it acknowledges beside it,
    /// weighed by the parent's network as a side is, a lone chain at a fraction of its own.
    pub blue_work: W,
    /// The block's finality point: the highest chain height stamped at least one finality horizon
    /// before the block. Nothing the block's children fold may fork off the chain below it.
    pub final_height: Height,
    /// What it knows of the forks along its chain.
    pub forks: ForksPerception<Id, W>,
    /// What it knows of the network.
    pub network: N,
}

impl<Id: BlockId, W: Work, N: NetworkPerception<W>> BlockPerception<Id, W, N> {
    /// Creates the genesis perception on `protocol_parameters` and the network it starts: a block
    /// like any other, with one block's work, at time zero.
    pub fn genesis(protocol_parameters: ProtocolParameters, network: N) -> Self {
        let work = network.block_work();
        Self {
            id: Id::GENESIS,
            work,
            time: 0,
            protocol_parameters,
            past_work: work,
            chain_work: work,
            blue_work: network.weigh(work, work),
            final_height: 0,
            forks: ForksPerception::default(),
            network,
        }
    }

    /// Derives a block's perception from its `merge`: its chain from the parent's, its forks from
    /// the parent's and the merged blocks, its weight from both, and the network from what the
    /// forks have closed, for the children. Every part is measured by the parent's network.
    pub fn derive(merge: &MergedPerception<'_, Id, W, N>) -> Self {
        let block = merge.block;
        let forks = ForksPerception::derive(merge);
        let chain_work = merge.parent.chain_work + block.work;
        Self {
            id: block.id,
            work: block.work,
            time: block.time,
            protocol_parameters: merge.parent.protocol_parameters,
            past_work: merge.past_work(),
            chain_work,
            blue_work: merge
                .parent
                .network
                .weigh(chain_work, chain_work + forks.acknowledged()),
            final_height: merge.final_height(),
            network: N::derive(merge, forks.folding_threshold()),
            forks,
        }
    }

    /// Returns what the topology keeps about the block.
    pub fn summary(&self) -> PerceptionSummary<W> {
        PerceptionSummary {
            work: self.work,
            time: self.time,
            chain_work: self.chain_work,
            past_work: self.past_work,
            folding_threshold: self.forks.folding_threshold(),
            final_height: self.final_height,
            blue_work: self.blue_work,
        }
    }
}

/// Heavier by weight, ties to the lower hash.
impl<Id: BlockId, W: Work, N: NetworkPerception<W>> Ord for BlockPerception<Id, W, N> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.blue_work
            .cmp(&other.blue_work)
            .then_with(|| other.id.cmp(&self.id))
    }
}

/// The total order of `Ord`.
impl<Id: BlockId, W: Work, N: NetworkPerception<W>> PartialOrd for BlockPerception<Id, W, N> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

use std::sync::Arc;

use daglight_protocol_block::{Block, BlockId, Work};
use daglight_protocol_topology::{BlockAddress, Chain, Height, PerceptionSummary};

use crate::{BlockPerception, ForksPerception, MergedBlock, NetworkPerception};

/// A block's merged perception: the block, the selected parent it extends, with all the parent
/// knew, the parent's ancestors read by height, and the blocks it merges. Everything a block's
/// perception derives, it derives from it.
pub struct MergedPerception<'a, Id, W, N> {
    /// The block.
    pub block: &'a Block<Id, W>,
    /// Its parents as the rules see them, by address, the selected one first: where it enters the
    /// topology.
    pub parents: Vec<BlockAddress>,
    /// The selected parent: what it knew of its chain, its forks, the network and the protocol.
    pub parent: Arc<BlockPerception<Id, W, N>>,
    /// The parent and its chain, read by height.
    pub ancestors: Chain<'a, Id, PerceptionSummary<W>>,
    /// The blocks it merges, in address order.
    pub merged: Vec<MergedBlock<Id, W>>,
    /// All work in the block's past, before its own: the parent's past and the merged blocks.
    pub seen: W,
    /// The height of the parent's pruning point, its finality point's finality point: nothing
    /// below it is read.
    pub pruning_height: Height,
}

impl<'a, Id: BlockId, W: Work, N: NetworkPerception<W>> MergedPerception<'a, Id, W, N> {
    /// Returns the merged perception of `block`, with `parents`, on `parent`, whose chain is
    /// `ancestors` and whose pruning point is at `pruning_height`, merging `merged`.
    pub fn new(
        block: &'a Block<Id, W>,
        parents: Vec<BlockAddress>,
        parent: Arc<BlockPerception<Id, W, N>>,
        ancestors: Chain<'a, Id, PerceptionSummary<W>>,
        merged: Vec<MergedBlock<Id, W>>,
        pruning_height: Height,
    ) -> Self {
        let seen = merged.iter().fold(parent.past_work, |a, m| a + m.work);
        Self {
            block,
            parents,
            parent,
            ancestors,
            merged,
            seen,
            pruning_height,
        }
    }

    /// Returns all work in the block's past, its own included.
    pub fn past_work(&self) -> W {
        self.seen + self.block.work
    }

    /// Returns the margin the parent knew: the handshake's delays of work.
    pub(crate) fn margin(&self) -> W {
        self.parent.network.margin(&self.parent.protocol_parameters)
    }

    /// Iterates over the merged blocks the block credits: all but those joining the chain below the
    /// parent's folding threshold, which are red and past work only.
    pub fn credited(&self) -> impl Iterator<Item = &MergedBlock<Id, W>> {
        self.merged
            .iter()
            .filter(move |m| m.join >= self.parent.forks.folding_threshold())
    }

    /// Returns the block's finality point: the parent's, moved up the chain to the highest block
    /// stamped at least one finality horizon before the block. Stamps never fall from parent to
    /// child, so it only ever moves up.
    pub(crate) fn final_height(&self) -> Height {
        // Within the first horizon the genesis stays the finality point.
        let mut height = self.parent.final_height;
        let Some(cut) = self
            .block
            .time
            .checked_sub(self.parent.protocol_parameters.finality)
        else {
            return height;
        };

        // Up the chain while the next block is stamped at or before the cut.
        while height < self.ancestors.height() && self.ancestors.at(height + 1).time <= cut {
            height += 1;
        }
        height
    }

    /// Returns the chain's continuation above height `k`, the block right above it, unless `k` is
    /// the parent's own height: there the continuation is the new block, with no work yet.
    pub(crate) fn continuation(&self, k: Height) -> Option<Id> {
        (k < self.ancestors.height()).then(|| self.ancestors.vertex(k + 1).id)
    }

    /// Returns the weight of the chain's side above height `k`, with `rivals_above` recorded above
    /// it: a cone of the parent's chain above `k`, the parent included, and those rivals. The new
    /// block's own work is not on it: it settles nothing.
    pub(crate) fn side(&self, k: Height, rivals_above: W) -> W {
        let chain = self.parent.chain_work - self.ancestors.at(k).chain_work;
        self.parent.network.weigh(chain, chain + rivals_above)
    }

    /// Returns whether the fork at the lowest open height `k` is decided by `forks`: the chain's
    /// side leads the rivals recorded there by the margin, and the work the block has seen since
    /// the chain block at `k` is a majority of the network's.
    pub(crate) fn decided(&self, k: Height, forks: &ForksPerception<Id, W>) -> bool {
        // The side leads the rivals, each weighed in its own cone, by the margin.
        let at = self.ancestors.at(k);
        let rivals = forks
            .get(k)
            .map_or_else(W::zero, |r| r.weighed(&self.parent.network));
        let leads = self.side(k, forks.above(k)) > rivals + self.margin();

        // The work seen since the fork block is a majority of what the network did meanwhile.
        let majority = self.parent.network.majority(
            &self.parent.protocol_parameters,
            self.seen - at.past_work,
            self.block.time - at.time,
        );
        leads && majority
    }
}

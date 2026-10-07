use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::iter::once;
use std::sync::Arc;

use daglight_protocol_block::{BlockId, Work};
use daglight_protocol_block_perception::{BlockPerception, NetworkPerception};
use daglight_protocol_topology::{BlockAddress, IdHasher, PerceptionSummary, Topology};

/// The blocks without children a DAG or a view of it has, with their perceptions; the heaviest of
/// them, the one to build on and follow: the most blue work, ties to the lower hash; and the
/// finality point, below which it never reorganises. Every tip's chain contains the finality point.
/// Never empty.
#[derive(Clone, Debug)]
pub struct Tips<Id, W, N> {
    /// Every tip's perception, by address.
    perceptions:
        HashMap<BlockAddress, Arc<BlockPerception<Id, W, N>>, BuildHasherDefault<IdHasher>>,
    /// The heaviest tip's address.
    heaviest: BlockAddress,
    /// The finality point: the heaviest tip's own, unless an earlier one lies higher.
    final_point: BlockAddress,
}

impl<Id: BlockId, W: Work, N: NetworkPerception<W>> Tips<Id, W, N> {
    /// Creates the tips of a DAG holding only the genesis, with its `perception`.
    pub fn new(perception: Arc<BlockPerception<Id, W, N>>) -> Self {
        let mut perceptions = HashMap::default();
        perceptions.insert(BlockAddress::GENESIS, perception);
        Self {
            perceptions,
            heaviest: BlockAddress::GENESIS,
            final_point: BlockAddress::GENESIS,
        }
    }

    /// Makes the block at `address` in `topology`, with its `perception`, a tip in place of its
    /// parents, unless its chain misses the finality point. Returns the heaviest tip before, if it
    /// changed.
    pub fn insert(
        &mut self,
        perception: Arc<BlockPerception<Id, W, N>>,
        address: BlockAddress,
        topology: &Topology<Id, PerceptionSummary<W>>,
    ) -> Option<BlockAddress> {
        // A block whose chain misses the finality point is never built on or merged.
        if !topology.on_chain(self.final_point, address) {
            return None;
        }

        // The block replaces its parents among the tips.
        let before = self.heaviest;
        let parents = &topology.vertex(address).parents;
        let (lost, heavier) = (
            parents.contains(&before),
            perception > *self.heaviest_perception(),
        );
        for p in parents {
            self.perceptions.remove(p);
        }
        self.perceptions.insert(address, perception);

        // A block outranking the heaviest outranks every tip; only when the heaviest left for a
        // lighter block does finding the next one take a look at every tip.
        if heavier {
            self.heaviest = address;
        } else if lost {
            self.reselect();
        }
        if self.heaviest == before {
            return None;
        }

        // A new heaviest tip moves the finality point up, and the tips whose chains miss it die.
        self.final_point = topology.ancestor(
            self.heaviest,
            topology
                .vertex(self.heaviest)
                .data
                .final_height
                .max(self.final_point.height),
        );
        let (heaviest, final_point) = (self.heaviest, self.final_point);
        self.perceptions
            .retain(|&t, _| t == heaviest || topology.on_chain(final_point, t));
        Some(before)
    }

    /// Forgets every tip for which `keep` fails, the heaviest too, and then finds the heaviest
    /// again; some tip must be kept.
    pub fn retain(&mut self, keep: impl Fn(BlockAddress) -> bool) {
        self.perceptions.retain(|&s, _| keep(s));
        if !self.perceptions.contains_key(&self.heaviest) {
            self.reselect();
        }
    }

    /// Returns the tips' ids, the heaviest first, then the others by id.
    pub fn by_weight(&self) -> Vec<Id> {
        once(self.heaviest)
            .chain(self.others(self.heaviest))
            .map(|t| self.perceptions[&t].id)
            .collect()
    }

    /// Returns every tip but `first`, by id.
    pub fn others(&self, first: BlockAddress) -> Vec<BlockAddress> {
        // Sorted by id, so that nothing depends on the map's order.
        let mut others: Vec<(Id, BlockAddress)> = self
            .perceptions
            .iter()
            .filter(|&(&s, _)| s != first)
            .map(|(&s, p)| (p.id, s))
            .collect();
        others.sort_unstable();
        others.into_iter().map(|(_, s)| s).collect()
    }

    /// Returns the heaviest tip.
    pub fn heaviest(&self) -> BlockAddress {
        self.heaviest
    }

    /// Returns the heaviest tip's perception.
    pub fn heaviest_perception(&self) -> &Arc<BlockPerception<Id, W, N>> {
        &self.perceptions[&self.heaviest]
    }

    /// Returns the finality point.
    pub fn final_point(&self) -> BlockAddress {
        self.final_point
    }

    /// Finds the heaviest tip by looking at every tip.
    fn reselect(&mut self) {
        let (&heaviest, _) = self
            .perceptions
            .iter()
            .max_by(|a, b| a.1.cmp(b.1))
            .expect("there is always a tip");
        self.heaviest = heaviest;
    }
}

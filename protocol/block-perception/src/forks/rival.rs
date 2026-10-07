use daglight_protocol_block::{BlockId, Work};

use crate::NetworkPerception;

/// The work recorded behind one rival child of a fork.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rival<W> {
    /// Work from lineages that still have the fork open; foldable once the chain's side leads.
    pub contested: W,
    /// Work from lineages that have folded the fork their way; evidence only, never foldable.
    pub settled: W,
    /// The deepest chain work above the fork among its blocks: the chain of its cone.
    pub chain: W,
}

impl<W: Work> Rival<W> {
    /// Returns all work behind the rival, contested and settled.
    pub fn weight(&self) -> W {
        self.contested + self.settled
    }

    /// Returns its weight as `network` weighs its cone.
    pub fn weighed<N: NetworkPerception<W>>(&self, network: &N) -> W {
        let weight = self.weight();
        network.weigh(self.chain.min(weight), weight)
    }

    /// Returns whether a side of weight `side`, its chain continuing with `continuation`, leads
    /// this rival, the child `child`: it is at least as heavy, a tie going to the lower hash of the
    /// continuation; without one (the new block itself, with no work yet) the tie is lost.
    pub fn led_by<Id: BlockId, N: NetworkPerception<W>>(
        &self,
        child: Id,
        side: W,
        continuation: Option<Id>,
        network: &N,
    ) -> bool {
        let weight = self.weighed(network);
        weight < side || (weight == side && continuation.is_some_and(|x| x < child))
    }
}

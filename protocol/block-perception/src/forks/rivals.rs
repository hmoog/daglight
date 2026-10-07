use std::collections::BTreeMap;

use daglight_protocol_block::{BlockId, Work};

use crate::{NetworkPerception, Rival};

/// The rivals at one open fork of the chain: the work recorded behind each rival child of the
/// fork block, with the totals and the bound the verdict reads.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rivals<Id, W> {
    /// The rival children, by id, with the work behind each.
    rivals: BTreeMap<Id, Rival<W>>,
    /// All work behind all rivals.
    weight: W,
    /// The contested work behind all rivals.
    contested: W,
    /// The weight of the heaviest rival.
    heaviest: W,
    /// The most contested work the lineage has had foldable here: listing a heavier rival stops the
    /// folding but never takes it back.
    held: W,
}

/// No rivals.
impl<Id, W: Work> Default for Rivals<Id, W> {
    fn default() -> Self {
        Self {
            rivals: BTreeMap::new(),
            weight: W::zero(),
            contested: W::zero(),
            heaviest: W::zero(),
            held: W::zero(),
        }
    }
}

impl<Id: BlockId, W: Work> Rivals<Id, W> {
    /// Returns the work behind one rival child, if recorded.
    pub fn rival(&self, child: Id) -> Option<&Rival<W>> {
        self.rivals.get(&child)
    }

    /// Adds `work` behind `child`, as settled or as contested, from a block whose chain reaches
    /// `depth` above the fork.
    pub(crate) fn record(&mut self, child: Id, work: W, settled: bool, depth: W) {
        // The rival's cone deepens and its work grows, settled or contested.
        let r = self.rivals.entry(child).or_default();
        r.chain = r.chain.max(depth);
        if settled {
            r.settled = r.settled + work;
        } else {
            r.contested = r.contested + work;
            self.contested = self.contested + work;
        }

        // Weights only grow, so the heaviest is the one just grown, or the one before.
        self.heaviest = self.heaviest.max(r.weight());
        self.weight = self.weight + work;
    }

    /// Returns the contested work behind all rivals.
    pub fn contested(&self) -> W {
        self.contested
    }

    /// Returns the settled work behind all rivals.
    pub fn settled(&self) -> W {
        self.weight - self.contested
    }

    /// Returns whether a side of weight `side`, its chain continuing with `continuation`, leads
    /// every rival outright: all their work is foldable, and none is settled.
    pub(crate) fn led_outright<N: NetworkPerception<W>>(
        &self,
        side: W,
        continuation: Option<Id>,
        network: &N,
    ) -> bool {
        self.settled() == W::zero()
            && self
                .rivals
                .iter()
                .all(|(&c, r)| r.led_by(c, side, continuation, network))
    }

    /// Returns all work behind all rivals, each weighed by `network` in its own cone.
    pub(crate) fn weighed<N: NetworkPerception<W>>(&self, network: &N) -> W {
        self.rivals
            .values()
            .fold(W::zero(), |a, r| a + r.weighed(network))
    }

    /// Returns the most contested work the lineage has had foldable here.
    pub(crate) fn held(&self) -> W {
        self.held
    }

    /// Holds `work` foldable here, if more than held so far; never more than is contested. Returns
    /// how much more is held.
    pub(crate) fn hold(&mut self, work: W) -> W {
        let raised = work.max(self.held) - self.held;
        self.held = self.held + raised;
        raised
    }

    /// Returns all work behind all rivals.
    pub(crate) fn weight(&self) -> W {
        self.weight
    }

    /// Returns the contested work of every rival that `side` leads, each rival weighed by
    /// `network` in its own cone.
    pub fn foldable<N: NetworkPerception<W>>(
        &self,
        side: W,
        continuation: Option<Id>,
        network: &N,
    ) -> W {
        // Most forks are decided by the bound: a rival weighs no more than all its work.
        if self.heaviest < side {
            return self.contested;
        }

        // Else each rival is weighed on its own.
        self.rivals
            .iter()
            .filter(|(&c, r)| r.led_by(c, side, continuation, network))
            .fold(W::zero(), |a, (_, r)| a + r.contested)
    }
}

use std::fmt::Debug;

use daglight_protocol_block::{BlockId, Work};
use daglight_protocol_parameters::ProtocolParameters;
use daglight_protocol_topology::Height;

use crate::MergedPerception;

/// What the rules know of the network a lineage runs on: the work a block must carry, the rate the
/// network works at, its delay and its width, the yardsticks every margin and weighing is measured
/// in, by the protocol's `ProtocolParameters`. How they come about is the implementation's: learned
/// from the DAG, or fixed by the genesis.
pub trait NetworkPerception<W: Work>: Clone + Debug + PartialEq + Eq {
    /// One, in millionths.
    const ONE: u128 = 1_000_000;

    /// Returns the network a genesis on `protocol_parameters` starts: blocks carrying `block_work`,
    /// a delay of `delay` milliseconds and a width of `width` millionths of a block per chain step.
    fn genesis(
        protocol_parameters: &ProtocolParameters,
        block_work: W,
        delay: u64,
        width: u64,
    ) -> Self;

    /// Returns the work a child block must carry.
    fn block_work(&self) -> W;

    /// Returns the work the network does per interval.
    fn rate(&self) -> W;

    /// Returns the network's delay, in milliseconds.
    fn delay_time(&self, protocol_parameters: &ProtocolParameters) -> u64;

    /// Returns one delay's worth of work.
    fn delay_work(&self) -> W;

    /// Returns the width chains are weighed against, in millionths of a block per chain step.
    fn width(&self) -> u64;

    /// Derives a block's network from its `merge` and its `folding_threshold`, for its children:
    /// the parent's, taught by the blocks the block credits and by the block itself.
    fn derive<Id: BlockId>(
        merge: &MergedPerception<'_, Id, W, Self>,
        folding_threshold: Height,
    ) -> Self {
        // Every credited block samples the delay, and so does the block itself, which missed
        // nothing of its parent's past.
        let mut network = merge.parent.network.clone();
        for m in merge.credited() {
            network.sample(merge, m.missed, m.work);
        }
        network.sample(merge, W::zero(), merge.block.work);

        // Then the block's past teaches what it has folded.
        network.learn(merge, folding_threshold.min(merge.ancestors.height()));
        network
    }

    /// Takes a block of work `work` that had missed `missed` of the work the chain had as a sample
    /// of the delay, for the block of `merge`.
    fn sample<Id: BlockId>(
        &mut self,
        merge: &MergedPerception<'_, Id, W, Self>,
        missed: W,
        work: W,
    );

    /// Learns, for a block's children, what the block's past shows of the network: the block of
    /// `merge`, whose chain is folded for good below `folding_threshold`.
    fn learn<Id: BlockId>(
        &mut self,
        merge: &MergedPerception<'_, Id, W, Self>,
        folding_threshold: Height,
    );

    /// Returns the handshake in work, its steps in delays of it: the lead that closes a fork.
    fn margin(&self, protocol_parameters: &ProtocolParameters) -> W {
        self.delay_work().times(protocol_parameters.handshake_steps)
    }

    /// Returns the work the network is expected to do in `time` milliseconds.
    fn expected(&self, protocol_parameters: &ProtocolParameters, time: u64) -> u128 {
        self.rate().wide() * u128::from(time) / u128::from(protocol_parameters.interval())
    }

    /// Returns whether `seen` work since a chain block stamped `elapsed` milliseconds ago is a
    /// majority: it leads the work expected but not seen by the margin, and by the handshake's
    /// steps in standard deviations of its count of blocks.
    fn majority(&self, protocol_parameters: &ProtocolParameters, seen: W, elapsed: u64) -> bool {
        // The work expected meanwhile, and one standard deviation of its count of blocks, as work.
        let expected = self.expected(protocol_parameters, elapsed);
        let chance = (expected * self.block_work().wide()).isqrt();

        // Seen against not seen, by the margin and the handshake's steps of chance.
        let seen = seen.wide();
        seen > expected.saturating_sub(seen)
            + self.margin(protocol_parameters).wide()
            + chance * u128::from(protocol_parameters.handshake_steps)
    }

    /// Returns the weight of a cone of `cone` work whose deepest chain has `chain` of it: the rest
    /// in full, and the chain in full at the expected width or more, in proportion below it.
    fn weigh(&self, chain: W, cone: W) -> W {
        // A cone as wide as the network is expected to be has `cone / width` of chain; a cone with
        // more chain than that is long rather than wide, as a lineage mined alone is, and only
        // the chain the width accounts for weighs.
        let scaled = cone.wide() * Self::ONE / u128::from(self.width());
        W::narrow(scaled.min(chain.wide())) + (cone - chain)
    }
}

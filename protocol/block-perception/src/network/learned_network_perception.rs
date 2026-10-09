use daglight_protocol_block::{BlockId, Work};
use daglight_protocol_parameters::ProtocolParameters;
use daglight_protocol_topology::Height;

use crate::network::HorizonSum;
use crate::{MergedPerception, NetworkPerception};

/// A network learned from the DAG over the finality horizon: its hash rate at the tip, as the work
/// a block must carry, and, from what the lineage has folded, the pace it turns the delay into
/// work by and the width it weighs by; its delay from merged blocks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LearnedNetworkPerception<W> {
    /// The work a child block must carry: the tip's recent work, per interval.
    block_work: W,
    /// The work added at the tip over the horizon.
    tip_work: HorizonSum<W>,
    /// The past work of the folded chain over the horizon.
    folded_past: HorizonSum<W>,
    /// The chain work of the folded chain over the horizon.
    folded_chain: HorizonSum<W>,
    /// The folded height the sums have taken in.
    fed: Height,
    /// The work the network does per interval, as the lineage has folded it.
    rate: W,
    /// The network's delay, in millionths of an interval: a running median of what merged
    /// blocks had not seen.
    delay: u64,
    /// One delay's worth of work at the folded rate.
    delay_work: W,
    /// The width chains are weighed against: the folded chain's past work over its chain work.
    width: u64,
}

/// Learned: every merged block samples the delay, every fold feeds the rate and the width.
impl<W: Work> NetworkPerception<W> for LearnedNetworkPerception<W> {
    fn genesis(
        protocol_parameters: &ProtocolParameters,
        block_work: W,
        delay: u64,
        width: u64,
    ) -> Self {
        // The delay in intervals; the sums as a steady network of the given width would have them.
        let interval = u128::from(protocol_parameters.interval());
        let delay = u64::try_from(u128::from(delay) * Self::ONE / interval).expect("a delay");
        let past = HorizonSum::steady(block_work, protocol_parameters.finality_intervals());
        let chain = W::narrow(past.work().wide() * Self::ONE / u128::from(width.max(1)));
        Self {
            block_work,
            tip_work: past,
            folded_past: past,
            folded_chain: HorizonSum::new(chain),
            fed: 0,
            rate: block_work,
            delay,
            delay_work: W::narrow(block_work.wide() * u128::from(delay) / Self::ONE),
            width,
        }
    }

    fn block_work(&self) -> W {
        self.block_work
    }

    fn delay_time(&self, protocol_parameters: &ProtocolParameters) -> u64 {
        u64::try_from(
            u128::from(self.delay) * u128::from(protocol_parameters.interval()) / Self::ONE,
        )
        .expect("a delay")
    }

    fn delay_work(&self) -> W {
        self.delay_work
    }

    fn width(&self) -> u64 {
        self.width
    }

    /// Moves the delay towards the running median of the samples: up for a sample above it, down
    /// for one below, by the sampled block's share of the horizon's work.
    fn sample<Id: BlockId>(
        &mut self,
        merge: &MergedPerception<'_, Id, W, Self>,
        missed: W,
        work: W,
    ) {
        // A block that had missed more than the handshake's worth of work was withheld, not
        // delayed: no sample.
        if missed > merge.margin() {
            return;
        }

        // The sample in intervals; a block counts for at most a delay of work.
        let sample = missed.wide() * Self::ONE / self.block_work.wide().max(1);
        let weight = work.wide().min(self.delay_work.wide().max(1));
        let horizon = u128::from(merge.parent.protocol_parameters.finality_intervals())
            * self.block_work.wide().max(1);
        let step = u128::from(self.delay) * weight / horizon;

        // One step towards the sample, never to zero.
        let delay = if sample > u128::from(self.delay) {
            u128::from(self.delay) + step
        } else {
            u128::from(self.delay).saturating_sub(step).max(1)
        };
        self.delay = u64::try_from(delay).expect("a delay");
    }

    /// Sets the block work from the work the tip added over the horizon, and the rate, the width
    /// and the delay work from the chain newly folded up to `folding_threshold`.
    fn learn<Id: BlockId>(
        &mut self,
        merge: &MergedPerception<'_, Id, W, Self>,
        folding_threshold: Height,
    ) {
        // The hash rate at the tip: what the block added to its past since its selected parent.
        let parent = &merge.parent;
        let protocol_parameters = &parent.protocol_parameters;
        self.tip_work.add(
            merge.past_work() - parent.past_work,
            merge.block.time - parent.time,
            protocol_parameters.finality,
        );
        self.block_work = self
            .tip_work
            .per_interval(protocol_parameters.finality_intervals());

        // What the lineage has folded, from at least the pruning point: a lineage learns nothing
        // lower from what it has not folded.
        self.fed = self.fed.max(merge.pruning_height);
        let folding = self.take_in(merge, folding_threshold);

        // The delay work rises with the delay at once and falls only with a fold.
        let now = W::narrow(self.rate.wide() * u128::from(self.delay) / Self::ONE);
        self.delay_work = if folding {
            now
        } else {
            self.delay_work.max(now)
        };
    }
}

impl<W: Work> LearnedNetworkPerception<W> {
    /// Returns the work the network does per interval, as the lineage has folded it.
    pub fn rate(&self) -> W {
        self.rate
    }

    /// Takes in the segment of the chain newly folded by the block of `merge`, from the height last
    /// taken in up to `folding_threshold`: its past and chain work set the rate and the width.
    /// Returns whether there was one.
    fn take_in<Id: BlockId>(
        &mut self,
        merge: &MergedPerception<'_, Id, W, Self>,
        folding_threshold: Height,
    ) -> bool {
        // Nothing new folded: nothing to take in.
        if folding_threshold <= self.fed {
            return false;
        }

        // The segment's past and chain work, over the time it took, go into the sums.
        let protocol_parameters = &merge.parent.protocol_parameters;
        let horizon = protocol_parameters.finality;
        let (start, end) = (
            merge.ancestors.at(self.fed),
            merge.ancestors.at(folding_threshold),
        );
        let time = end.time - start.time;
        self.folded_past
            .add(end.past_work - start.past_work, time, horizon);
        self.folded_chain
            .add(end.chain_work - start.chain_work, time, horizon);
        self.fed = folding_threshold;

        // The rate is the folded past per interval; the width its ratio to the folded chain, at
        // least one block per step.
        self.rate = self
            .folded_past
            .per_interval(protocol_parameters.finality_intervals());
        let width =
            self.folded_past.work().wide() * Self::ONE / self.folded_chain.work().wide().max(1);
        self.width = u64::try_from(width.max(Self::ONE)).expect("a width");
        true
    }
}

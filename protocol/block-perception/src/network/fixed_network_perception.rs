use daglight_protocol_block::{BlockId, Work};
use daglight_protocol_parameters::ProtocolParameters;
use daglight_protocol_topology::Height;

use crate::{MergedPerception, NetworkPerception};

/// A network the genesis describes once and for all: its block work, delay and width never change,
/// whatever the DAG shows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FixedNetworkPerception<W> {
    /// The work every block carries.
    block_work: W,
    /// The delay, in milliseconds.
    delay: u64,
    /// One delay's worth of work.
    delay_work: W,
    /// The width chains are weighed against, in millionths of a block per chain step.
    width: u64,
}

/// Fixed: samples and folds teach it nothing.
impl<W: Work> NetworkPerception<W> for FixedNetworkPerception<W> {
    fn genesis(
        protocol_parameters: &ProtocolParameters,
        block_work: W,
        delay: u64,
        width: u64,
    ) -> Self {
        let interval = u128::from(protocol_parameters.interval());
        Self {
            block_work,
            delay,
            delay_work: W::narrow(block_work.wide() * u128::from(delay) / interval),
            width,
        }
    }

    fn block_work(&self) -> W {
        self.block_work
    }

    fn delay_time(&self, _protocol_parameters: &ProtocolParameters) -> u64 {
        self.delay
    }

    fn delay_work(&self) -> W {
        self.delay_work
    }

    fn width(&self) -> u64 {
        self.width
    }

    fn sample<Id: BlockId>(&mut self, _: &MergedPerception<'_, Id, W, Self>, _missed: W, _work: W) {
    }

    fn learn<Id: BlockId>(
        &mut self,
        _: &MergedPerception<'_, Id, W, Self>,
        _folding_threshold: Height,
    ) {
    }
}

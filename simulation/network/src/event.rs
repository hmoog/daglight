use std::cmp::Ordering;

use daglight_protocol_topology::BlockAddress;

/// A scheduled event of the simulation.
#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    /// Some miner finds a block.
    Mine {
        /// The time it happens, in seconds.
        time: f64,
    },
    /// A block arrives at a miner.
    Deliver {
        /// The time it happens, in seconds.
        time: f64,
        /// The receiving miner.
        miner: usize,
        /// The block, already in the shared store.
        address: BlockAddress,
    },
    /// A miner's clock reaches the stamp of a block it holds back.
    Wake {
        /// The time it happens, in seconds.
        time: f64,
        /// The miner.
        miner: usize,
    },
}

impl Event {
    /// Returns when the event happens.
    pub fn time(&self) -> f64 {
        match self {
            Event::Mine { time } | Event::Deliver { time, .. } | Event::Wake { time, .. } => *time,
        }
    }
}

/// Times are never NaN, so equality is total.
impl Eq for Event {}

/// The total order of `Ord`.
impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Earlier events are greater, so that a max-heap pops them first.
impl Ord for Event {
    fn cmp(&self, other: &Self) -> Ordering {
        other.time().total_cmp(&self.time())
    }
}

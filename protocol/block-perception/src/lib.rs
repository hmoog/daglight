#![forbid(unsafe_code)]
//! The perception a node derives for every block and never transmits: the block's own facts and all
//! work in its past, with what it knows of its own chain, of the forks along it and of the
//! network. Every block's perception forms from its merged perception: the perception of the
//! selected parent it extends, merged with the blocks it merges.

use std::sync::Arc;

mod block_perception;
mod forks;
mod merged_block;
mod merged_perception;
mod network;

pub use block_perception::BlockPerception;
pub use forks::{ForksPerception, Rival, Rivals};
pub use merged_block::MergedBlock;
pub use merged_perception::MergedPerception;
pub use network::{FixedNetworkPerception, LearnedNetworkPerception, NetworkPerception};

/// The number of consecutive heights a chunk of forks spans.
pub(crate) const CHUNK: usize = 16;

/// The rivals of one chunk of consecutive heights, `None` where no fork is open.
pub(crate) type Chunk<Id, W> = [Option<Arc<Rivals<Id, W>>>; CHUNK];

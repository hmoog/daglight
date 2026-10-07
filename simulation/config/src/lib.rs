#![forbid(unsafe_code)]
//! What a simulated network is: its miners, its delays, the attack it faces and how long it runs;
//! and the node every simulated miner runs.

use daglight_protocol_node::{Node, SharedDag};
use daglight_protocol_store::MemoryStore;

mod attack;
mod config;
mod rate_switch;
mod reveal;
mod strategy;

pub use attack::Attack;
pub use config::Config;
pub use rate_switch::RateSwitch;
pub use reveal::Reveal;
pub use strategy::Strategy;

/// A miner's node: its view of the network's one DAG, shared with every other miner.
pub type MinerNode<N> = Node<SharedDag<MemoryStore<u64, u64, N>>>;

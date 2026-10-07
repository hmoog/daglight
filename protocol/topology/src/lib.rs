#![forbid(unsafe_code)]
//! The DAG's structure: blocks by address, their selected chains and their ancestry.

mod block_address;
mod chain;
mod id_hasher;
mod lane;
mod link;
mod mergeset;
mod perception_summary;
mod topology;
mod vertex;

pub use block_address::BlockAddress;
pub use chain::Chain;
pub use id_hasher::IdHasher;
pub use mergeset::Mergeset;
pub use perception_summary::PerceptionSummary;
pub use topology::Topology;
pub use vertex::Vertex;

/// A position on a selected chain, the genesis at `0`.
pub type Height = u64;

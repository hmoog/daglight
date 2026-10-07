//! What a lineage knows of the network it runs on, and how it comes to know it: learned from the
//! DAG, or fixed by the genesis.

mod fixed_network_perception;
mod horizon_sum;
mod learned_network_perception;
mod network_perception;

pub use fixed_network_perception::FixedNetworkPerception;
pub(crate) use horizon_sum::HorizonSum;
pub use learned_network_perception::LearnedNetworkPerception;
pub use network_perception::NetworkPerception;

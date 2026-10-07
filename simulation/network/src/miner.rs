use daglight_protocol_block_perception::NetworkPerception;

use daglight_simulation_config::MinerNode;

/// A miner: its node on the network's shared DAG, with its share of the hash rate.
#[derive(Debug)]
pub struct Miner<N: NetworkPerception<u64>> {
    /// Its share of the total hash rate.
    pub share: f64,
    /// Its node.
    pub node: MinerNode<N>,
}

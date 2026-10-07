//! What the node tests share: the protocol's parameters, a node on a DAG of its own, and blocks
//! mined as the network would.
#![allow(dead_code)]

use daglight_protocol_block::Block;
use daglight_protocol_block_perception::{
    BlockPerception, LearnedNetworkPerception, NetworkPerception,
};
use daglight_protocol_dag::{Dag, DagError};
use daglight_protocol_node::Node;
use daglight_protocol_parameters::ProtocolParameters;
use daglight_protocol_store::MemoryStore;

/// The protocol's parameters: a block a second, and a finality horizon of five minutes.
pub const PROTOCOL_PARAMETERS: ProtocolParameters = ProtocolParameters::DEFAULT
    .with_bps(1)
    .with_finality(300_000);

/// The store the tests keep their DAGs in.
pub type TestStore = MemoryStore<u32, u64>;

/// A node on a DAG of its own, in memory.
pub type TestNode = Node<Dag<TestStore>>;

/// Returns a node holding only a genesis of work 10 that guesses a delay of `delay` milliseconds
/// and a width of `width` millionths of a block per chain step.
pub fn dag(delay: u64, width: u64) -> TestNode {
    TestNode::new(Dag::new(MemoryStore::new(
        PROTOCOL_PARAMETERS,
        LearnedNetworkPerception::genesis(&PROTOCOL_PARAMETERS, 10, delay, width),
    )))
}

/// Returns the perception `d` derives for `block`, or why it is invalid.
pub fn derive(
    d: &TestNode,
    block: &Block<u32, u64>,
) -> Result<BlockPerception<u32, u64>, DagError<u32, u64>> {
    Ok(BlockPerception::derive(&d.dag().merge(block)?))
}

/// Returns block `id` on `parents`, the selected one first, at `time` in milliseconds, with the
/// work its selected parent requires.
pub fn mine_at(d: &TestNode, id: u32, parents: Vec<u32>, time: u64) -> Block<u32, u64> {
    let work = d.perception(parents[0]).network.block_work();
    Block::new(id, parents[0], parents[1..].to_vec(), work, time)
}

/// Returns block `id` on `parents`, the selected one first, as the network mines it: with the work
/// its selected parent requires, stamped when the genesis's hash rate, a block a second, has done
/// the work before it, so that the difficulty holds.
pub fn mine(d: &TestNode, id: u32, parents: Vec<u32>) -> Block<u32, u64> {
    let latest = parents.iter().map(|&p| d.perception(p).time).max();
    let mut block = mine_at(d, id, parents, latest.expect("a parent"));
    if let Ok(m) = derive(d, &block) {
        block.time = (m.past_work - block.work) * 1000 / block.work;
    }
    block
}

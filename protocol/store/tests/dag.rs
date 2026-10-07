//! A DAG used on its own, without a node: it admits blocks, keeps its tips and builds on them.

use daglight_protocol_block::Block;
use daglight_protocol_block_perception::{
    BlockPerception, LearnedNetworkPerception, NetworkPerception,
};
use daglight_protocol_dag::{Dag, DagStore};
use daglight_protocol_parameters::ProtocolParameters;
use daglight_protocol_store::MemoryStore;

/// The protocol's parameters: a block a second, and a finality horizon of five minutes.
const PROTOCOL_PARAMETERS: ProtocolParameters = ProtocolParameters::DEFAULT
    .with_bps(1)
    .with_finality(300_000);

/// Returns a DAG holding only a genesis of work 10.
fn genesis() -> Dag<MemoryStore<u32, u64>> {
    Dag::new(MemoryStore::new(
        PROTOCOL_PARAMETERS,
        LearnedNetworkPerception::genesis(&PROTOCOL_PARAMETERS, 10, 1000, 1_500_000),
    ))
}

/// Returns `block` stamped when the genesis's hash rate, a block a second, has done the work
/// before it.
fn stamped(dag: &Dag<MemoryStore<u32, u64>>, mut block: Block<u32, u64>) -> Block<u32, u64> {
    let store = dag.store();
    block.time = block
        .parents()
        .map(|p| store.perception(p).time)
        .max()
        .unwrap();
    let past_work = BlockPerception::derive(&dag.merge(&block).unwrap()).past_work;
    block.time = (past_work - block.work) * 1000 / block.work;
    block
}

/// Inserting blocks keeps the tips, and the next block builds on the heaviest, folding the rest.
#[test]
fn builds_on_its_own_tips() {
    let mut dag = genesis();
    let work = dag.store().perception(0).network.block_work();
    for id in [1, 2] {
        let block = stamped(&dag, Block::new(id, 0, vec![], work, 0));
        dag.insert(&block).unwrap();
    }
    assert_eq!(dag.tips().by_weight().len(), 2, "two rivals on the genesis");
    let heaviest = dag.tips().heaviest_perception().id;
    assert_eq!(heaviest, 1, "equal blue work: the lower hash");

    let next = stamped(&dag, dag.next_block(3, 0));
    assert_eq!(next.selected_parent, 1);
    assert_eq!(
        next.parents().collect::<Vec<_>>(),
        vec![1, 2],
        "folds the rival"
    );
    let perception = dag.insert(&next).unwrap();
    assert_eq!(dag.tips().by_weight().len(), 1, "one tip left");
    assert_eq!(dag.tips().heaviest_perception(), &perception);
    assert_eq!(dag.insert(&next).unwrap(), perception, "added again");
}

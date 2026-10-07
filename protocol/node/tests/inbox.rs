//! Out-of-order arrival: blocks wait in the node until their parents are there and its clock has
//! reached their stamps.

use daglight_protocol_block::Block;
use daglight_protocol_dag::Admission;

mod common;

use common::*;

/// Blocks that arrive before their parents wait, then enter the node in causal order.
#[test]
fn out_of_order_arrival() {
    let mut node = dag(1000, 1_500_000);
    let entered = |e: Vec<Admission<TestStore>>| -> Vec<u32> {
        e.into_iter()
            .map(|r| r.expect("a valid block").id)
            .collect()
    };
    assert_eq!(
        entered(node.receive(Block::new(3, 2, vec![1], 10, 3000), 3000)),
        vec![]
    );
    assert_eq!(
        entered(node.receive(Block::new(2, 1, vec![], 10, 2000), 3000)),
        vec![]
    );
    let mut missing = node.missing();
    missing.sort();
    assert_eq!(missing, vec![1, 2], "3 waits on 2 and 1, 2 on 1");

    let in_order = entered(node.receive(Block::new(1, 0, vec![], 10, 1000), 3000));
    assert_eq!(in_order, vec![1, 2, 3]);
    assert!(node.missing().is_empty());
    assert_eq!(node.tips(), vec![3]);
    // The redundant parent 1, already in the past of 2, is not kept.
    assert_eq!(node.block(3), Block::new(3, 2, vec![], 10, 3000));
}

/// A block stamped beyond the node's clock waits until the clock gets there, and whatever builds on
/// it waits with it.
#[test]
fn early_blocks_wait_for_the_clock() {
    let mut node = dag(1000, 1_500_000);
    let entered = |e: Vec<Admission<TestStore>>| -> Vec<u32> {
        e.into_iter()
            .map(|r| r.expect("a valid block").id)
            .collect()
    };
    assert_eq!(
        entered(node.receive(Block::new(1, 0, vec![], 10, 5000), 1000)),
        vec![]
    );
    assert_eq!(node.next_due(), Some(5000));
    assert_eq!(
        entered(node.receive(Block::new(2, 1, vec![], 10, 6000), 6000)),
        vec![],
        "2 waits on 1"
    );
    assert_eq!(entered(node.tick(4999)), vec![]);
    assert_eq!(entered(node.tick(5000)), vec![1, 2]);
    assert_eq!(node.next_due(), None);
    assert_eq!(node.tips(), vec![2]);
}

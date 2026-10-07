//! The ways a view refuses a block.

use daglight_protocol_block::Block;
use daglight_protocol_dag::{DagError, DagStore};

mod common;

use common::*;

/// A block added again returns its perception; one with an unknown parent and invalid ones are
/// refused.
#[test]
fn refused_blocks() {
    let mut dag = dag(1000, 1_500_000);
    dag.add_block(Block::new(1, 0, vec![], 10, 1000)).unwrap();
    dag.add_block(Block::new(2, 0, vec![], 10, 1000)).unwrap();
    dag.add_block(Block::new(3, 2, vec![], 10, 2000)).unwrap();

    assert_eq!(
        dag.add_block(Block::new(1, 0, vec![], 10, 1000)).unwrap(),
        dag.perception(1),
        "the block it has already"
    );
    assert_eq!(
        dag.add_block(Block::new(4, 1, vec![9], 10, 3000))
            .unwrap_err(),
        DagError::UnknownParent(9)
    );

    // 3 weighs the genesis, 2 and itself, more than 1, so a block on 1 may not list it.
    assert_eq!(
        dag.add_block(Block::new(4, 1, vec![3], 10, 3000))
            .unwrap_err(),
        DagError::Heavier {
            selected: 1,
            heavier: 3
        }
    );

    // A block must carry the work its selected parent requires, and be stamped no earlier than any
    // parent.
    assert_eq!(
        dag.add_block(Block::new(4, 3, vec![], 11, 3000))
            .unwrap_err(),
        DagError::Work {
            required: 10,
            carried: 11
        }
    );
    assert_eq!(
        dag.add_block(Block::new(4, 3, vec![], 10, 1999))
            .unwrap_err(),
        DagError::Time {
            parent: 2000,
            time: 1999
        }
    );
    assert_eq!(
        dag.dag().store().topology().len(),
        4,
        "nothing refused was added"
    );
}

//! How a block's blue work is made up: its own chain in full, its own block included, and the
//! rival work it acknowledges.
//!
//! Work is in tenths and the delay work is one block, so a fork folds once its side leads by more
//! than four. The width is guessed at one and a half blocks per step, so a lone chain and a lone
//! rival are weighed alike against each other, and a tip is weighed by the same width: a lone
//! chain weighs two thirds of its work, a cone with a third beside its chain all of it.

use daglight_protocol_dag::DagError;

mod common;

use common::*;

/// The ids of the drawing in `build`.
const J: u32 = 0;
const F2: u32 = 4;
const T1: u32 = 5;
const S: u32 = 29;
const T_LAST: u32 = T1 + 23;
/// An id not yet in the DAG.
const NEXT: u32 = 40;

/// Builds a long chain next to a short lineage that has acknowledged more.
///
/// ```text
///        +- T1 -+- T2 -- T3 -- T4 -- ... -- T24      (T4 folds S)
///        |      +- S . . . . . :
///   J ---+- F1 --- F2                                (F2 folds F1' and F1'')
///        +- F1' . . :
///        +- F1'' . .:
/// ```
///
/// Every block has work 1. J=0 F1=1 F1'=2 F1''=3 F2=4 T1..T24=5..28 S=29.
fn build() -> TestNode {
    let mut d = dag(1000, 1_500_000);
    for (id, parents) in [(1, vec![0]), (2, vec![0]), (3, vec![0]), (4, vec![1, 2, 3])] {
        d.add_block(mine(&d, id, parents)).unwrap();
    }
    d.add_block(mine(&d, T1, vec![J])).unwrap();
    d.add_block(mine(&d, S, vec![T1])).unwrap();
    for t in T1 + 1..=T_LAST {
        let parents = if t == T1 + 3 {
            vec![t - 1, S]
        } else {
            vec![t - 1]
        };
        d.add_block(mine(&d, t, parents)).unwrap();
    }
    d
}

/// The drawing's blocks have the acknowledged work and chain it implies.
#[test]
fn the_state_is_as_drawn() {
    let d = build();
    assert_eq!(PROTOCOL_PARAMETERS.handshake_steps, 4);
    let f2 = d.perception(F2);
    assert_eq!(
        f2.forks.acknowledged(),
        20,
        "F2's side at J, F1, ties F1' and F1'', and F1 has the lowest hash"
    );
    assert_eq!(f2.chain_work, 30, "J, F1 and F2");
    assert_eq!(
        f2.blue_work, 50,
        "a cone of 50 allows 33 of chain: all 30, and 20"
    );
    let t = d.perception(T_LAST);
    assert_eq!(
        t.forks.acknowledged(),
        10,
        "S foldable, and nothing else ever acknowledged"
    );
    assert_eq!(
        t.past_work, 260,
        "the genesis, S and twenty-four chain blocks"
    );
    // The threshold trails the tip by six: at two thirds, a lone chain leads the margin of four
    // only from six blocks up.
    assert_eq!(t.forks.folding_threshold(), 18);
    assert_eq!(t.chain_work, 250, "the genesis and T1 to T24 in full");
    assert_eq!(
        t.blue_work, 187,
        "a cone of 260 at the width T's folds taught, 1.47, counts 177 of its chain, and S's 10"
    );
}

/// A longer chain outranks a lineage that has acknowledged more.
#[test]
fn the_chain_chooses_and_lists() {
    let d = build();

    // The chain is selected, 187 against 50.
    assert!(d.perception(T_LAST) > d.perception(F2));
    assert_eq!(d.heaviest(), T_LAST);
    let virt = mine(&d, 0, d.next_block(0, 0).parents().collect());
    assert_eq!(virt.parents().collect::<Vec<_>>(), vec![T_LAST, F2]);

    // The chain closed J long ago, so F's side joins below the threshold and is red: in the past,
    // never recorded or folded. S, folded at T1, is all the chain ever acknowledged.
    let v = &derive(&d, &virt).unwrap();
    assert!(v.forks.is_empty());
    assert_eq!(v.forks.folded(), 10, "S at T1; F's 4 at J are red");
    assert_eq!(
        v.forks.folding_threshold(),
        19,
        "T24 on top closes one more"
    );
    assert_eq!(v.past_work, 310, "the chain's 26, F's 4 and its own");

    // No block on F's side may list the chain's tip.
    assert_eq!(
        derive(&d, &mine(&d, NEXT, vec![F2, T_LAST])).unwrap_err(),
        DagError::Heavier {
            selected: F2,
            heavier: T_LAST
        }
    );
}

/// A child always outranks its parent, and a tip that folds a rival outranks a chain as long that
/// does not.
///
/// ```text
///        +- 1 -- 3 -- 4
///   0 ---+- 2 -- 5 -- 7       (7 folds 6)
///        +- 6 . . . . :
/// ```
#[test]
fn a_child_outranks_its_parent() {
    let mut d = dag(1000, 1_500_000);
    for (id, parents) in [
        (1, vec![0]),
        (2, vec![0]),
        (3, vec![1]),
        (4, vec![3]),
        (5, vec![2]),
        (6, vec![0]),
        (7, vec![5, 6]),
    ] {
        d.add_block(mine(&d, id, parents)).unwrap();
    }
    assert!(d.perception(1) > d.perception(0));
    assert!(d.perception(3) > d.perception(1));
    assert!(d.perception(4) > d.perception(3));
    assert_eq!(
        d.perception(4).blue_work,
        26,
        "a chain of four, alone: two thirds of 40"
    );

    // 7's side at the genesis, 2 and 5, outweighs 6.
    assert_eq!(
        d.perception(7).blue_work,
        43,
        "a cone of 50 allows 33 of its chain of 40, and 6's 10"
    );
    assert_eq!(d.heaviest(), 7);
}

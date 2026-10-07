//! Worked examples of the rules on small DAGs.
//!
//! Work is in tenths: a block, the genesis included, is 1, `10`. The genesis guesses a delay of a
//! fifth of a block interval, so the delay work is 0.2: a fork folds once its side leads by more
//! than 0.8 blocks and the work seen since it leads the work not seen by as much and by four
//! standard deviations of its count, which takes about twenty blocks where a block takes five
//! delays. A chain block counts at once, for the rank and for the sides of the forks below it.
//! Every block is mined as the network would: with the work its selected parent requires, stamped
//! when a block a second has done the work before it, so the difficulty holds at one block. These
//! DAGs are narrow, so the width stays at one block per step and weighing leaves a chain whole. Ids
//! are chosen so that hash ties fall as described. In the drawings solid edges are selected parents
//! and dotted edges folded parents.

use daglight_protocol_block_perception::{NetworkPerception, Rival};
use daglight_protocol_dag::{DagError, SequencedBlock, Update};

mod common;

use common::*;

/// The ids of the natural fork: its common blocks, the F side and the G side by height, a block
/// that meets both and one that folds the G side late.
const A0: u32 = 0;
const C2: u32 = 2;
const E3: u32 = F + 3;
const D3: u32 = G + 3;
const H: u32 = 400;
const K: u32 = 500;
/// The ids of the F and the G side start here, plus the height.
const F: u32 = 100;
const G: u32 = 200;
/// The height both sides reach.
const TOP: u32 = 26;

/// Builds a natural fork at C2 whose sides never meet again, each side a block a second.
///
/// ```text
///                        +- E3 -- F4 -- F5 -- ... -- F26
///   A0 -- B1 -- C2 ------+        :
///                        +- D3 .. : -- G4 -- ... -- G26     (F4 folds D3)
/// ```
///
/// A0 is the genesis, 0; B1=1 C2=2, the F side 100 plus its height, the G side 200 plus its.
fn natural_fork_dag() -> TestNode {
    let mut d = dag(200, 1_000_000);
    d.follow();
    for (id, parent) in [(1, A0), (C2, 1), (E3, C2), (D3, C2)] {
        d.add_block(mine(&d, id, vec![parent])).unwrap();
    }

    // F4 on E3 folds D3: both weigh the genesis alone, and E3 has the lower hash.
    let f4 = mine(&d, F + 4, d.next_block(F + 4, 0).parents().collect());
    assert_eq!(f4.parents().collect::<Vec<_>>(), vec![E3, D3]);
    d.add_block(f4).unwrap();
    for h in 4..=TOP {
        d.add_block(mine(&d, G + h, vec![G + h - 1])).unwrap();
    }
    for h in 5..=TOP {
        d.add_block(mine(&d, F + h, vec![F + h - 1])).unwrap();
    }
    d
}

/// A natural fork that both sides fold their own way.
#[test]
fn natural_fork() {
    let d = natural_fork_dag();

    // F4 records D3 at C2, where its side is E3: 1 against D3's 1, and E3's lower hash wins.
    let f4 = d.perception(F + 4);
    assert_eq!(
        f4.forks.get(2).and_then(|r| r.rival(D3)),
        Some(&Rival {
            contested: 10,
            settled: 0,
            chain: 10
        })
    );
    assert_eq!(f4.forks.held(), 10, "D3 foldable");
    assert_eq!(f4.blue_work(), 60, "the chain A0 to F4, and D3");

    // Every block is a sample of the delay, by its share of the finality horizon's work, a step of
    // 0.006 ms here. B1, C2, E3 and F4 extend their selected parents and missed nothing: four steps
    // down. D3 had missed E3, a whole block, more than the handshake's 0.8: withheld, as far as the
    // delay can tell, and no sample.
    assert_eq!(
        f4.network.delay_time(&PROTOCOL_PARAMETERS),
        199,
        "in milliseconds, 199.98"
    );
    assert_eq!(f4.network.delay_work(), 2);

    // Nothing closes while the work seen does not lead the work not seen by four standard
    // deviations of its count as well: a block every five delays, it takes twenty of them.
    assert_eq!(d.perception(F + 19).forks.folding_threshold(), 0);
    assert_eq!(d.perception(F + 20).forks.folding_threshold(), 1);

    // F22 folds C2 for good: F's chain leads D3 by far more than the margin there.
    let f22 = d.perception(F + 22);
    assert_eq!(f22.forks.folding_threshold(), 3, "C2 closed");
    assert!(f22.forks.is_empty());
    assert_eq!(f22.forks.folded(), 10, "D3 folded for good");
    assert_eq!(f22.network.block_work(), 10, "the difficulty held");

    // The G side, seeing nothing against it and as much work as the network is known to do,
    // folds C2 its own way a block later.
    assert_eq!(d.perception(G + 22).forks.folding_threshold(), 2);
    assert_eq!(d.perception(G + 23).forks.folding_threshold(), 3);
    assert_eq!(d.perception(G + TOP).forks.acknowledged(), 0);

    // H on F26 folds G26: G4 to G26 join at C2, below F26's folding threshold, and are red.
    let h = mine(&d, H, d.next_block(H, 0).parents().collect());
    assert_eq!(h.parents().collect::<Vec<_>>(), vec![F + TOP, G + TOP]);
    let h = derive(&d, &h).unwrap();
    assert_eq!(h.forks.acknowledged(), 10, "only D3");
    assert_eq!(h.past_work, 520, "red work is past work all the same");

    // A block on G26 may not list the heavier F26.
    assert_eq!(
        derive(&d, &mine(&d, H, vec![G + TOP, F + TOP])).unwrap_err(),
        DagError::Heavier {
            selected: G + TOP,
            heavier: F + TOP
        }
    );
}

/// Settled work weighs against folding its fork but is never folded.
///
/// ```text
///                        +- E3 -- F4 -- F5 -- ... -- F21 -- K -- K1 -- ...
///   A0 -- B1 -- C2 ------+                                 :
///                        +- D3 - G4 - G5 - ... - G20 . . . .      (K folds G20)
/// ```
///
/// The genesis knows a block a second, and the F side mines one: F3 at 3 s, F21 at 21 s. The G
/// side mines two: D3 at 3 s, then half a second apart, G20 at 11.5 s. K at 21.5 s and on, a second
/// apart. A0 is the genesis, 0; B1=1 C2=2, the F side 100 plus its height, the G side 200 plus
/// its, K=500 and on.
#[test]
fn settled_work_is_never_folded() {
    let mut d = dag(200, 1_000_000);
    for (id, parent, time) in [(1, A0, 1000), (C2, 1, 2000), (E3, C2, 3000), (D3, C2, 3000)] {
        d.add_block(mine_at(&d, id, vec![parent], time)).unwrap();
    }
    for h in 4..=21 {
        d.add_block(mine_at(&d, F + h, vec![F + h - 1], u64::from(h) * 1000))
            .unwrap();
    }
    for h in 4..=20 {
        let time = 1500 + u64::from(h) * 500;
        d.add_block(mine_at(&d, G + h, vec![G + h - 1], time))
            .unwrap();
    }

    // The G side sees twice the work the network is known to do and closes C2 its own way, once
    // the work it has seen leads by four standard deviations of its count.
    assert_eq!(d.perception(G + 13).forks.folding_threshold(), 0);
    assert_eq!(d.perception(G + 14).forks.folding_threshold(), 3);

    // K on F21 folds G20, stamped 10 s before K, 6 s later than honest blocks come. D3 to G13 are
    // contested; G14 on have closed C2 themselves and are settled. K's side at C2, E3 to F21, is 19
    // blocks against all 18 behind D3, settled included, and leads by more than the margin of 0.8;
    // the work it has seen since C2 is far ahead of the work it has not. It folds C2 at once, and
    // only the contested work with it.
    d.add_block(mine_at(&d, K, vec![F + 21, G + 20], 21_500))
        .unwrap();
    let k = d.perception(K);
    assert!(k.forks.is_empty());
    assert_eq!(
        k.forks.folded(),
        110,
        "G14 to G20 weighed against folding and stay evidence"
    );
}

/// A lone lineage closes nothing on a few blocks, and joins below a chain that has closed.
///
/// ```text
///        +- H1 -- H2 -- H3 -- ... -- H24 -- N     (N folds X5)
///   J ---+  :
///        +- X1 -- X2 -- X3 -- X4 -- X5 . . .      (M on H3 folds X3)
/// ```
///
/// Each lineage mines a block a second. J is the genesis, 0; H1..H24=1..24 X1..X5=31..35 M=40
/// N=41.
#[test]
fn hidden_lineage() {
    const J: u32 = 0;
    const H1: u32 = 1;
    const H3: u32 = 3;
    const H24: u32 = 24;
    const X1: u32 = 31;
    const X3: u32 = 33;
    const X5: u32 = 35;
    const M: u32 = 40;
    const N: u32 = 41;

    let mut d = dag(200, 1_000_000);
    for h in H1..=H24 {
        d.add_block(mine(&d, h, vec![if h == H1 { J } else { h - 1 }]))
            .unwrap();
    }
    for x in X1..=X5 {
        d.add_block(mine(&d, x, vec![if x == X1 { J } else { x - 1 }]))
            .unwrap();
    }
    assert_eq!(
        d.perception(X5).forks.folding_threshold(),
        0,
        "five blocks are too few for the work seen to lead by four standard deviations"
    );
    assert_eq!(
        d.perception(H24).forks.folding_threshold(),
        4,
        "twenty blocks are enough"
    );

    // X1 and H1 both weigh the genesis and themselves, and H1 has the lower hash, so a block on X1
    // may not list it.
    assert_eq!(
        derive(&d, &mine(&d, 50, vec![X1, H1])).unwrap_err(),
        DagError::Heavier {
            selected: X1,
            heavier: H1
        }
    );

    // M on H3 folds X3: both weigh four blocks, and H3 has the lower hash. X1 to X3 are all
    // contested, and H3's side at J, H1 to H3, ties them: H1's lower hash wins.
    let m = derive(&d, &mine(&d, M, vec![H3, X3])).unwrap();
    assert_eq!(
        m.forks.get(0).and_then(|r| r.rival(X1)),
        Some(&Rival {
            contested: 30,
            settled: 0,
            chain: 30
        })
    );
    assert_eq!(m.forks.held(), 30, "3 against 3");
    assert_eq!(m.forks.folding_threshold(), 0);

    // N on H24 folds X5, but it joins at J, below H24's folding threshold, and is red.
    let n = derive(&d, &mine(&d, N, vec![H24, X5])).unwrap();
    assert!(n.forks.get(0).is_none());
    assert_eq!(n.forks.acknowledged(), 0);
    assert_eq!(n.past_work, 310, "the red 5 in the past work");
}

/// Listing an ancestor of the selected parent changes nothing; listing its child is refused.
#[test]
fn redundant_parents_need_no_rule() {
    let mut d = dag(500, 1_000_000);
    d.add_block(mine(&d, 1, vec![0])).unwrap();
    let plain = derive(&d, &mine(&d, 2, vec![1])).unwrap();
    let redundant = derive(&d, &mine(&d, 2, vec![1, 0, 0])).unwrap();
    assert_eq!(plain, redundant);

    // A child weighs its parent and itself, so it always outranks its parent.
    assert_eq!(
        derive(&d, &mine(&d, 2, vec![0, 1])).unwrap_err(),
        DagError::Heavier {
            selected: 0,
            heavier: 1
        }
    );
}

/// The own chain and entangled work both support a side at once.
///
/// ```text
///             +- B --- T
///   G --- A --+- C . . :     (T folds C and E)
///   +---- E . . . . . .:
/// ```
///
/// G=0 A=1 B=2 C=3 E=4 T=5.
#[test]
fn entangled_work_supports_at_once() {
    const G: u32 = 0;
    const A: u32 = 1;
    const B: u32 = 2;
    const C: u32 = 3;
    const E: u32 = 4;
    const T: u32 = 5;
    let mut d = dag(200, 1_000_000);
    for (id, parent) in [(A, G), (E, G), (B, A), (C, A)] {
        d.add_block(mine(&d, id, vec![parent])).unwrap();
    }

    // B and C weigh three blocks, E two, so T selects B by hash and folds the others.
    let t = mine(&d, T, d.next_block(T, 0).parents().collect());
    assert_eq!(t.parents().collect::<Vec<_>>(), vec![B, C, E]);
    let t = derive(&d, &t).unwrap();
    let lone = Rival {
        contested: 10,
        settled: 0,
        chain: 10,
    };
    assert_eq!(t.forks.get(1).and_then(|r| r.rival(C)), Some(&lone));
    assert_eq!(t.forks.get(0).and_then(|r| r.rival(E)), Some(&lone));

    // At A, C ties B, and B has the lower hash. At G, A and B are on the side, and C above the fork
    // supports it: 3 against E's 1.
    assert_eq!(t.forks.held(), 20, "C and E foldable");
    assert_eq!(t.blue_work(), 60, "the chain G, A, B and T, with C and E");

    // Without C, A and B alone outweigh E at G.
    let alone = derive(&d, &mine(&d, T, vec![B, E])).unwrap();
    assert_eq!(alone.forks.held(), 10);
}

/// A lineage that sees less than half the work the network is known to do closes nothing.
///
/// ```text
///        +- H1 -- H2 -- H3 -- ... -- H30 -- N       (N folds X15)
///   J ---+
///        +- X1 ------ X2 ------ ... ------ X15 . .
/// ```
///
/// The genesis knows a block a second. H1..H30 come a second apart, H1 at 1 s; X1..X15, hidden, with
/// half the hash rate, each block in the time half the network mines its work. J is the genesis,
/// 0; H1..H30=1..30 X1..X15=41..55 N=60.
#[test]
fn a_minority_closes_nothing() {
    const J: u32 = 0;
    const H30: u32 = 30;
    const X1: u32 = 41;
    const X15: u32 = 55;
    const N: u32 = 60;
    let mut d = dag(200, 1_000_000);
    for h in 1..=H30 {
        let parent = if h == 1 { J } else { h - 1 };
        d.add_block(mine_at(&d, h, vec![parent], u64::from(h) * 1000))
            .unwrap();
    }
    let mut time = 0;
    for x in X1..=X15 {
        let parent = if x == X1 { J } else { x - 1 };
        time += d.perception(parent).network.block_work() * 200;
        d.add_block(mine_at(&d, x, vec![parent], time)).unwrap();
    }
    let x15 = d.perception(X15);
    assert_eq!(
        x15.forks.folding_threshold(),
        0,
        "X saw half the work: it closes nothing"
    );
    // Its difficulty follows the half it sees only over the finality horizon, 6000 intervals:
    // fifteen blocks two seconds apart take it from 60000 to about 59850 of work, still 10 a
    // block. Its expected rate learns from what it has folded, which is nothing.
    assert_eq!(
        x15.network.block_work(),
        10,
        "fifteen blocks barely move it"
    );
    assert_eq!(
        x15.network.rate(),
        10,
        "the network's, as it last folded it"
    );
    assert_eq!(
        d.perception(H30).forks.folding_threshold(),
        10,
        "H sees all there is and closes"
    );

    // N on H30 folds X15 at 31 s: X joins at J, below H30's folding threshold, and is red.
    let n = derive(&d, &mine_at(&d, N, vec![H30, X15], 31_000)).unwrap();
    assert!(n.forks.get(0).is_none());
    assert_eq!(n.forks.acknowledged(), 0);
}

/// A node sequences each chain block's credited mergeset, then its red one, then the chain block,
/// and reports a reorg whenever its heaviest tip changes sides, before what it sequences on the new
/// side.
///
/// On the natural fork above it follows F4, then the G side once G5 outweighs it, then the F side
/// again once that overtakes: two reorgs at C2. H on F26 then sequences G4 to G26 as red.
#[test]
fn sequencing() {
    let mut d = natural_fork_dag();
    let h = mine(&d, H, d.next_block(H, 0).parents().collect());
    d.add_block(h).unwrap();

    let updates = d.updates();
    let kinds: Vec<char> = updates
        .iter()
        .map(|u| match u {
            Update::Reorg(_) => 'r',
            Update::Advance(_) => 'a',
        })
        .collect();
    assert!(
        kinds.windows(2).all(|w| w[0] != 'r' || w[1] == 'a'),
        "a reorg, then what the new side sequences"
    );
    let mut sequence = Vec::new();
    let mut reorgs = Vec::new();
    for update in updates {
        match update {
            Update::Reorg(reorg) => {
                for reverted in &reorg.reverted {
                    assert_eq!(sequence.pop().as_ref(), Some(reverted));
                }
                reorgs.push(reorg);
            }
            Update::Advance(sequenced) => sequence.extend(sequenced),
        }
    }
    assert_eq!(reorgs.len(), 2);
    assert!(reorgs.iter().all(|r| r.common.id == C2));
    let g_side: Vec<u32> = (4..=TOP).rev().map(|h| G + h).chain([D3]).collect();
    let chain: Vec<u32> = reorgs[1].chain().map(|c| c.id).collect();
    assert_eq!(chain, g_side, "the G side, the highest first");

    // B1 and C2; E3; F4 credits D3, which joined at C2 above E3's folding threshold; F5 to F26; H
    // sequences the G side, which joined below F26's, as red, then itself. Every block comes with
    // its perception.
    assert!(sequence
        .iter()
        .all(|s| *s.perception() == d.perception(s.id())));
    let seen: Vec<(char, u32)> = sequence
        .iter()
        .map(|s| match s {
            SequencedBlock::Blue(m) => ('b', m.id),
            SequencedBlock::Red(m) => ('r', m.id),
            SequencedBlock::Chain(m) => ('c', m.id),
        })
        .collect();
    let mut expected = vec![('c', 1), ('c', C2), ('c', E3), ('b', D3)];
    expected.extend((4..=TOP).map(|h| ('c', F + h)));
    expected.extend((4..=TOP).map(|h| ('r', G + h)));
    expected.push(('c', H));
    assert_eq!(seen, expected);
}

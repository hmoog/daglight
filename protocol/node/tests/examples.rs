//! Worked examples of the rules on small DAGs.
//!
//! Work is in tenths: a block, the genesis included, is 1, `10`. The genesis guesses a delay of a
//! fifth of a block interval, so the delay work is 0.2: a fork folds once its side leads by more
//! than 0.8 blocks, and a chain nothing rivals closes every height below its parent's as it goes.
//! Where an example needs its forks open longer, its genesis guesses a delay of a whole interval,
//! a margin of four blocks. A chain block counts at once, for the rank and for the sides of the
//! forks below it.
//! Every block is mined as the network would: with the work its selected parent requires, stamped
//! when a block a second has done the work before it, so the difficulty holds at one block. These
//! DAGs are narrow, so the width stays at one block per step and weighing leaves a chain whole. Ids
//! are chosen so that hash ties fall as described. In the drawings solid edges are selected parents
//! and dotted edges folded parents.

use daglight_protocol_block_perception::{NetworkPerception, Rival};
use daglight_protocol_dag::{DagError, SequencedBlock, Update};

mod common;

use common::*;

/// The ids of the natural fork: its common blocks, the F side and the G side by height, and a
/// block that meets both.
const A0: u32 = 0;
const C2: u32 = 2;
const E3: u32 = F + 3;
const D3: u32 = G + 3;
const H: u32 = 400;
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
    assert_eq!(f4.blue_work, 60, "the chain A0 to F4, and D3");

    // Every block is a sample of the delay, by its share of the finality horizon's work, a step of
    // 0.006 ms here. B1, C2, E3 and F4 extend their selected parents and missed nothing: four steps
    // down. D3 had missed E3, a whole block, more than the handshake's 0.4: withheld, as far as the
    // delay can tell, and no sample. The delay work follows the delay down at each fold, and
    // 1.9999 is 1 in tenths: from C2's children on, the margin is 0.4 blocks.
    assert_eq!(
        f4.network.delay_time(&PROTOCOL_PARAMETERS),
        199,
        "in milliseconds, 199.98"
    );
    assert_eq!(f4.network.delay_work(), 1);

    // A chain nothing rivals closes every height below its parent's: C2 closed A0, E3 closed B1.
    // F4 stops at C2, where E3 alone does not lead D3 by the margin.
    assert_eq!(d.perception(E3).forks.folding_threshold(), 2);
    assert_eq!(f4.forks.folding_threshold(), 2);

    // F5 folds C2 for good: E3 and F4 lead D3 by more than the margin, and F4 alone closes E3's
    // height.
    let f5 = d.perception(F + 5);
    assert_eq!(f5.forks.folding_threshold(), 4, "C2 and E3's height closed");
    assert!(f5.forks.is_empty());
    assert_eq!(f5.forks.folded(), 10, "D3 folded for good");
    assert_eq!(
        d.perception(F + TOP).network.block_work(),
        10,
        "the difficulty held"
    );

    // The G side, seeing nothing against it, closes C2 its own way at G4, folding nothing.
    assert_eq!(d.perception(D3).forks.folding_threshold(), 2);
    assert_eq!(d.perception(G + 4).forks.folding_threshold(), 3);
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
///                        +- E3 -- F4 -- F5 -- F6 -- F7 -- F8 -- F9 -- ... -- F14
///   A0 -- B1 -- C2 ------+        :     :     :     :     :     :
///                        +- D3 .. : G4 .: G5 .: G6 .: G7 .: G8 .:      (F4 folds D3, F5 G4, ...)
/// ```
///
/// The genesis guesses a delay of a block interval: a margin of four blocks. Both sides mine a
/// block a second, E3 and D3 at 3 s; the F side folds the G block of the second before, the G side
/// lists nothing and stops at G8. A0 is the genesis, 0; B1=1 C2=2, the F side 100 plus its
/// height, the G side 200 plus its.
#[test]
fn settled_work_is_never_folded() {
    let mut d = dag(1000, 1_000_000);
    for (id, parent, time) in [(1, A0, 1000), (C2, 1, 2000), (E3, C2, 3000), (D3, C2, 3000)] {
        d.add_block(mine_at(&d, id, vec![parent], time)).unwrap();
    }
    for h in 4..=14 {
        let time = u64::from(h) * 1000;
        if h <= 8 {
            d.add_block(mine_at(&d, G + h, vec![G + h - 1], time))
                .unwrap();
        }
        let g = G + (h - 1).min(8);
        d.add_block(mine_at(&d, F + h, vec![F + h - 1, g], time))
            .unwrap();
    }

    // The G side, seeing nothing against it, closes C2 its own way at G7: G7 and G8 are settled
    // to the F side, D3 to G6 contested.
    assert_eq!(d.perception(G + 6).forks.folding_threshold(), 1);
    assert_eq!(d.perception(G + 7).forks.folding_threshold(), 3);

    // By F12 the F side has recorded all of them at C2 and leads all six, so the contested four
    // are held; the settled two weigh against the fold, which the lead is short of.
    let f12 = d.perception(F + 12);
    let rivals = f12.forks.get(2).expect("C2 open");
    assert_eq!(rivals.contested(), 40, "D3 to G6");
    assert_eq!(rivals.settled(), 20, "G7 and G8");
    assert_eq!(f12.forks.held(), 40, "the contested work, led");
    assert_eq!(f12.forks.folding_threshold(), 2);

    // By F14 the side, E3 to F13, leads all six by more than the margin and has folded C2: the
    // contested work only.
    let f14 = d.perception(F + 14);
    assert!(f14.forks.is_empty());
    assert_eq!(
        f14.forks.folded(),
        40,
        "G7 and G8 weighed against folding and stay evidence"
    );
}

/// A lone lineage closes as it goes, and joins below a chain that has closed.
///
/// ```text
///        +- H1 -- H2 -- H3 -- ... -- H24 -- N     (N folds X5)
///   J ---+  :
///        +- X1 -- X2 -- X3 -- X4 -- X5 . . .      (M on H1 folds X1)
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
    // Each lineage, alone as it is, closes every height below its parent's as it goes.
    assert_eq!(d.perception(X5).forks.folding_threshold(), 4);
    assert_eq!(d.perception(H24).forks.folding_threshold(), 23);

    // X1 and H1 both weigh the genesis and themselves, and H1 has the lower hash, so a block on X1
    // may not list it.
    assert_eq!(
        derive(&d, &mine(&d, 50, vec![X1, H1])).unwrap_err(),
        DagError::Heavier {
            selected: X1,
            heavier: H1
        }
    );

    // M on H1 folds X1: both weigh two blocks, and H1 has the lower hash. H1 has closed nothing,
    // so X1 is recorded at J, and H1 alone ties it there: the lower hash wins.
    let m = derive(&d, &mine(&d, M, vec![H1, X1])).unwrap();
    assert_eq!(
        m.forks.get(0).and_then(|r| r.rival(X1)),
        Some(&Rival {
            contested: 10,
            settled: 0,
            chain: 10
        })
    );
    assert_eq!(m.forks.held(), 10, "1 against 1");
    assert_eq!(m.forks.folding_threshold(), 0);

    // On H3 already, X3 would join below the threshold, red.
    assert_eq!(d.perception(H3).forks.folding_threshold(), 2);
    let m = derive(&d, &mine(&d, M, vec![H3, X3])).unwrap();
    assert!(m.forks.get(0).is_none());
    assert_eq!(m.forks.acknowledged(), 0);

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
/// G=0 A=1 B=2 C=3 E=4 T=5. The genesis guesses a delay of a block interval, so that B has not
/// closed G's height when T comes.
#[test]
fn entangled_work_supports_at_once() {
    const G: u32 = 0;
    const A: u32 = 1;
    const B: u32 = 2;
    const C: u32 = 3;
    const E: u32 = 4;
    const T: u32 = 5;
    let mut d = dag(1000, 1_000_000);
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
    assert_eq!(t.blue_work, 60, "the chain G, A, B and T, with C and E");

    // Without C, A and B alone outweigh E at G.
    let alone = derive(&d, &mine(&d, T, vec![B, E])).unwrap();
    assert_eq!(alone.forks.held(), 10);
}

/// A lineage that sees less than half the work the network is known to do closes all the same, as
/// nothing it sees rivals its chain; to the majority, which closes its own way, it is red.
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
fn a_minority_closes_on_its_own() {
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
        14,
        "X saw half the work and nothing against it: it closes as it goes"
    );
    // Its difficulty and its pace follow the half it sees only over the finality horizon, 300
    // intervals: fifteen blocks two seconds apart barely move either from 10 a block.
    assert_eq!(x15.network.block_work(), 10);
    assert_eq!(x15.network.rate(), 10);
    assert_eq!(
        d.perception(H30).forks.folding_threshold(),
        29,
        "H sees all there is and closes as it goes"
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

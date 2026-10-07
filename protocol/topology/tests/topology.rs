//! The topology against pasts computed by brute force on random DAGs.

use std::collections::BTreeSet;

use daglight_protocol_topology::{BlockAddress, Topology};

/// A small deterministic generator for the random DAGs.
struct Lcg(u64);

impl Lcg {
    /// Returns the next value below `n`.
    fn below(&mut self, n: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.0 >> 33) % n as u64) as usize
    }
}

/// A block's strict past, as one bit per block id.
#[derive(Clone, Default)]
struct Past(Vec<u64>);

impl Past {
    /// Returns whether block `id` is in the past.
    fn contains(&self, id: u32) -> bool {
        self.0
            .get(id as usize / 64)
            .is_some_and(|w| w >> (id % 64) & 1 == 1)
    }

    /// Adds block `id`.
    fn insert(&mut self, id: u32) {
        let i = id as usize / 64;
        if self.0.len() <= i {
            self.0.resize(i + 1, 0);
        }
        self.0[i] |= 1 << (id % 64);
    }

    /// Adds every block of `other`.
    fn extend(&mut self, other: &Past) {
        if self.0.len() < other.0.len() {
            self.0.resize(other.0.len(), 0);
        }
        for (w, o) in self.0.iter_mut().zip(&other.0) {
            *w |= o;
        }
    }
}

/// A random DAG built into a topology, with every block's past kept by brute force.
struct Fixture {
    /// The topology under test.
    topology: Topology<u32, ()>,
    /// Every block's address, by id.
    addresses: Vec<BlockAddress>,
    /// Every block's strict past, by id.
    pasts: Vec<Past>,
    /// Whether some block lists the block as a parent, by id.
    referenced: Vec<bool>,
}

impl Fixture {
    /// Builds `n` blocks, each on a recent selected parent, folding a few other recent blocks and
    /// the oldest recent block nobody has referenced yet, as honest nodes fold every tip.
    fn new(seed: u64, n: usize) -> Self {
        let mut rng = Lcg(seed);
        let mut f = Self {
            topology: Topology::new(0, ()),
            addresses: vec![BlockAddress {
                height: 0,
                offset: 0,
            }],
            pasts: vec![Past::default()],
            referenced: vec![false; n],
        };
        for id in 1..n as u32 {
            let recent = |rng: &mut Lcg| id - 1 - rng.below((id as usize).min(12)) as u32;
            let s = recent(&mut rng);
            let mut parents = vec![s];
            let oldest = (id.saturating_sub(12)..id).find(|&x| !f.referenced[x as usize]);
            for p in (0..rng.below(4)).map(|_| recent(&mut rng)).chain(oldest) {
                if !parents.contains(&p) {
                    parents.push(p);
                }
            }
            for &p in &parents {
                f.referenced[p as usize] = true;
            }

            // The past is the parents and their pasts.
            let mut past = Past::default();
            for &p in &parents {
                past.insert(p);
                past.extend(&f.pasts[p as usize]);
            }

            let parent_addresses = parents.iter().map(|&p| f.addresses[p as usize]).collect();
            let address = f.topology.insert(id, parent_addresses, ());
            f.addresses.push(address);
            f.pasts.push(past);
        }
        f
    }

    /// Returns the ids on a block's selected chain, by walking selected parents one at a time.
    fn chain(&self, id: u32) -> Vec<u32> {
        let mut out = vec![id];
        let mut cur = self.addresses[id as usize];
        while let Some(p) = self.topology.selected_parent(cur) {
            out.push(self.topology.vertex(p).id);
            cur = p;
        }
        out
    }
}

/// Ancestry agrees with the brute-force pasts for every pair of blocks.
#[test]
fn ancestry_matches_brute_force() {
    let f = Fixture::new(3, 400);
    for a in 0..400u32 {
        for b in 0..400u32 {
            let (sa, sb) = (f.addresses[a as usize], f.addresses[b as usize]);
            assert_eq!(
                f.topology.is_ancestor(sa, sb),
                f.pasts[b as usize].contains(a),
                "{a} in the past of {b}"
            );
        }
    }
}

/// Chain lookups, joins and branches agree with walking the chains one step at a time.
#[test]
fn chain_queries_match_walks() {
    let f = Fixture::new(5, 600);
    let mut rng = Lcg(9);
    for _ in 0..3_000 {
        let (a, b) = (rng.below(600) as u32, rng.below(600) as u32);
        let (ca, cb) = (f.chain(a), f.chain(b));
        let (sa, sb) = (f.addresses[a as usize], f.addresses[b as usize]);

        // Every height on `a`'s chain.
        for (i, &c) in ca.iter().enumerate() {
            let h = sa.height - i as u64;
            assert_eq!(f.topology.vertex(f.topology.ancestor(sa, h)).id, c);
        }

        // The join is the first common block; the branch the one before it on `a`'s chain.
        if cb.contains(&a) {
            continue;
        }
        let join = *ca.iter().find(|c| cb.contains(c)).expect("chains meet");
        let above = ca[ca.iter().position(|&c| c == join).unwrap() - 1];
        assert_eq!(f.topology.vertex(f.topology.join(sa, sb).unwrap()).id, join);
        assert_eq!(
            f.topology.vertex(f.topology.branch(sa, sb).unwrap()).id,
            above
        );
    }
}

/// A mergeset is exactly the folded blocks' past outside the selected parent's past.
#[test]
fn mergeset_matches_brute_force() {
    let f = Fixture::new(7, 500);
    let mut rng = Lcg(21);
    for _ in 0..2_000 {
        let s = rng.below(500) as u32;
        let folded: Vec<u32> = (0..1 + rng.below(3))
            .map(|_| rng.below(500) as u32)
            .collect();

        let mut reached = Past::default();
        for &p in &folded {
            reached.insert(p);
            reached.extend(&f.pasts[p as usize]);
        }
        let expected: BTreeSet<u32> = (0..500u32)
            .filter(|&x| reached.contains(x) && x != s && !f.pasts[s as usize].contains(x))
            .collect();

        let got: BTreeSet<u32> = f
            .topology
            .mergeset(
                f.addresses[s as usize],
                folded.iter().map(|&p| f.addresses[p as usize]),
            )
            .addresses()
            .iter()
            .map(|&x| f.topology.vertex(x).id)
            .collect();
        assert_eq!(got, expected);
    }
}

/// Ancestry far below the cached top of a chain agrees with the brute-force pasts.
#[test]
fn deep_ancestry_matches_brute_force() {
    let f = Fixture::new(17, 3_000);
    let top = f.addresses.iter().map(|s| s.height).max().unwrap_or(0);
    assert!(
        top > 300,
        "the DAG reaches {top}, deep enough to leave the cache"
    );
    let mut rng = Lcg(23);
    for _ in 0..20_000 {
        let (a, b) = (rng.below(3_000) as u32, rng.below(3_000) as u32);
        let (sa, sb) = (f.addresses[a as usize], f.addresses[b as usize]);
        assert_eq!(
            f.topology.is_ancestor(sa, sb),
            f.pasts[b as usize].contains(a),
            "{a} in the past of {b}"
        );
    }
}

/// Every lane is a lane: each block is in the past of the next, and the cover stays narrow.
#[test]
fn lanes_are_chains() {
    let f = Fixture::new(29, 2_000);
    let mut lanes: Vec<Vec<(u32, u32)>> = vec![Vec::new(); f.topology.lanes()];
    for id in 0..2_000u32 {
        let e = f.topology.vertex(f.addresses[id as usize]);
        lanes[e.lane as usize].push((e.position, id));
    }
    for lane in &mut lanes {
        lane.sort();
        for (i, pair) in lane.windows(2).enumerate() {
            assert_eq!(pair[0].0 as usize, i, "positions are dense");
            assert!(f.pasts[pair[1].1 as usize].contains(pair[0].1));
        }
    }

    // Every block gets referenced, so the cover stays within the DAG's window of twelve.
    assert!(f.topology.lanes() <= 24, "{} lanes", f.topology.lanes());
}

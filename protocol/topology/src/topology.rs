use std::collections::{HashMap, VecDeque};
use std::hash::BuildHasherDefault;
use std::iter::once;

use daglight_protocol_block::BlockId;

use crate::lane::Lane;
use crate::link::Link;
use crate::{BlockAddress, Chain, Height, IdHasher, Mergeset, PerceptionSummary, Vertex};

/// Every block of a DAG by address, with the structure to answer chain and ancestry queries fast.
///
/// Ancestry comes from a lane cover: every block extends a lane, a sequence of blocks each in the
/// past of the next, so any block's past meets every lane in a prefix, and one count per lane,
/// its reach, describes the whole past. It is computed once, on insertion, and never changes.
///
/// Pruning drops blocks for good: heights below `base` entirely, and above it the blocks nothing
/// can need, leaving holes. Positions and reach stay as they were.
#[derive(Clone, Debug)]
pub struct Topology<Id, T> {
    /// The lowest height kept.
    base: Height,
    /// The vertices by height from `base`, then by position within the height; `None` where
    /// pruned.
    vertices: VecDeque<Vec<Option<Vertex<Id, T>>>>,
    /// The selected-chain links, laid out like `vertices`.
    links: VecDeque<Vec<Link>>,
    /// The blocks of every lane, in order.
    lanes: Vec<Lane>,
    /// The address of every block, by id.
    addresses: HashMap<Id, BlockAddress, BuildHasherDefault<IdHasher>>,
}

impl<Id: BlockId, T> Topology<Id, T> {
    /// Creates a topology holding only the genesis.
    pub fn new(genesis: Id, data: T) -> Self {
        let address = BlockAddress::GENESIS;
        let mut addresses = HashMap::default();
        addresses.insert(genesis, address);
        Self {
            base: 0,
            vertices: VecDeque::from([vec![Some(Vertex {
                id: genesis,
                parents: Vec::new(),
                lane: 0,
                position: 0,
                reach: Box::new([1]),
                data,
            })]]),
            links: VecDeque::from([vec![Link {
                parent: address,
                jump: address,
            }]]),
            lanes: vec![Lane::of(address)],
            addresses,
        }
    }

    /// Returns the number of blocks kept.
    pub fn len(&self) -> usize {
        self.addresses.len()
    }

    /// Returns `false`: a topology always keeps a block.
    pub fn is_empty(&self) -> bool {
        false
    }

    /// Returns the number of lanes.
    pub fn lanes(&self) -> usize {
        self.lanes.len()
    }

    /// Returns the address of a block, if known.
    pub fn address(&self, id: Id) -> Option<BlockAddress> {
        self.addresses.get(&id).copied()
    }

    /// Returns the vertex at an address; panics if it was pruned.
    #[track_caller]
    pub fn vertex(&self, address: BlockAddress) -> &Vertex<Id, T> {
        match self.kept(address) {
            Some(v) => v,
            None => panic!("{address:?} was pruned"),
        }
    }

    /// Returns the vertex at an address, unless it was pruned.
    pub fn kept(&self, address: BlockAddress) -> Option<&Vertex<Id, T>> {
        self.vertices
            .get(address.height.checked_sub(self.base)? as usize)?
            .get(address.offset as usize)?
            .as_ref()
    }

    /// Adds a block on known parents, the selected parent first.
    pub fn insert(&mut self, id: Id, parents: Vec<BlockAddress>, data: T) -> BlockAddress {
        // A block is placed once, on its selected parent.
        assert!(!self.addresses.contains_key(&id), "duplicate block {id:?}");
        let p = *parents.first().expect("a block has a selected parent");

        // The block takes the next position one height above its selected parent.
        let height = p.height + 1;
        let level = (height - self.base) as usize;
        if self.vertices.len() <= level {
            self.vertices.push_back(Vec::new());
            self.links.push_back(Vec::new());
        }
        let address = BlockAddress {
            height,
            offset: self.vertices[level].len() as u32,
        };

        // The past is the parents' pasts together; the block extends a lane and reaches one block
        // further along it.
        let jump = self.jump(p);
        let mut reach = self.union(parents.iter().copied());
        let lane = self.lane_for(self.vertex(p).lane as usize, &reach);
        let position = self.append(lane, address);
        if reach.len() <= lane {
            reach.resize(lane + 1, 0);
        }
        reach[lane] = position + 1;

        // The vertex, its chain links and its address are stored side by side.
        self.vertices[level].push(Some(Vertex {
            id,
            parents,
            lane: lane as u32,
            position,
            reach: reach.into_boxed_slice(),
            data,
        }));
        self.links[level].push(Link { parent: p, jump });
        self.addresses.insert(id, address);
        address
    }

    /// Returns the jump of a block on `p`: it spans twice the previous one whenever the two before
    /// it are equal, so that the jump's height depends on the height alone and walks take
    /// logarithmic steps. A jump below the base is kept by its height alone.
    fn jump(&self, p: BlockAddress) -> BlockAddress {
        // A jump of one step is the parent itself.
        let to = Self::jump_height(p.height + 1);
        if to == p.height {
            return p;
        }

        // A longer jump lands where the parent's jump's jump does; below the base only the height
        // is known, which is all a walk needs there.
        let pj = self.link(p).jump;
        if pj.height >= self.base {
            self.link(pj).jump
        } else {
            BlockAddress {
                height: to,
                offset: u32::MAX,
            }
        }
    }

    /// Returns the lane a block whose past has `reach` extends: its selected parent's lane `own` if
    /// that ends in its past, else the first lane it has seen to the end, else a new lane.
    fn lane_for(&self, own: usize, reach: &[u32]) -> usize {
        // A lane can be extended only by a block whose past holds all of it.
        let seen_whole = |l: usize| reach.get(l).copied().unwrap_or(0) == self.lanes[l].len();
        if seen_whole(own) {
            return own;
        }

        // Else the first such lane, else a new one.
        (0..self.lanes.len())
            .find(|&l| seen_whole(l))
            .unwrap_or(self.lanes.len())
    }

    /// Appends `address` to `lane`, a new lane if it is one past the last, and returns its position
    /// there.
    fn append(&mut self, lane: usize, address: BlockAddress) -> u32 {
        if lane == self.lanes.len() {
            self.lanes.push(Lane::of(address));
        } else {
            self.lanes[lane].push(address);
        }
        self.lanes[lane].len() - 1
    }

    /// Returns the selected parent; `None` for the genesis.
    pub fn selected_parent(&self, address: BlockAddress) -> Option<BlockAddress> {
        (address.height > 0).then(|| self.link(address).parent)
    }

    /// Returns the selected chain of `address`.
    pub fn chain(&self, address: BlockAddress) -> Chain<'_, Id, T> {
        Chain::new(self, address)
    }

    /// Returns the block at height `h` on the chain of `address`.
    pub fn ancestor(&self, mut address: BlockAddress, h: Height) -> BlockAddress {
        // The height must lie on the chain and be kept.
        assert!(h <= address.height, "height {h} above {address:?}");
        assert!(h >= self.base, "height {h} was pruned");

        // Jump whenever the jump does not overshoot, else step.
        while address.height > h {
            let link = self.link(address);
            address = if link.jump.height >= h {
                link.jump
            } else {
                link.parent
            };
        }
        address
    }

    /// Returns the highest block on both the chain of `a` and the chain of `b`; `None` if it was
    /// pruned.
    pub fn join(&self, a: BlockAddress, b: BlockAddress) -> Option<BlockAddress> {
        self.meet(a, b, self.base).map(|m| m.0)
    }

    /// Returns the block on the chain of `a` right above its join with the chain of `b`; `None` if
    /// the join was pruned.
    pub fn branch(&self, a: BlockAddress, b: BlockAddress) -> Option<BlockAddress> {
        self.meet(a, b, self.base).map(|m| m.1)
    }

    /// Returns whether `a` is `b` or on its selected chain.
    pub fn on_chain(&self, a: BlockAddress, b: BlockAddress) -> bool {
        a.height <= b.height && self.ancestor(b, a.height) == a
    }

    /// Returns whether `a` is in the past of `b`.
    pub fn is_ancestor(&self, a: BlockAddress, b: BlockAddress) -> bool {
        a != b && self.vertex(b).sees(self.vertex(a))
    }

    /// Returns, for every block of `merged`, the mergeset of `s`, in address order, the block on its
    /// chain right above the join with the chain of `s`; `None` where the join lies below height
    /// `floor` or was pruned.
    pub fn branches(
        &self,
        s: BlockAddress,
        merged: &Mergeset,
        floor: Height,
    ) -> Vec<Option<BlockAddress>> {
        // In address order a block comes after its selected parent, so a merged parent's branch
        // is known by the time its child is asked.
        let addresses = merged.addresses();
        let mut branches: Vec<Option<BlockAddress>> = Vec::with_capacity(addresses.len());
        for &x in addresses {
            let branch = match self.selected_parent(x) {
                // The parent was merged too: the same chain, the same branch.
                Some(p) if let Ok(i) = addresses.binary_search(&p) => branches[i],
                // The parent lies below the floor: so does the join.
                Some(p) if p.height < floor.max(self.base) => None,
                // The parent is on the chain of `s`: the block itself branches off.
                Some(p) if self.on_chain(p, s) => Some(x),
                // Else the chains are walked down together.
                _ => self.meet(x, s, floor).map(|m| m.1),
            };
            branches.push(branch);
        }
        branches
    }

    /// Returns the blocks in `folded` or their past that are neither `s` nor in its past.
    pub fn mergeset(
        &self,
        s: BlockAddress,
        folded: impl IntoIterator<Item = BlockAddress>,
    ) -> Mergeset {
        // Per lane, what the folded blocks reach beyond `S`; the lanes partition the blocks, so
        // these runs are disjoint and their union needs no deduplication.
        let base = &self.vertex(s).reach;
        let reach = self.union(once(s).chain(folded));
        let mut runs = Vec::new();
        for (lane, &to) in reach.iter().enumerate() {
            let from = base.get(lane).copied().unwrap_or(0);
            let run: Vec<BlockAddress> = self.lanes[lane].range(from, to).collect();
            if !run.is_empty() {
                runs.push((lane as u32, from.max(self.lanes[lane].start()), run));
            }
        }
        Mergeset::of(runs)
    }

    /// Returns the reach of the blocks together: per lane, the most any of them reaches.
    fn union(&self, blocks: impl IntoIterator<Item = BlockAddress>) -> Vec<u32> {
        // Lane by lane, the most any block reaches; a lane none reaches stays at zero.
        let mut reach: Vec<u32> = Vec::with_capacity(self.lanes.len() + 1);
        for b in blocks {
            let theirs = &self.vertex(b).reach;
            if reach.len() < theirs.len() {
                reach.resize(theirs.len(), 0);
            }
            for (r, &t) in reach.iter_mut().zip(theirs.iter()) {
                *r = (*r).max(t);
            }
        }
        reach
    }

    /// Returns the selected-chain links of a block.
    fn link(&self, address: BlockAddress) -> Link {
        self.links[(address.height - self.base) as usize][address.offset as usize]
    }

    /// Returns the height a block at `height` jumps to: `height` less the smallest term of its
    /// skew-binary form, the sums of `2^k - 1` taken greedily.
    fn jump_height(height: Height) -> Height {
        // Take the largest term that fits, again and again; the last one taken is the smallest.
        let (mut rest, mut term) = (height, 0);
        while rest > 0 {
            term = (1..)
                .map(|k| (1 << k) - 1)
                .take_while(|&t| t <= rest)
                .last()
                .unwrap_or(1);
            rest -= term;
        }
        height - term
    }

    /// Returns the join of the chains of `a` and `b`, and the block above it on `a`'s chain;
    /// `None` if the join lies below `floor` or was pruned, found without reading below either.
    fn meet(
        &self,
        a: BlockAddress,
        b: BlockAddress,
        floor: Height,
    ) -> Option<(BlockAddress, BlockAddress)> {
        // Start level, at the lower of the two heights, unless that is below the floor.
        let floor = floor.max(self.base);
        let h = a.height.min(b.height);
        if h < floor {
            return None;
        }

        // The lower block may be on the other's chain: then it is the join itself.
        let (mut x, mut y) = (self.ancestor(a, h), self.ancestor(b, h));
        if x == y {
            assert!(h < a.height, "join of {a:?} with its own chain");
            return Some((x, self.ancestor(a, h + 1)));
        }

        // Descend in step until the parents meet; jumps at equal heights have equal heights, and a
        // jump is safe whenever it does not reach a common block or a pruned height.
        loop {
            if x.height == floor {
                return None;
            }
            let (lx, ly) = (self.link(x), self.link(y));
            if lx.parent == ly.parent {
                return Some((lx.parent, x));
            }
            (x, y) = if lx.jump != ly.jump && lx.jump.height >= floor {
                (lx.jump, ly.jump)
            } else {
                (lx.parent, ly.parent)
            };
        }
    }

    /// Prunes every block that is neither one of `points` nor has one in its past, at heights below
    /// the highest point, and returns the addresses pruned. Heights left without a block go entirely.
    pub fn prune(&mut self, points: &[BlockAddress]) -> Vec<BlockAddress> {
        // The unneeded blocks leave holes; their ids are forgotten.
        let Some(top) = points.iter().map(|p| p.height).max() else {
            return Vec::new();
        };
        let gone = self.unneeded(points, top);
        for &address in &gone {
            let v = self.vertices[(address.height - self.base) as usize][address.offset as usize]
                .take()
                .expect("a kept block");
            self.addresses.remove(&v.id);
        }

        // Emptied heights at the bottom go, and the base moves up past them.
        while self.vertices.len() > 1 && self.vertices[0].iter().all(Option::is_none) {
            self.vertices.pop_front();
            self.links.pop_front();
            self.base += 1;
        }

        // So do the pruned blocks at the start of each lane.
        let (vertices, base) = (&self.vertices, self.base);
        let pruned = |s: BlockAddress| {
            s.height < base || vertices[(s.height - base) as usize][s.offset as usize].is_none()
        };
        for lane in &mut self.lanes {
            lane.prune(pruned);
        }
        gone
    }

    /// Returns every block below height `top` that is neither one of `points` nor has one in its
    /// past.
    fn unneeded(&self, points: &[BlockAddress], top: Height) -> Vec<BlockAddress> {
        // A block is needed while some point is it or sees it.
        let points: Vec<&Vertex<Id, T>> = points.iter().map(|&p| self.vertex(p)).collect();
        let keeps = |v: &Vertex<Id, T>| points.iter().any(|p| v.sees(p));
        (self.base..top)
            .flat_map(|height| self.kept_at(height))
            .filter(|(_, v)| !keeps(v))
            .map(|(address, _)| address)
            .collect()
    }

    /// Iterates over the blocks kept at `height`, with their addresses.
    fn kept_at(&self, height: Height) -> impl Iterator<Item = (BlockAddress, &Vertex<Id, T>)> {
        self.vertices[(height - self.base) as usize]
            .iter()
            .enumerate()
            .filter_map(move |(offset, v)| {
                v.as_ref().map(|v| {
                    (
                        BlockAddress {
                            height,
                            offset: offset as u32,
                        },
                        v,
                    )
                })
            })
    }
}

/// The points a block's chain is read down to, from what the topology keeps of every block.
impl<Id: BlockId, W> Topology<Id, PerceptionSummary<W>> {
    /// Returns the finality point of `s`: the highest block on its chain stamped at least one
    /// finality horizon before it.
    pub fn final_point(&self, s: BlockAddress) -> BlockAddress {
        self.ancestor(s, self.vertex(s).data.final_height)
    }

    /// Returns the pruning point of `s`: its finality point's finality point.
    pub fn pruning_point(&self, s: BlockAddress) -> BlockAddress {
        self.final_point(self.final_point(s))
    }

    /// Returns whether a block on `s` can be judged: the chain of `s` is kept down to its pruning
    /// point, the finality point's finality point, below which judging reads nothing.
    pub fn judgeable(&self, s: BlockAddress) -> bool {
        // The finality point first, as the pruning point is read off it.
        let kept = |h: Height| h >= self.base && self.kept(self.ancestor(s, h)).is_some();
        kept(self.vertex(s).data.final_height)
            && kept(self.vertex(self.final_point(s)).data.final_height)
    }

    /// Returns the point on the chain of `x` that keeps every block on `x` judgeable, with
    /// `da_offset` milliseconds of history beyond: the highest chain block stamped `da_offset`
    /// before the pruning point of `x`.
    pub fn retention_point(&self, x: BlockAddress, da_offset: u64) -> BlockAddress {
        // The chain of `x`, read by height.
        let at = |h: Height| self.ancestor(x, h);
        let data = |h: Height| &self.vertex(at(h)).data;

        // A chain is kept from some height on: whatever is pruned lies in the past of what is kept.
        let lowest = Self::first(self.base, x.height, |h| self.kept(at(h)).is_some());

        // The pruning point of `x`, the finality point's finality point, as far as kept; every
        // block whose chain contains `x` has its own no lower.
        let final_height = data(x.height).final_height.max(lowest);
        let pruning_height = data(final_height).final_height.max(lowest);

        // The highest chain block stamped `da_offset` before it, as far as kept.
        let height = match data(pruning_height).time.checked_sub(da_offset) {
            Some(cut) => Self::first(lowest, pruning_height + 1, |h| data(h).time > cut)
                .saturating_sub(1)
                .max(lowest),
            None => lowest,
        };
        at(height)
    }

    /// Returns the lowest height from `low` below `high` at which `pred` holds, `high` if none;
    /// `pred` must hold from some height on.
    fn first(mut low: Height, mut high: Height, pred: impl Fn(Height) -> bool) -> Height {
        // Binary search: the answer is always in `low..=high`.
        while low < high {
            let h = low + (high - low) / 2;
            if pred(h) {
                high = h;
            } else {
                low = h + 1;
            }
        }
        low
    }
}

use std::iter::once;
use std::sync::Arc;

use daglight_protocol_block::{Block, Work};
use daglight_protocol_block_perception::{
    BlockPerception, MergedBlock, MergedPerception, NetworkPerception,
};
use daglight_protocol_topology::{BlockAddress, Height, Mergeset};

use crate::{
    Admission, AdmitResult, DagError, DagStore, MergeResult, Reorg, Sequence, SequencedBlock, Tips,
    Update,
};

/// A block DAG: its blocks, held in a store, its tips, and the rules on them: which blocks it
/// admits, what a new block merges, which tips a new block may fold, and how a chain block
/// sequences its mergeset.
#[derive(Clone, Debug)]
pub struct Dag<S: DagStore> {
    /// The store holding the blocks.
    store: S,
    /// The tips of the blocks it holds, the heaviest and the finality point.
    tips: Tips<S::Id, S::Work, S::Network>,
}

impl<S: DagStore> Dag<S> {
    /// Returns the DAG of the blocks in `store`, which holds only the genesis.
    pub fn new(store: S) -> Self {
        Self {
            tips: Tips::new(store.perception_at(BlockAddress::GENESIS)),
            store,
        }
    }

    /// Returns the store holding the blocks.
    pub fn store(&self) -> &S {
        &self.store
    }

    /// Returns its tips: every block without children whose chain contains the finality point.
    pub fn tips(&self) -> &Tips<S::Id, S::Work, S::Network> {
        &self.tips
    }

    /// Returns the block to mine next, `id`, stamped `time`: on the heaviest tip, folding every
    /// other tip it may.
    pub fn next_block(&self, id: S::Id, time: u64) -> Block<S::Id, S::Work> {
        self.build_on(id, self.tips.heaviest(), &self.tips, time)
    }

    /// Returns the block `id`, stamped `time`, built on `s`, folding every other of `tips` it may.
    pub fn build_on(
        &self,
        id: S::Id,
        s: BlockAddress,
        tips: &Tips<S::Id, S::Work, S::Network>,
        time: u64,
    ) -> Block<S::Id, S::Work> {
        // The block lists ids; it carries the work `s` requires.
        let id_of = |p: BlockAddress| self.store.topology().vertex(p).id;
        Block::new(
            id,
            id_of(s),
            self.foldable(s, tips.others(s))
                .into_iter()
                .map(id_of)
                .collect(),
            self.store.perception_at(s).network.block_work(),
            time,
        )
    }

    /// Prunes every block that is neither one of `points` nor has one in its past, below the
    /// highest point, and every tip pruned with them.
    pub fn prune(&mut self, points: &[BlockAddress]) {
        self.store.prune(points);
        self.tips
            .retain(|t| self.store.topology().kept(t).is_some());
    }

    /// Admits `block` and places it, derived, unless it is already known; returns its perception
    /// either way, or why it is refused: a parent is unknown, what judging it would read was
    /// pruned, or it breaks the rules.
    pub fn insert(&mut self, block: &Block<S::Id, S::Work>) -> Admission<S> {
        // A known block is not derived again.
        let topology = self.store.topology();
        if let Some(address) = topology.address(block.id) {
            return Ok(self.store.perception_at(address));
        }

        // Its parents must be known, and its selected parent's chain kept down to the pruning point.
        if let Some(p) = block.parents().find(|&p| topology.address(p).is_none()) {
            return Err(DagError::UnknownParent(p));
        }
        if !topology.judgeable(
            topology
                .address(block.selected_parent)
                .expect("a known parent"),
        ) {
            return Err(DagError::Pruned);
        }

        // The rules admit it, its perception is derived and placed, and it becomes a tip.
        let merge = self.merge(block)?;
        let perception = BlockPerception::derive(&merge);
        let address = self.store.place(merge.parents, perception);
        let perception = self.store.perception_at(address);
        self.tips
            .insert(Arc::clone(&perception), address, self.store.topology());
        Ok(perception)
    }

    /// Returns the merged perception of `block`, whose parents are known, or why it is invalid:
    /// the rules admit it on its selected parent and merge the blocks of its mergeset.
    pub fn merge<'b>(&'b self, block: &'b Block<S::Id, S::Work>) -> MergeResult<'b, S> {
        // The parents as the rules see them, and what the selected parent knew.
        let (s, folded) = self.parents(block);
        let s_perception = self.store.perception_at(s);
        let pruning_height = self.store.topology().pruning_point(s).height;

        // The rules admit the block; its mergeset becomes what it merges.
        let (mergeset, branches) = self.admit(block, s, &s_perception, &folded, pruning_height)?;
        let merged = self.merged(&s_perception, &mergeset, &branches);
        Ok(MergedPerception::new(
            block,
            once(s).chain(folded).collect(),
            s_perception,
            self.store.topology().chain(s),
            merged,
            pruning_height,
        ))
    }

    /// Returns the `others` a block on `s` may fold: less every one the rules refuse as a folded
    /// parent, and every one through which the block would fold a chain that meets `s`'s chain
    /// below its pruning point. It is the rules' admission, kept rather than refused.
    pub fn foldable(&self, s: BlockAddress, mut others: Vec<BlockAddress>) -> Vec<BlockAddress> {
        // Every tip must pass the folded parent's check on its own.
        let topology = self.store.topology();
        let check = self.folded_parent(s);
        others.retain(|&p| check(p).is_ok());

        // A block that would be folded below the pruning point goes, with every tip that would
        // fold it, until none is left.
        let pruning_height = topology.pruning_point(s).height;
        loop {
            let mergeset = topology.mergeset(s, others.iter().copied());
            let Some(y) =
                Self::forks_below(&mergeset, &topology.branches(s, &mergeset, pruning_height))
            else {
                return others;
            };
            others.retain(|&p| p != y && !topology.is_ancestor(y, p));
        }
    }

    /// Returns what chain block `c` sequences before itself: its mergeset, the credited blocks
    /// first, then the red ones, each by past work, then by id.
    pub fn sequence(&self, c: BlockAddress) -> Sequence {
        // The genesis sequences nothing before itself.
        let topology = self.store.topology();
        let vertex = topology.vertex(c);
        let Some(&s) = vertex.parents.first() else {
            return Sequence::default();
        };

        // A merged block is red as its derivation found it: its chain meets the selected parent's
        // below that parent's folding threshold.
        let threshold = topology.vertex(s).data.folding_threshold;
        let mergeset = topology.mergeset(s, vertex.parents[1..].iter().copied());
        let mut sequence = Sequence::default();
        for (&x, branch) in mergeset
            .addresses()
            .iter()
            .zip(topology.branches(s, &mergeset, threshold))
        {
            match branch {
                Some(_) => sequence.blues.push(x),
                None => sequence.reds.push(x),
            }
        }

        // Each part by past work, then by id, so that every block comes after its past.
        let order = |&x: &BlockAddress| {
            let v = topology.vertex(x);
            (v.data.past_work, v.id)
        };
        sequence.blues.sort_by_key(order);
        sequence.reds.sort_by_key(order);
        sequence
    }

    /// Returns how the order changes as the chosen chain moves from tip `from` to tip `to`: the
    /// reorg, if it changes sides, then the blocks it newly sequences.
    pub fn order_change(
        &self,
        from: BlockAddress,
        to: BlockAddress,
    ) -> Vec<Update<S::Id, S::Work, S::Network>> {
        // What the old chain sequenced above the join is reverted, what the new one does is new.
        let common = self.common(from, to);
        let reverted = self.chain_above(from, common);
        let sequenced = self.sequenced(self.chain_above(to, common));

        // A reorg only if something is reverted, an advance only if something is new.
        (!reverted.is_empty())
            .then(|| Update::Reorg(self.reorg(common, &reverted)))
            .into_iter()
            .chain((!sequenced.is_empty()).then_some(Update::Advance(sequenced)))
            .collect()
    }

    /// Returns the highest block on the chains of both `from` and `to`.
    fn common(&self, from: BlockAddress, to: BlockAddress) -> BlockAddress {
        // One may be on the chain of the other; else the chains join somewhere above finality.
        let topology = self.store.topology();
        if topology.on_chain(from, to) {
            from
        } else if topology.on_chain(to, from) {
            to
        } else {
            topology
                .join(from, to)
                .expect("a join above the finality point")
        }
    }

    /// Returns the blocks on the chain of `tip` above `common`, lowest first.
    fn chain_above(&self, tip: BlockAddress, common: BlockAddress) -> Vec<BlockAddress> {
        // Walked down from the tip, then turned around.
        let mut chain: Vec<BlockAddress> = self
            .store
            .topology()
            .chain(tip)
            .addresses()
            .take_while(|c| c.height > common.height)
            .collect();
        chain.reverse();
        chain
    }

    /// Returns the reorg that reverts the chain blocks `reverted` above `common`, with what they
    /// sequenced, the last first.
    fn reorg(
        &self,
        common: BlockAddress,
        reverted: &[BlockAddress],
    ) -> Reorg<S::Id, S::Work, S::Network> {
        Reorg {
            common: self.store.perception_at(common),
            reverted: reverted
                .iter()
                .rev()
                .flat_map(|&c| self.unsequenced(c))
                .collect(),
        }
    }

    /// Returns what chain block `c` sequenced, the last first: itself, then its red blocks, then
    /// its credited ones, backwards.
    fn unsequenced(
        &self,
        c: BlockAddress,
    ) -> impl Iterator<Item = SequencedBlock<S::Id, S::Work, S::Network>> + '_ {
        self.sequenced_by(c).collect::<Vec<_>>().into_iter().rev()
    }

    /// Returns what the chain blocks `chain` sequence, in order.
    fn sequenced(
        &self,
        chain: Vec<BlockAddress>,
    ) -> Vec<SequencedBlock<S::Id, S::Work, S::Network>> {
        chain
            .into_iter()
            .flat_map(|c| self.sequenced_by(c))
            .collect()
    }

    /// Returns what chain block `c` sequences, in order: its credited blocks, its red ones, then
    /// itself.
    fn sequenced_by(
        &self,
        c: BlockAddress,
    ) -> impl Iterator<Item = SequencedBlock<S::Id, S::Work, S::Network>> + '_ {
        // Every sequenced block comes with its perception.
        let at = move |x| self.store.perception_at(x);
        let s = self.sequence(c);
        s.blues
            .into_iter()
            .map(move |x| SequencedBlock::Blue(at(x)))
            .chain(s.reds.into_iter().map(move |x| SequencedBlock::Red(at(x))))
            .chain([SequencedBlock::Chain(at(c))])
    }

    /// Returns the parents of `block` as the rules see them: the selected parent, and the folded
    /// parents that are not in its past, deduplicated and by id, so that nothing depends on the
    /// listing order.
    fn parents(&self, block: &Block<S::Id, S::Work>) -> (BlockAddress, Vec<BlockAddress>) {
        // A folded parent in the selected parent's past adds nothing and is dropped.
        let topology = self.store.topology();
        let address = |id| topology.address(id).expect("a known parent");
        let s = address(block.selected_parent);
        let mut folded: Vec<(S::Id, BlockAddress)> = block
            .folded
            .iter()
            .map(|&p| (p, address(p)))
            .filter(|&(_, p)| p != s && !topology.is_ancestor(p, s))
            .collect();

        // The rest by id, each once.
        folded.sort_unstable();
        folded.dedup();
        (s, folded.into_iter().map(|(_, p)| p).collect())
    }

    /// Returns the check a folded parent of a block on `s` must pass: it may not outrank `s`, and
    /// its chain must contain `s`'s finality point.
    fn folded_parent(
        &self,
        s: BlockAddress,
    ) -> impl Fn(BlockAddress) -> Result<(), DagError<S::Id, S::Work>> + '_ {
        // What the check compares against is read once.
        let topology = self.store.topology();
        let selected = self.store.perception_at(s);
        let final_point = topology.final_point(s);
        move |p| {
            // Not heavier than the selected parent: a block extends the heaviest parent it lists.
            let vertex = topology.vertex(p);
            if self.store.perception_at(p) > selected {
                return Err(DagError::Heavier {
                    selected: selected.id,
                    heavier: vertex.id,
                });
            }

            // On a chain through the finality point: what forks off below it enters only through
            // a block that agrees on finality, and only as past work.
            if !topology.on_chain(final_point, p) {
                return Err(DagError::BelowFinality { folded: vertex.id });
            }
            Ok(())
        }
    }

    /// Admits `block` on its selected parent `s`, with `s_perception`, folding `folded`: returns
    /// its mergeset with every merged block's branch off `s`'s chain, or why the rules refuse it.
    fn admit(
        &self,
        block: &Block<S::Id, S::Work>,
        s: BlockAddress,
        s_perception: &BlockPerception<S::Id, S::Work, S::Network>,
        folded: &[BlockAddress],
        pruning_height: Height,
    ) -> AdmitResult<S> {
        let topology = self.store.topology();

        // It must carry the work its selected parent requires.
        let required = s_perception.network.block_work();
        if block.work != required {
            return Err(DagError::Work {
                required,
                carried: block.work,
            });
        }

        // In one pass over the folded parents: each must pass its check, and the block may be
        // stamped no earlier than any of them, for a block stamped back can list nothing stamped
        // since.
        let check = self.folded_parent(s);
        let mut latest = s_perception.time;
        for &p in folded {
            check(p)?;
            latest = latest.max(topology.vertex(p).data.time);
        }
        if block.time < latest {
            return Err(DagError::Time {
                parent: latest,
                time: block.time,
            });
        }

        // Everything it folds must meet `s`'s chain at or above `s`'s pruning point, the finality
        // point's own finality point: nothing below it is ever read again.
        let mergeset = topology.mergeset(s, folded.iter().copied());
        let branches = topology.branches(s, &mergeset, pruning_height);
        if let Some(y) = Self::forks_below(&mergeset, &branches) {
            return Err(DagError::BelowPruning {
                folded: topology.vertex(y).id,
            });
        }

        Ok((mergeset, branches.into_iter().flatten().collect()))
    }

    /// Returns the first block of `mergeset` without one of its `branches`: its chain forks off below
    /// the floor they were found down to.
    fn forks_below(mergeset: &Mergeset, branches: &[Option<BlockAddress>]) -> Option<BlockAddress> {
        mergeset
            .addresses()
            .iter()
            .zip(branches)
            .find_map(|(&x, branch)| branch.is_none().then_some(x))
    }

    /// Returns what a block on `s`, with `s_perception`, merges of the blocks in its `mergeset`, in
    /// address order, each leaving `s`'s chain at its one of `branches`: all the topology tells
    /// the block's perception.
    fn merged(
        &self,
        s_perception: &BlockPerception<S::Id, S::Work, S::Network>,
        mergeset: &Mergeset,
        branches: &[BlockAddress],
    ) -> Vec<MergedBlock<S::Id, S::Work>> {
        // The mergeset's work along every lane, summed once for all its blocks.
        let topology = self.store.topology();
        let lane_work = self.lane_work(mergeset);
        mergeset
            .addresses()
            .iter()
            .zip(branches)
            .map(|(&x, &rival)| {
                // The block, the rival it backs, and the join right below the rival.
                let vertex = topology.vertex(x);
                let branch = topology.vertex(rival);
                let join = rival.height - 1;

                // The block had missed the part of `s`'s past that is not in its own: its past,
                // less the part outside `s`'s, is what it shares with `s`. That part is the block
                // and the merged blocks in its past, the first ones of every lane up to its reach.
                let outside = mergeset
                    .within(vertex)
                    .zip(&lane_work)
                    .fold(S::Work::zero(), |a, (n, sums)| a + sums[n]);

                // The depth is the chain work above the join: the rival's selected parent.
                MergedBlock {
                    id: vertex.id,
                    join,
                    rival: branch.id,
                    work: vertex.data.work,
                    depth: vertex.data.chain_work - (branch.data.chain_work - branch.data.work),
                    missed: s_perception.past_work - (vertex.data.past_work - outside),
                    settled: vertex.data.folding_threshold > join,
                }
            })
            .collect()
    }

    /// Returns, per lane the `mergeset` meets, the work of its first blocks there, from none to
    /// all.
    fn lane_work(&self, mergeset: &Mergeset) -> Vec<Vec<S::Work>> {
        // Per lane, the running sums of its run: `sums[n]` is the work of the first `n` blocks.
        let topology = self.store.topology();
        mergeset
            .lanes()
            .map(|run| {
                let mut sums = vec![S::Work::zero()];
                for &x in run {
                    sums.push(*sums.last().expect("a sum") + topology.vertex(x).data.work);
                }
                sums
            })
            .collect()
    }
}

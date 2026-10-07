use std::collections::VecDeque;
use std::fmt;
use std::sync::Arc;

use daglight_protocol_block::{BlockId, Work};
use daglight_protocol_topology::Height;

use crate::{Chunk, MergedBlock, MergedPerception, NetworkPerception, Rivals, CHUNK};

/// What a block knows of the forks along its chain: one number for the forks closed for good below
/// the folding threshold, and the rivals recorded and held at every open fork above it. The open
/// forks are kept in chunks of consecutive heights, shared with the ancestors that hold them
/// unchanged: a child copies the chunk pointers, a write copies one chunk.
#[derive(Clone)]
pub struct ForksPerception<Id, W> {
    /// The lowest chain height whose fork is still open: a merged block that joins below it is red,
    /// no record is kept below it, and the network is learned from below it.
    folding_threshold: Height,
    /// The contested work at the forks folded for good.
    folded: W,
    /// The first height of the first chunk.
    base: Height,
    /// The chunks from `base` upwards; the first is never empty.
    chunks: VecDeque<Arc<Chunk<Id, W>>>,
    /// All work behind all rivals of all forks, kept with every record and fold.
    weight: W,
    /// All contested work held foldable at all forks, kept with every hold and fold.
    held: W,
}

/// No open forks.
impl<Id, W: Work> Default for ForksPerception<Id, W> {
    fn default() -> Self {
        Self {
            folding_threshold: 0,
            folded: W::zero(),
            base: 0,
            chunks: VecDeque::new(),
            weight: W::zero(),
            held: W::zero(),
        }
    }
}

impl<Id, W> ForksPerception<Id, W> {
    /// The number of heights per chunk.
    const CHUNK: Height = CHUNK as Height;

    /// Iterates over the open forks by height, lowest first, with their rivals.
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = (Height, &Rivals<Id, W>)> {
        self.chunks.iter().enumerate().flat_map(move |(c, chunk)| {
            chunk
                .iter()
                .enumerate()
                .filter_map(move |(i, fork)| fork.as_deref().map(|f| (self.height_at(c, i), f)))
        })
    }

    /// Returns the height at place `i` of chunk `c`.
    fn height_at(&self, c: usize, i: usize) -> Height {
        self.base + c as Height * Self::CHUNK + i as Height
    }
}

impl<Id: BlockId, W: Work> ForksPerception<Id, W> {
    /// Derives a block's forks from its `merge`: the parent's forks with the credited merged blocks
    /// recorded, the forks below the pruning point dropped, the decided ones closed for good, the
    /// ones older than a horizon closed as judged, and the ones left open judged.
    pub(crate) fn derive<N: NetworkPerception<W>>(merge: &MergedPerception<'_, Id, W, N>) -> Self {
        let mut forks = merge.parent.forks.clone();
        for m in merge.credited() {
            forks.record(m);
        }
        // Nothing below the pruning point is read again, two horizons past anything a lineage can
        // still adopt.
        forks.drop_below(merge.pruning_height);
        forks.decide(merge);
        forks.expire(merge);
        forks.judge(merge);
        forks
    }

    /// Returns the folding threshold.
    pub fn folding_threshold(&self) -> Height {
        self.folding_threshold
    }

    /// Returns the contested work at the forks folded for good.
    pub fn folded(&self) -> W {
        self.folded
    }

    /// Returns the work acknowledged from other lineages: folded for good, and held foldable at the
    /// forks still open.
    pub fn acknowledged(&self) -> W {
        self.folded + self.held
    }

    /// Closes, from the folding threshold up, every fork that is decided: its contested work is
    /// folded, its settled work is dropped with the record. The side is empty at the parent's own
    /// height, so deciding stops there at the latest.
    fn decide<N: NetworkPerception<W>>(&mut self, merge: &MergedPerception<'_, Id, W, N>) {
        // Each fork closed brings the next one into question.
        let mut k = self.folding_threshold.max(merge.pruning_height);
        while merge.decided(k, self) {
            self.folded = self.folded + self.fold(k);
            k += 1;
        }
        self.folding_threshold = k;
    }

    /// Ends a stalemate: forces the folding threshold up through the forks older than the finality
    /// horizon, closing each as judged while the side leads every rival there outright.
    fn expire<N: NetworkPerception<W>>(&mut self, merge: &MergedPerception<'_, Id, W, N>) {
        // Up the chain while the fork block is older than the horizon.
        let (ancestors, finality) = (&merge.ancestors, merge.parent.protocol_parameters.finality);
        let mut f = self.folding_threshold;
        while f < ancestors.height() && ancestors.at(f).time + finality <= merge.block.time {
            // At the first fork the side does not lead outright, the lineage has lost it, and it
            // stays open on the scale.
            if let Some(rivals) = self.get(f) {
                if !rivals.led_outright(
                    merge.side(f, self.above(f)),
                    merge.continuation(f),
                    &merge.parent.network,
                ) {
                    break;
                }
                self.folded = self.folded + self.fold(f);
            }
            f += 1;
        }
        self.folding_threshold = f;
    }

    /// Returns where height `k` lies: its chunk and its place in it, unless below the first.
    fn locate(&self, k: Height) -> Option<(usize, usize)> {
        let offset = k.checked_sub(self.base)?;
        Some((
            (offset / Self::CHUNK) as usize,
            (offset % Self::CHUNK) as usize,
        ))
    }

    /// Returns the rivals of the fork at height `k`, if open.
    pub fn get(&self, k: Height) -> Option<&Rivals<Id, W>> {
        let (c, i) = self.locate(k)?;
        self.chunks.get(c)?[i].as_deref()
    }

    /// Returns all rival work recorded at the open forks other than the one at height `k`: above
    /// it, as no fork stays open below the heights it is asked about.
    pub(crate) fn above(&self, k: Height) -> W {
        self.weight - self.get(k).map_or_else(W::zero, Rivals::weight)
    }

    /// Records a merged block behind its rival at the height where its chain joins, opening the
    /// fork if needed.
    fn record(&mut self, merged: &MergedBlock<Id, W>) {
        self.open(merged.join)
            .record(merged.rival, merged.work, merged.settled, merged.depth);
        self.weight = self.weight + merged.work;
    }

    /// Returns the rivals of the fork at height `k`, opening it if needed: chunks are added below
    /// or above, aligned to the chunk size.
    fn open(&mut self, k: Height) -> &mut Rivals<Id, W> {
        // The first chunk is aligned to the height; chunks below the base are added in front.
        if self.chunks.is_empty() {
            self.base = k - k % Self::CHUNK;
        }
        while k < self.base {
            self.chunks.push_front(Arc::default());
            self.base -= Self::CHUNK;
        }

        // Chunks up to the height are added behind.
        let (c, i) = self.locate(k).expect("a height at or above the base");
        while self.chunks.len() <= c {
            self.chunks.push_back(Arc::default());
        }

        // The chunk and the fork are copied if shared with an ancestor.
        let fork = &mut Arc::make_mut(&mut self.chunks[c])[i];
        Arc::make_mut(fork.get_or_insert_with(Arc::default))
    }

    /// Removes the fork at height `k` and returns the contested work behind its rivals.
    fn fold(&mut self, k: Height) -> W {
        // A fork that is not open folds nothing.
        let Some((c, i)) = self.locate(k).filter(|_| self.get(k).is_some()) else {
            return W::zero();
        };

        // The fork leaves the chunk, and the totals.
        let fork = Arc::make_mut(&mut self.chunks[c])[i]
            .take()
            .expect("an open fork");
        self.weight = self.weight - fork.weight();
        self.held = self.held - fork.held();

        // Emptied chunks at the bottom go.
        while self
            .chunks
            .front()
            .is_some_and(|chunk| chunk.iter().all(Option::is_none))
        {
            self.chunks.pop_front();
            self.base += Self::CHUNK;
        }
        fork.contested()
    }

    /// Judges every fork left open: holds the contested work of every rival the side leads, and
    /// keeps what the lineage held before, since listing a heavier rival never takes back a hold.
    fn judge<N: NetworkPerception<W>>(&mut self, merge: &MergedPerception<'_, Id, W, N>) {
        // From the top down: the side above a fork is the chain above it plus the rivals of the
        // forks passed.
        let mut rivals_above = W::zero();
        let mut raised = Vec::new();
        for (k, rivals) in self.iter().rev() {
            raised.push((
                k,
                rivals.foldable(
                    merge.side(k, rivals_above),
                    merge.continuation(k),
                    &merge.parent.network,
                ),
            ));
            rivals_above = rivals_above + rivals.weight();
        }

        // The lineage holds what it now has foldable.
        for (k, work) in raised {
            self.hold(k, work);
        }
    }

    /// Holds `work` foldable at the open fork at height `k`, if more than held so far; a fork it
    /// does not raise stays shared with the ancestors.
    fn hold(&mut self, k: Height, work: W) {
        // Nothing to raise: the fork stays as it is, shared or not.
        if self.get(k).is_none_or(|rivals| work <= rivals.held()) {
            return;
        }

        // The chunk and the fork are copied if shared with an ancestor, and the hold raised.
        let (c, i) = self.locate(k).expect("an open fork");
        let fork = Arc::make_mut(&mut self.chunks[c])[i]
            .as_mut()
            .expect("an open fork");
        self.held = self.held + Arc::make_mut(fork).hold(work);
    }

    /// Drops the forks below height `k`: what the lineage held there stays acknowledged, as
    /// folded, so weight never falls; the rest never becomes weight.
    fn drop_below(&mut self, k: Height) {
        // The open forks below, with what is held at each.
        let below: Vec<(Height, W)> = self
            .iter()
            .take_while(|&(h, _)| h < k)
            .map(|(h, rivals)| (h, rivals.held()))
            .collect();

        // Each is folded for what it held, not for all it contested.
        for (h, held) in below {
            self.fold(h);
            self.folded = self.folded + held;
        }
    }

    /// Returns all contested work held foldable at the open forks.
    pub fn held(&self) -> W {
        self.held
    }

    /// Iterates over the rivals of the open forks, lowest first.
    pub fn values(&self) -> impl Iterator<Item = &Rivals<Id, W>> {
        self.iter().map(|(_, f)| f)
    }

    /// Returns whether no fork is open: every fork records some work.
    pub fn is_empty(&self) -> bool {
        self.weight == W::zero()
    }
}

/// Equal when the same forks are closed and the same are open at the same heights, however they
/// are chunked.
impl<Id: PartialEq, W: PartialEq> PartialEq for ForksPerception<Id, W> {
    fn eq(&self, other: &Self) -> bool {
        self.folding_threshold == other.folding_threshold
            && self.folded == other.folded
            && self.weight == other.weight
            && self.iter().eq(other.iter())
    }
}

impl<Id: Eq, W: Eq> Eq for ForksPerception<Id, W> {}

/// Printed as a map from height to rivals.
impl<Id: fmt::Debug, W: fmt::Debug> fmt::Debug for ForksPerception<Id, W> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

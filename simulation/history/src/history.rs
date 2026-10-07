use std::collections::HashMap;
use std::iter::once;

use daglight_protocol_dag::DagStore;
use daglight_protocol_topology::BlockAddress;

use crate::{FinalChain, MinedBlock};

/// What a run records along the way: who mined what when, how honest miners switched tips, and
/// what became final before it could be pruned.
#[derive(Clone, Debug)]
pub struct History {
    /// Every block mined, the genesis aside.
    pub blocks: HashMap<u64, MinedBlock>,
    /// The blocks in the order they were mined.
    pub order: Vec<u64>,
    /// The seconds between every block and the chain block at its folding threshold, summed.
    pub folding_lag: f64,
    /// Honest miner 0's final chain as far as recorded.
    pub final_chain: FinalChain,
    /// The most chain work an honest miner's switch of selected parent reverted.
    pub max_reorg: u64,
    /// The honest switches that reverted a height below the miner's folding threshold.
    pub folded_reorgs: usize,
    /// Each honest miner's last selected parent.
    last_tip: Vec<Option<u64>>,
}

impl History {
    /// Creates an empty history for `miners` miners.
    pub fn new(miners: usize) -> Self {
        Self {
            blocks: HashMap::new(),
            order: Vec::new(),
            folding_lag: 0.0,
            final_chain: FinalChain::default(),
            max_reorg: 0,
            folded_reorgs: 0,
            last_tip: vec![None; miners],
        }
    }

    /// Returns every block's number in the order mined, the genesis 0.
    pub fn numbering(&self) -> HashMap<u64, usize> {
        once(0)
            .chain(self.order.iter().copied())
            .enumerate()
            .map(|(i, id)| (id, i))
            .collect()
    }

    /// Returns when block `id` was mined, the genesis at 0.
    pub fn time_of(&self, id: u64) -> f64 {
        self.blocks.get(&id).map_or(0.0, |b| b.time)
    }

    /// Records that `miner` mined the block placed at `address` at `time`, with its work and how far
    /// back its folding threshold lies.
    pub fn mined<S: DagStore<Id = u64, Work = u64>>(
        &mut self,
        address: BlockAddress,
        time: f64,
        miner: usize,
        store: &S,
    ) {
        // How far back the block's folding threshold lies, in time.
        let topology = store.topology();
        let vertex = topology.vertex(address);
        let folding_threshold = topology.ancestor(address, vertex.data.folding_threshold);
        self.folding_lag += time - self.time_of(topology.vertex(folding_threshold).id);

        // The block itself.
        self.blocks.insert(
            vertex.id,
            MinedBlock {
                time,
                miner,
                work: vertex.data.work,
            },
        );
        self.order.push(vertex.id);
    }

    /// Records that honest miner `who` built on `tip`, measuring any switch from its last tip.
    pub fn chose<S: DagStore<Id = u64, Work = u64>>(&mut self, who: usize, tip: u64, store: &S) {
        if let Some(last) = self.last_tip[who] {
            self.switched(store, store.address(last), store.address(tip));
        }
        self.last_tip[who] = Some(tip);
    }

    /// Measures an honest miner's move from tip `from` to `to`: a switch off `from`'s chain reverts
    /// `from`'s chain work above the join, and is folded if the join lies below `from`'s folding
    /// threshold.
    fn switched<S: DagStore<Id = u64, Work = u64>>(
        &mut self,
        store: &S,
        from: BlockAddress,
        to: BlockAddress,
    ) {
        // Building on, or past, the last tip reverts nothing.
        let topology = store.topology();
        if from == to || topology.is_ancestor(from, to) {
            return;
        }

        // The chain work above the join is reverted; below the threshold, that is a folded reorg.
        let join = topology.join(from, to).expect("a switch above finality");
        let (left, joined) = (&topology.vertex(from).data, &topology.vertex(join).data);
        self.max_reorg = self.max_reorg.max(left.chain_work - joined.chain_work);
        if join.height < left.folding_threshold {
            self.folded_reorgs += 1;
        }
    }
}

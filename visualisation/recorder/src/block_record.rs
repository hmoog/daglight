use daglight_protocol_block_perception::NetworkPerception;
use daglight_protocol_dag::{Dag, DagStore};
use daglight_protocol_store::MemoryStore;
use daglight_protocol_topology::Height;

/// A block as the player shows it: who mined it when, its parents, and what every node derives
/// for it. Blocks are numbered in the order they were mined, the genesis 0.
#[derive(Clone, Debug, PartialEq)]
pub struct BlockRecord {
    /// The block's number.
    pub index: usize,
    /// The miner that mined it; none for the genesis.
    pub miner: Option<usize>,
    /// When it was mined, in seconds.
    pub time: f64,
    /// Its parents' numbers, the selected one first.
    pub parents: Vec<usize>,
    /// Its own work.
    pub work: u64,
    /// Its blue work.
    pub blue_work: u64,
    /// All work in its past, its own included.
    pub past_work: u64,
    /// Its height on its own chain.
    pub height: Height,
    /// Its folding threshold: the lowest height on its chain whose fork is still open.
    pub folding_threshold: Height,
    /// The height of its finality point.
    pub final_height: Height,
    /// The delay it learned, in milliseconds.
    pub delay: u64,
    /// The topology lane it extends.
    pub lane: u32,
}

impl BlockRecord {
    /// Returns the record of block `id`, numbered `index`, as `dag` derived it, mined by `miner` at
    /// `time` on the parents numbered `parents`.
    pub fn of(
        dag: &Dag<MemoryStore<u64, u64>>,
        id: u64,
        index: usize,
        miner: Option<usize>,
        time: f64,
        parents: Vec<usize>,
    ) -> Self {
        let store = dag.store();
        let perception = store.perception(id);
        let address = store.address(id);
        Self {
            index,
            miner,
            time,
            parents,
            work: perception.work,
            blue_work: perception.blue_work(),
            past_work: perception.past_work,
            height: address.height,
            folding_threshold: perception.forks.folding_threshold(),
            final_height: perception.final_height,
            delay: perception
                .network
                .delay_time(&perception.protocol_parameters),
            lane: store.topology().vertex(address).lane,
        }
    }

    /// Returns the record as a JSON object.
    pub fn json(&self) -> String {
        let parents: Vec<String> = self.parents.iter().map(usize::to_string).collect();
        format!(
            "{{\"i\":{},\"miner\":{},\"t\":{:.3},\"parents\":[{}],\"work\":{},\"blueWork\":{},\"past\":{},\"height\":{},\"folding\":{},\"finalHeight\":{},\"delay\":{},\"lane\":{}}}",
            self.index,
            self.miner.map_or(-1, |m| m as i64),
            self.time,
            parents.join(","),
            self.work,
            self.blue_work,
            self.past_work,
            self.height,
            self.folding_threshold,
            self.final_height,
            self.delay,
            self.lane
        )
    }
}

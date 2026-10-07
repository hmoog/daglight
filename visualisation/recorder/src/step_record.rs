use daglight_protocol_dag::{Dag, DagStore};
use daglight_protocol_node::Node;
use daglight_protocol_store::MemoryStore;

use crate::UpdateRecord;

/// A node's view once one more block is in, with blocks by number.
#[derive(Clone, Debug, PartialEq)]
pub struct StepRecord {
    /// The block that came in.
    pub block: usize,
    /// When it was mined, in seconds.
    pub time: f64,
    /// The node's tips.
    pub tips: Vec<usize>,
    /// The heaviest tip: the one it builds on and follows.
    pub perception: usize,
    /// The folding threshold on the perception's chain.
    pub folding_threshold: usize,
    /// The finality point.
    pub final_point: usize,
    /// The pruning point.
    pub pruning_point: usize,
    /// What it reported.
    pub updates: Vec<UpdateRecord>,
}

impl StepRecord {
    /// Returns `node`'s view once block number `block`, mined at `time`, is in, with blocks
    /// numbered by `number`.
    pub fn of(
        node: &mut Node<Dag<MemoryStore<u64, u64>>>,
        block: usize,
        time: f64,
        number: &impl Fn(u64) -> usize,
    ) -> Self {
        // What the node reported since the last step.
        let updates = node
            .updates()
            .into_iter()
            .map(|u| UpdateRecord::of(u, number))
            .collect();

        // The cut-offs, read off the heaviest tip's chain.
        let store = node.dag().store();
        let topology = store.topology();
        let tip = store.address(node.heaviest());
        let at = |s| number(topology.vertex(s).id);
        Self {
            block,
            time,
            tips: node.tips().into_iter().map(number).collect(),
            perception: number(node.heaviest()),
            folding_threshold: at(
                topology.ancestor(tip, topology.vertex(tip).data.folding_threshold)
            ),
            final_point: at(node.final_point()),
            pruning_point: at(topology.pruning_point(tip)),
            updates,
        }
    }

    /// Returns the step as a JSON object.
    pub fn json(&self) -> String {
        let tips: Vec<String> = self.tips.iter().map(usize::to_string).collect();
        let updates: Vec<String> = self.updates.iter().map(UpdateRecord::json).collect();
        format!(
            "{{\"block\":{},\"t\":{:.3},\"tips\":[{}],\"perception\":{},\"folding\":{},\"final\":{},\"pruning\":{},\"updates\":[{}]}}",
            self.block,
            self.time,
            tips.join(","),
            self.perception,
            self.folding_threshold,
            self.final_point,
            self.pruning_point,
            updates.join(",")
        )
    }
}

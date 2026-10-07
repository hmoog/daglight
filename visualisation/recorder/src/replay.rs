use daglight_protocol_block::{Block, BlockId};
use daglight_protocol_dag::{Dag, DagStore};
use daglight_protocol_node::{DagHandle, Node};
use daglight_simulation_network::Simulation;

use crate::{BlockRecord, Scenario, StepRecord};

/// A recorded run: the scenario, every block, and every step of a node that receives each block
/// the moment it is mined and is followed throughout.
#[derive(Clone, Debug)]
pub struct Replay {
    /// The scenario run.
    pub scenario: Scenario,
    /// The attacker's miner number, if there is one.
    pub attacker: Option<usize>,
    /// Every block, in the order mined.
    pub blocks: Vec<BlockRecord>,
    /// The node's view after each block.
    pub steps: Vec<StepRecord>,
}

impl Replay {
    /// Runs `scenario` and records it.
    pub fn record(scenario: &Scenario) -> Self {
        // The run itself.
        let config = &scenario.config;
        let mut sim = Simulation::new(config.clone());
        sim.run();

        // Blocks are numbered in the order they were mined, the genesis 0.
        let history = &sim.history;
        let number = history.numbering();
        let n = |id: u64| number[&id];

        // The blocks as they were mined, and a node of its own that receives each at once.
        let mined_blocks: Vec<Block<u64, u64>> = sim.dag().with(|dag| {
            history
                .order
                .iter()
                .map(|&id| dag.store().block(id))
                .collect()
        });
        let mut node = Node::new(Dag::new(config.store()));
        node.follow();
        let mut blocks = vec![BlockRecord::of(
            node.dag(),
            u64::GENESIS,
            0,
            None,
            0.0,
            Vec::new(),
        )];
        // Each block enters the node, which records the block and the node's view after it.
        let mut steps = Vec::new();
        for block in mined_blocks {
            let (id, mined) = (block.id, history.blocks[&block.id]);
            let parents = block.parents().map(n).collect();
            node.add_block(block).expect("a mined block is valid");
            blocks.push(BlockRecord::of(
                node.dag(),
                id,
                n(id),
                Some(mined.miner),
                mined.time,
                parents,
            ));
            steps.push(StepRecord::of(&mut node, n(id), mined.time, &n));
        }
        Self {
            scenario: scenario.clone(),
            attacker: config.attacker_index(),
            blocks,
            steps,
        }
    }

    /// Returns the replay as a JSON object.
    pub fn json(&self) -> String {
        let config = &self.scenario.config;
        let blocks: Vec<String> = self.blocks.iter().map(BlockRecord::json).collect();
        let steps: Vec<String> = self.steps.iter().map(StepRecord::json).collect();
        format!(
            "{{\"id\":\"{}\",\"label\":\"{}\",\"rate\":{},\"delay\":{},\"miners\":{},\"seed\":{},\"attacker\":{},\"blocks\":[{}],\"frames\":[{}]}}",
            self.scenario.id,
            self.scenario.label,
            config.rate,
            config.delay,
            config.shares.len(),
            config.seed,
            self.attacker.map_or(-1, |a| a as i64),
            blocks.join(","),
            steps.join(",")
        )
    }
}

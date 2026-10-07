use std::mem::forget;

use daglight_protocol_block_perception::{
    BlockPerception, FixedNetworkPerception, LearnedNetworkPerception, NetworkPerception,
};
use daglight_protocol_dag::DagStore;
use daglight_protocol_node::DagHandle;
use daglight_simulation_config::Config;
use daglight_simulation_history::FinalChain;
use daglight_simulation_network::Simulation;

/// The numbers a finished run is judged by.
#[derive(Clone, Debug, Default)]
pub struct Metrics {
    /// The number of blocks mined.
    pub blocks: usize,
    /// The length of the final selected chain.
    pub chain_len: usize,
    /// The share of honest work, older than three delays, that the final chain never counts.
    pub honest_red: f64,
    /// The contested work at the closing block's open forks that is not yet foldable, in delays.
    pub pending: f64,
    /// The settled work at the closing block's open forks, in delays.
    pub grey: f64,
    /// The mean seconds between a block and the chain block at its folding threshold.
    pub folding_lag: f64,
    /// The most chain work an honest miner's switch of selected parent reverted, in delays.
    pub max_reorg: f64,
    /// The honest switches that reverted a height below the miner's folding threshold.
    pub folded_reorgs: usize,
    /// The delay the closing block has learned, in seconds.
    pub delay: f64,
    /// The closing block's delay work, in delays of work at the true delay and hash rate.
    pub yardstick: f64,
    /// The number of blocks the attacker mined.
    pub attacker_blocks: usize,
    /// The attacker's share of the final chain's work.
    pub attacker_on_chain: f64,
    /// The number of reveals.
    pub reveals: usize,
    /// The time of the first reveal.
    pub first_reveal: Option<f64>,
    /// How often archived perceptions were loaded back; not a column.
    pub archive_loads: usize,
    /// The topology's lanes; not a column.
    pub lanes: usize,
}

impl Metrics {
    /// Runs the network of `config` on the network perception it names, learned or fixed, and
    /// returns what it measured.
    pub fn outcome(config: Config) -> Self {
        if config.fixed {
            Self::measured::<FixedNetworkPerception<u64>>(config)
        } else {
            Self::measured::<LearnedNetworkPerception<u64>>(config)
        }
    }

    /// Runs the network of `config` with network perception `N` and returns what it measured.
    fn measured<N: NetworkPerception<u64>>(config: Config) -> Self {
        let mut sim = Simulation::<N>::start(config);
        sim.run();
        let metrics = Self::measure(&sim);

        // Freeing every block would only cost time.
        forget(sim);
        metrics
    }

    /// Returns the printed columns as name and value.
    fn columns(&self) -> Vec<(&'static str, String)> {
        vec![
            ("blocks", self.blocks.to_string()),
            ("chain", self.chain_len.to_string()),
            ("hred", format!("{:.2}%", 100.0 * self.honest_red)),
            ("pend", format!("{:.2}", self.pending)),
            ("grey", format!("{:.2}", self.grey)),
            ("folding", format!("{:.2}s", self.folding_lag)),
            ("reorg", format!("{:.2}", self.max_reorg)),
            ("fold_reorg", self.folded_reorgs.to_string()),
            ("delay", format!("{:.2}s", self.delay)),
            ("yard", format!("{:.2}", self.yardstick)),
            ("att", self.attacker_blocks.to_string()),
            ("att_chn", format!("{:.1}%", 100.0 * self.attacker_on_chain)),
            (
                "reveal",
                self.first_reveal
                    .map_or("-".to_string(), |t| format!("{}@{t:.0}s", self.reveals)),
            ),
        ]
    }

    /// Returns the column names as one line.
    pub fn header(&self) -> String {
        self.line(|name, _| name)
    }

    /// Returns the values as one line.
    pub fn row(&self) -> String {
        self.line(|_, value| value)
    }

    /// Returns one line of the columns, each `cell` of its name and value right-aligned to the
    /// column's width.
    fn line(&self, cell: impl for<'a> Fn(&'a str, &'a str) -> &'a str) -> String {
        self.columns()
            .iter()
            .map(|(name, value)| format!("{:>w$}", cell(name, value), w = name.len().max(8)))
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Measures a finished run on honest miner 0's node: what an honest node concluded, once
    /// everything published has arrived.
    pub fn measure<N: NetworkPerception<u64>>(sim: &Simulation<N>) -> Self {
        // The closing block and the final chain, and what the run recorded.
        let (chain, v) = Self::close(sim);
        let history = &sim.history;
        let attacker = sim.config.attacker_index();
        let blocks = history.order.len();
        let reveals = sim.attacker.as_ref().map_or(&[][..], |a| a.reveals());
        let (archive_loads, lanes) = sim.dag().with(|dag| {
            let store = dag.store();
            (store.archive().loads(), store.topology().lanes())
        });

        // Work is reported in delays of work at the genesis's guess.
        Metrics {
            blocks,
            chain_len: chain.blocks.len() - 1,
            honest_red: Self::red_share(sim, &chain),
            pending: Self::in_delays(
                sim,
                v.forks.values().map(|r| r.contested()).sum::<u64>() - v.forks.held(),
            ),
            grey: Self::in_delays(sim, v.forks.values().map(|r| r.settled()).sum()),
            folding_lag: history.folding_lag / blocks.max(1) as f64,
            max_reorg: Self::in_delays(sim, history.max_reorg),
            folded_reorgs: history.folded_reorgs,
            delay: v.network.delay_time(&v.protocol_parameters) as f64 / 1000.0,
            yardstick: v.network.delay_work() as f64 / (sim.hash_rate * sim.config.delay),
            attacker_blocks: history
                .blocks
                .values()
                .filter(|b| Some(b.miner) == attacker)
                .count(),
            attacker_on_chain: Self::attacker_share(sim, &chain),
            reveals: reveals.len(),
            first_reveal: reveals.first().copied(),
            archive_loads,
            lanes,
        }
    }

    /// Returns the final chain and the closing block's perception: what honest miner 0 would mine
    /// on its heaviest tip once everything published has arrived, which measures the network.
    fn close<N: NetworkPerception<u64>>(
        sim: &Simulation<N>,
    ) -> (FinalChain, BlockPerception<u64, u64, N>) {
        // The closing block on honest miner 0's heaviest tip, stamped no earlier than any tip.
        let node = &sim.miners[0].node;
        let end = Config::millis(sim.end);
        let top = node.heaviest();
        let closing = node.block_on(0, top, Self::latest_stamp(sim).max(end));
        sim.dag().with(|dag| {
            // The chain runs as recorded, then up to the tip, and credits also what the closing
            // block merges: honest blocks that only lack a block after them.
            let merge = dag
                .merge(&closing)
                .expect("a block on the heaviest tip is valid");
            let mut chain = sim.history.final_chain.clone();
            chain.extend_to(dag, dag.store().address(top));
            chain.credited.extend(merge.credited().map(|m| m.id));
            (chain, BlockPerception::derive(&merge))
        })
    }

    /// Returns the latest stamp among honest miner 0's tips.
    fn latest_stamp<N: NetworkPerception<u64>>(sim: &Simulation<N>) -> u64 {
        let node = &sim.miners[0].node;
        node.tips()
            .into_iter()
            .map(|t| node.perception(t).time)
            .max()
            .expect("a node always has a tip")
    }

    /// Returns the attacker's share of the work of the final `chain`'s blocks, the genesis aside.
    fn attacker_share<N: NetworkPerception<u64>>(sim: &Simulation<N>, chain: &FinalChain) -> f64 {
        // The attacker's work against all work on the chain.
        let attacker = sim.config.attacker_index();
        let (mut own, mut all) = (0u64, 0u64);
        for c in &chain.blocks[1..] {
            let b = sim.history.blocks[c];
            all += b.work;
            if Some(b.miner) == attacker {
                own += b.work;
            }
        }
        own as f64 / all.max(1) as f64
    }

    /// Returns `work` in delays of work.
    fn in_delays<N: NetworkPerception<u64>>(sim: &Simulation<N>, work: u64) -> f64 {
        work as f64 / sim.config.delay_work() as f64
    }

    /// Returns the share of honest work, older than three delays, that the final `chain` and a
    /// block on its tip do not credit.
    fn red_share<N: NetworkPerception<u64>>(sim: &Simulation<N>, chain: &FinalChain) -> f64 {
        // Younger blocks may still be in flight and are not judged.
        let attacker = sim.config.attacker_index();
        let (mut red, mut all) = (0u64, 0u64);
        for (id, b) in &sim.history.blocks {
            if b.time <= sim.end - 3.0 * sim.config.delay && Some(b.miner) != attacker {
                all += b.work;
                if !chain.credited.contains(id) {
                    red += b.work;
                }
            }
        }
        red as f64 / all.max(1) as f64
    }
}

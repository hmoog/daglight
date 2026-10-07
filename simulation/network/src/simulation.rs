use std::collections::BinaryHeap;

use daglight_protocol_block::Block;
use daglight_protocol_block_perception::{LearnedNetworkPerception, NetworkPerception};
use daglight_protocol_dag::{Dag, DagStore};
use daglight_protocol_node::{DagHandle, Node, SharedDag};
use daglight_protocol_store::MemoryStore;
use daglight_protocol_topology::BlockAddress;

use daglight_simulation_attacker::Attacker;
use daglight_simulation_config::{Config, RateSwitch, Strategy};
use daglight_simulation_history::History;

use crate::{Event, Miner, Rng};

/// A network of miners, each a node on one shared DAG that derives every block once for all of
/// them, driven by a queue of events, on network perception `N`: learned by the blocks unless
/// given.
pub struct Simulation<N: NetworkPerception<u64> = LearnedNetworkPerception<u64>> {
    /// The network being simulated.
    pub config: Config,
    /// The DAG every miner's node holds.
    dag: SharedDag<MemoryStore<u64, u64, N>>,
    /// The miners, the attacker last.
    pub miners: Vec<Miner<N>>,
    /// The attacker, if any.
    pub attacker: Option<Attacker>,
    /// The pending events, earliest first.
    events: BinaryHeap<Event>,
    /// The source of all randomness.
    rng: Rng,
    /// The network's hash rate now, in work per second.
    pub hash_rate: f64,
    /// The rate switch still to come, if any.
    rate_switch: Option<RateSwitch>,
    /// What happened so far.
    pub history: History,
    /// The time of the last block mined or delivered.
    pub end: f64,
}

impl Simulation {
    /// Creates the network of `config`, its blocks learning the network, with the first block's
    /// mining scheduled.
    pub fn new(config: Config) -> Self {
        Self::start(config)
    }
}

impl<N: NetworkPerception<u64>> Simulation<N> {
    /// How many blocks, the genesis counted, pass between two rounds of recording what became final
    /// and releasing what no miner needs.
    const RELEASE_EVERY: usize = 16;

    /// Creates the network of `config`, with network perception `N`, and schedules the first
    /// block's mining.
    pub fn start(config: Config) -> Self {
        // Every miner's node holds the one DAG, from its genesis.
        let dag = SharedDag::new(Dag::new(config.store()));
        let miners: Vec<Miner<N>> = config
            .miner_shares()
            .into_iter()
            .map(|share| Miner {
                share,
                node: Node::new(dag.clone()),
            })
            .collect();
        // The attacker, if any, is the last miner.
        let n = miners.len();
        let attacker = config.attack.map(|a| Attacker::new(a, n - 1));
        let mut sim = Self {
            hash_rate: Config::HASH_RATE,
            rate_switch: config.rate_switch,
            rng: Rng::new(config.seed),
            config,
            dag,
            miners,
            attacker,
            events: BinaryHeap::new(),
            history: History::new(n),
            end: 0.0,
        };
        sim.schedule(0.0);
        sim
    }

    /// Returns the DAG the miners share.
    pub fn dag(&self) -> &SharedDag<MemoryStore<u64, u64, N>> {
        &self.dag
    }

    /// Runs until every event is processed.
    pub fn run(&mut self) {
        while let Some(event) = self.events.pop() {
            match event {
                Event::Mine { time } => self.next_block(time),
                Event::Deliver {
                    time,
                    miner,
                    address,
                } => self.deliver(time, miner, address),
                Event::Wake { time, miner } => self.wake(time, miner),
            }
        }
    }

    /// Mines the next block at `time`, unless past the duration, where mining stops and the
    /// attacker publishes what it still holds; then schedules the one after.
    fn next_block(&mut self, time: f64) {
        // Past the duration mining stops, and what is still withheld comes out.
        if time > self.config.duration {
            self.publish_withheld(self.config.duration);
            return;
        }
        self.end = time;

        // The hash rate switches; the difficulty follows it by itself.
        if let Some(switch) = self.rate_switch.filter(|s| time >= s.at) {
            self.hash_rate = Config::HASH_RATE * switch.rate / self.config.rate;
            self.rate_switch = None;
        }

        // A block is mined; the attacker may reveal after it.
        let own = self.mine(time);
        if self
            .attacker
            .as_mut()
            .is_some_and(|a| a.should_reveal(&self.miners[a.miner].node, time, own))
        {
            self.publish_withheld(time);
        }

        // Every so many blocks what became final is recorded and what nobody needs released.
        if (self.history.order.len() + 1).is_multiple_of(Self::RELEASE_EVERY) {
            self.release();
        }
        self.schedule(time);
    }

    /// Delivers the block at `address` to `miner` at `time`, unless it or one of its parents was
    /// pruned on its way: then it can never matter to the miner, which would wait for them forever.
    /// A block stamped beyond the miner's clock waits in its node until the clock gets there.
    fn deliver(&mut self, time: f64, miner: usize, address: BlockAddress) {
        // The block as relayed, unless pruned on its way.
        self.end = self.end.max(time);
        let Some(block) = self.relayed(address) else {
            return;
        };

        // The miner's node takes it, and whatever waited on it.
        let (now, stamp) = (Config::millis(time), block.time);
        for entered in self.miners[miner].node.receive(block, now) {
            entered.expect("a relayed block is valid");
        }

        // A block stamped ahead of the clock is taken in when the clock gets there.
        if stamp > now {
            self.events.push(Event::Wake {
                time: stamp as f64 / 1000.0,
                miner,
            });
        }
    }

    /// Lets `miner` take in, at `time`, the blocks it held back until then.
    fn wake(&mut self, time: f64, miner: usize) {
        for entered in self.miners[miner].node.tick(Config::millis(time)) {
            entered.expect("a relayed block is valid");
        }
    }

    /// Returns each miner's block rate at `time`: its hash rate over the work its next block
    /// requires, which its tip sets, or the attacker's private tip while it hides.
    fn block_rates(&self, time: f64) -> Vec<f64> {
        self.miners
            .iter()
            .enumerate()
            .map(|(i, m)| {
                // The tip the miner builds on sets the work its next block needs.
                let tip = match self.attacker.as_ref().filter(|a| a.hides(i, time)) {
                    Some(a) => a.parent(&m.node),
                    None => m.node.heaviest(),
                };
                m.share * self.hash_rate / m.node.perception(tip).network.block_work() as f64
            })
            .collect()
    }

    /// Schedules the next block after `time`, at the network's block rate then.
    fn schedule(&mut self, time: f64) {
        let rate: f64 = self.block_rates(time).iter().sum();
        self.events.push(Event::Mine {
            time: time + self.rng.exponential(rate),
        });
    }

    /// Returns a block as it is relayed, unless it or one of its parents was pruned.
    fn relayed(&self, address: BlockAddress) -> Option<Block<u64, u64>> {
        self.dag.with(|dag| {
            // The block and its parents must all be kept.
            let store = dag.store();
            let topology = store.topology();
            let vertex = topology.kept(address)?;
            if vertex.parents.iter().any(|&p| topology.kept(p).is_none()) {
                return None;
            }
            Some(store.block(vertex.id))
        })
    }

    /// Records what became final and, if nodes prune, prunes what no miner will ever need again.
    fn release(&mut self) {
        // The final chain is recorded up to honest miner 0's finality point.
        self.dag.with(|dag| {
            self.history
                .final_chain
                .extend_to(dag, self.miners[0].node.final_point())
        });

        // If nodes prune, the DAG keeps only the future of every retained point.
        if let Some(da_offset) = self.config.prune {
            let points = self.retained(Config::millis(da_offset));
            self.dag.with_mut(|dag| dag.prune(&points));
        }
    }

    /// Returns what the miners retain, keeping `da_offset` milliseconds of history beyond what
    /// they can still need: every node's retained point and every private lineage's.
    fn retained(&mut self, da_offset: u64) -> Vec<BlockAddress> {
        // Every node's point, and the attacker's private lineages'.
        let mut points: Vec<BlockAddress> = self
            .miners
            .iter_mut()
            .map(|m| m.node.retain(da_offset))
            .collect();
        if let Some(a) = &self.attacker {
            points.extend(a.retained(&self.miners[a.miner].node, da_offset));
        }
        points
    }

    /// Mines one block at `time` by a miner picked by block rate; returns whether it is withheld.
    fn mine(&mut self, time: f64) -> bool {
        // The miner builds its block honestly, or privately if it is the attacker in hiding.
        let who = self.pick_miner(time);
        let id = self.rng.block_id();
        let hidden = self.attacker.as_ref().is_some_and(|a| a.hides(who, time));
        let block = match self.attacker.as_mut().filter(|_| hidden) {
            Some(a) => a.block(&self.miners[who].node, id, a.attack.stamp(time)),
            None => self.honest_block(who, id, time),
        };

        // The miner's node derives the block in the shared DAG and takes it at once.
        self.miners[who]
            .node
            .add_block(block)
            .expect("a mined block is valid");
        let address = self.dag.with(|dag| {
            let address = dag.store().address(id);
            self.history.mined(address, time, who, dag.store());
            address
        });

        // A withheld block waits for a reveal; a balancing attacker's goes out to the half that
        // favours its side first; any other block goes out at once.
        match self.attacker.as_mut().filter(|_| hidden) {
            Some(a) if a.attack.strategy == Strategy::Balance => {
                let side = a.side_of(&self.miners[who].node, id);
                self.broadcast(address, who, time, side);
            }
            Some(a) => a.withhold(id),
            None => self.broadcast(address, who, time, None),
        }
        hidden
    }

    /// Returns the block miner `who` builds honestly, recording its choice of tip.
    fn honest_block(&mut self, who: usize, id: u64, time: f64) -> Block<u64, u64> {
        // The attacker's honest blocks, before its attack starts, are not measured as switches.
        let block = self.miners[who].node.next_block(id, Config::millis(time));
        if Some(who) != self.config.attacker_index() {
            self.dag
                .with(|dag| self.history.chose(who, block.selected_parent, dag.store()));
        }
        block
    }

    /// Broadcasts every block the attacker has withheld so far.
    fn publish_withheld(&mut self, time: f64) {
        // Nothing to publish without an attacker.
        let Some(a) = self.attacker.as_mut() else {
            return;
        };

        // A withheld block that was pruned can never be folded or built on: there is nothing to
        // reveal.
        let from = a.miner;
        let revealed = a.reveal(time);
        let addresses: Vec<BlockAddress> = self.dag.with(|dag| {
            revealed
                .into_iter()
                .filter_map(|id| dag.store().topology().address(id))
                .collect()
        });

        // The rest go out as any block does.
        for address in addresses {
            self.broadcast(address, from, time, None);
        }
    }

    /// Sends a block from miner `from` to every other miner, each with its own delay; a balancing
    /// attacker's block on `side` reaches the half of the honest miners that does not favour the
    /// side a delay later.
    fn broadcast(&mut self, address: BlockAddress, from: usize, time: f64, side: Option<usize>) {
        for other in 0..self.miners.len() {
            if other != from {
                // The delay, jittered; the disfavoured half waits one more.
                let u = 2.0 * self.rng.uniform() - 1.0;
                let delay = self.config.delay * (1.0 + self.config.jitter * u).max(0.0);
                let late = side.is_some_and(|s| other % 2 != s);
                self.events.push(Event::Deliver {
                    time: time + delay + if late { self.config.delay } else { 0.0 },
                    miner: other,
                    address,
                });
            }
        }
    }

    /// Returns a miner picked at `time` with probability proportional to its block rate.
    fn pick_miner(&mut self, time: f64) -> usize {
        // A point on the line of rates laid end to end; the last miner catches rounding.
        let rates = self.block_rates(time);
        let mut u = self.rng.uniform() * rates.iter().sum::<f64>();
        for (i, r) in rates.iter().enumerate() {
            if u < *r {
                return i;
            }
            u -= r;
        }
        self.miners.len() - 1
    }
}

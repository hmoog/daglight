use std::cmp::Reverse;
use std::mem::take;

use daglight_protocol_block::Block;
use daglight_protocol_block_perception::{BlockPerception, NetworkPerception};
use daglight_protocol_dag::DagStore;
use daglight_protocol_node::DagHandle;
use daglight_protocol_topology::BlockAddress;

use daglight_simulation_config::{Attack, MinerNode, Reveal, Strategy};

/// The miner running an attack: it mines private lineages and withholds its blocks until a reveal,
/// or, balancing, builds two sides and shows each block first to the honest half that favours it.
#[derive(Clone, Debug)]
pub struct Attacker {
    /// The attack it runs.
    pub attack: Attack,
    /// Its index among the miners.
    pub miner: usize,
    /// The blocks mined since the last reveal.
    withheld: Vec<u64>,
    /// The tip of each private lineage, `None` until its first block.
    lineage_tips: Vec<Option<u64>>,
    /// The lineage the next block extends.
    turn: usize,
    /// The times of all reveals so far.
    reveals: Vec<f64>,
    /// The time of the next periodic reveal; infinite if none is scheduled.
    next_reveal: f64,
    /// The heaviest foreign tip of its view at each of its recent blocks, oldest first.
    seen: Vec<u64>,
    /// A balancing attacker's two sides, by the first block of each; the first alone, with its
    /// parent, until the second is mined.
    sides: Vec<u64>,
    /// The parent of a balancing attacker's two first blocks.
    split_at: Option<u64>,
}

impl Attacker {
    /// How many recent foreign tips a greedy attacker keeps as candidates.
    const SEEN: usize = 8;

    /// Creates the attacker for `attack` as miner number `miner`.
    pub fn new(attack: Attack, miner: usize) -> Self {
        // Only a periodic reveal is scheduled; only a private DAG runs two lineages.
        let next_reveal = match attack.reveal {
            Reveal::Every(t) => t,
            Reveal::Immediately | Reveal::WhenAhead => f64::INFINITY,
        };
        let lineages = match attack.strategy {
            Strategy::Withhold | Strategy::Harvest | Strategy::Greedy | Strategy::Balance => 1,
            Strategy::Dag => 2,
        };
        Self {
            attack,
            miner,
            withheld: Vec::new(),
            lineage_tips: vec![None; lineages],
            turn: 0,
            reveals: Vec::new(),
            next_reveal,
            seen: Vec::new(),
            sides: Vec::new(),
            split_at: None,
        }
    }

    /// Returns whether a block by `miner` at `time` is withheld.
    pub fn hides(&self, miner: usize, time: f64) -> bool {
        miner == self.miner && time >= self.attack.start
    }

    /// Returns the block its next private block builds on, the tip of its lineage; for a balancing
    /// attacker, whose side it picks when it mines, the one whose work it must carry.
    pub fn parent<N: NetworkPerception<u64>>(&self, node: &MinerNode<N>) -> u64 {
        self.lineage_tips[self.turn].unwrap_or_else(|| node.heaviest())
    }

    /// Returns what its private lineages still need, keeping `da_offset` milliseconds of history
    /// beyond: every lineage tip's retention point.
    pub fn retained<N: NetworkPerception<u64>>(
        &self,
        node: &MinerNode<N>,
        da_offset: u64,
    ) -> Vec<BlockAddress> {
        node.dag().with(|dag| {
            let store = dag.store();
            self.own_tips()
                .map(|tip| {
                    store
                        .topology()
                        .retention_point(store.address(tip), da_offset)
                })
                .collect()
        })
    }

    /// Returns the side of a balancing attacker a block is on, if any.
    pub fn side_of<N: NetworkPerception<u64>>(
        &self,
        node: &MinerNode<N>,
        id: u64,
    ) -> Option<usize> {
        node.dag().with(|dag| {
            let topology = dag.store().topology();
            let address = topology.address(id)?;
            self.sides.iter().position(|&s| {
                topology
                    .address(s)
                    .is_some_and(|s| topology.on_chain(s, address))
            })
        })
    }

    /// Returns the parent of a balancing attacker's next block: the tip it splits at for the two
    /// siblings that start the sides, then the lighter side's heaviest tip, the first side on a tie.
    fn balance<N: NetworkPerception<u64>>(&mut self, node: &MinerNode<N>, id: u64) -> u64 {
        let parent = match (self.sides.len(), self.split_at) {
            // The first block splits at the heaviest tip.
            (0, _) => {
                let tip = node.heaviest();
                self.split_at = Some(tip);
                tip
            }
            // The second is its sibling.
            (1, Some(tip)) => tip,
            // From then on, the lighter side's heaviest tip.
            _ => [0, 1]
                .into_iter()
                .filter_map(|side| self.heaviest_on(node, side))
                .min_by_key(|&t| Self::blue_work(node, t))
                .unwrap_or_else(|| node.heaviest()),
        };

        // The first two blocks start the sides.
        if self.sides.len() < 2 {
            self.sides.push(id);
        }
        parent
    }

    /// Returns the heaviest of the node's tips on a balancing attacker's `side`, the lower id on a
    /// tie.
    fn heaviest_on<N: NetworkPerception<u64>>(
        &self,
        node: &MinerNode<N>,
        side: usize,
    ) -> Option<u64> {
        node.tips()
            .into_iter()
            .filter(|&t| self.side_of(node, t) == Some(side))
            .max_by_key(|&t| (Self::blue_work(node, t), Reverse(t)))
    }

    /// Returns the attacker's next private block at `time`, built on its own node: on its side for
    /// a balancing attacker, else on its lineage's tip, folding what the strategy lists if the
    /// block stays valid. A lineage starts where an honest miner would build.
    pub fn block<N: NetworkPerception<u64>>(
        &mut self,
        node: &MinerNode<N>,
        id: u64,
        time: u64,
    ) -> Block<u64, u64> {
        // The parent: the balancing side, or the lineage's tip, or a lineage still to start.
        let parent = if self.attack.strategy == Strategy::Balance {
            self.balance(node, id)
        } else {
            let Some(own) = self.lineage_tips[self.turn] else {
                return Self::start_lineage(node, id, time);
            };
            own
        };

        // The block listing nothing is always valid; what else it lists is the strategy's.
        let alone = Block::new(
            id,
            parent,
            Vec::new(),
            node.perception(parent).network.block_work(),
            time.max(Self::stamp(node, parent)),
        );
        let listed: Vec<u64> = match self.attack.strategy {
            Strategy::Withhold | Strategy::Balance => return alone,
            Strategy::Greedy => return self.greedy(node, parent, alone),
            Strategy::Harvest => Self::tips_besides(node, parent),
            Strategy::Dag => self.own_tips().filter(|&t| t != parent).collect(),
        };
        Self::valid_or(
            node,
            Block {
                folded: listed,
                ..alone.clone()
            },
            alone,
        )
    }

    /// Returns the first block of a lineage at `time`: where an honest miner would build, but
    /// stamped no earlier than the tips it lists, since the attacker's stamps need not keep to the
    /// node's clock; listing only the tip it builds on if the later stamp ranks the tips otherwise.
    fn start_lineage<N: NetworkPerception<u64>>(
        node: &MinerNode<N>,
        id: u64,
        time: u64,
    ) -> Block<u64, u64> {
        // An honest block, stamped no earlier than any parent.
        let mut block = node.next_block(id, time);
        block.time = block
            .parents()
            .map(|p| Self::stamp(node, p))
            .fold(time, u64::max);

        // The later stamp may rank the tips otherwise: then only the selected parent is listed.
        let alone = Block {
            folded: Vec::new(),
            ..block.clone()
        };
        Self::valid_or(node, block, alone)
    }

    /// Returns `block` if the rules admit it, else `fallback`.
    fn valid_or<N: NetworkPerception<u64>>(
        node: &MinerNode<N>,
        block: Block<u64, u64>,
        fallback: Block<u64, u64>,
    ) -> Block<u64, u64> {
        if node.dag().with(|dag| dag.merge(&block).is_ok()) {
            block
        } else {
            fallback
        }
    }

    /// Returns the valid block like `alone`, which lists nothing on `own`, with the most blue work
    /// among the greedy listings, the first on a tie: nothing, every tip, then the single tips.
    fn greedy<N: NetworkPerception<u64>>(
        &mut self,
        node: &MinerNode<N>,
        own: u64,
        alone: Block<u64, u64>,
    ) -> Block<u64, u64> {
        // The candidates: nothing, every other tip, and each single tip, recent one or importable
        // chain block.
        let others = Self::tips_besides(node, own);
        let heaviest_other = others.first().copied();
        let singles: Vec<Vec<u64>> = others
            .iter()
            .chain(&self.seen)
            .chain(&heaviest_other.and_then(|t| Self::importable(node, own, t)))
            .map(|&t| vec![t])
            .collect();
        let best = Self::heaviest_listing(
            node,
            &alone,
            [Vec::new(), others].into_iter().chain(singles),
        );

        // The heaviest foreign tip now is a candidate for the next blocks.
        if let Some(t) = heaviest_other {
            if self.seen.len() == Self::SEEN {
                self.seen.remove(0);
            }
            self.seen.push(t);
        }
        best
    }

    /// Returns the node's tips other than `own`, the heaviest first.
    fn tips_besides<N: NetworkPerception<u64>>(node: &MinerNode<N>, own: u64) -> Vec<u64> {
        node.tips().into_iter().filter(|&t| t != own).collect()
    }

    /// Returns the newest block on the chain of `foreign` that the attacker's tip `own` still
    /// outranks: the most honest work it may import.
    fn importable<N: NetworkPerception<u64>>(
        node: &MinerNode<N>,
        own: u64,
        foreign: u64,
    ) -> Option<u64> {
        let rank = Self::blue_work(node, own);
        node.dag().with(|dag| {
            // Down the foreign chain to the first block lighter than the attacker's tip.
            let store = dag.store();
            let topology = store.topology();
            topology
                .chain(store.address(foreign))
                .addresses()
                .map(|c| topology.vertex(c))
                .find(|e| e.data.blue_work < rank)
                .map(|e| e.id)
        })
    }

    /// Returns the valid block like `alone` with the most blue work among `listings`, the first on a
    /// tie.
    fn heaviest_listing<N: NetworkPerception<u64>>(
        node: &MinerNode<N>,
        alone: &Block<u64, u64>,
        listings: impl Iterator<Item = Vec<u64>>,
    ) -> Block<u64, u64> {
        // Invalid listings drop out; the first of the heaviest wins.
        listings
            .map(|folded| Block {
                folded,
                ..alone.clone()
            })
            .filter_map(|b| Some((Self::blue_work_of(node, &b)?, b)))
            .reduce(|best, next| if next.0 > best.0 { next } else { best })
            .map(|(_, b)| b)
            .expect("its own tip alone is valid")
    }

    /// Returns the blue work the DAG would derive for `block`, unless the rules refuse it.
    fn blue_work_of<N: NetworkPerception<u64>>(
        node: &MinerNode<N>,
        block: &Block<u64, u64>,
    ) -> Option<u64> {
        node.dag().with(|dag| {
            dag.merge(block)
                .ok()
                .map(|merge| BlockPerception::derive(&merge).blue_work())
        })
    }

    /// Returns a block's blue work as the DAG keeps it.
    fn blue_work<N: NetworkPerception<u64>>(node: &MinerNode<N>, id: u64) -> u64 {
        node.dag().with(|dag| dag.store().vertex(id).data.blue_work)
    }

    /// Returns a block's stamp as the DAG keeps it.
    fn stamp<N: NetworkPerception<u64>>(node: &MinerNode<N>, id: u64) -> u64 {
        node.dag().with(|dag| dag.store().vertex(id).data.time)
    }

    /// Withholds a newly mined block as the tip of the current lineage and moves to the next.
    pub fn withhold(&mut self, id: u64) {
        self.withheld.push(id);
        self.lineage_tips[self.turn] = Some(id);
        self.turn = (self.turn + 1) % self.lineage_tips.len();
    }

    /// Iterates over the tips of the lineages started so far.
    fn own_tips(&self) -> impl Iterator<Item = u64> + '_ {
        self.lineage_tips.iter().flatten().copied()
    }

    /// Returns whether to reveal after a block mined at `time`, `own` if it was the attacker's.
    pub fn should_reveal<N: NetworkPerception<u64>>(
        &mut self,
        node: &MinerNode<N>,
        time: f64,
        own: bool,
    ) -> bool {
        match self.attack.reveal {
            Reveal::Immediately => true,
            Reveal::WhenAhead => own && self.ahead(node),
            Reveal::Every(t) => {
                // Each due reveal schedules the next one.
                let due = time >= self.next_reveal;
                if due {
                    self.next_reveal += t;
                }
                due
            }
        }
    }

    /// Returns the withheld blocks to publish at `time`, recording the reveal if there are any.
    pub fn reveal(&mut self, time: f64) -> Vec<u64> {
        if !self.withheld.is_empty() {
            self.reveals.push(time);
        }
        take(&mut self.withheld)
    }

    /// Returns the times of all reveals so far.
    pub fn reveals(&self) -> &[f64] {
        &self.reveals
    }

    /// Returns whether the heaviest tip of the attacker's node is one of its own.
    fn ahead<N: NetworkPerception<u64>>(&self, node: &MinerNode<N>) -> bool {
        let heaviest = node.heaviest();
        self.own_tips().any(|t| t == heaviest)
    }
}

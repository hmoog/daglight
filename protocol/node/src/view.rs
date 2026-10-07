use std::mem::take;
use std::sync::Arc;

use daglight_protocol_block::Block;
use daglight_protocol_block_perception::BlockPerception;
use daglight_protocol_dag::{Admission, Dag, DagError, DagStore, Tips, Update};
use daglight_protocol_topology::BlockAddress;

use crate::inbox::Inbox;

/// What one party has of a DAG it may share with others: its tips, the heaviest of which it
/// follows, its retained point, the received blocks still waiting to enter, and, while followed,
/// the updates as its heaviest tip moves. A view is closed under parents, so a reach per lane says
/// which blocks it has.
#[derive(Debug)]
pub(crate) struct View<S: DagStore> {
    /// For each lane, how many of its blocks the view has; a prefix of the lane.
    reach: Vec<u32>,
    /// The view's blocks without children, the heaviest it follows and its finality point.
    tips: Tips<S::Id, S::Work, S::Network>,
    /// On its chain, the block below which it keeps nothing; it only advances.
    retained: BlockAddress,
    /// The received blocks still missing parents or stamped beyond the view's clock.
    inbox: Inbox<S::Id, S::Work>,
    /// Whether the view is followed: then it records the updates as its heaviest tip moves.
    recording: bool,
    /// The updates recorded and not yet taken.
    updates: Vec<Update<S::Id, S::Work, S::Network>>,
}

impl<S: DagStore> View<S> {
    /// Creates a view of `dag` that has only the genesis.
    pub fn new(dag: &Dag<S>) -> Self {
        let store = dag.store();
        Self {
            reach: store
                .topology()
                .vertex(BlockAddress::GENESIS)
                .reach
                .to_vec(),
            tips: Tips::new(store.perception_at(BlockAddress::GENESIS)),
            retained: BlockAddress::GENESIS,
            inbox: Inbox::default(),
            recording: false,
            updates: Vec::new(),
        }
    }

    /// Returns whether the view has the block.
    pub fn contains(&self, dag: &Dag<S>, id: S::Id) -> bool {
        let store = dag.store();
        store
            .topology()
            .address(id)
            .is_some_and(|address| self.has(store, address))
    }

    /// Adds a block whose parents the view has, deriving it in `dag` unless the DAG has it already,
    /// at whatever time it is stamped: the caller vouches that the view's clock has reached it.
    /// Returns its perception, also if the view had it already.
    pub fn add_block(&mut self, dag: &mut Dag<S>, block: Block<S::Id, S::Work>) -> Admission<S> {
        // A block the view has is not entered again.
        if self.contains(dag, block.id) {
            return Ok(dag.store().perception(block.id));
        }

        // The DAG derives it, the view enters it, and a followed view records what changed.
        let perception = dag.insert(&block)?;
        if let Some(from) = self
            .enter(dag.store(), &perception)?
            .filter(|_| self.recording)
        {
            self.updates
                .extend(dag.order_change(from, self.tips.heaviest()));
        }
        Ok(perception)
    }

    /// Starts recording the updates as the view's heaviest tip moves, from where it is now: every
    /// block it sequences, and every reorg with the blocks it stops sequencing.
    pub fn follow(&mut self) {
        self.recording = true;
    }

    /// Returns the updates recorded since it was last asked, in the order they happened; none
    /// unless it is followed.
    pub fn updates(&mut self) -> Vec<Update<S::Id, S::Work, S::Network>> {
        take(&mut self.updates)
    }

    /// Takes a block from the network when the view's clock reads `now`: it waits until the clock
    /// reaches its stamp and its parents are there, then enters with whatever waited on it. Returns
    /// every block that tried to enter, in order. A parent that was pruned never arrives.
    pub fn receive(
        &mut self,
        dag: &mut Dag<S>,
        block: Block<S::Id, S::Work>,
        now: u64,
    ) -> Vec<Admission<S>> {
        // A block already waiting waits on as it is.
        if self.inbox.holds(block.id) {
            return Vec::new();
        }

        // A block stamped ahead of the clock waits for it; one missing parents waits for them.
        if block.time > now {
            self.inbox.park(block);
            return Vec::new();
        }
        let absent = block
            .parents()
            .filter(|&p| !self.contains(dag, p))
            .collect();
        let Some(block) = self.inbox.wait(block, absent) else {
            return Vec::new();
        };

        // The block enters, then whatever its entry released, and so on.
        let mut entered = Vec::new();
        let mut ready = vec![block];
        while let Some(b) = ready.pop() {
            let id = b.id;
            let result = self.add_block(dag, b);
            if result.is_ok() {
                ready.extend(self.inbox.arrived(id));
            }
            entered.push(result);
        }
        entered
    }

    /// Takes in, now that the view's clock reads `now`, every block stamped up to then, as
    /// `receive` would. Returns every block that tried to enter, in order.
    pub fn tick(&mut self, dag: &mut Dag<S>, now: u64) -> Vec<Admission<S>> {
        let mut entered = Vec::new();
        for block in self.inbox.due(now) {
            entered.extend(self.receive(dag, block, now));
        }
        entered
    }

    /// Returns the earliest time on the view's clock a received block waits for, if any does.
    pub fn next_due(&self) -> Option<u64> {
        self.inbox.next()
    }

    /// Returns the parents the waiting blocks miss, those waiting for the clock included.
    pub fn missing(&self) -> Vec<S::Id> {
        self.inbox.missing().collect()
    }

    /// Returns the tips, the heaviest first, then the others by id.
    pub fn tips(&self) -> Vec<S::Id> {
        self.tips.by_weight()
    }

    /// Returns the tip to build on: the one with the most blue work, ties to the lower hash.
    pub fn heaviest(&self) -> S::Id {
        self.tips.heaviest_perception().id
    }

    /// Returns the block this view would mine at `time`: on the heaviest tip, with the work it
    /// requires, folding every other tip the rules let it. A view takes in no block stamped beyond
    /// its clock, so its own stamp is no earlier than any parent's.
    pub fn next_block(&self, dag: &Dag<S>, id: S::Id, time: u64) -> Block<S::Id, S::Work> {
        dag.build_on(id, self.tips.heaviest(), &self.tips, time)
    }

    /// Returns the block this view would mine on its tip `tip` at `time`: with the work `tip`
    /// requires, folding every other tip the rules let it.
    pub fn block_on(
        &self,
        dag: &Dag<S>,
        id: S::Id,
        tip: S::Id,
        time: u64,
    ) -> Block<S::Id, S::Work> {
        dag.build_on(id, dag.store().address(tip), &self.tips, time)
    }

    /// Returns the view's finality point: below it the view never reorganises.
    pub fn final_point(&self) -> BlockAddress {
        self.tips.final_point()
    }

    /// Advances the view's retained point, keeping `da_offset` milliseconds of history beyond what
    /// the view can still need, and returns it: the DAG may prune everything outside its future.
    pub fn retain(&mut self, dag: &Dag<S>, da_offset: u64) -> BlockAddress {
        // Every block the view can still take has the finality point on its chain, so judging it
        // reads nothing below the finality point's pruning point; the point never moves down.
        let topology = dag.store().topology();
        let f = self.tips.final_point();
        self.retained = topology.ancestor(
            f,
            topology
                .retention_point(f, da_offset)
                .height
                .max(self.retained.height),
        );
        self.retained
    }

    /// Returns whether the view has a block the store placed; a pruned block is as good as one that
    /// never arrived.
    fn has(&self, store: &S, address: BlockAddress) -> bool {
        store.topology().kept(address).is_some_and(|vertex| {
            vertex.position < self.reach.get(vertex.lane as usize).copied().unwrap_or(0)
        })
    }

    /// Enters a block the store already holds, once the view has its parents; returns the heaviest
    /// tip before if the block changed it.
    fn enter(
        &mut self,
        store: &S,
        perception: &Arc<BlockPerception<S::Id, S::Work, S::Network>>,
    ) -> Result<Option<BlockAddress>, DagError<S::Id, S::Work>> {
        // Every parent must be in the view.
        let topology = store.topology();
        let address = store.address(perception.id);
        let vertex = topology.vertex(address);
        if let Some(&p) = vertex.parents.iter().find(|&&p| !self.has(store, p)) {
            return Err(DagError::UnknownParent(topology.vertex(p).id));
        }

        // The block becomes a tip, maybe the heaviest.
        let advanced = self.tips.insert(Arc::clone(perception), address, topology);

        // The block's past is in the view already, so the view reaches one block further along the
        // block's own lane and nowhere else.
        let lane = vertex.lane as usize;
        if self.reach.len() <= lane {
            self.reach.resize(lane + 1, 0);
        }
        self.reach[lane] = vertex.position + 1;
        Ok(advanced)
    }
}

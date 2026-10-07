use std::sync::Arc;

use daglight_protocol_block::Block;
use daglight_protocol_block_perception::BlockPerception;
use daglight_protocol_dag::{Admission, Dag, DagStore, Update};
use daglight_protocol_topology::BlockAddress;

use crate::view::View;
use crate::DagHandle;
use crate::{Id, Network, Work};

/// A node: the DAG it holds, its own or one shared with other nodes, and its view of it, the
/// received blocks still waiting for parents or for its clock, and, while followed, the updates as
/// its heaviest tip moves. The DAG derives every block once, whichever node has it.
#[derive(Debug)]
pub struct Node<D: DagHandle> {
    /// The DAG it holds.
    dag: D,
    /// What it has of the DAG.
    view: View<D::Store>,
}

impl<D: DagHandle> Node<D> {
    /// Creates a node on `dag` that has only the genesis.
    pub fn new(dag: D) -> Self {
        Self {
            view: dag.with(View::new),
            dag,
        }
    }

    /// Returns the DAG the node holds.
    pub fn dag(&self) -> &D {
        &self.dag
    }

    /// Returns whether the node has the block.
    pub fn contains(&self, id: Id<D>) -> bool {
        self.dag.with(|dag| self.view.contains(dag, id))
    }

    /// Adds a block whose parents the node has, deriving it unless the DAG has it already, at
    /// whatever time it is stamped: the caller vouches that the node's clock has reached it.
    pub fn add_block(&mut self, block: Block<Id<D>, Work<D>>) -> Admission<D::Store> {
        self.dag.with_mut(|dag| self.view.add_block(dag, block))
    }

    /// Takes a block from the network when the node's clock reads `now`: it waits until the clock
    /// reaches its stamp and its parents are there, then enters with whatever waited on it. Returns
    /// every block that tried to enter, in order. A parent that was pruned never arrives.
    pub fn receive(&mut self, block: Block<Id<D>, Work<D>>, now: u64) -> Vec<Admission<D::Store>> {
        self.dag.with_mut(|dag| self.view.receive(dag, block, now))
    }

    /// Takes in, now that the node's clock reads `now`, every block stamped up to then, as
    /// `receive` would. Returns every block that tried to enter, in order.
    pub fn tick(&mut self, now: u64) -> Vec<Admission<D::Store>> {
        self.dag.with_mut(|dag| self.view.tick(dag, now))
    }

    /// Returns the earliest time on the node's clock a received block waits for, if any does.
    pub fn next_due(&self) -> Option<u64> {
        self.view.next_due()
    }

    /// Returns the parents the waiting blocks miss, those waiting for the clock included.
    pub fn missing(&self) -> Vec<Id<D>> {
        self.view.missing()
    }

    /// Starts recording the updates as the node's heaviest tip moves, from where it is now: every
    /// block it sequences, and every reorg with the blocks it stops sequencing.
    pub fn follow(&mut self) {
        self.view.follow();
    }

    /// Returns the updates recorded since it was last asked, in the order they happened; none
    /// unless it is followed.
    pub fn updates(&mut self) -> Vec<Update<Id<D>, Work<D>, Network<D>>> {
        self.view.updates()
    }

    /// Returns the tips, the heaviest first, then the others by id.
    pub fn tips(&self) -> Vec<Id<D>> {
        self.view.tips()
    }

    /// Returns the tip to build on: the one with the most blue work, ties to the lower hash.
    pub fn heaviest(&self) -> Id<D> {
        self.view.heaviest()
    }

    /// Returns the block this node would mine at `time`: on the tip it builds on then, with the
    /// work it requires, folding every tip it may.
    pub fn next_block(&self, id: Id<D>, time: u64) -> Block<Id<D>, Work<D>> {
        self.dag.with(|dag| self.view.next_block(dag, id, time))
    }

    /// Returns the block this node would mine on its tip `tip` at `time`: with the work `tip`
    /// requires, folding every other tip it may.
    pub fn block_on(&self, id: Id<D>, tip: Id<D>, time: u64) -> Block<Id<D>, Work<D>> {
        self.dag.with(|dag| self.view.block_on(dag, id, tip, time))
    }

    /// Returns the node's finality point: below it the node never reorganises.
    pub fn final_point(&self) -> BlockAddress {
        self.view.final_point()
    }

    /// Returns the perception of a block the DAG holds.
    pub fn perception(&self, id: Id<D>) -> Arc<BlockPerception<Id<D>, Work<D>, Network<D>>> {
        self.dag.with(|dag| dag.store().perception(id))
    }

    /// Returns a block the DAG holds as the node relays it, without its redundant parents.
    pub fn block(&self, id: Id<D>) -> Block<Id<D>, Work<D>> {
        self.dag.with(|dag| dag.store().block(id))
    }

    /// Advances the node's retained point, keeping `da_offset` milliseconds of history beyond what
    /// it can still need, and returns it: whoever owns the DAG may prune everything outside its
    /// future.
    pub fn retain(&mut self, da_offset: u64) -> BlockAddress {
        self.dag.with(|dag| self.view.retain(dag, da_offset))
    }
}

impl<S: DagStore> Node<Dag<S>> {
    /// Prunes the node's own DAG of everything outside the future of its retained point, keeping
    /// `da_offset` milliseconds of history beyond what it can still need.
    pub fn prune(&mut self, da_offset: u64) {
        let point = self.retain(da_offset);
        self.dag.prune(&[point]);
    }
}

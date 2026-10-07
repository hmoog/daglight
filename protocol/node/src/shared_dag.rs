use std::cell::RefCell;
use std::rc::Rc;

use daglight_protocol_dag::{Dag, DagStore};

use crate::DagHandle;

/// A handle on a DAG shared by many nodes, which derives every block once for all of them. Every
/// clone is one more holder of the same DAG. What none of them needs any more, their owner prunes.
#[derive(Debug)]
pub struct SharedDag<S: DagStore>(Rc<RefCell<Dag<S>>>);

impl<S: DagStore> SharedDag<S> {
    /// Shares `dag`, returning its first holder.
    pub fn new(dag: Dag<S>) -> Self {
        Self(Rc::new(RefCell::new(dag)))
    }
}

/// Another holder of the same DAG.
impl<S: DagStore> Clone for SharedDag<S> {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

/// The DAG borrowed for the length of one call, which no node call ever nests.
impl<S: DagStore> DagHandle for SharedDag<S> {
    type Store = S;

    fn with<R>(&self, f: impl FnOnce(&Dag<S>) -> R) -> R {
        f(&self.0.borrow())
    }

    fn with_mut<R>(&mut self, f: impl FnOnce(&mut Dag<S>) -> R) -> R {
        f(&mut self.0.borrow_mut())
    }
}

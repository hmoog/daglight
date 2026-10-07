use daglight_protocol_dag::{Dag, DagStore};

/// How a node holds its DAG: a DAG of its own, or a handle on one it shares with other nodes. The
/// node reaches it only through calls that end before it returns, so no access outlives a call.
pub trait DagHandle {
    /// The store the DAG keeps its blocks in.
    type Store: DagStore;

    /// Returns what `f` makes of the DAG.
    fn with<R>(&self, f: impl FnOnce(&Dag<Self::Store>) -> R) -> R;

    /// Returns what `f` makes of the DAG, changing it.
    fn with_mut<R>(&mut self, f: impl FnOnce(&mut Dag<Self::Store>) -> R) -> R;
}

/// A DAG of the node's own.
impl<S: DagStore> DagHandle for Dag<S> {
    type Store = S;

    fn with<R>(&self, f: impl FnOnce(&Dag<S>) -> R) -> R {
        f(self)
    }

    fn with_mut<R>(&mut self, f: impl FnOnce(&mut Dag<S>) -> R) -> R {
        f(self)
    }
}

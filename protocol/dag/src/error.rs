/// Why a DAG or a node refuses a block: a parent is missing, it cannot be judged, or it breaks the
/// rules.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DagError<Id, W> {
    /// A listed parent is not in the view yet.
    UnknownParent(Id),
    /// What judging the block would read was pruned: it is as good as never arrived.
    Pruned,
    /// A folded parent outranks the selected parent.
    Heavier {
        /// The selected parent.
        selected: Id,
        /// The folded parent that outranks it.
        heavier: Id,
    },
    /// The block carries other work than its selected parent requires.
    Work {
        /// The work required.
        required: W,
        /// The work carried.
        carried: W,
    },
    /// A folded parent's chain misses the selected parent's finality point.
    BelowFinality {
        /// The folded parent.
        folded: Id,
    },
    /// The block folds a block that does not have the selected parent's pruning point in its past.
    BelowPruning {
        /// The folded block.
        folded: Id,
    },
    /// The block is stamped before a parent.
    Time {
        /// The latest parent's time.
        parent: u64,
        /// The block's time.
        time: u64,
    },
}

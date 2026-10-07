#![forbid(unsafe_code)]
//! A block DAG held in a store, usable on its own, with its consensus rules: how it admits a new
//! block and merges what the block's perception derives from, its tips and the block to build on
//! them, and its order: how a chain block sequences its mergeset and how the order changes as the
//! chosen chain moves from one tip to another.

use std::sync::Arc;

use daglight_protocol_block_perception::{BlockPerception, MergedPerception};
use daglight_protocol_topology::{BlockAddress, Mergeset};

mod dag;
mod error;
mod reorg;
mod sequence;
mod sequenced_block;
mod store;
mod tips;
mod update;

pub use dag::Dag;
pub use error::DagError;
pub use reorg::Reorg;
pub use sequence::Sequence;
pub use sequenced_block::SequencedBlock;
pub use store::DagStore;
pub use tips::Tips;
pub use update::Update;

/// A block's admission to a DAG in store `S`, or to a node on one: its perception, or why it is
/// refused.
pub type Admission<S> = Result<
    Arc<BlockPerception<<S as DagStore>::Id, <S as DagStore>::Work, <S as DagStore>::Network>>,
    DagError<<S as DagStore>::Id, <S as DagStore>::Work>,
>;

/// What merging a block on the DAG in store `S` returns: its merged perception, or why the rules
/// refuse it.
pub type MergeResult<'b, S> = Result<
    MergedPerception<'b, <S as DagStore>::Id, <S as DagStore>::Work, <S as DagStore>::Network>,
    DagError<<S as DagStore>::Id, <S as DagStore>::Work>,
>;

/// What admitting a block on the DAG in store `S` returns: its mergeset, with every merged block's
/// branch off the selected parent's chain, or why the rules refuse it.
pub(crate) type AdmitResult<S> =
    Result<(Mergeset, Vec<BlockAddress>), DagError<<S as DagStore>::Id, <S as DagStore>::Work>>;

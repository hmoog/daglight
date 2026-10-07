#![forbid(unsafe_code)]
//! A node: its view of a DAG it holds, its own or one shared with other nodes, the tips it builds
//! on and follows, the blocks it mines and what it may forget; and what it tells the outside: its
//! heaviest tip, its finality point and, while followed, the `Update`s as its heaviest tip moves,
//! every block in them with its perception.

use daglight_protocol_dag::DagStore;

mod dag_handle;
mod inbox;
mod node;
mod shared_dag;
mod view;

pub use dag_handle::DagHandle;
pub use node::Node;
pub use shared_dag::SharedDag;

/// The id of the blocks of the DAG behind handle `D`.
pub(crate) type Id<D> = <<D as DagHandle>::Store as DagStore>::Id;

/// The work of the blocks of the DAG behind handle `D`.
pub(crate) type Work<D> = <<D as DagHandle>::Store as DagStore>::Work;

/// The network perception the blocks of the DAG behind handle `D` carry.
pub(crate) type Network<D> = <<D as DagHandle>::Store as DagStore>::Network;

#![forbid(unsafe_code)]
//! The block as it travels between nodes, and the primitives it is made of.

mod block;
mod id;
mod work;

pub use block::Block;
pub use id::BlockId;
pub use work::Work;

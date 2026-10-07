#![forbid(unsafe_code)]
//! What a simulated run records along the way: who mined what when, how honest miners switched
//! tips, and the final chain as it became final.

mod final_chain;
mod history;
mod mined_block;

pub use final_chain::FinalChain;
pub use history::History;
pub use mined_block::MinedBlock;

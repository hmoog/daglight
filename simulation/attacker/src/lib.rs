#![forbid(unsafe_code)]
//! The simulated adversary: a miner that withholds, harvests, builds a private DAG, lists greedily
//! or balances two sides, and reveals at once, when ahead or periodically.

mod attacker;

pub use attacker::Attacker;

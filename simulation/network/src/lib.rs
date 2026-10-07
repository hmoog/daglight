#![forbid(unsafe_code)]
//! The simulated network: miners, each a node on one shared DAG, finding blocks at their hash rate
//! and delivering them with per-link delays, driven by a queue of events.

mod event;
mod miner;
mod rng;
mod simulation;

pub(crate) use event::Event;
pub use miner::Miner;
pub(crate) use rng::Rng;
pub use simulation::Simulation;

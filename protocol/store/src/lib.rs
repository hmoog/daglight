#![forbid(unsafe_code)]
//! A store for a DAG in memory: every block and its perception held once, the perception of older
//! heights in an archive.

mod address_map;
mod archive;
mod memory_archive;
mod memory_store;

pub use archive::Archive;
pub use memory_archive::MemoryArchive;
pub use memory_store::MemoryStore;

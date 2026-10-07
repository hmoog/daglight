use std::cell::Cell;

use daglight_protocol_block_perception::{BlockPerception, LearnedNetworkPerception};
use daglight_protocol_topology::BlockAddress;

use crate::address_map::AddressMap;
use crate::Archive;

/// An archive kept in memory, counting how often perception is loaded back.
#[derive(Clone, Debug)]
pub struct MemoryArchive<Id, W, N = LearnedNetworkPerception<W>> {
    /// The archived perception by address.
    perception: AddressMap<BlockPerception<Id, W, N>>,
    /// The number of loads so far.
    loads: Cell<usize>,
}

/// An empty archive.
impl<Id, W, N> Default for MemoryArchive<Id, W, N> {
    fn default() -> Self {
        Self {
            perception: AddressMap::default(),
            loads: Cell::new(0),
        }
    }
}

impl<Id, W, N> MemoryArchive<Id, W, N> {
    /// Returns how often perception was loaded back.
    pub fn loads(&self) -> usize {
        self.loads.get()
    }
}

/// Owned copies in and out, as a disk would give them.
impl<Id: Clone, W: Clone, N: Clone> Archive<Id, W, N> for MemoryArchive<Id, W, N> {
    fn store(&mut self, address: BlockAddress, perception: BlockPerception<Id, W, N>) {
        if self.perception.insert(address, perception).is_err() {
            unreachable!("the archive evicts nothing");
        }
    }

    fn forget(&mut self, address: BlockAddress) {
        self.perception.remove(address);
    }

    fn load(&self, address: BlockAddress) -> BlockPerception<Id, W, N> {
        self.loads.set(self.loads.get() + 1);
        self.perception
            .get(address)
            .unwrap_or_else(|| panic!("{address:?} was never archived"))
            .clone()
    }
}

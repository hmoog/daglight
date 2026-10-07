use std::collections::VecDeque;

use crate::BlockAddress;

/// The blocks of one lane in order, each in the past of the next, from the first one kept.
#[derive(Clone, Debug)]
pub(crate) struct Lane {
    /// The position of the first block kept.
    start: u32,
    /// The blocks from `start` on.
    blocks: VecDeque<BlockAddress>,
}

impl Lane {
    /// Returns a lane of one block.
    pub fn of(address: BlockAddress) -> Self {
        Self {
            start: 0,
            blocks: VecDeque::from([address]),
        }
    }

    /// Returns the number of blocks the lane ever had: the position its next block takes.
    pub fn len(&self) -> u32 {
        self.start + self.blocks.len() as u32
    }

    /// Returns the position of the first block kept.
    pub fn start(&self) -> u32 {
        self.start
    }

    /// Returns the blocks from `from` to `to`, the kept ones.
    pub fn range(&self, from: u32, to: u32) -> impl Iterator<Item = BlockAddress> + '_ {
        // Positions below the start were pruned; the rest index the kept blocks.
        let from = from.max(self.start) - self.start;
        let to = to.max(self.start) - self.start;
        self.blocks.range(from as usize..to as usize).copied()
    }

    /// Appends a block.
    pub fn push(&mut self, address: BlockAddress) {
        self.blocks.push_back(address);
    }

    /// Drops the first blocks while `gone` says they were pruned.
    pub fn prune(&mut self, gone: impl Fn(BlockAddress) -> bool) {
        while self.blocks.front().is_some_and(|&s| gone(s)) {
            self.blocks.pop_front();
            self.start += 1;
        }
    }
}

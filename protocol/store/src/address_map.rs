use std::collections::VecDeque;

use daglight_protocol_topology::{BlockAddress, Height};

/// Values by address, stored by height from a base height upwards, then by position.
#[derive(Clone, Debug)]
pub(crate) struct AddressMap<T> {
    /// The values by height from `base`, then by position; `None` where no value is held.
    layers: VecDeque<Vec<Option<T>>>,
    /// The lowest height held; lower heights have been evicted.
    base: Height,
}

/// An empty map.
impl<T> Default for AddressMap<T> {
    fn default() -> Self {
        Self {
            layers: VecDeque::new(),
            base: 0,
        }
    }
}

impl<T> AddressMap<T> {
    /// Returns the value at an address, if held.
    pub fn get(&self, address: BlockAddress) -> Option<&T> {
        self.layers
            .get(address.height.checked_sub(self.base)? as usize)?
            .get(address.offset as usize)?
            .as_ref()
    }

    /// Holds a value at an address, or hands it back if the address's height was evicted.
    pub fn insert(&mut self, address: BlockAddress, value: T) -> Result<(), T> {
        // The height, as a layer above the base; an evicted height takes nothing.
        let Some(layer) = address.height.checked_sub(self.base) else {
            return Err(value);
        };
        let layer = layer as usize;

        // Layers up to the height, and slots up to the position, are made as needed.
        if self.layers.len() <= layer {
            self.layers.resize_with(layer + 1, Vec::new);
        }
        let slots = &mut self.layers[layer];
        let slot = address.offset as usize;
        if slots.len() <= slot {
            slots.resize_with(slot + 1, || None);
        }

        slots[slot] = Some(value);
        Ok(())
    }

    /// Removes the value at an address, and the heights at the bottom left without one.
    pub fn remove(&mut self, address: BlockAddress) {
        // Clear the slot, if the height and the position are held at all.
        let Some(layer) = address
            .height
            .checked_sub(self.base)
            .and_then(|h| self.layers.get_mut(h as usize))
        else {
            return;
        };
        if let Some(value) = layer.get_mut(address.offset as usize) {
            *value = None;
        }

        // Emptied layers at the bottom go, and the base moves up past them.
        while self
            .layers
            .front()
            .is_some_and(|l| l.iter().all(Option::is_none))
        {
            self.layers.pop_front();
            self.base += 1;
        }
    }

    /// Removes every height below `floor` and returns its values by address.
    pub fn evict_below(&mut self, floor: Height) -> Vec<(BlockAddress, T)> {
        let mut out = Vec::new();
        while self.base < floor {
            // The bottom layer goes, its held values with their addresses; a height never held
            // has no layer and contributes nothing.
            let layer = self.layers.pop_front().unwrap_or_default();
            for (offset, value) in layer.into_iter().enumerate() {
                if let Some(v) = value {
                    out.push((
                        BlockAddress {
                            height: self.base,
                            offset: offset as u32,
                        },
                        v,
                    ));
                }
            }
            self.base += 1;
        }
        out
    }
}

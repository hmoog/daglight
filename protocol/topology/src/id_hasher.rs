use std::hash::Hasher;

/// A fast hasher for block ids, which are already hashes: a multiply-rotate mix per word.
#[derive(Clone, Copy, Debug, Default)]
pub struct IdHasher(u64);

impl IdHasher {
    /// The multiplier of the mix.
    const K: u64 = 0x517c_c1b7_2722_0a95;

    /// Mixes one word into the state.
    fn add(&mut self, word: u64) {
        self.0 = (self.0.rotate_left(5) ^ word).wrapping_mul(Self::K);
    }
}

/// Words go in whole, bytes eight at a time.
impl Hasher for IdHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        // Whole words first.
        let mut chunks = bytes.chunks_exact(8);
        for chunk in &mut chunks {
            self.add(u64::from_le_bytes(chunk.try_into().expect("eight bytes")));
        }

        // The remainder, if any, padded to a word.
        let rest = chunks.remainder();
        if !rest.is_empty() {
            let mut word = [0; 8];
            word[..rest.len()].copy_from_slice(rest);
            self.add(u64::from_le_bytes(word));
        }
    }

    fn write_u32(&mut self, n: u32) {
        self.add(u64::from(n));
    }

    fn write_u64(&mut self, n: u64) {
        self.add(n);
    }

    fn write_usize(&mut self, n: usize) {
        self.add(n as u64);
    }
}

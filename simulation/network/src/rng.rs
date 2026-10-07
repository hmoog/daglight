/// A deterministic splitmix64 random number generator.
#[derive(Clone, Debug)]
pub struct Rng {
    /// The counter the output is mixed from.
    state: u64,
}

impl Rng {
    /// Creates a generator from a seed.
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// Returns the next 64 random bits.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Returns a uniform sample in `[0, 1)`.
    pub fn uniform(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Returns an exponential sample with the given rate.
    pub fn exponential(&mut self, rate: f64) -> f64 {
        -(1.0 - self.uniform()).ln() / rate
    }

    /// Returns a random block id other than the genesis.
    pub fn block_id(&mut self) -> u64 {
        loop {
            let id = self.next_u64();
            if id != 0 {
                return id;
            }
        }
    }
}

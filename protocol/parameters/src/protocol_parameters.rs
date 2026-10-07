/// The protocol's parameters, carried by every block's perception and inherited from its selected
/// parent: what the rules measure time and margins in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProtocolParameters {
    /// The blocks the network is to make per second; the difficulty follows the hash rate to hold
    /// it.
    pub bps: u64,
    /// The finality horizon, in milliseconds: how far back a lineage looks to know the network, and
    /// how deep a fork may stay open before it is closed as judged.
    pub finality: u64,
    /// The handshake, in delays: how long honest blocks may be on their way. In work, it is the
    /// lead by which a fork's continuation must beat its rivals to fold it.
    pub handshake_steps: u64,
}

impl ProtocolParameters {
    /// Twenty blocks a second, a finality horizon of twelve hours and a handshake of four delays.
    pub const DEFAULT: Self = Self {
        bps: 20,
        finality: 43_200_000,
        handshake_steps: 4,
    };

    /// Returns these parameters with `bps` blocks a second.
    pub const fn with_bps(mut self, bps: u64) -> Self {
        self.bps = bps;
        self
    }

    /// Returns these parameters with a finality horizon of `finality` milliseconds.
    pub const fn with_finality(mut self, finality: u64) -> Self {
        self.finality = finality;
        self
    }

    /// Returns the time the network is to take per block, in milliseconds, at least one.
    pub fn interval(&self) -> u64 {
        (1000 / self.bps.max(1)).max(1)
    }

    /// Returns the finality horizon in block intervals, at least one.
    pub fn finality_intervals(&self) -> u64 {
        (self.finality / self.interval()).max(1)
    }
}

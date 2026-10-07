use crate::{Config, Reveal, Strategy};

/// The settings of an attack by the last miner.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Attack {
    /// The attacker's share of the total hash rate.
    pub share: f64,
    /// How the attacker chooses parents.
    pub strategy: Strategy,
    /// When the attacker publishes its withheld blocks.
    pub reveal: Reveal,
    /// The time the attack begins; before it the attacker mines honestly.
    pub start: f64,
    /// How fast the attacker's stamps advance against the time from the start, if not with it:
    /// below one it stamps behind time, above one ahead of it.
    pub stamp_pace: Option<f64>,
}

impl Attack {
    /// Returns the attacker's stamp, in milliseconds, for a block mined at `time`: the time, or, at
    /// a stamp pace of its own, the start plus the time since at that pace.
    pub fn stamp(&self, time: f64) -> u64 {
        Config::millis(
            self.stamp_pace
                .map_or(time, |pace| self.start + (time - self.start) * pace),
        )
    }
}

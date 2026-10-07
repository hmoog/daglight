/// A change of the network's hash rate during a run, which the difficulty then follows.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RateSwitch {
    /// The time of the switch.
    pub at: f64,
    /// The blocks per second the network mines from then on at the genesis's block work.
    pub rate: f64,
}

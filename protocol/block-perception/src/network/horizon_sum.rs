use daglight_protocol_block::Work;

/// Work summed over the finality horizon, older work counting less: every millisecond that passes
/// takes its share of the horizon off the sum, so the sum of a steady network is its work over one
/// horizon.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct HorizonSum<W>(W);

impl<W: Work> HorizonSum<W> {
    /// Returns the sum of a network that has done `per_interval` of work in every one of the
    /// horizon's `intervals`.
    pub fn steady(per_interval: W, intervals: u64) -> Self {
        Self(W::narrow(per_interval.wide() * u128::from(intervals)))
    }

    /// Returns a sum of `work`.
    pub fn new(work: W) -> Self {
        Self(work)
    }

    /// Takes in `work` done `elapsed` milliseconds after the last, on a horizon of `horizon`
    /// milliseconds.
    pub fn add(&mut self, work: W, elapsed: u64, horizon: u64) {
        // The elapsed share of the horizon decays off the sum before the work goes in.
        let horizon = u128::from(horizon.max(1));
        let kept = horizon.saturating_sub(u128::from(elapsed));
        self.0 = W::narrow(self.0.wide() * kept / horizon + work.wide());
    }

    /// Returns the sum as work per interval of a horizon of `intervals`, rounded, at least one.
    pub fn per_interval(&self, intervals: u64) -> W {
        let horizon = u128::from(intervals.max(1));
        W::narrow(((self.0.wide() + horizon / 2) / horizon).max(1))
    }

    /// Returns the sum itself.
    pub fn work(&self) -> W {
        self.0
    }
}

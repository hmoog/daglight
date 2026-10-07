use std::fmt::Debug;
use std::ops::{Add, Sub};

/// An amount of proof of work: totally ordered, additive, with a zero, and exact in a `u128`.
pub trait Work:
    Copy + Ord + Add<Output = Self> + Sub<Output = Self> + Default + Debug + Into<u128> + TryFrom<u128>
{
    /// Returns no work.
    fn zero() -> Self {
        Self::default()
    }

    /// Returns `self · n`; panics on overflow.
    fn times(self, n: u64) -> Self {
        Self::narrow(
            self.wide()
                .checked_mul(u128::from(n))
                .expect("work overflow"),
        )
    }

    /// Returns the work as a `u128`, for exact ratios.
    fn wide(self) -> u128 {
        self.into()
    }

    /// Returns work given as a `u128`; panics if it does not fit.
    fn narrow(work: u128) -> Self {
        Self::try_from(work).unwrap_or_else(|_| panic!("work overflow"))
    }
}

/// Work counted in a `u32`.
impl Work for u32 {}

/// Work counted in a `u64`.
impl Work for u64 {}

/// Work counted in a `u128`.
impl Work for u128 {}

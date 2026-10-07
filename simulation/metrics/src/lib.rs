#![forbid(unsafe_code)]
//! What a simulated run is judged by, read from an honest node once everything published has
//! arrived; the `daglight-sim` binary and the measured table.

mod metrics;

pub use metrics::Metrics;

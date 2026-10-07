#![forbid(unsafe_code)]
//! Records DAGLight runs as replays for the player: every block with what a node derives for it,
//! and every step of a node that receives the blocks as they are mined, with its tips, its heaviest
//! tip, its cut-offs and the updates it reports, written as JSON.

mod block_record;
mod replay;
mod scenario;
mod step_record;
mod update_record;

pub use block_record::BlockRecord;
pub use replay::Replay;
pub use scenario::Scenario;
pub use step_record::StepRecord;
pub use update_record::UpdateRecord;

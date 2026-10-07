//! What a block knows of the lineages that conflict with its chain: the forks along it closed for
//! good, and the rival work recorded at every open one, contested and settled.

mod forks_perception;
mod rival;
mod rivals;

pub use forks_perception::ForksPerception;
pub use rival::Rival;
pub use rivals::Rivals;

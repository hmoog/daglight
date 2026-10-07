use std::collections::HashSet;
use std::iter::once;

use daglight_protocol_dag::{Dag, DagStore};
use daglight_protocol_topology::{BlockAddress, Height};

/// A selected chain as far as recorded, from the genesis up, with the blocks it credits: its own
/// and their mergesets but the red blocks.
#[derive(Clone, Debug, Default)]
pub struct FinalChain {
    /// The chain blocks, the genesis first.
    pub blocks: Vec<u64>,
    /// The blocks the chain credits.
    pub credited: HashSet<u64>,
}

impl FinalChain {
    /// Extends the chain up to `point`, on the chain recorded so far, with what each new chain
    /// block credits: nothing it still needs can have been pruned.
    pub fn extend_to<S: DagStore<Id = u64, Work = u64>>(
        &mut self,
        dag: &Dag<S>,
        point: BlockAddress,
    ) {
        // The chain blocks above the recorded ones, lowest first.
        let topology = dag.store().topology();
        let from = self.blocks.len() as Height;
        let mut added: Vec<BlockAddress> = topology
            .chain(point)
            .addresses()
            .take_while(|s| s.height >= from)
            .collect();
        added.reverse();

        // Each credits itself and the blue part of its mergeset.
        for c in added {
            self.credited.extend(
                once(c)
                    .chain(dag.sequence(c).blues)
                    .map(|x| topology.vertex(x).id),
            );
            self.blocks.push(topology.vertex(c).id);
        }
    }
}

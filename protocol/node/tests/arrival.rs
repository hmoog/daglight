//! A block's perception depends on its past alone, not on the order blocks arrive in, and neither
//! does what a node sequences.

use std::cmp::Reverse;
use std::collections::HashSet;

use daglight_protocol_block::Block;
use daglight_protocol_block_perception::{LearnedNetworkPerception, NetworkPerception};
use daglight_protocol_dag::{Dag, DagError, DagStore, SequencedBlock, Update};
use daglight_protocol_store::MemoryStore;

mod common;

use common::*;

/// A small deterministic generator for the random DAG and the arrival order.
struct Lcg(u64);

impl Lcg {
    /// Returns the next value below `n`.
    fn below(&mut self, n: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.0 >> 33) % n as u64) as usize
    }
}

/// What a node sequences, with the perception it derives.
type Sequence = Vec<SequencedBlock<u32, u64, LearnedNetworkPerception<u64>>>;

/// Replays the updates of a followed node: what it sequences in the end.
fn replay(updates: Vec<Update<u32, u64, LearnedNetworkPerception<u64>>>) -> Sequence {
    let mut sequence: Sequence = Vec::new();
    for update in updates {
        match update {
            Update::Reorg(reorg) => {
                for reverted in reorg.reverted {
                    assert_eq!(sequence.pop(), Some(reverted), "reverted last first");
                }
            }
            Update::Advance(sequenced) => sequence.extend(sequenced),
        }
    }
    sequence
}

/// Returns what node `n` sequences along the chain of its perception: each chain block's credited
/// mergeset, its red one, then the chain block.
fn sequence_of(n: &TestNode) -> Sequence {
    let topology = n.dag().store().topology();
    let tip = n.dag().store().address(n.heaviest());
    let mut chain: Vec<_> = topology.chain(tip).addresses().collect();
    chain.pop();
    chain.reverse();
    let at = |x| n.dag().store().perception_at(x);
    chain
        .into_iter()
        .flat_map(|c| {
            let s = n.dag().sequence(c);
            let blues = s.blues.into_iter().map(|x| SequencedBlock::Blue(at(x)));
            let reds = s.reds.into_iter().map(|x| SequencedBlock::Red(at(x)));
            blues
                .chain(reds)
                .chain([SequencedBlock::Chain(at(c))])
                .collect::<Vec<_>>()
        })
        .collect()
}

/// Two nodes given the same blocks in different causal orders derive the same perception and
/// sequence the same blocks the same way, each block of the perception's past exactly once.
#[test]
fn perception_is_independent_of_arrival_order() {
    // A random DAG: each block lists some tips and recent blocks, the heaviest first, at random
    // times, so that the difficulty moves; a parent that outranks the selected one at the block's
    // time is selected instead.
    let mut rng = Lcg(5);
    let network = LearnedNetworkPerception::genesis(&PROTOCOL_PARAMETERS, 10, 2000, 2_000_000);
    let mut first = TestNode::new(Dag::new(MemoryStore::new(
        PROTOCOL_PARAMETERS,
        network.clone(),
    )));
    first.follow();
    let mut blocks = Vec::new();
    let mut time = 0;
    for id in 1..600u32 {
        let tips: Vec<u32> = first.tips();
        let mut parents: Vec<u32> = tips.into_iter().filter(|_| rng.below(3) > 0).collect();
        parents.push(id - 1 - rng.below((id as usize).min(10)) as u32);
        parents.sort_by_key(|&p| Reverse(first.perception(p)));
        parents.dedup();
        time += rng.below(2000) as u64;
        let block = loop {
            let work = first.perception(parents[0]).network.block_work();
            let block = Block::new(id, parents[0], parents[1..].to_vec(), work, time);
            let refused = first.dag().merge(&block).err();
            match refused {
                Some(DagError::Heavier { heavier, .. }) => {
                    parents.retain(|&p| p != heavier);
                    parents.insert(0, heavier);
                }
                _ => break block,
            }
        };
        first.add_block(block.clone()).unwrap();
        blocks.push(block);
    }

    // The same blocks again, each time a random one whose parents have arrived.
    let mut second = TestNode::new(Dag::new(MemoryStore::new(PROTOCOL_PARAMETERS, network)));
    second.follow();
    let mut waiting = blocks.clone();
    while !waiting.is_empty() {
        let ready: Vec<usize> = (0..waiting.len())
            .filter(|&i| waiting[i].parents().all(|p| second.contains(p)))
            .collect();
        let block = waiting.swap_remove(ready[rng.below(ready.len())]);
        second.add_block(block).unwrap();
    }

    // Equal parents, compared by id since addresses follow the arrival order, and equal perception.
    for b in &blocks {
        assert_eq!(first.block(b.id), second.block(b.id));
        assert_eq!(
            first.perception(b.id),
            second.perception(b.id),
            "block {}",
            b.id
        );
    }

    // The same sequence, replayed from the advances or read off the final chain, and every block
    // of the perception's past in it once.
    let sequenced = replay(first.updates());
    assert_eq!(sequenced, replay(second.updates()));
    assert_eq!(sequenced, sequence_of(&first));
    let topology = first.dag().store().topology();
    let tip = first.dag().store().address(first.heaviest());
    let past: HashSet<u32> = blocks
        .iter()
        .map(|b| b.id)
        .filter(|&id| {
            let x = topology.address(id).unwrap();
            x == tip || topology.is_ancestor(x, tip)
        })
        .collect();
    let ids: Vec<u32> = sequenced.iter().map(|s| s.id()).collect();
    assert_eq!(ids.len(), past.len(), "each once");
    assert_eq!(ids.into_iter().collect::<HashSet<_>>(), past);
}

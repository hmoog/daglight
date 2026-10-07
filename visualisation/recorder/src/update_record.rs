use daglight_protocol_block_perception::NetworkPerception;
use daglight_protocol_dag::{SequencedBlock, Update};

/// An update a node reported, with blocks by number.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UpdateRecord {
    /// The perception changed sides at the fork `common`.
    Reorg {
        /// The fork both chains share.
        common: usize,
        /// The chain blocks reverted, the highest first.
        chain: Vec<usize>,
        /// Every block no longer sequenced, the last first.
        reverted: Vec<usize>,
    },
    /// Blocks newly sequenced, in order, each `b`lue, `r`ed or a `c`hain block.
    Advance(Vec<(usize, char)>),
}

impl UpdateRecord {
    /// Returns the record of `update`, with blocks numbered by `number`.
    pub fn of<N: NetworkPerception<u64>>(
        update: Update<u64, u64, N>,
        number: &impl Fn(u64) -> usize,
    ) -> Self {
        match update {
            Update::Reorg(reorg) => Self::Reorg {
                common: number(reorg.common.id),
                chain: reorg.chain().map(|c| number(c.id)).collect(),
                reverted: reorg.reverted.iter().map(|r| number(r.id())).collect(),
            },
            Update::Advance(blocks) => Self::Advance(
                blocks
                    .iter()
                    .map(|b| (number(b.id()), Self::kind(b)))
                    .collect(),
            ),
        }
    }

    /// Returns the letter a sequenced block's kind goes by: `b`lue, `r`ed or `c`hain.
    fn kind<N>(block: &SequencedBlock<u64, u64, N>) -> char {
        match block {
            SequencedBlock::Blue(_) => 'b',
            SequencedBlock::Red(_) => 'r',
            SequencedBlock::Chain(_) => 'c',
        }
    }

    /// Returns the update as a JSON object.
    pub fn json(&self) -> String {
        // Numbers as a bare list.
        let list = |xs: &[usize]| {
            xs.iter()
                .map(usize::to_string)
                .collect::<Vec<_>>()
                .join(",")
        };
        match self {
            Self::Reorg {
                common,
                chain,
                reverted,
            } => format!(
                "{{\"reorg\":{{\"common\":{common},\"chain\":[{}],\"reverted\":[{}]}}}}",
                list(chain),
                list(reverted)
            ),
            Self::Advance(blocks) => {
                let blocks: Vec<String> = blocks
                    .iter()
                    .map(|(b, kind)| format!("[{b},\"{kind}\"]"))
                    .collect();
                format!("{{\"advance\":[{}]}}", blocks.join(","))
            }
        }
    }
}

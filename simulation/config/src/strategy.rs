/// How the attacker chooses the parents of its private blocks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Strategy {
    /// Extends its own tip and folds nothing.
    Withhold,
    /// Extends its own tip and folds the other tips of its view whenever the block stays valid.
    Harvest,
    /// Extends two private lineages in turn, each folding the other's tip whenever valid.
    Dag,
    /// Extends its own tip with whichever listing gives the block the highest blue work: nothing,
    /// every tip, any single tip, a foreign tip it saw at one of its recent blocks, or the newest
    /// foreign chain block its tip outranks.
    Greedy,
    /// Keeps the honest miners split: it mines two siblings that start two sides, then always
    /// extends the lighter side's heaviest tip, folding nothing, and shows each block to the half
    /// of the honest miners that favours its side a delay before the other half.
    Balance,
}

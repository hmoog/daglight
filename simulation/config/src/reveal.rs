/// When the attacker publishes its withheld blocks.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Reveal {
    /// After every block mined in the network: a public attacker.
    Immediately,
    /// Every this many seconds.
    Every(f64),
    /// Whenever, after one of its own blocks, the heaviest tip of its view is one of its own.
    WhenAhead,
}

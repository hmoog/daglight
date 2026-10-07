use daglight_protocol_block_perception::NetworkPerception;
use daglight_protocol_parameters::ProtocolParameters;
use daglight_protocol_store::MemoryStore;

use crate::{Attack, RateSwitch, Reveal, Strategy};

/// The network to simulate; times in seconds.
#[derive(Clone, Debug)]
pub struct Config {
    /// The blocks per second the network targets: the protocol's block rate.
    pub rate: f64,
    /// The mean one-way propagation delay between two miners.
    pub delay: f64,
    /// The relative spread of each delivery's delay around `delay`.
    pub jitter: f64,
    /// The honest miners' relative hash rates.
    pub shares: Vec<f64>,
    /// The attack, if any.
    pub attack: Option<Attack>,
    /// The time mining stops; pending deliveries still arrive.
    pub duration: f64,
    /// The random seed.
    pub seed: u64,
    /// The genesis's guess of the delay; `None` for the true `delay`.
    pub assumed_delay: Option<f64>,
    /// The finality horizon, in seconds: five minutes unless set, far shorter than a real network's
    /// so that runs reach it.
    pub finality: f64,
    /// A change of the hash rate during the run, if any.
    pub rate_switch: Option<RateSwitch>,
    /// Whether the blocks keep the genesis's network as it is instead of learning it.
    pub fixed: bool,
    /// The seconds of history nodes keep beyond what they need, if they prune at all.
    pub prune: Option<f64>,
}

impl Config {
    /// The network's total hash rate, in work per second.
    pub const HASH_RATE: f64 = 1_000_000.0;

    /// Creates a network of `miners` equal miners, without jitter or attack, for 60 s.
    pub fn new(rate: f64, delay: f64, miners: usize) -> Self {
        Self {
            rate,
            delay,
            jitter: 0.0,
            shares: vec![1.0; miners.max(1)],
            attack: None,
            duration: 60.0,
            seed: 1,
            assumed_delay: None,
            finality: 300.0,
            rate_switch: None,
            fixed: false,
            prune: None,
        }
    }

    /// Sets the true propagation delay.
    pub fn delay(mut self, seconds: f64) -> Self {
        self.delay = seconds;
        self
    }

    /// Sets the relative spread of each delivery's delay.
    pub fn jitter(mut self, jitter: f64) -> Self {
        self.jitter = jitter;
        self
    }

    /// Replaces the honest miners by `n` equal ones.
    pub fn miners(mut self, n: usize) -> Self {
        self.shares = vec![1.0; n.max(1)];
        self
    }

    /// Gives miner 0 this share of the honest hash rate, the others staying equal.
    pub fn pool(mut self, share: f64) -> Self {
        // The others keep a share of one each; miner 0's is scaled to make up its share.
        let others = (self.shares.len() - 1) as f64;
        self.shares[0] = share / (1.0 - share) * others;
        self
    }

    /// Sets the time mining stops.
    pub fn duration(mut self, seconds: f64) -> Self {
        self.duration = seconds;
        self
    }

    /// Sets the random seed.
    pub fn seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    /// Sets the finality horizon, in seconds.
    pub fn finality(mut self, seconds: f64) -> Self {
        self.finality = seconds;
        self
    }

    /// Sets the genesis's guess of the delay.
    pub fn assumed_delay(mut self, seconds: f64) -> Self {
        self.assumed_delay = Some(seconds);
        self
    }

    /// Has the nodes prune what they can never need again, keeping `da_offset` seconds of history
    /// beyond that.
    pub fn prune(mut self, da_offset: f64) -> Self {
        self.prune = Some(da_offset);
        self
    }

    /// Has the blocks keep the genesis's network as it is: its block work, delay and width.
    pub fn fixed(mut self) -> Self {
        self.fixed = true;
        self
    }

    /// Switches the hash rate at time `at` to the one that mines `rate` blocks per second at the
    /// genesis's block work.
    pub fn rate_switch(mut self, at: f64, rate: f64) -> Self {
        self.rate_switch = Some(RateSwitch { at, rate });
        self
    }

    /// Adds an attacker with this share of the total hash rate, from time `start`.
    pub fn attack(mut self, strategy: Strategy, share: f64, reveal: Reveal, start: f64) -> Self {
        self.attack = Some(Attack {
            share,
            strategy,
            reveal,
            start,
            stamp_pace: None,
        });
        self
    }

    /// Lets the attacker stamp its blocks as if time since the start ran `pace` times as fast.
    pub fn stamp_pace(mut self, pace: f64) -> Self {
        if let Some(a) = self.attack.as_mut() {
            a.stamp_pace = Some(pace);
        }
        self
    }

    /// Returns a time in seconds in milliseconds, as stamps and the protocol count time.
    pub fn millis(seconds: f64) -> u64 {
        (seconds * 1000.0).round() as u64
    }

    /// Returns every miner's share of the total hash rate, the attacker last. The attacker's share
    /// is of the total, so its rate relative to the honest miners' is scaled to their sum.
    pub fn miner_shares(&self) -> Vec<f64> {
        // The attacker's share is of the total, so its weight is scaled to the honest sum.
        let mut shares = self.shares.clone();
        if let Some(a) = self.attack {
            let honest: f64 = shares.iter().sum();
            shares.push(a.share / (1.0 - a.share) * honest);
        }

        // Normalised to one.
        let total: f64 = shares.iter().sum();
        shares.iter().map(|s| s / total).collect()
    }

    /// Returns the attacker's index among the miners, the last, if there is an attack.
    pub fn attacker_index(&self) -> Option<usize> {
        self.attack.map(|_| self.shares.len())
    }

    /// Returns a store holding only the genesis this network starts from.
    pub fn store<N: NetworkPerception<u64>>(&self) -> MemoryStore<u64, u64, N> {
        MemoryStore::new(self.protocol_parameters(), self.network())
    }

    /// Returns the work of one delay, as the genesis guesses it, at the starting hash rate.
    pub fn delay_work(&self) -> u64 {
        (Self::HASH_RATE * self.assumed_delay.unwrap_or(self.delay)).round() as u64
    }

    /// Returns the protocol's parameters: `rate` blocks a second, the run's finality horizon, the
    /// handshake at its default.
    pub fn protocol_parameters(&self) -> ProtocolParameters {
        assert!(
            self.rate >= 1.0 && self.rate.fract() == 0.0,
            "the block rate is a whole number of blocks a second"
        );
        ProtocolParameters::DEFAULT
            .with_bps(self.rate as u64)
            .with_finality(Self::millis(self.finality))
    }

    /// Returns the network the genesis starts: the block work that the starting hash rate mines at
    /// the interval, the guessed delay, and a guess of the width from it, one block plus half the
    /// blocks of a delay, at most eight.
    pub fn network<N: NetworkPerception<u64>>(&self) -> N {
        // The work per block at the starting hash rate, the guessed delay, and the width it implies.
        let block_work = ((Self::HASH_RATE / self.rate).round() as u64).max(1);
        let delay = self.assumed_delay.unwrap_or(self.delay);
        let width = (1.0 + self.rate * delay / 2.0).min(8.0);
        N::genesis(
            &self.protocol_parameters(),
            block_work,
            Self::millis(delay),
            (1e6 * width).round() as u64,
        )
    }
}

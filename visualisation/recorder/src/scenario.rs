use daglight_simulation_config::{Config, Reveal, Strategy};

/// A run the player shows: a simulated network and the name it goes by.
#[derive(Clone, Debug)]
pub struct Scenario {
    /// The scenario's short name, unique among scenarios.
    pub id: &'static str,
    /// What the player calls it.
    pub label: &'static str,
    /// The network simulated.
    pub config: Config,
}

impl Scenario {
    /// Returns the scenarios the player ships with.
    pub fn all() -> Vec<Self> {
        let network = |rate: f64, miners: usize, duration: f64, finality: f64, seed: u64| {
            Config::new(rate, 1.0, miners)
                .jitter(0.4)
                .duration(duration)
                .finality(finality)
                .seed(seed)
        };
        vec![
            Self {
                id: "wide",
                label: "Wide network, 10 blocks/s",
                config: network(10.0, 8, 180.0, 30.0, 1),
            },
            Self {
                id: "wider",
                label: "Very wide network, 100 blocks/s",
                config: network(100.0, 30, 60.0, 30.0, 1),
            },
            Self {
                id: "reveal",
                label: "Withheld chain revealed late",
                config: network(5.0, 8, 150.0, 30.0, 1).attack(
                    Strategy::Withhold,
                    0.2,
                    Reveal::Every(24.0),
                    10.0,
                ),
            },
            Self {
                id: "slow",
                label: "One block a second",
                config: network(1.0, 4, 300.0, 60.0, 12),
            },
        ]
    }
}

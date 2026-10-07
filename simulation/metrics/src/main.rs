use std::env::args;
use std::time::Instant;

use daglight_simulation_config::{Config, Reveal, Strategy};
use daglight_simulation_metrics::Metrics;

/// Runs one simulation from command-line flags and prints its metrics.
fn main() {
    // Flags are `--name value` pairs; `opt` is a flag's value if given.
    let args: Vec<String> = args().skip(1).collect();
    let opt = |name: &str| -> Option<String> {
        args.iter()
            .position(|a| a == name)
            .and_then(|i| args.get(i + 1))
            .cloned()
    };
    let get =
        |name: &str, default: &str| -> String { opt(name).unwrap_or_else(|| default.to_string()) };

    // The network.
    let mut config = Config::new(
        get("--rate", "5").parse().expect("rate"),
        get("--delay", "1").parse().expect("delay"),
        get("--miners", "10").parse().expect("miners"),
    )
    .jitter(get("--jitter", "0.3").parse().expect("jitter"))
    .duration(get("--duration", "60").parse().expect("duration"))
    .seed(get("--seed", "1").parse().expect("seed"));
    if let Some(f) = opt("--finality") {
        config = config.finality(f.parse().expect("finality"));
    }
    if let Some(d) = opt("--assumed-delay") {
        config = config.assumed_delay(d.parse().expect("assumed-delay"));
    }
    if let Some(at) = opt("--rate-switch") {
        let rate = get("--rate2", "1").parse().expect("rate2");
        config = config.rate_switch(at.parse().expect("rate-switch"), rate);
    }
    if let Some(share) = opt("--pool") {
        config = config.pool(share.parse().expect("pool"));
    }

    // The attack, if any.
    if let Some(attack) = opt("--attack") {
        let strategy = match attack.as_str() {
            "withhold" => Strategy::Withhold,
            "harvest" => Strategy::Harvest,
            "dag" => Strategy::Dag,
            "greedy" => Strategy::Greedy,
            "balance" => Strategy::Balance,
            _ => panic!("--attack withhold|harvest|dag|greedy|balance"),
        };
        let reveal = match get("--reveal", "ahead").as_str() {
            "ahead" => Reveal::WhenAhead,
            "now" => Reveal::Immediately,
            t => Reveal::Every(t.parse().expect("--reveal ahead|now|<seconds>")),
        };
        config = config.attack(
            strategy,
            get("--share", "0.3").parse().expect("share"),
            reveal,
            get("--attack-start", "0").parse().expect("attack-start"),
        );
        if let Some(pace) = opt("--stamp-pace") {
            config = config.stamp_pace(pace.parse().expect("stamp-pace"));
        }
    }

    if let Some(da_offset) = opt("--prune") {
        config = config.prune(da_offset.parse().expect("prune"));
    }
    if args.iter().any(|a| a == "--fixed") {
        config = config.fixed();
    }

    // Run, measure and print.
    let start = Instant::now();
    let metrics = Metrics::outcome(config.clone());
    println!(
        "rate {} delay {} jitter {} miners {} duration {} seed {} attack {:?} fixed {}",
        config.rate,
        config.delay,
        config.jitter,
        config.shares.len(),
        config.duration,
        config.seed,
        config.attack,
        config.fixed
    );
    println!("{}", metrics.header());
    println!("{}", metrics.row());
    println!(
        "{:.1} s, {} archive loads, {} lanes",
        start.elapsed().as_secs_f64(),
        metrics.archive_loads,
        metrics.lanes
    );
}

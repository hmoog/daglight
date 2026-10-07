//! Pruning changes nothing a node concludes: every miner ends on the same tip with the same
//! perception, the run measures the same, and less is kept.

use daglight_protocol_dag::DagStore;
use daglight_protocol_node::DagHandle;
use daglight_simulation_config::{Config, Reveal, Strategy};
use daglight_simulation_metrics::Metrics;
use daglight_simulation_network::Simulation;

/// Runs `config` with and without pruning and compares every miner's final tip and perception, and
/// the measured row.
fn same_with_and_without(config: Config) {
    let mut full = Simulation::new(config.clone());
    full.run();
    let mut pruned = Simulation::new(config.prune(0.0));
    pruned.run();
    for (a, b) in full.miners.iter().zip(&pruned.miners) {
        let (ta, tb) = (a.node.heaviest(), b.node.heaviest());
        assert_eq!(ta, tb, "the same tip");
        assert_eq!(
            a.node.perception(ta),
            b.node.perception(tb),
            "the same perception"
        );
    }
    assert_eq!(
        Metrics::measure(&full).row(),
        Metrics::measure(&pruned).row()
    );
    let blocks = |sim: &Simulation| sim.dag().with(|dag| dag.store().topology().len());
    let (kept, all) = (blocks(&pruned), blocks(&full));
    assert!(kept < all, "{kept} of {all} blocks kept");
}

/// An honest network over more than three finality horizons.
#[test]
#[ignore]
fn honest() {
    same_with_and_without(Config::new(20.0, 1.0, 10).jitter(0.3).duration(1500.0));
}

/// A withholder that reveals every minute, whose late blocks fork below finality.
#[test]
#[ignore]
fn withholding() {
    same_with_and_without(
        Config::new(20.0, 1.0, 10)
            .jitter(0.3)
            .duration(1500.0)
            .attack(Strategy::Withhold, 0.45, Reveal::Every(60.0), 0.0),
    );
}

/// A greedy attacker that imports honest work while its lineage can.
#[test]
#[ignore]
fn greedy() {
    same_with_and_without(
        Config::new(20.0, 1.0, 10)
            .jitter(0.3)
            .duration(1500.0)
            .attack(Strategy::Greedy, 0.47, Reveal::WhenAhead, 0.0),
    );
}

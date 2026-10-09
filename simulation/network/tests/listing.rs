//! Listing never costs a lineage what it had: what it once had foldable at a fork stays foldable,
//! so no honest block weighs less than its selected parent at the width the block was weighed by,
//! whatever heavier rival it lists. What listing can cost is a hold not yet taken: a block whose
//! merged blocks lift a rival past its side holds nothing there, where listing nothing it would
//! have; about one honest block in a hundred, by a few blocks of work, and not asserted away here.

use daglight_protocol_block_perception::NetworkPerception;
use daglight_protocol_dag::DagStore;
use daglight_protocol_node::DagHandle;
use daglight_simulation_config::{Config, Reveal, Strategy};
use daglight_simulation_network::Simulation;

/// Runs `config` and checks every honest block against its selected parent at the same width.
fn never_lighter_than_its_parent(config: Config) {
    let mut sim = Simulation::new(config.clone());
    sim.run();
    let attacker = config.attacker_index();
    let checked = sim.dag().with(|dag| {
        let store = dag.store();
        let mut checked = 0;
        for (&id, mined) in &sim.history.blocks {
            if Some(mined.miner) == attacker || store.topology().address(id).is_none() {
                continue;
            }
            let parent = store.perception(store.block(id).selected_parent);
            let own = store.perception(id).blue_work;
            let selected = parent.network.weigh(
                parent.chain_work,
                parent.chain_work + parent.forks.acknowledged(),
            );
            assert!(
                own >= selected,
                "block {id} weighs {own}, its parent {selected} at the same width"
            );
            checked += 1;
        }
        checked
    });
    assert!(checked > 0, "honest blocks checked");
}

/// An honest network, whose tips list one another's lineages all the time.
#[test]
fn honest() {
    never_lighter_than_its_parent(Config::new(20.0, 1.0, 10).jitter(0.3).duration(30.0));
}

/// A public spine, whose lineage honest blocks list while it leads.
#[test]
fn spine() {
    never_lighter_than_its_parent(
        Config::new(20.0, 1.0, 10)
            .jitter(0.3)
            .duration(30.0)
            .seed(2)
            .attack(Strategy::Withhold, 0.45, Reveal::Immediately, 10.0),
    );
}

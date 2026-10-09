//! The chain leans to the largest miner: the heaviest tip is the one with the most work in its
//! past, and a miner's own tip always holds its own last delay of blocks, which no other tip can
//! yet. Any rule that chooses the heaviest past has this, GHOSTDAG included; weighing by width
//! tempers it, as that run of own blocks is a lone chain until the network entangles it.

use daglight_protocol_dag::DagStore;
use daglight_protocol_node::DagHandle;
use daglight_simulation_config::Config;
use daglight_simulation_network::Simulation;

/// Runs `config` and returns how many blocks of miner 0's final chain miner 0 mined, and the
/// chain's length, once everything published has arrived.
fn chain_blocks_of_miner_0(config: Config) -> (usize, usize) {
    let mut sim = Simulation::new(config);
    sim.run();

    // The chain as recorded, then up to miner 0's heaviest tip.
    let top = sim.miners[0].node.heaviest();
    let mut chain = sim.history.final_chain.clone();
    sim.dag()
        .with(|dag| chain.extend_to(dag, dag.store().address(top)));
    let own = chain.blocks[1..]
        .iter()
        .filter(|&c| sim.history.blocks[c].miner == 0)
        .count();
    (own, chain.blocks.len() - 1)
}

/// Ten miners at twenty blocks a second with a one-second delay and 30% jitter, for 120 s.
fn network() -> Config {
    Config::new(20.0, 1.0, 10).jitter(0.3).duration(120.0)
}

/// One of ten equal miners mines some of the chain blocks, as each of them does.
#[test]
fn equal_miners_share_the_chain() {
    assert_eq!(chain_blocks_of_miner_0(network()), (35, 291));
}

/// A pool of a fifth of the hash rate mines nearly half the chain blocks.
#[test]
fn a_fifth_mines_nearly_half() {
    assert_eq!(chain_blocks_of_miner_0(network().pool(0.2)), (134, 293));
}

/// A pool of 30% mines two in three.
#[test]
fn thirty_percent_mines_two_in_three() {
    assert_eq!(chain_blocks_of_miner_0(network().pool(0.3)), (271, 408));
}

/// A pool of 40% mines six in seven.
#[test]
fn forty_percent_mines_six_in_seven() {
    assert_eq!(chain_blocks_of_miner_0(network().pool(0.4)), (551, 644));
}

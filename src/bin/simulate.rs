use std::env;

use property_trading_rl::{Action, GameState, SimpleRng};

fn main() {
    let games = arg_or_default(1, 1000).max(1);
    let players = arg_or_default(2, 4).clamp(2, 8);
    let max_steps = arg_or_default(3, 10_000).max(1);

    let mut overall_rng = SimpleRng::seeded(0x5EED_2026);
    let mut wins = vec![0usize; players];
    let mut completed = 0usize;
    let mut total_steps = 0usize;

    for _ in 0..games {
        let seed = ((overall_rng.range(u32::MAX as usize) as u64) << 32)
            ^ (overall_rng.range(u32::MAX as usize) as u64)
            ^ 0xA5A5_5A5A_1234_5678;
        let mut game = GameState::new(players, seed);
        let mut agent_rng = SimpleRng::seeded(seed ^ 0xC0FFEE);
        let mut steps = 0usize;

        while !game.is_done() && steps < max_steps {
            let legal = game.legal_actions();
            if legal.is_empty() {
                break;
            }
            let action = pick_action(&legal, &mut agent_rng);
            let outcome = game.step(action);
            steps += 1;

            if outcome.done {
                break;
            }
        }

        if game.is_done() {
            if let Some((winner, _)) = game.players.iter().enumerate().find(|(_, p)| p.active) {
                wins[winner] += 1;
            }
            completed += 1;
        }
        total_steps += steps;
    }

    println!("Simulated {} games with {} players.", games, players);
    println!("Completed: {}", completed);
    println!("Average steps: {:.1}", total_steps as f64 / games as f64);
    for (idx, count) in wins.iter().enumerate() {
        println!("Player {} wins: {}", idx + 1, count);
    }
}

fn arg_or_default(index: usize, default: usize) -> usize {
    env::args()
        .nth(index)
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(default)
}

fn pick_action(actions: &[Action], rng: &mut SimpleRng) -> Action {
    let idx = rng.range(actions.len());
    actions[idx].clone()
}

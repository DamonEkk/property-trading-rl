use std::io::{self, Write};
use std::time::{SystemTime, UNIX_EPOCH};

use monopoly_cli::{Action, GameState, TradeOffer};

fn main() {
    println!("CLI Monopoly (ML-friendly core)");
    println!("--------------------------------");

    let player_count = prompt_player_count();
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(1);

    let mut game = GameState::new(player_count, seed);

    loop {
        print_status(&game);
        if game.is_done() {
            println!("Game over.");
            break;
        }

        let action = if let Some(square) = game.pending_purchase_square() {
            let price = game.pending_purchase_price().unwrap_or(0);
            loop {
                print!("Buy {} for ${}? (y/n): ", square.name(), price);
                io::stdout().flush().expect("flush stdout");

                let mut input = String::new();
                io::stdin().read_line(&mut input).expect("read input");
                match input.trim().to_lowercase().as_str() {
                    "y" | "yes" => break Action::BuyProperty,
                    "n" | "no" => break Action::DeclineProperty,
                    _ => println!("Please type y or n."),
                }
            }
        } else {
            print!("Enter=roll | t=trade | h=house | o=hotel | j=json | q=quit: ");
            io::stdout().flush().expect("flush stdout");

            let mut input = String::new();
            io::stdin().read_line(&mut input).expect("read input");

            match input.trim().to_lowercase().as_str() {
                "q" => {
                    println!("Goodbye.");
                    break;
                }
                "t" => prompt_trade(&game),
                "h" => Action::BuildHouse {
                    position: prompt_position("Build house on board position") ,
                },
                "o" => Action::BuildHotel {
                    position: prompt_position("Build hotel on board position") ,
                },
                "j" => {
                    println!("{}", game.export_json());
                    continue;
                }
                _ => Action::Roll,
            }
        };

        let outcome = game.step(action);
        for line in outcome.messages {
            println!("{line}");
        }
        println!("Reward: {}", outcome.reward);
        if outcome.done {
            println!("Game over.");
            break;
        }
        println!();
    }
}

fn prompt_player_count() -> usize {
    loop {
        print!("How many players want to play? (2-8): ");
        io::stdout().flush().expect("flush stdout");

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("read input");

        match input.trim().parse::<usize>() {
            Ok(count) if (2..=8).contains(&count) => return count,
            _ => println!("Please enter a number from 2 to 8."),
        }
    }
}

fn print_status(game: &GameState) {
    println!();
    println!("Turn {} | Current player: Player {}", game.turn, game.current_player + 1);
    if game.extra_turn_pending {
        println!("Extra turn pending for the current player.");
    }
    if let Some(square) = game.pending_purchase_square() {
        println!("Pending purchase: {} for ${}", square.name(), game.pending_purchase_price().unwrap_or(0));
    }
    println!("Free Parking pot: ${}", game.free_parking_pot);
    for (index, player) in game.players.iter().enumerate() {
        let square_name = &game.board[player.position];
        println!(
            "P{} {} | cash ${} | pos {} ({}){}",
            index + 1,
            player.name,
            player.cash,
            player.position,
            square_name,
            if player.in_jail { " | in jail" } else { "" }
        );
    }
    println!();
    println!("Board:");
    print!("{}", game.board_snapshot());
}

fn prompt_position(label: &str) -> usize {
    loop {
        print!("{}: ", label);
        io::stdout().flush().expect("flush stdout");

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("read input");
        if let Ok(pos) = input.trim().parse::<usize>() {
            return pos;
        }
        println!("Please enter a valid board position.");
    }
}

fn prompt_trade(game: &GameState) -> Action {
    let from = game.current_player;
    let to = loop {
        let target = prompt_position("Trade with player number (1-based)");
        if target >= 1 && target <= game.players.len() && target - 1 != from && game.players[target - 1].active {
            break target - 1;
        }
        println!("Choose another active player.");
    };

    let cash_from_offer = prompt_i32("Cash you give");
    let cash_to_offer = prompt_i32("Cash you receive");
    let give_positions = prompt_positions("Board positions you give (comma-separated, or blank)");
    let receive_positions = prompt_positions("Board positions you receive (comma-separated, or blank)");

    Action::Trade(TradeOffer {
        from,
        to,
        cash_from_offer,
        cash_to_offer,
        give_positions,
        receive_positions,
    })
}

fn prompt_i32(label: &str) -> i32 {
    loop {
        print!("{}: ", label);
        io::stdout().flush().expect("flush stdout");

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("read input");
        if input.trim().is_empty() {
            return 0;
        }
        if let Ok(value) = input.trim().parse::<i32>() {
            return value.max(0);
        }
        println!("Please enter a valid number.");
    }
}

fn prompt_positions(label: &str) -> Vec<usize> {
    print!("{}: ", label);
    io::stdout().flush().expect("flush stdout");

    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("read input");
    input
        .trim()
        .split(',')
        .filter_map(|part| part.trim().parse::<usize>().ok())
        .collect()
}

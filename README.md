# Property Trading RL

This is a small Rust terminal game built to be easy to extend for machine learning.

## Features

- asks for the number of players
- rolls 2 six-sided dice
- moves around a 40-space board
- includes Jail, Free Parking, Chance, and Community Chest
- supports buying properties and paying rent
- exposes an ML-friendly environment API
- keeps the core game logic separate from the terminal UI

## Notes for ML work

The reusable game core lives in `src/lib.rs`.
That makes it easier to add:

- state serialization
- action spaces
- reward shaping
- self-play or training loops

## CLI commands

- `Enter` = roll
- `t` = trade
- `h` = build house
- `o` = build hotel
- `j` = print JSON state
- `q` = quit

## Run

Install Rust and run:

```bash
cargo run
```

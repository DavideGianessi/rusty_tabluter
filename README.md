# Tabluter (Rust Hashton Tablut bot)

A Rust bot that plays the *Chesani* variant of *Hashton Tablut* for the exam of "Foundation in artificial intelligence".

Compatible with [the tournament server](https://github.com/AGalassi/TablutCompetition)

Internally it represents the 9×9 board as **bitboards** (`u128`) and picks moves using a **Negamax + Alpha-Beta** search with **iterative deepening** and a fixed-size **transposition table**. Leaf positions are scored by a **linear, feature-based evaluation** implemented in `src/eval.rs` and parameterized by weights in `src/weights.rs`.

## Quick start

Requirements: a Rust toolchain that supports edition 2024.

Build a release binary:

```bash
cargo build --release
```

Run interactive debugging mode (local TUI):

```bash
cargo run --release -- --interactive
```

Run as an online bot:

```bash
# <white|black> <time_limit_seconds> <server_ip>
cargo run --release -- white 5 127.0.0.1
```

Play as a human through the TUI while connected to the server:

```bash
cargo run --release -- black 10 127.0.0.1 --human
```

Notes:

- The program expects a plain `server_ip` and connects to port `5800` for White and `5801` for Black (see `src/client.rs`).
- The help text in `src/main.rs` uses `bot ...` as the executable name; when running via Cargo you’ll pass args after `--` as shown above.

## High-level control flow

1. `src/main.rs` parses CLI arguments.
2. If `--interactive`, it starts a local TUI (`src/interactive.rs`).
3. Otherwise it connects to the server (`src/client.rs`), receives a `State`, and whenever it is the bot’s turn:
	 - runs `search::search(...)` (`src/search.rs`)
	 - extracts `best_move`
	 - sends it back to the server.

So the “AI core” is: **move generation** (`State::generate_moves`) → **search** (`src/search.rs`) → **leaf evaluation** (`src/eval.rs`).

## Code map

### Root

- `Cargo.toml`: crate config (name `tabluter`, edition 2024) and dependencies (`termion`, `serde`, `serde_json`).
- `Cargo.lock`: Cargo lockfile.
- `LICENSE`: license.

### `src/`

- `main.rs`
	- Entry point + CLI.
	- Modes:
		- `--interactive`: local TUI for debugging.
		- `<white|black> <time_limit> <server_ip>`: connect and play as a bot.
		- `... --human`: connect to the server but play manually via the TUI.
	- Keeps `history: Vec<u64>` (position hashes) to help draw detection and to feed search.

- `client.rs`
	- TCP client for the game server.
	- Protocol: length-prefixed JSON (4-byte big-endian length + UTF-8 payload).
	- `get_game_state()` maps JSON into a bitboard `State` and sets `win/draw` based on the `turn` field.
	- `send_move()` sends moves in the server’s expected format (`from`/`to` like `a1`, `b4`, ...).

- `board.rs`
	- Board representation and game rules (movement + captures).
	- Data structures:
		- `State { white, black, king, white_to_move, win, draw, hash }`
		- `Move { fr, fc, tr, tc, captured }` where `captured` is a `u128` mask
	- Key operations:
		- parsing/printing: `from_position_string`, `to_position_string`
		- legal move generation: `generate_moves` (ray moves along ranks/files)
		- applying moves: `apply_move` (updates bitboards, captures, side-to-move, win/draw)
		- hashing: `compute_full_hash` + incremental `update_hash` (Zobrist)
	- Variant-specific rules: throne + citadels are encoded as constant masks.

- `zobrist_keys.rs`
	- Zobrist constants (`Z_WHITE`, `Z_BLACK`, `Z_KING`, `Z_TURN`).
	- Used by the transposition table and repetition/draw tracking (`history.contains(hash)`).

- `search.rs`
	- Move search engine.
	- Implemented techniques:
		- iterative deepening (keeps increasing depth until time expires)
		- Negamax with alpha-beta pruning
		- fixed-size transposition table (TT)
		- move ordering using current/previous TT data
		- time control via a helper thread that triggers an early abort near the deadline
	- Returns `SearchResult { value, best_move, stats }`.

- `eval.rs`
	- Feature-based evaluation function.
	- The score is a linear combination of features extracted from the bitboards.

- `weights.rs`
	- `Weights` container + `Weights::new(is_white)`.
	- Two weight sets are used (one when playing White, one when playing Black).

- `interactive.rs`
	- Terminal UI (termion): local debugging and “online human” mode.
	- Renders the board with highlights (citadels/throne/escape squares, cursor, legal targets, captures).

- `bitboardmaker.html`
	- Small HTML/JS tool to build 9×9 masks and obtain their numeric `u128` value (useful for constants in `board.rs` / `eval.rs`).

## Evaluation: what it measures

At a high level, `evaluate(state, weights)` computes a “raw score” as a weighted sum:

`raw_score = Σ f_i(state) * w_i`

Then it applies a special-case correction (`square_formation`) and finally flips the sign so that **higher is better for the side to move** (this is convenient for Negamax).

Some notable features (names as in `src/eval.rs`):

- `white_count`, `black_count`: piece counts (the King is included in `white_count` because it is also in the `white` bitboard).
- `ready`, `balance`: measures how evenly Black is deployed across key sectors/quadrants.
- `first_line`, `second_line`, `third_line`, `solid_control`: pattern checks for Black’s “blocking lines” (with infiltration checks to avoid over-counting).
- `black_in`, `white_out`: presence in inner/outer regions (pressure vs. escape support).
- `white_moves`: White mobility (capped).
- `distance_from_unblocked`: how close the King is to an open side/quadrant.
- `quadrant_advantage`: local material advantage in unblocked quadrants.
- `encirclement`: how many orthogonally adjacent squares around the King are occupied by Black or a citadel.

### Special case: `square_formation`

When there are enough White pieces, the evaluation checks for a 2×2 “square” formation around the King (excluding the throne). If enabled by weights, it can clamp overly optimistic scores in some situations.

## Why search + eval work well together

- `src/search.rs` explores candidate lines deeper and deeper.
- At leaves it calls `evaluate`.
- Alpha-beta prunes branches that cannot affect the final decision.
- The transposition table avoids recomputing positions and improves move ordering.

In short: evaluation estimates position quality, and search uses that estimate to choose the best move under a time budget.

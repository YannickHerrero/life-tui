# life-tui

Conway's Game of Life in your terminal — a minimalist ratatui front-end with
a two-phase workflow: design your seed, then watch it evolve.

```
╭──────────────────── life-tui · run · gen 142 ────────────────────╮
│ ▄▄                                            ╭──── stats ─────╮ │
│ █  ▄▀                                         │ phase     RUN  │ │
│  ▀█▀                                          │ generation 142 │ │
│       ▄▄▄                                     │ living    87   │ │
│       ▀ ▀                                     │ speed     10/s │ │
│           ▄  ▄                                │ births Δ  +12  │ │
│            ██                                 │ deaths Δ  -9   │ │
│             ▀▀                                │                │ │
│   ▄▄▄                                         │ population     │ │
│  ▀   ▀                                        │ ▄▄▄▄██████▄▄▄▄ │ │
│                                               ╰────────────────╯ │
│                   space pause · +/- speed · e edit · ? help · q  │
╰──────────────────────────────────────────────────────────────────╯
```

## Highlights

- **Two phases**: an *edit* phase to place cells, and a *run* phase to watch
  the simulation evolve, with a one-key toggle between them.
- **Half-block rendering** — each terminal row holds two cell rows, so the
  grid looks roughly square and uses the available area densely.
- **Toroidal grid** — edges wrap, so gliders and spaceships keep going.
- **Mouse paint** in edit mode: click and drag to draw cells.
- **Pattern library**: stamp glider, blinker, pulsar, Gosper glider gun,
  and LWSS at the cursor with one keystroke.
- **Random fill** at 25% density for quickly exploring emergent behavior.
- **Live control panel**: generation count, living cells, speed, births
  and deaths per tick, and a population sparkline over recent generations.
- **Adjustable speed** from 1 to 60 generations per second, plus pause and
  single-step.
- **RLE save / load** — patterns persist as standard Game of Life RLE files.
- **Theme-friendly**: uses your terminal's default foreground and background
  for cells, with cyan/magenta accents only for the cursor.

## Install

You need a recent Rust toolchain (edition 2024, currently `cargo` 1.85+).

```sh
cargo install --git https://github.com/YannickHerrero/life-tui life-tui
```

This drops a `life-tui` binary into `~/.cargo/bin/` (make sure that's on
your `PATH`). To run from a clone instead:

```sh
git clone https://github.com/YannickHerrero/life-tui
cd life-tui
cargo run --release
```

A release binary then lands in `target/release/life-tui`.

## Controls

### Edit phase

| Key                    | Action                       |
| ---------------------- | ---------------------------- |
| `arrows` / `hjkl`      | move the cursor              |
| `space`                | toggle the cell at cursor    |
| `mouse click` / `drag` | paint cells (toroidal-safe)  |
| `1`                    | stamp a glider               |
| `2`                    | stamp a blinker              |
| `3`                    | stamp a pulsar               |
| `4`                    | stamp a Gosper glider gun    |
| `5`                    | stamp an LWSS                |
| `r`                    | random fill at 25% density   |
| `c`                    | clear the grid               |
| `w`                    | save current pattern as RLE  |
| `L`                    | load most recent saved RLE   |
| `enter`                | start the simulation         |

### Run phase

| Key      | Action                          |
| -------- | ------------------------------- |
| `space`  | pause / resume                  |
| `s`      | step one generation (paused)    |
| `+` / `-`| adjust speed by 1 generation/s  |
| `e`      | back to edit (state preserved)  |

### Global

| Key          | Action            |
| ------------ | ----------------- |
| `?`          | toggle help       |
| `q` / `esc`  | quit              |

The cursor turns **cyan** when it sits over a dead cell and **magenta** when
it sits over a living one, so you always know whether `space` will birth or
kill the cell underneath.

## Patterns

The built-in library includes the most popular small patterns from the
Game of Life canon:

- **Glider** — the classic lightweight spaceship that travels diagonally.
- **Blinker** — period-2 oscillator, three cells flipping orientation.
- **Pulsar** — period-3 oscillator with striking radial symmetry.
- **Gosper glider gun** — emits a glider every 30 generations forever.
- **LWSS** (Lightweight Spaceship) — moves horizontally across the grid.

Patterns stamp centered on the cursor and wrap around grid edges.

## Save / load

Saving (`w`) writes the bounding box of currently living cells to:

```
$XDG_DATA_HOME/life-tui/saves/<unix-timestamp>.rle
```

(falling back to `~/.local/share/life-tui/saves/` when `XDG_DATA_HOME` is
unset). Loading (`L`) picks the most recently modified `.rle` in that
directory and stamps it at the cursor. The format is the standard
[RLE](https://conwaylife.com/wiki/Run_Length_Encoded) used across the Life
community, so files round-trip with LifeWiki, Golly, and other tools.

## Project layout

```
src/
  main.rs       terminal setup, event loop, key/mouse routing
  app.rs        application state, tick logic, save/load
  grid.rs       toroidal Game of Life step + tests
  patterns.rs   built-in pattern library
  phase.rs      Edit / Run phase enum
  rle.rs        RLE encode + decode + tests
  ui.rs         layout, grid widget, stats panel, footer, help overlay
```

Run the test suite with `cargo test`. It covers the grid step rules
(blinker, glider, block, toroidal wrap) and RLE roundtrips.

## License

MIT or Apache-2.0, at your option. Add a `LICENSE` file before publishing.

## Acknowledgements

- John Conway, for the cellular automaton.
- The [ratatui](https://ratatui.rs) project for the TUI framework.
- The [LifeWiki](https://conwaylife.com/wiki) community for cataloguing
  every pattern under the sun, including the ones bundled here.

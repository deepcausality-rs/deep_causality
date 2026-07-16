## 1. Move the folders (history-preserving)

- [ ] 1.1 Create the parent folder `examples/avionics_examples/cfd/plasma_blackout/` and
      `git mv examples/avionics_examples/cfd/plasma_blackout_corridor examples/avionics_examples/cfd/plasma_blackout/corridor`
- [ ] 1.2 `git mv examples/avionics_examples/cfd/plasma_blackout_weather examples/avionics_examples/cfd/plasma_blackout/weather`
- [ ] 1.3 Confirm `git status` shows renames (R) for every moved file and the old flat folders no longer exist

## 2. Functional path fixups

- [ ] 2.1 `examples/avionics_examples/Cargo.toml`: update the two `[[example]] path` entries to
      `cfd/plasma_blackout/corridor/main.rs` and `cfd/plasma_blackout/weather/main.rs`; binary
      names stay `plasma_blackout_corridor` / `plasma_blackout_weather`
- [ ] 2.2 Corridor `main.rs` (`table_path` closure, was line 65): record path becomes
      `cfd/plasma_blackout/corridor/corridor_branches.csv`
- [ ] 2.3 Weather `model.rs` (`get_audit_dir` / `get_table_path`, were lines 401/406): paths become
      `cfd/plasma_blackout/weather/audit` and `cfd/plasma_blackout/weather/weather_table.csv`

## 3. Docs and live-note sweep

- [ ] 3.1 `examples/avionics_examples/README.md` (lines 24-25): repoint both example links to
      `cfd/plasma_blackout/corridor/README.md` and `cfd/plasma_blackout/weather/README.md`
- [ ] 3.2 Corridor `README.md`: cross-link becomes `../weather/README.md`; correct the stale
      module name `avionics_examples::blackout` to `avionics_examples::shared` (same paragraph,
      "Where Things Live")
- [ ] 3.3 Weather `README.md` (line 12): cross-link becomes `../corridor/README.md`
- [ ] 3.4 Live openspec notes: repoint the old paths in
      `openspec/notes/cfd-plasma-blackout/finite-rate-cfd-chemistry.md` (line 182),
      `openspec/notes/cfd-plasma-blackout/gap-analysis.md` (line 295), and
      `openspec/notes/cfd-plasma-retropulsion/plasma-retropulsion-descent.md` §12 (update the
      two example paths and drop the "paths as of this note; §8 moves them" parenthetical).
      Do not touch `openspec/changes/archive/` or `openspec/notes/archive/`

## 4. Verification (the spec's scenarios)

- [ ] 4.1 `cargo build -p avionics_examples` compiles clean
- [ ] 4.2 `cargo run --release -p avionics_examples --example plasma_blackout_corridor` exits 0
      with all gates passing, and `corridor_branches.csv` is rewritten inside
      `cfd/plasma_blackout/corridor/`
- [ ] 4.3 `cargo run --release -p avionics_examples --example plasma_blackout_weather` exits 0
      with all gates passing (~4 min, 48 descents; run it, do not skip), and
      `weather_table.csv` plus `audit/` land inside `cfd/plasma_blackout/weather/`
- [ ] 4.4 Grep gate: searching the repo for `cfd/plasma_blackout_corridor` or
      `cfd/plasma_blackout_weather` (excluding `openspec/changes/archive/`,
      `openspec/notes/archive/`, `target/`) returns zero matches
- [ ] 4.5 `git log --follow --oneline -- examples/avionics_examples/cfd/plasma_blackout/corridor/main.rs`
      lists the pre-move history
- [ ] 4.6 Prepare the commit message and hand it to the user (never commit; golden rule)

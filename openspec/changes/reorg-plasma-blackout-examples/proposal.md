## Why

The plasma-blackout example family is about to grow a third sibling: the retropulsion descent
designed in `openspec/notes/cfd-plasma-retropulsion/plasma-retropulsion-descent.md` (§8, §9).
Its build order puts Stage R, the folder reorganization, first and alone, so the new example
lands in its final layout once and no later change mixes a mechanical move with new code. This
change is Stage R only: a history-preserving move with path fixups, no new code, no behavior
change.

## What Changes

- `git mv examples/avionics_examples/cfd/plasma_blackout_corridor` →
  `examples/avionics_examples/cfd/plasma_blackout/corridor` (history-preserving).
- `git mv examples/avionics_examples/cfd/plasma_blackout_weather` →
  `examples/avionics_examples/cfd/plasma_blackout/weather`.
- `examples/avionics_examples/Cargo.toml`: the two `[[example]]` entries keep their names
  (`plasma_blackout_corridor`, `plasma_blackout_weather` are user-facing run commands and CI
  strings) and change only their `path`.
- Embedded `CARGO_MANIFEST_DIR`-relative paths move with the folders: the corridor's
  `corridor_branches.csv` record path (`main.rs`), the weather example's audit directory and
  `weather_table.csv` paths (`model.rs`).
- README link sweep: `examples/avionics_examples/README.md` example links, the two example
  READMEs' cross-links to each other; the corridor README's stale module name
  `avionics_examples::blackout` is corrected to `avionics_examples::shared`.
- Live openspec notes that point at the old paths are repointed
  (`cfd-plasma-blackout/finite-rate-cfd-chemistry.md`, `cfd-plasma-blackout/gap-analysis.md`,
  `cfd-plasma-retropulsion/plasma-retropulsion-descent.md` §12). Archived changes and archived
  notes stay untouched; they are historical record.

Out of scope: the `retropulsion/` subfolder (it arrives with its own change), any change to
example behavior, gates, constants, or the shared library `avionics_examples::shared` (which
does not move).

## Capabilities

### New Capabilities

- `blackout-example-layout`: the folder contract of the plasma-blackout example family — one
  parent folder (`cfd/plasma_blackout/`) with one subfolder per example, stable example binary
  names across moves, and each example's recorded artifacts (branch tables, dispersion table,
  audit logs) living inside its own subfolder.

### Modified Capabilities

None. No spec-level behavior changes: `plasma-blackout-flagship` and the other blackout specs
pin behavior and gates, not folder paths, and both examples must produce byte-identical results
before and after the move.

## Impact

- **Affected code**: `examples/avionics_examples/` only — `Cargo.toml`, the crate README, the
  two moved example folders (`main.rs` / `model.rs` path constants, READMEs). No library crate
  is touched; `src/shared/` stays where it is.
- **Build systems**: cargo only. The examples have no Bazel wiring (verified: no `BUILD.bazel`
  under `examples/avionics_examples/`), and all documented run commands use example binary
  names, which do not change.
- **Docs**: three live openspec notes repointed; `deep_causality_cfd/README.md` and
  `CFD_MDAO_PRESENTATION.md` reference binary names only and need no edit.
- **Verification**: `cargo build -p avionics_examples`, then both self-verifying examples run
  green (`plasma_blackout_corridor` ≈ 42 s; `plasma_blackout_weather` ≈ 4 min, 48 descents) and
  write their recorded artifacts into the new subfolders.

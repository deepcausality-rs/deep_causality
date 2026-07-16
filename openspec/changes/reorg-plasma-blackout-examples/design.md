## Context

Two plasma-blackout examples sit today as flat siblings under `examples/avionics_examples/cfd/`:
`plasma_blackout_corridor/` and `plasma_blackout_weather/`. A third sibling (the retropulsion
descent) is designed in `openspec/notes/cfd-plasma-retropulsion/plasma-retropulsion-descent.md`,
whose build order (§9, Stage R) puts this reorganization first and alone so the new example is
born into the final layout.

The full live-reference inventory (grep over `*.rs`, `*.toml`, `*.md`, `*.bazel`, `*.yml`,
excluding `openspec/changes/archive/` and `target/`):

| Reference | Kind |
|---|---|
| `examples/avionics_examples/Cargo.toml:38,42` — two `[[example]] path` entries | functional |
| corridor `main.rs:65` — `join("cfd/plasma_blackout_corridor/corridor_branches.csv")` | functional |
| weather `model.rs:401,406` — `join("cfd/plasma_blackout_weather/audit")`, `join(".../weather_table.csv")` | functional |
| `examples/avionics_examples/README.md:24-25` — links to both example READMEs | docs |
| corridor `README.md:239` — link `../plasma_blackout_weather/README.md`; stale module name `avionics_examples::blackout` nearby | docs |
| weather `README.md:12` — link `../plasma_blackout_corridor/README.md` | docs |
| `openspec/notes/cfd-plasma-blackout/finite-rate-cfd-chemistry.md:182`, `gap-analysis.md:295`, `cfd-plasma-retropulsion/plasma-retropulsion-descent.md:532-533` — path mentions in live notes | docs |
| `deep_causality_cfd/README.md:34`, both `main.rs` doc headers, `CFD_MDAO_PRESENTATION.md:200`, `src/shared/mod.rs:6-7` — binary names only, no paths | none needed |
| archives (`openspec/changes/archive/…`, `openspec/notes/archive/…`) | historical, untouched |

No `BUILD.bazel` exists under `examples/avionics_examples/`; the examples are cargo-run
artifacts only. No `.gitignore` pattern names these folders. The shared library
(`avionics_examples::shared`, `src/shared/`) is imported by module path, not folder path, and
does not move.

## Goals / Non-Goals

**Goals:**

- Both examples live at `examples/avionics_examples/cfd/plasma_blackout/{corridor,weather}/`
  with git history preserved.
- Every live reference resolves; both examples build and their self-verifying gates run green,
  byte-identical in behavior to the pre-move runs.
- Recorded artifacts (`corridor_branches.csv`, `weather_table.csv`, the audit directory) are
  written inside the new subfolders.

**Non-Goals:**

- No `retropulsion/` folder (an empty directory cannot be committed; it arrives with its own
  change).
- No behavior, constant, gate, or shared-library change of any kind.
- No edits to archived changes or archived notes; they are historical record.
- No Bazel work (nothing to update) and no CI edits (run commands use binary names, which do
  not change).

## Decisions

- **Parent folder named `plasma_blackout`, subfolders `corridor` / `weather`.** Matches the
  binary-name prefix and the note's §8 layout verbatim. Alternative considered: keeping full
  names as subfolders (`plasma_blackout/plasma_blackout_corridor/`); rejected as redundant
  stutter in every path.
- **Binary names stay; only `path` changes in `Cargo.toml`.** The names are user-facing run
  commands quoted in five READMEs/docs and possibly muscle memory and CI strings. Renaming
  would turn a mechanical move into a breaking one for zero benefit.
- **`git mv`, never delete-and-add.** Preserves per-file history (AGENTS.md authorizes mv;
  deletion is barred).
- **Reference-sweep policy: functional paths and live docs are repointed; archives are not.**
  Archived proposals/notes describe where things lived when they landed; rewriting them would
  falsify the record. The retropulsion note's §12 parenthetical ("paths as of this note; §8
  moves them") is replaced by the new paths once they are real.
- **Verification is the examples' own gate suites plus a grep gate.** Both binaries
  self-verify and exit nonzero on regression, so "run both, expect exit 0" is the behavior
  proof; a repo grep for the old paths (archives excluded) proves the sweep is complete. No new
  test scaffolding is warranted for a move.

## Risks / Trade-offs

- [Missed embedded path → example writes artifacts to a dead location or errors] → The grep
  inventory above is exhaustive at proposal time; the tasks re-run the same grep after the move
  as a gate, and both examples are executed end to end.
- [Weather example wall-clock (~4 min, 48 descents) tempts skipping its run] → Run it anyway;
  it is the only proof that `get_table_path()`/`get_audit_dir()` write into the new folder and
  that all eight gates still pass.
- [History display: `git log` on new paths needs `--follow`] → Inherent to any move; `git mv`
  keeps rename detection cheap and reliable.
- [Docs elsewhere (website, blog) linking to GitHub paths of the old folders] → Repo grep found
  none outside archives; external links cannot be fixed from this repo and the old GitHub URLs
  will 404 on the moved folders. Accepted; the README at `examples/avionics_examples/` remains
  the stable entry point.

## Migration Plan

Single-session, single-commit move (the user commits; a prepared commit message is handed over
at the end):

1. `git mv` both folders into `cfd/plasma_blackout/`.
2. Path fixups (Cargo.toml, the two embedded path constants, READMEs, live notes).
3. `cargo build -p avionics_examples`, run both examples, re-run the reference grep.
4. Rollback, if ever needed, is `git revert` of the single commit.

## Open Questions

None. The layout, naming, and sweep policy are pinned by the design note §8 and the decisions
above.

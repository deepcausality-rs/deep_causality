# Proposal: add-cfd-file-io

## Why

The everyday CFD example set (`openspec/notes/cfd-examples/common-examples.md`) is blocked on
its Group 1 precondition: every planned example reads an input table and writes a result table,
example 5 ingests measured sensor traces, and the everyday research workflow needs to suspend a
running `CfdFlow` state to disk and resume it from a different workflow days later. None of
these file seams exist; `deep_causality_file` today ships only the RINEX GNSS loaders, and the
CFD side owns only bare `write_csv` / `write_xy_csv` output.

## What Changes

- `deep_causality_file` gains a **generic typed table reader** for delimited numeric tables
  (test matrices, atmosphere tables, flow-rate schedules), expressed over the `IoAction`
  pattern like the existing GNSS loaders, precision-generic (parse to exact `f64`, lift via the
  caller's convention).
- `deep_causality_file` gains a **result-table writer** that carries column headers and units
  with the data, designed as the table payload of the future self-describing results archive
  (roadmap item 5) so everyday outputs become archive-ready without rework.
- `deep_causality_file` gains a **sensor-trace loader**: time-stamped noisy samples per
  channel, shaped for `Uncertain<T>` / `MaybeUncertain<T>` construction on the way in (an
  intermittently reporting channel maps to `MaybeUncertain`).
- **Snapshot and resume** for a running CFD state, in two tiers: a *field snapshot* (tensor
  cores with ranks plus metadata) and a *full resume package* (cores, carried scalar fields,
  navigation engine state, provenance log, step index, scalar type, and a world-description
  fingerprint). Saving is one line in the flow (a path); loading from a different workflow
  restores the package bit-exactly. A hash checksum over the whole package is computed at save
  and verified at load; a mismatch reports a corrupt-file error naming the file, overridable
  with an explicit `force_load` so the user is always informed first. Loading into a world
  whose fingerprint does not match is refused loudly.
- `deep_causality_cfd` surfaces the snapshot seam on the flow side (save from a paused march,
  resume a march from a loaded package). No solver or physics behavior changes.

## Capabilities

### New Capabilities

- `typed-table-io`: reading delimited numeric tables into typed, precision-generic rows and
  writing result tables with headers and units, both as lazy `IoAction` values.
- `sensor-trace-io`: loading time-stamped, per-channel noisy sensor traces shaped for
  uncertain-type construction, as a lazy `IoAction`.
- `cfd-state-snapshot`: the two-tier snapshot package (field snapshot, full resume), its
  bit-exact binary format, the whole-package checksum with corrupt-file reporting and
  `force_load`, the world-fingerprint validation, and the one-line save / load seam on the
  CFD flow side.

### Modified Capabilities

<!-- none: the CfdFlow additions are covered by the new cfd-state-snapshot capability; no
existing spec's requirements change -->

## Impact

- `deep_causality_file`: new loader and writer modules beside the GNSS loaders; new snapshot
  package module; new error variants for corrupt files and fingerprint mismatches; tests
  mirrored under `tests/`.
- `deep_causality_cfd`: a save hook on the paused-march surface and a resume entry point that
  accepts a loaded package; re-exports of the file-crate types it consumes.
- `deep_causality_tensor` (read-only dependency): the snapshot serializes existing tensor-train
  state; no tensor API changes expected beyond accessors that already exist.
- Downstream: unblocks all five everyday examples (Group 2 and Group 3 of the note) and
  back-fills the payload convention that roadmap items 4 (ROM export) and 5 (self-describing
  results) both need.
- No breaking changes; all additions.

# Design: add-cfd-file-io

## Context

`deep_causality_file` ships IO-monad loaders (`deep_causality_haft::IoAction`, lazy until
`.run()`) for RINEX GNSS products, precision-generic over `R`. The CFD side owns only
`write_csv` / `write_xy_csv`. The everyday example set (Group 1 of
`openspec/notes/cfd-examples/common-examples.md`) needs four file seams: a typed table reader,
a units-aware result-table writer, a sensor-trace loader, and two-tier snapshot/resume for a
running CFD state with whole-package checksum integrity. Tensor-train states make snapshots
cheap: the serialized artifact is the compressed representation itself, order `chi^2 * L`
numbers.

## Goals / Non-Goals

**Goals:**

- All four seams as lazy `IoAction` values, matching the GNSS loader pattern: describing a read
  or write performs no side effect until `.run()`.
- Bit-exact round-trip for every supported scalar (`f32`, `f64`, `Float106`), so a resumed run
  preserves the house bit-reproducibility guarantee.
- Integrity by construction: a checksum over the whole package, verified before any content is
  interpreted; corruption is reported, never silently consumed; `force_load` exists for salvage
  and is always explicit.
- Clean dependency direction: `deep_causality_file` knows containers and tables, never CFD
  types; `deep_causality_cfd` packs and unpacks its own state.

**Non-Goals:**

- The self-describing results archive itself (roadmap item 5); this change only shapes the
  table writer and snapshot container so the archive can adopt them unchanged.
- Canonical config serialization; the world fingerprint is a seam (caller-supplied bytes,
  hashed) until item 5 defines the canonical form.
- `Uncertain<T>` construction inside the file crate; the loader returns plain typed samples and
  the uncertain lift happens in the consumer (keeps `deep_causality_file` free of the
  `deep_causality_uncertain` dependency).
- Any solver, physics, or gate behavior change.

## Decisions

**D1: One binary container type, tier-agnostic, in `deep_causality_file`.** The snapshot format
is a versioned binary container: a fixed header (magic, format version, scalar-type tag, tier
tag, world-fingerprint hash, package checksum) followed by named sections, each a length-
prefixed byte blob. Little-endian, explicit layout, no padding ambiguity. The container knows
nothing about CFD; it stores named blobs. `deep_causality_cfd` defines what goes into the
sections (D5). This keeps `deep_causality_file` dependency-clean and makes the container
reusable as the field payload of the future archive.

**D2: Checksum is corruption detection, not cryptography.** A 64-bit FNV-1a implemented in the
crate (a dozen lines, no new dependency, workspace no-external-deps posture preserved) computed
over everything after the checksum field, written at save, verified at load before any section
is parsed. On mismatch the loader returns a corrupt-file error naming the path.
`force_load` skips the checksum and fingerprint checks, never the scalar-type check: a wrong
scalar cannot be reinterpreted, only refused.

**D3: Bit-exact scalar encoding.** Values are stored as raw bit patterns: `f64` as `u64` LE,
`f32` as `u32` LE, `Float106` as its two `f64` components. The header's scalar-type tag is
authoritative; loading a package into a program whose `FloatType` differs is an error with no
override. This is what makes a resumed run bit-identical to an unsuspended one.

**D4: The world fingerprint is a seam, not a schema.** The saver hashes caller-supplied
fingerprint bytes (an example hashes its constants; later, the canonical config serialization
from roadmap item 5 becomes the input) and stores the digest in the header. The loader recomputes
from the current world's bytes and refuses a mismatch loudly (`force_load` overrides, with the
mismatch still reported). The seam ships now; the canonical input arrives with item 5.

**D5: Two tiers, packed by the CFD crate.** `deep_causality_file` owns the container;
`deep_causality_cfd` owns the packing. Tier one, the *field snapshot*: named tensor-train
fields (cores with ranks and mode metadata) plus grid metadata. Tier two, the *full resume
package*: the field snapshot sections plus carried scalar fields, the navigation engine state
(filter vector and covariance), the provenance log entries, and the step index. The flow-side
surface is one line each way: save from a paused march to a path, and a resume entry point that
accepts a loaded package in place of an initial field. Dependency direction stays acyclic
(`deep_causality_cfd` depends on `deep_causality_file`, never the reverse).

**D6: Tables are CSV with a two-row header.** The typed table reader parses delimited numeric
tables: row one is column names, an optional row two prefixed `#units` carries units, then
numeric rows. Values parse to exact `f64` and lift into `R` at the boundary (the house `ft`
convention), so specification tables keep exact literals. The result-table writer emits the
same shape (name row, `#units` row, data rows), which makes every everyday output
self-labeling, spreadsheet-readable, and ready to become the archive's table payload verbatim.

**D7: Sensor traces are plain typed samples with gaps.** The trace loader returns per-channel
time-stamped samples; a missing sample at a timestamp is represented as absent, not as a
sentinel value. The mapping to `MaybeUncertain` (presence) and `Uncertain` (noise) happens in
the consumer, per the presence-gate note's recorded design.

## Risks / Trade-offs

- **A format is forever.** Mitigated by the version field in the header, refusing unknown
  versions loudly, and keeping the container minimal (named blobs, not schema).
- **FNV-1a is not tamper-proof.** Accepted: the threat model is corruption (truncated writes,
  bit rot, partial copies), not adversaries. The choice is recorded in the container docs so
  nobody mistakes it for a security boundary.
- **Fingerprint quality depends on the caller until item 5 lands.** Accepted and documented; a
  weak fingerprint degrades to the status quo (no validation), never to silent corruption,
  because the checksum still covers the content.
- **Nav-engine and provenance serialization couples the package to those types' shapes.**
  Mitigated by the section structure: each section carries its own version byte, so a nav-state
  layout change bumps one section, not the container.
- **Stale snapshots as a research cache.** The fingerprint check is the guard; the recorded
  workflow (load for days while research concludes) is safe exactly because a config edit
  mid-week flips the fingerprint and the loader says so.

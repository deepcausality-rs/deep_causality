## 1. Table IO (deep_causality_file)

- [x] 1.1 The typed table reader as an `IoAction`: two-row header (names, optional `#units`),
  exact-`f64` parse lifted into `R` at the boundary, descriptive errors naming path and row for
  ragged rows, non-numeric cells, and missing headers. Module beside the GNSS loaders; types in
  `types/`; registered in `lib.rs`.
  -> `loaders/read_table.rs`: `ReadTable`/`read_table`, two-row header, exact-f64 lift,
  errors name path and significant row. `NumericTable`/`TableColumn` in `types/table_types.rs`.
- [x] 1.2 The result-table writer emitting the same two-row-header shape; write-read round trip
  preserves names, units, and `f64` bits.
  -> `writers/write_table.rs`: `WriteTable`/`write_table`, shortest-round-trip f64 formatting;
  round trip verified bit-for-bit. Units row convention: `#units` marker + one cell per column.
- [x] 1.3 Mirrored tests: laziness (no filesystem touch before `.run()`), typed round trip,
  malformed-input errors, empty table, single-column table. Bazel registration for the new test
  modules.
  -> 9 reader + 3 writer + 2 table-type tests; laziness proven by describing the read before
  the file exists. Bazel: new `writers` suite; `types`/`loaders` globs cover the rest.

## 2. Sensor-trace IO (deep_causality_file)

- [x] 2.1 The trace loader as an `IoAction`: per-channel `(timestamp, value)` samples in `R`,
  absent entries for missing samples (no sentinels), no `deep_causality_uncertain` dependency.
  -> `loaders/read_trace.rs`: `ReadSensorTrace`/`read_sensor_trace`; empty cell = `None`,
  no uncertain dependency; `SensorTraceSet`/`SensorChannel` in `types/trace_types.rs`.
- [x] 2.2 Mirrored tests: gap preservation on an intermittent channel, laziness, typed lift,
  malformed-trace errors.
  -> 5 loader + 1 type test: gap preservation, laziness, missing-timestamp and
  non-numeric-sample errors, minimum-channel check.

## 3. Snapshot container (deep_causality_file)

- [x] 3.1 The binary container: versioned header (magic, version, scalar-type tag, tier tag,
  fingerprint digest, checksum), named length-prefixed sections with per-section version bytes,
  little-endian. In-crate 64-bit FNV-1a for the package checksum (documented as corruption
  detection, not a security boundary).
  -> `snapshot/container.rs` (layout documented in the module header) + `snapshot/checksum.rs`
  (in-crate FNV-1a 64, documented as corruption detection, not a security boundary).
- [x] 3.2 Save and load as `IoAction`s: checksum computed at save, verified at load before any
  section is parsed; corrupt-file error naming the path; `force_load` skipping checksum and
  fingerprint (mismatch still reported) but never the scalar-type check; unknown versions
  refused.
  -> `snapshot/io.rs`: `save_snapshot`, `load_snapshot` (checksum verified before any section
  is parsed), `force_load_snapshot` returning `(package, warnings)`; unknown versions refuse in
  both modes; scalar mismatch never overridable.
- [x] 3.3 Bit-exact scalar encoding for `f32`, `f64`, `Float106` (raw bit patterns, tag
  authoritative, no cross-scalar loading).
  -> `BitCodec` in `types/snapshot_types.rs`: raw LE bit patterns for `f32`/`f64`/`Float106`
  (hi/lo components); header scalar tag authoritative.
- [x] 3.4 Mirrored tests: header round trip, checksum catches a flipped byte, `force_load`
  semantics, scalar-tag refusal, unknown-version refusal, per-section versioning, empty and
  multi-section packages.
  -> 8 container tests incl. flipped-byte corruption (content-level, structure intact),
  force-load warning semantics, scalar/version refusals, empty package, inspection mode
  (fingerprint `None`), bad-magic file. 99 file-crate tests green; Bazel 19/19.

## 4. CFD packing and the flow seam (deep_causality_cfd)

- [x] 4.1 Field-snapshot packing: named tensor-train fields (cores, ranks, mode metadata) plus
  grid metadata into container sections; unpacking restores identical TT states.
  -> `types/flow/state_snapshot.rs`: `pack_tt_fields`/`unpack_tt_fields` over
  `CausalTensorTrain::cores()`/`from_cores`; cores round-trip bit-exact (verified).
- [x] 4.2 Full resume package: field snapshot plus carried scalar fields, navigation engine
  state, provenance log, and step index; the restored log appends after the recorded entries.
  -> `pack_resume`/`unpack_resume`: scalars, aero/control channels, ambient (body-forced
  ambients refused, section v1), full nav engine (new `restore` surfaces on `ReentryNavEngine`
  and `NavFilter`, `covariance()` accessor, `EffectLog::messages()` in core), provenance
  message sequence (re-stamped; EffectLog equality ignores timestamps by contract), step index.
- [x] 4.3 The one-line flow surface: save from a paused march to a path; a resume entry point
  accepting a loaded package in place of an initial field; fingerprint bytes supplied by the
  caller, digest stored and verified per the container rules.
  -> One-liners: `CarrierPause::save_state_snapshot(path, fingerprint)` on any paused march;
  `save_resume_state`/`load_resume_state` free functions (strict load validates checksum,
  scalar, and world fingerprint). Fingerprint = caller bytes via `fingerprint64`.
- [x] 4.4 Mirrored tests: save/resume bit-identity (a suspended-and-resumed march matches the
  unsuspended march bit for bit), stale-fingerprint refusal, cross-workflow resume (save in one
  test body, load in another), tier separation.
  -> 5 mirrored tests: disk round trip bit-exact (scalars, nav state + covariance, log,
  step), post-resume stage step bit-identical to the unsuspended field, stale-fingerprint
  refusal at the seam, tier separation both directions, TT-core bit-exact round trip.

## 5. Finalize

- [x] 5.1 `make format && make fix`; full file, cfd, and physics suites green; clippy clean
  (fix, never allow); Bazel registration for every new test module; no `unsafe`/`dyn`/lib
  macros; float literals only in tests and cited constants.
  -> fmt clean; clippy 0 across file + cfd (all targets); 99 file + 604 cfd tests green,
  physics suites green; stagline 7/7 and corridor 13/13 gates unchanged; Bazel 19/19 on the
  file crate and the extended `types_flow` suite.
- [x] 5.2 `openspec validate add-cfd-file-io --strict`; update the common-examples note (Group
  1 status) and the roadmap note (items 4 and 5 gain their payload foundation); prepare the
  commit message for review (never commit).
  -> Validates strict. Notes updated (common-examples Group 1 status, roadmap items 4/5
  foundation). Commit message prepared and handed to the user (never committed).

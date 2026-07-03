## 1. Table IO (deep_causality_file)

- [ ] 1.1 The typed table reader as an `IoAction`: two-row header (names, optional `#units`),
  exact-`f64` parse lifted into `R` at the boundary, descriptive errors naming path and row for
  ragged rows, non-numeric cells, and missing headers. Module beside the GNSS loaders; types in
  `types/`; registered in `lib.rs`.
- [ ] 1.2 The result-table writer emitting the same two-row-header shape; write-read round trip
  preserves names, units, and `f64` bits.
- [ ] 1.3 Mirrored tests: laziness (no filesystem touch before `.run()`), typed round trip,
  malformed-input errors, empty table, single-column table. Bazel registration for the new test
  modules.

## 2. Sensor-trace IO (deep_causality_file)

- [ ] 2.1 The trace loader as an `IoAction`: per-channel `(timestamp, value)` samples in `R`,
  absent entries for missing samples (no sentinels), no `deep_causality_uncertain` dependency.
- [ ] 2.2 Mirrored tests: gap preservation on an intermittent channel, laziness, typed lift,
  malformed-trace errors.

## 3. Snapshot container (deep_causality_file)

- [ ] 3.1 The binary container: versioned header (magic, version, scalar-type tag, tier tag,
  fingerprint digest, checksum), named length-prefixed sections with per-section version bytes,
  little-endian. In-crate 64-bit FNV-1a for the package checksum (documented as corruption
  detection, not a security boundary).
- [ ] 3.2 Save and load as `IoAction`s: checksum computed at save, verified at load before any
  section is parsed; corrupt-file error naming the path; `force_load` skipping checksum and
  fingerprint (mismatch still reported) but never the scalar-type check; unknown versions
  refused.
- [ ] 3.3 Bit-exact scalar encoding for `f32`, `f64`, `Float106` (raw bit patterns, tag
  authoritative, no cross-scalar loading).
- [ ] 3.4 Mirrored tests: header round trip, checksum catches a flipped byte, `force_load`
  semantics, scalar-tag refusal, unknown-version refusal, per-section versioning, empty and
  multi-section packages.

## 4. CFD packing and the flow seam (deep_causality_cfd)

- [ ] 4.1 Field-snapshot packing: named tensor-train fields (cores, ranks, mode metadata) plus
  grid metadata into container sections; unpacking restores identical TT states.
- [ ] 4.2 Full resume package: field snapshot plus carried scalar fields, navigation engine
  state, provenance log, and step index; the restored log appends after the recorded entries.
- [ ] 4.3 The one-line flow surface: save from a paused march to a path; a resume entry point
  accepting a loaded package in place of an initial field; fingerprint bytes supplied by the
  caller, digest stored and verified per the container rules.
- [ ] 4.4 Mirrored tests: save/resume bit-identity (a suspended-and-resumed march matches the
  unsuspended march bit for bit), stale-fingerprint refusal, cross-workflow resume (save in one
  test body, load in another), tier separation.

## 5. Finalize

- [ ] 5.1 `make format && make fix`; full file, cfd, and physics suites green; clippy clean
  (fix, never allow); Bazel registration for every new test module; no `unsafe`/`dyn`/lib
  macros; float literals only in tests and cited constants.
- [ ] 5.2 `openspec validate add-cfd-file-io --strict`; update the common-examples note (Group
  1 status) and the roadmap note (items 4 and 5 gain their payload foundation); prepare the
  commit message for review (never commit).

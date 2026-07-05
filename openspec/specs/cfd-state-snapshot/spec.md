# cfd-state-snapshot Specification

## Purpose
The two-tier snapshot of a running CFD state: a checksummed, fingerprinted, bit-exact binary container in deep_causality_file, packed and unpacked by deep_causality_cfd (field snapshots of tensor-train fields; full resume packages carrying scalars, channels, ambient, navigation engine, provenance log, and step), with a one-line save on a paused march and a strict cross-workflow load.

## Requirements
### Requirement: A versioned, checksummed snapshot container

`deep_causality_file` SHALL provide a binary snapshot container: a versioned header (magic,
format version, scalar-type tag, tier tag, world-fingerprint digest, package checksum) followed
by named, length-prefixed sections, little-endian throughout. A checksum over the entire
package after the checksum field SHALL be computed at save and verified at load **before any
section is interpreted**. On mismatch the loader SHALL return a corrupt-file error naming the
path. An explicit `force_load` SHALL skip the checksum and fingerprint checks (with the
mismatch still reported), and SHALL NOT skip the scalar-type check. Unknown format versions
SHALL be refused loudly.

#### Scenario: Corruption is reported, never consumed

- **WHEN** a snapshot file whose bytes were altered after save is loaded normally
- **THEN** the load fails with a corrupt-file error naming the file, and no section content is
  interpreted

#### Scenario: Force load informs and proceeds

- **WHEN** the same corrupted file is loaded with `force_load`
- **THEN** the checksum mismatch is reported, the load proceeds at the caller's explicit risk,
  and a scalar-type mismatch still refuses

### Requirement: Bit-exact scalar round trip

Snapshot values SHALL be stored as raw bit patterns (`f64` as little-endian `u64`, `f32` as
`u32`, `Float106` as its two `f64` components), with the header's scalar-type tag
authoritative. Loading a package into a program whose working scalar differs from the tag
SHALL be an error with no override. A state saved and reloaded SHALL be bit-identical to the
state that was never suspended.

#### Scenario: Suspend and resume changes no bits

- **WHEN** a paused state is saved, the process ends, and a new workflow loads the package and
  continues the march
- **THEN** the continued run produces bit-identical results to the same march run without
  suspension

### Requirement: Two snapshot tiers packed by the CFD crate

The container SHALL support two tiers. The *field snapshot* carries named tensor-train fields
(cores, ranks, mode metadata) and grid metadata. The *full resume package* additionally
carries the carried scalar fields, the navigation engine state, the provenance log, and the
step index. `deep_causality_cfd` SHALL own the packing and unpacking of its own state;
`deep_causality_file` SHALL know only the container. Each section SHALL carry its own version
byte so a single section's layout can evolve without breaking the container.

#### Scenario: The full package carries the state's passengers

- **WHEN** a full resume package is saved from a paused march and inspected after load
- **THEN** it contains the tensor fields, the scalar fields, the navigation engine state, the
  provenance log entries, and the step index, and the restored provenance log continues
  appending after the recorded entries

### Requirement: One-line save, cross-workflow resume, fingerprint-guarded

The CFD flow surface SHALL expose saving as a single call on a paused march (a file path in)
and resuming as an entry point that accepts a loaded package in place of an initial field, so
one workflow can suspend and a different workflow can continue days later. The saver SHALL
store a digest of caller-supplied world-fingerprint bytes; the loader SHALL recompute the
digest from the current world and refuse a mismatch loudly (`force_load` overrides with the
mismatch reported). The fingerprint input is a seam: when the canonical config serialization
lands (roadmap item 5), it becomes the fingerprint's input without changing the container.

#### Scenario: A stale snapshot against an edited world is refused

- **WHEN** a snapshot saved under one world description is loaded after the world's constants
  were edited
- **THEN** the load fails with a fingerprint-mismatch error stating that the snapshot belongs
  to a different world, and `force_load` proceeds only with the mismatch reported

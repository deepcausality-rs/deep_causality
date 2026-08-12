## ADDED Requirements

### Requirement: The Mathlib artifact set is cut to the import closure
The ruleset SHALL support restricting the fetched and packed Mathlib artifacts to the transitive import closure of the consumer's own proofs, and SHALL expand any declared root to its full closure so that no imported module can be omitted.

Measured on this repository's formalization at the pinned revisions: 1,918 of 9,450 modules, 452 MB
of oleans against 1,874 MB, and 1,461 MB of total artifacts against 6,233 MB.

#### Scenario: Closure expansion is sound
- **WHEN** a root module is declared for tree-shaking
- **THEN** that module and every module it transitively imports are included

#### Scenario: An unshaken consumer is unaffected
- **WHEN** no roots are declared
- **THEN** the full artifact set is fetched

### Requirement: The packed artifact archive is byte-reproducible
The ruleset's packing tool SHALL produce a byte-identical archive from identical inputs, independent of the output path, the packing host's clock, and filesystem enumeration order.

A pin that nobody can reproduce cannot be audited: the sha256 in the lockfile is the only thing
distinguishing the published artifact from a substituted one. Reproducibility also means re-hosting
an artifact does not invalidate the pin.

#### Scenario: Repacking yields the same digest
- **WHEN** the same workspace is packed twice to different output paths
- **THEN** both archives have the same sha256

#### Scenario: Per-module completeness
- **WHEN** a module is included in the archive
- **THEN** every build artifact belonging to that module is included, not a chosen subset of extensions

### Requirement: Artifacts must carry the closure of the consumer's imports
The consumer-side check SHALL fail when the tree-shaken artifacts no longer cover what the proofs import, naming the offending import and the command that repairs it.

Without the check, an import falling outside the shaken set surfaces as an unknown-module or
missing-artifact error against a dependency file nobody touched, far from the change that caused it.

#### Scenario: An import outside the declared roots
- **WHEN** a proof gains an import not covered by the declared roots
- **THEN** the check fails, naming that import

#### Scenario: An archive cut from different inputs
- **WHEN** the pinned archive was cut from a different set of imports or dependency revisions
- **THEN** the check fails and states that the archive must be repacked

#### Scenario: The check needs no Lean and no network
- **WHEN** the check runs on a bare checkout with no toolchain, no workspace and no network
- **THEN** it still reaches a verdict

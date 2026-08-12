## ADDED Requirements

### Requirement: The repository carries registry metadata from its first commit
The extracted repository SHALL contain the `.bcr` templates the Bazel Central Registry consumes — module metadata, source template and presubmit configuration — before its first release rather than as a later retrofit.

Registry requirements shape the repository layout, the example modules and the release artifact
naming. Retrofitting them means rewriting all three.

#### Scenario: Templates are present and complete
- **WHEN** the repository is prepared for release
- **THEN** module metadata naming maintainers and the repository is present
- **AND** a source template resolving to the release archive is present
- **AND** a presubmit configuration naming a test module is present

#### Scenario: Module identity matches the registry entry
- **WHEN** the module is submitted
- **THEN** the module name declared in `MODULE.bazel` matches the registry directory name
- **AND** the module declares a semantic version and a compatibility level

### Requirement: Registry presubmit stays fast
The presubmit test module SHALL exercise the toolchain without fetching Mathlib.

Presubmit runs on the registry's infrastructure. A test module that pulls a multi-gigabyte Mathlib
makes every submission slow and flaky, and it demonstrates to reviewers the exact cost this ruleset
exists to remove. The Mathlib-dependent examples belong in the ruleset's own CI, where the caches
that make them cheap are available.

#### Scenario: Presubmit module scope
- **WHEN** the registry runs presubmit
- **THEN** the test module compiles Lean sources that import only Lean core
- **AND** no Mathlib artifacts are fetched

#### Scenario: Mathlib coverage is not lost
- **WHEN** the ruleset's own CI runs
- **THEN** the Mathlib-dependent examples are built there

### Requirement: Guardrails are tested, not only documented
The ruleset's CI SHALL assert that each declared failure mode fails, with the message the consumer is meant to act on.

Every guardrail in this ruleset — hash enforcement, lock validation, platform selection, drift
detection — is worthless if it silently stops firing. Testing the happy path cannot detect that.

#### Scenario: Hash enforcement is exercised
- **WHEN** a toolchain is requested with no hash and no override
- **THEN** CI asserts the build fails with the message directing the consumer to supply one

#### Scenario: Platform coverage is exercised
- **WHEN** CI runs
- **THEN** every declared platform's toolchain archive is verified to download and extract under its recorded sha256

#### Scenario: Version skew is exercised
- **WHEN** a Lean toolchain and a Mathlib revision that do not correspond are configured
- **THEN** CI asserts the build fails with a message naming the mismatch

### Requirement: Releases are reproducible and automated
The release process SHALL produce the archive the source template resolves to, and SHALL open the registry submission from that release without hand-editing.

Hand-assembled release archives drift from the tag they claim to represent, and a hand-written
registry pull request is a second place for the integrity hash to be wrong.

#### Scenario: Release artifact matches the tag
- **WHEN** a version is released
- **THEN** the archive is produced from that tag
- **AND** its integrity hash is recorded in the registry submission

#### Scenario: Submission is automated
- **WHEN** a release is published
- **THEN** the registry submission is opened automatically from it

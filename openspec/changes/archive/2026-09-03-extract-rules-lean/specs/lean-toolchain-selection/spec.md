## ADDED Requirements

### Requirement: Toolchain declarations live in a downloadless repository
The ruleset SHALL emit `toolchain()` declarations into a repository that downloads nothing, separate from the repositories holding the toolchain binaries and the Lake workspace.

Registering a toolchain forces Bazel to fetch the repository the `toolchain()` target lives in, so
resolution can evaluate it — for every target in every build, Lean-related or not. Declaring next to
the implementation puts a multi-gigabyte download on the critical path of builds that compile no
Lean at all.

#### Scenario: A non-Lean build fetches no Lean
- **WHEN** a target that compiles no Lean is built from a cold output base
- **THEN** neither the Lake workspace nor any toolchain distribution is fetched
- **AND** only the declaration repository is present

#### Scenario: Package loading does not force a fetch
- **WHEN** a target pattern is expanded that loads a package declaring Lean targets but builds none of them
- **THEN** the Lake workspace is not fetched

#### Scenario: Building Lean fetches what it needs
- **WHEN** a Lean target is built
- **THEN** the Lake workspace and the selected toolchain distribution are fetched

### Requirement: Toolchains are selected by execution platform
The ruleset SHALL declare one toolchain per supported execution platform, constrained by `exec_compatible_with`, and SHALL NOT select a toolchain based on the host that launched the build.

Repository rules always run on the host, so host detection cannot answer which platform an action
will execute on. An unconstrained declaration matches every execution platform, which ships the
host's binary to whatever worker runs the action — a failure that appears only under remote
execution, at execution time, as an exec-format error.

#### Scenario: Remote execution on a different platform than the host
- **WHEN** a Lean action is executed remotely on a platform differing from the host
- **THEN** the toolchain matching the execution platform is used
- **AND** the action succeeds

#### Scenario: Unselected platforms are never downloaded
- **WHEN** toolchains for several platforms are declared and one is selected
- **THEN** only the selected platform's distribution is fetched

### Requirement: Missing toolchain hashes fail the build
The ruleset SHALL fail when a required toolchain archive has no sha256, and SHALL NOT download it unverified, unless the consumer explicitly opts out for local development.

The value of this ruleset is a trustworthy proof-checking toolchain. Silently accepting an
unverified binary is the weakest possible link in that claim, and a warning is not a control.

#### Scenario: Unpinned toolchain version
- **WHEN** a consumer requests a Lean version with no recorded hash and provides no override
- **THEN** the build fails, naming the platform and showing how to supply the hash

#### Scenario: Consumer-supplied override
- **WHEN** a consumer supplies a per-platform sha256 override for an unrecorded version
- **THEN** the toolchain is fetched and verified against it

#### Scenario: Explicit development opt-out
- **WHEN** a consumer explicitly disables hash enforcement
- **THEN** the build proceeds and states that verification is disabled

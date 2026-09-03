## ADDED Requirements

### Requirement: Lockfile pins every dependency by sha256
The ruleset SHALL materialize a Lake workspace entirely from sha256-addressed archives when a lockfile is supplied, using only Bazel's downloader and no subprocess-driven dependency resolution.

Fetching through `ctx.download*` is what makes the workspace reachable by `--repository_cache` and
`--experimental_remote_downloader`, and what makes it materialize byte-identically on every machine.
Byte-identical inputs are in turn what allow Lean action results to be shared across machines.

#### Scenario: Workspace materializes without git or the Mathlib cache client
- **WHEN** a lockfile pinning every Lake package and a prebuilt olean archive is supplied
- **THEN** the repository rule performs only `download_and_extract` calls
- **AND** runs neither `lake update`, nor `git clone`, nor `lake exe cache get`

#### Scenario: A warm repository cache serves the workspace offline
- **WHEN** every pinned archive is already present in Bazel's repository cache
- **THEN** the workspace materializes with no network access

### Requirement: Lock validation fails closed on drift
The ruleset SHALL validate the lockfile against the Lake manifest and the pinned Lean toolchain on every fetch, and fail with a message naming each disagreement.

A lock that disagrees with the manifest would check the proofs against a different Mathlib than the
manifest claims. That is the exact failure a lockfile exists to prevent, so it is an error rather
than a warning.

#### Scenario: Package revision drift
- **WHEN** a lock entry pins a revision that differs from the manifest's revision for that package
- **THEN** the fetch fails, naming the package, both revisions, and the regeneration command

#### Scenario: Toolchain drift
- **WHEN** the lock records a Lean version different from the pinned toolchain file
- **THEN** the fetch fails, naming both versions

#### Scenario: Unpinned entry
- **WHEN** a lock entry omits its URL or sha256
- **THEN** the fetch fails rather than downloading unverified

### Requirement: The lockfile fast path is all-or-nothing
The ruleset SHALL ignore a lockfile that pins package sources without a prebuilt olean archive, falling back to the subprocess path and emitting a diagnostic that says so.

Mathlib's cache client reads the checked-out commit and the `origin` remote from `mathlib/.git`, and
`lake` deletes and re-clones any package whose recorded URL does not match its checkout. A workspace
whose sources came from tarballs therefore cannot fall back to `lake exe cache get`; a partially
applied lock would destroy the very tree it just fetched.

#### Scenario: Lock without an olean archive
- **WHEN** a lockfile pins package sources but records no olean archive
- **THEN** the ruleset uses the subprocess path unchanged
- **AND** emits a diagnostic naming the missing archive and how to produce it

### Requirement: The subprocess fallback fetches shallowly
The ruleset SHALL fetch each Lake package at its pinned revision without full history when no lockfile is in use.

The fallback exists for consumers who have not adopted a lock. A full-history clone of Mathlib costs
hundreds of megabytes of git metadata per fetch and is the dominant cost of that path.

#### Scenario: Pinned revision fetched shallowly
- **WHEN** the subprocess path fetches a package pinned to a revision
- **THEN** only that revision's tree is transferred
- **AND** the checkout retains the `.git` directory and `origin` remote that the Mathlib cache client requires

#### Scenario: Both tags and bare commit hashes resolve
- **WHEN** a package is pinned to either a tag or a bare commit hash
- **THEN** the fetch succeeds for both forms

### Requirement: Machine-local pins are rejected
The ruleset's verification tooling SHALL reject a lockfile pinning any URL that is not fetchable over http or https.

A `file://` pin is what bootstrapping an archive locally produces. It builds green on the machine
that wrote it and fails on every other machine and in CI, so it must not reach a shared branch.

#### Scenario: A file:// URL is pinned
- **WHEN** the lockfile pins an archive under a `file://` URL
- **THEN** the check fails, naming the entry and stating that the artifact must be hosted

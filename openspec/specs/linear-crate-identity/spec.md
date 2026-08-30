# linear-crate-identity Specification

## Purpose
TBD - created by archiving change add-linear-algebra-crate. Update Purpose after archive.
## Requirements
### Requirement: The crate owns matrix representations and the algorithms over them
`deep_causality_linear` SHALL own every general matrix representation in the workspace and the algorithms defined over them, and SHALL NOT own N-dimensional tensor operations.

The split is by arity, not by density. A strided N-d tensor is not a matrix, so `ein_sum`,
`broadcast`, `kronecker`, `reduction`, `view` and the tensor-train stack stay in
`deep_causality_tensor`. Everything whose domain is a two-index object belongs here.

#### Scenario: A matrix algorithm has one home
- **WHEN** a rank, determinant, kernel or decomposition is needed anywhere in the workspace
- **THEN** it is provided by `deep_causality_linear`
- **AND** no consuming crate carries its own copy

#### Scenario: N-d operations are untouched
- **WHEN** the crate's public surface is enumerated
- **THEN** it contains no operation whose domain is a tensor of rank other than two

### Requirement: The crate sits below every crate that supplies a representation
`deep_causality_linear` SHALL NOT depend on `deep_causality_tensor` or on any crate above it, under any feature flag.

`CausalTensor::svd` and its siblings delegate into this crate, and a crate can only call into what it
depends on, which fixes the edge as tensor → linear. Given that edge, the access-trait impl for
`CausalTensor` can only live in `deep_causality_tensor`: a third crate writing it fails E0117 with
both trait and type foreign, and writing it here would need the reverse edge and close a cycle. An
optional dependency closes the cycle just as a mandatory one does.

#### Scenario: The forbidden direction does not compile
- **WHEN** a crate that defines neither the access trait nor the representation implements one for the other
- **THEN** compilation fails with E0117

#### Scenario: No feature reopens the edge
- **WHEN** the crate is built under every combination of its features
- **THEN** `deep_causality_tensor` appears in no resolved dependency graph

#### Scenario: The tier graph stays acyclic
- **WHEN** the workspace dependency graph is computed
- **THEN** it contains no cycle
- **AND** `AGENTS.md` records the crate's tier and its edges

### Requirement: The crate builds without the standard library
`deep_causality_linear` SHALL support `no_std` with `alloc`, matching the crates that depend on it.

`deep_causality_sparse` and `deep_causality_tensor` are both `no-std`-capable. A crate below them
that required `std` would remove that capability from everything above it.

#### Scenario: A no-std consumer builds
- **WHEN** the crate is built with `--no-default-features --features no-std`
- **THEN** the build succeeds
- **AND** every crate that depends on it still builds in the same configuration

### Requirement: The crate obeys the workspace lint and language policy
`deep_causality_linear` SHALL declare `[lints] workspace = true`, contain no `unsafe` code, and use no macros under `src/`.

`unsafe_code = "forbid"` is workspace-wide with three documented exemptions, and this crate is not
one of them. Bit-packing invites `unsafe` for unchecked indexing; the prototype reaches its measured
throughput without any.

#### Scenario: Lints are inherited
- **WHEN** the crate's manifest is read
- **THEN** it declares `[lints] workspace = true`

#### Scenario: The forbid holds
- **WHEN** the crate is compiled
- **THEN** no `unsafe` block is present and the workspace `forbid` is not overridden

#### Scenario: Clippy is clean without suppression
- **WHEN** clippy runs over the crate
- **THEN** it reports no warnings
- **AND** no `#[allow(clippy::…)]` was added to reach that state


# dec-stencil-operators

## Purpose

Compiled stencil tables for the cubical-lattice DEC operators, and the fused
allocation-free rate assembly built on them. The generic operator path stays as the
equivalence oracle; this capability is the compiled fast path and the measured speedup
that justifies it.

## Requirements
### Requirement: Compiled stencil tables for cubical-lattice DEC operators
`deep_causality_topology` SHALL provide a per-manifold compiled
representation (`DecStencilTables<R>` or equivalent) of the DEC operator
pipeline used by the NS rate on cubical lattices — exterior derivative,
codifferential, diagonal Hodge factors, interior-product transport, and
sharp — as flat index/coefficient arrays built once from the lattice's
shape, periodicity, and metric. Hot-path evaluation through the tables
SHALL perform no CSR traversal, no per-cell index arithmetic, and no heap
allocation. Construction SHALL be explicit (owned by the caller, e.g. the
solver), not hidden inside `Manifold`.

#### Scenario: Stencil evaluation matches the generic operators
- **WHEN** each compiled operator is evaluated on randomized fields on 2D and 3D lattices (periodic and mixed-periodicity), at f64 and Float106
- **THEN** results match the generic compositional operators within 100·ε of the scalar

#### Scenario: Tables embed the metric
- **WHEN** tables are compiled for unit, uniform, and per-axis spacings
- **THEN** stencil results match the generic operators under the same metric (the Hodge factors are folded into the coefficients)

#### Scenario: Hot path is allocation-free
- **WHEN** a compiled operator is applied repeatedly with caller-provided buffers
- **THEN** no per-application heap allocation occurs

### Requirement: Fused allocation-free rate assembly
The NS rate assembly SHALL stream through a reusable workspace over the
compiled tables: one stage evaluation is a fixed sequence of passes with no
intermediate `CausalTensor` materialization. The generic compositional
assembly SHALL remain available as the reference path.

#### Scenario: Fused rate equals compositional rate
- **WHEN** the fused assembly and the generic composition evaluate the same velocity 1-form on the same manifold
- **THEN** the unprojected rates agree within 100·ε at f64 and Float106

#### Scenario: Convergence tables are preserved
- **WHEN** the existing 2D Taylor–Green convergence table runs through the fused assembly
- **THEN** observed spatial orders and table values match the generic path at tolerance

### Requirement: Measured speedup gate
The change SHALL record, in the solver benchmark, the rate-assembly
speedup of the stencil path against the generic baseline at 32³ f64, and
SHALL NOT make the stencil path the solver default unless the serial
speedup is at least 2×.

#### Scenario: Benchmark records the gate
- **WHEN** `dec_solver_benchmark` runs with the stencil configuration
- **THEN** it reports rate-assembly timings comparable against the recorded generic baseline (30 ms serial / 11.2 ms parallel at 32³)

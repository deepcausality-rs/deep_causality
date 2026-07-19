# propulsion-performance-kernels Specification

## Purpose
Rocket-performance and nozzle exit-state pointwise kernels in `deep_causality_physics`:
propellant mass flow from specific impulse, the Tsiolkovsky Δv relation, the branch-selected
inverse area-Mach relation, and the isentropic nozzle exit-state composition. These make the SRP
momentum-flux ratio computable from a commanded throttle and size the propellant reserve for the
plasma-retropulsion descent; they compose the existing compressible-flow kernels rather than
restating them.
## Requirements
### Requirement: Propellant mass flow from specific impulse

The crate SHALL provide `propellant_mass_flow_kernel` computing `ṁ = T / (Isp · g₀)` from
thrust and specific impulse, returning a typed mass-flow-rate quantity, with the full citation
(Sutton & Biblarz, *Rocket Propulsion Elements*, edition and equation pinpoint) in its
docstring. The kernel MUST reject non-positive specific impulse and negative thrust with typed
`PhysicsError`s and never panic.

#### Scenario: Published-value check

- **WHEN** the kernel evaluates a textbook thrust/Isp pair (values and source pinned in the
  test comment)
- **THEN** the returned mass flow matches the published value within the stated tolerance

#### Scenario: Rejection paths

- **WHEN** the kernel is called with `Isp ≤ 0` or `thrust < 0`
- **THEN** it returns a typed `PhysicsError` (no panic, no NaN)

### Requirement: Tsiolkovsky delta-v

The crate SHALL provide `tsiolkovsky_delta_v_kernel` computing `Δv = Isp · g₀ · ln(m₀/m₁)`,
returning a `Speed`. The kernel MUST reject `m₁ ≤ 0`, `m₀ < m₁`, and `Isp ≤ 0` with typed
errors, because a burn cannot end heavier than it began.

#### Scenario: Textbook value

- **WHEN** the kernel evaluates a published mass-ratio/Isp example
- **THEN** the Δv matches the source within the stated tolerance

#### Scenario: Mass-ratio rejection

- **WHEN** `m₀ < m₁` or `m₁ ≤ 0`
- **THEN** the kernel returns a typed `PhysicsError`

### Requirement: Inverse area-Mach relation

The crate SHALL provide `inverse_area_mach_kernel` returning the Mach number for a given area
ratio `A/A* ≥ 1` and γ, with the branch (subsonic or supersonic) selected by an explicit
caller argument, promoting the bisection currently hand-rolled in the nozzle-operating-map
example. The kernel MUST reject `A/A* < 1` and MUST be consistent with the existing forward
`area_mach_ratio_kernel` (round-trip within tolerance), never reimplementing the forward
relation.

#### Scenario: Round-trip consistency

- **WHEN** a supersonic Mach `M` is passed through `area_mach_ratio_kernel` and the result is
  inverted with the supersonic branch
- **THEN** the recovered Mach equals `M` within the stated tolerance, and likewise for a
  subsonic `M` on the subsonic branch

#### Scenario: Published area-ratio check

- **WHEN** the kernel inverts a tabulated area ratio (e.g. a standard γ = 1.4 table entry,
  source pinned in the test)
- **THEN** the returned Mach matches the table within tolerance

#### Scenario: Domain rejection

- **WHEN** `A/A* < 1` or `γ ≤ 1`
- **THEN** the kernel returns a typed `PhysicsError`

### Requirement: Nozzle exit state composition

The crate SHALL provide `nozzle_exit_state_kernel` computing the exit state (exit Mach,
pressure, temperature, density, velocity) from chamber conditions (p₀, T₀), area ratio, γ, and
the specific gas constant — the input set that makes the SRP momentum-flux ratio computable
from a commanded throttle. The kernel MUST compose the existing isentropic-ratio kernels and
`inverse_area_mach_kernel` rather than restating any isentropic formula, and MUST reject
non-physical chamber states (p₀ ≤ 0, T₀ ≤ 0) with typed errors.

#### Scenario: Composed exit state matches a published nozzle case

- **WHEN** the kernel evaluates a published chamber/area-ratio case (source pinned in the test)
- **THEN** exit Mach, pressure ratio, and exit velocity match the source within tolerance

#### Scenario: No formula duplication

- **WHEN** the family's sources are inspected
- **THEN** the isentropic ratios appear only as calls into the existing
  `kernels/fluids/compressible.rs` kernels, not as restated formulas

### Requirement: Family conventions hold

The family SHALL follow the crate's kernel conventions end to end: `R: RealField
(+ FromPrimitive)` generics, `Result<_, PhysicsError>` returns, a `PropagatingEffect` wrapper
per kernel in `wrappers.rs`, flat crate-root exports, mirrored `tests/kernels/propulsion/`
with a Bazel `rust_test_suite`, SPDX headers, and physics-from-publication docstrings with
PDFs in `papers/` for publicly hosted sources.

#### Scenario: Exports and test suites resolve

- **WHEN** `cargo test -p deep_causality_physics` and `bazel test //...` run after the family
  lands
- **THEN** every new kernel is importable from the crate root, all tests pass, and the new
  Bazel suite `kernels_propulsion` executes


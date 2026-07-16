## ADDED Requirements

### Requirement: SRP thrust coefficient

The crate SHALL provide `srp_thrust_coefficient_kernel` computing the freestream-normalized
thrust coefficient `C_T = T / (q∞ · S_ref)`, the SRP similarity number of the Jarvinen–Adams
dataset. The docstring MUST state the distinction from the duct solver's nozzle thrust
coefficient (normalized by `p₀ · A*`), and the kernel MUST reject `q∞ ≤ 0` and `S_ref ≤ 0`
with typed errors.

#### Scenario: Definition check

- **WHEN** the kernel evaluates a thrust with known `q∞` and `S_ref`
- **THEN** the returned `C_T` equals `T/(q∞·S_ref)` exactly for representative values

#### Scenario: Degenerate freestream rejection

- **WHEN** `q∞ ≤ 0` (no freestream to normalize by)
- **THEN** the kernel returns a typed `PhysicsError`

### Requirement: Jet-to-freestream momentum-flux ratio

The crate SHALL provide `momentum_flux_ratio_kernel` computing `J = (ρ_j · u_j²) / (ρ_∞ ·
u_∞²)`, the Cordell–Braun plume model's second input, rejecting non-positive freestream
momentum flux with a typed error.

#### Scenario: Definition check

- **WHEN** jet and freestream density/velocity pairs are supplied
- **THEN** the returned ratio equals the definition for representative values, and a zero
  freestream momentum flux is rejected

### Requirement: Preserved-drag correlation (Jarvinen–Adams, central nozzle)

The crate SHALL provide `srp_preserved_drag_fraction_kernel` evaluating the digitized
Jarvinen–Adams central-nozzle preserved-drag fraction as a function of `C_T`, interpolating a
constants table whose source block pinpoints the report figure/table digitized. The kernel
MUST reproduce the published low-C_T structure (preserved drag collapsing almost entirely by
`C_T ≈ 1`), MUST bracket by value over the digitized domain, and MUST reject `C_T` outside the
digitized domain rather than extrapolate — a silent clamp would fabricate physics.

#### Scenario: Digitized points reproduced

- **WHEN** the kernel evaluates each digitized `C_T` abscissa
- **THEN** every returned fraction matches its digitized ordinate within the pinned tolerance

#### Scenario: Drag-collapse structure present

- **WHEN** the kernel is evaluated across the low-C_T band
- **THEN** the preserved fraction near `C_T ≈ 1` is a small remnant of its unpowered value,
  matching the report's central-nozzle finding

#### Scenario: Out-of-domain rejection

- **WHEN** `C_T` lies outside the digitized domain
- **THEN** the kernel returns a typed `PhysicsError` instead of extrapolating

### Requirement: Unpowered baseline axial coefficient

The crate SHALL provide `jarvinen_adams_baseline_axial_coefficient_kernel` evaluating the
digitized unpowered axial-force coefficient `C_A0(M)` of the report's large-angle cone over
the tested Mach 0.4–2.0 envelope — the baseline the preserved-drag fraction multiplies. The
kernel MUST reject Mach inputs outside the digitized envelope with a typed error.

#### Scenario: Digitized baseline reproduced

- **WHEN** the kernel evaluates each digitized Mach abscissa
- **THEN** every returned `C_A0` matches its digitized ordinate within the pinned tolerance,
  and out-of-envelope Mach inputs are rejected

### Requirement: Total axial force composition

The crate SHALL provide `srp_total_axial_force_coefficient_kernel` composing
`C_A,total = C_T + preserved(C_T) · C_A0` from the sibling kernels, so the non-monotone
net-deceleration band the retropulsion note's gate (4b) later measures is computable pointwise.
The composition MUST call the sibling kernels rather than restating the correlation.

#### Scenario: Non-monotone band exists

- **WHEN** `C_A,total` is evaluated across the digitized `C_T` domain at a fixed baseline
- **THEN** the marginal gain `d(C_A,total)/d(C_T)` is materially below 1 in the low-C_T band
  (thrust replacing destroyed drag), matching the report's central-nozzle behavior, with the
  band edges recorded by the test

### Requirement: SRP stability margin

The crate SHALL provide `srp_stability_margin_kernel` returning the margin between a given
`C_T` and the published bow-shock instability onset (`C_T ≈ 3`, Jarvinen–Adams and
Keyes–Hefner observations as surveyed by Korzun–Braun–Cruz), with the onset exported as a cited
constant so the future envelope and classifier stages read the same bound.

#### Scenario: Margin and bound agree

- **WHEN** the kernel evaluates `C_T` below, at, and above the onset constant
- **THEN** the margin is positive, zero, and negative respectively, and the onset constant
  carries its citation in the constants source block

### Requirement: Cordell–Braun plume-boundary geometry

The crate SHALL provide `cordell_braun_plume_boundary_kernel` computing the analytic
plume-as-effective-obstruction geometry — maximum plume radius, penetration length, and
terminal-shock standoff, returned as a typed `PlumeGeometry` of `Length`s — from the nozzle
exit state and freestream conditions, on-axis, per Cordell & Braun (JSR 50(4), 2013). The
kernel MUST enforce the model's on-axis validity envelope by rejecting out-of-envelope inputs
(the note's §6 discipline pin), MUST NOT discretize space or produce a mask (the CFD stage's
job), and MUST validate against the paper's published comparison cases.

#### Scenario: Published comparison cases reproduced

- **WHEN** the kernel evaluates the paper's tabulated/plotted comparison conditions (pinpoints
  recorded beside the digitized values)
- **THEN** the returned geometry matches the published analytic values within the pinned
  tolerance

#### Scenario: Geometry responds to throttle

- **WHEN** the same freestream is evaluated at two different exit states (two throttle
  settings mapped through the nozzle exit-state kernel)
- **THEN** the two geometries differ — the dynamic-by-construction invariant: two different
  states return two different outputs

#### Scenario: Validity envelope enforced

- **WHEN** inputs leave the model's validated envelope
- **THEN** the kernel returns a typed `PhysicsError` instead of extrapolating

### Requirement: Family conventions hold

The family SHALL follow the crate's kernel conventions end to end (generics, typed errors,
wrappers, flat exports, mirrored tests with the Bazel suite, docstring citations with public
PDFs in `papers/`), with digitized coefficient tables living in `constants/propulsion.rs`
under source-block comments naming publication, figure/table, and units.

#### Scenario: Constants carry their sources

- **WHEN** `constants/propulsion.rs` is inspected
- **THEN** every digitized table and bound carries a source-block comment with publication and
  figure/table pinpoint, and the PDFs for publicly hosted sources exist in `papers/`

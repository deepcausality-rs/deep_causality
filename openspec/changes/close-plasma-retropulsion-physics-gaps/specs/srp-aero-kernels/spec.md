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

### Requirement: SRP flow-regime margin

The crate SHALL provide `srp_flow_regime_margin_kernel` returning the margin `C_T −
C_T,transition` between a given central-nozzle `C_T` and a caller-supplied transition
coefficient, positive in the steady blunt-flow regime and negative in the unsteady
jet-penetration regime. The digitized sources overturned the original assumption that a
`C_T ≈ 3` bow-shock instability governs the central nozzle: the Jarvinen–Adams report shows the
*central*-nozzle transition is the jet-penetration → blunt-flow change near `C_T ≈ 1` at
M∞ = 2.0 (fixed in jet-exit pressure ratio across conditions), while the `C_T ≈ 3` rippling
onset is a *peripheral*-configuration phenomenon (Keyes–Hefner via the Korzun survey). Both the
central transition and the peripheral onset MUST be exported as cited constants so the future
envelope and classifier stages read the same bounds.

#### Scenario: Margin sign tracks the flow regime

- **WHEN** the kernel evaluates `C_T` below, at, and above the supplied transition coefficient
- **THEN** the margin is negative, zero, and positive respectively, and both the central
  transition and the peripheral rippling onset carry their citations in the constants source
  block

#### Scenario: Degenerate inputs rejected

- **WHEN** `C_T` is negative or the transition coefficient is non-positive
- **THEN** the kernel returns a typed `PhysicsError`

### Requirement: Cordell plume sub-relations

The plume model SHALL be decomposed into independently testable pointwise sub-kernels, each
citing the dissertation equation it implements: a Prandtl–Meyer function
(`prandtl_meyer_kernel`, Eq. 9), a choked throat mass flow (`choked_mass_flow_kernel`), the
freestream post-bow-shock stagnation pressure (`srp_post_bow_shock_total_pressure_kernel`,
Eqs. 13–14), the terminal (Mach-disk) shock Mach number
(`srp_terminal_shock_mach_kernel`, Eqs. 13–15), and the barrel-shock jet-edge Mach number
(`srp_jet_edge_mach_kernel`, Eq. 19). The jet-edge and terminal-shock relations MUST reproduce
the dissertation's printed Table 13 and Fig. 54 values, which depend only on stagnation-pressure
ratios and are therefore exact anchors.

#### Scenario: Jet-edge Mach reproduces Table 13

- **WHEN** `srp_jet_edge_mach_kernel` is evaluated at the single-nozzle wind-tunnel conditions
  for the tabulated thrust coefficients
- **THEN** the returned jet-edge Mach matches the printed Table 13 values (C_T 0.47 → 3.86,
  4.04 → 5.63, 10.0 → 6.53) within the pinned tolerance

#### Scenario: Terminal-shock Mach matches Fig. 54

- **WHEN** `srp_terminal_shock_mach_kernel` is evaluated at C_T = 10
- **THEN** the terminal Mach matches the Fig. 54 analytic value (≈ 15.5), and a lower thrust
  yields a weaker terminal shock; a jet stagnation pressure below the post-bow-shock stagnation
  pressure is rejected as the no-terminal-shock low-thrust regime

### Requirement: Cordell plume-boundary geometry

The crate SHALL provide `cordell_braun_plume_boundary_kernel` composing the sub-relations into
the analytic plume-as-effective-obstruction geometry — maximum plume radius, penetration
length, and terminal-shock standoff, returned as a typed `PlumeGeometry` of `Length`s — from
the nozzle exit state and freestream conditions, on-axis, per Cordell's Georgia Tech
dissertation (2013), Ch. III. The kernel MUST enforce the model's validity envelope by
rejecting out-of-envelope inputs (freestream Mach outside [2, 4], jet γ outside [1.2, 1.4], and
the jet-penetration regime below the blunt-flow pressure-ratio transition — the note's §6
discipline pin), and MUST NOT discretize space or produce a mask. The bow-shock construction
(dissertation §3.4) is deliberately excluded: the marched CFD layer forms its own bow shock
around the obstruction this kernel returns.

#### Scenario: Terminal-shock standoff matches the published anchor

- **WHEN** the kernel is evaluated at the single-nozzle C_T = 4.04 wind-tunnel condition
- **THEN** the terminal-shock standoff normalized by body diameter matches the Fig. 55 analytic
  anchor (≈ 1.28, consistently slightly underpredicted) within the pinned tolerance, and the
  penetration length is at least the standoff

#### Scenario: Geometry responds to throttle

- **WHEN** the same freestream is evaluated at two different jet stagnation pressures (two
  throttle settings)
- **THEN** the two geometries differ and the larger jet drives a larger plume (radius,
  penetration, and standoff all grow) — the dynamic-by-construction invariant

#### Scenario: Validity envelope enforced

- **WHEN** inputs leave the model's validated envelope (freestream Mach, jet γ, or the
  jet-penetration pressure-ratio regime)
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

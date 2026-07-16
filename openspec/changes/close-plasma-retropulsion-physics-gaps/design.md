## Context

The retropulsion design note demands six kernels by name (§3.1, §7: mass flow from Isp, C_T,
momentum-flux ratio, Cordell–Braun plume boundary, drag-decrement correlation, stopping
distance) and implies four more inputs it never names (nozzle exit state behind the
momentum-flux ratio; freestream q∞ normalization; the unpowered baseline drag the decrement
multiplies; the C_T stability bound the envelope clamps against). A four-way audit (existing
kernel surface, duct-solver embedded math, crate conventions, note demands) pinned the gap
list below.

**What already exists and is reused** (`kernels/fluids/compressible.rs`
unless noted): `isentropic_{pressure,temperature,density}_ratio_kernel` (Mach-parameterized),
`total_{pressure,temperature}_isentropic_kernel`, `area_mach_ratio_kernel` (forward A/A*),
`speed_of_sound_ideal_gas_kernel` (T-form), `mach_number_kernel`, `dynamic_pressure_kernel`
(`fluids/ideal_flow.rs:24`), `rankine_hugoniot_temperature_kernel` (`hypersonic/shock.rs:31`),
the Park-2T chemistry family, `plasma_frequency_kernel` (mhd). The audit's direct answers:
choked mass flow NO, coefficient-based drag/lift force NO, ballistic coefficient NO,
thrust/rocket/nozzle kernels NO, flight kinematics in `dynamics/` NO.

**The definitive gap inventory** (kernel → physics → citation anchor):

| # | Kernel | Physics | Anchor |
|---|---|---|---|
| 1 | `propellant_mass_flow_kernel` | ṁ = T/(Isp·g₀) | Sutton & Biblarz, *Rocket Propulsion Elements* (edition/eq. pinpoint in docstring) |
| 2 | `tsiolkovsky_delta_v_kernel` | Δv = Isp·g₀·ln(m₀/m₁) | Tsiolkovsky (1903); Sutton & Biblarz |
| 3 | `inverse_area_mach_kernel` | M from A/A*, branch-selected (subsonic/supersonic) | Anderson, *Modern Compressible Flow* (the forward kernel's own anchor); promotes the bisection hand-rolled in the nozzle example (`model.rs:132-153`) |
| 4 | `nozzle_exit_state_kernel` | (M_e, p_e, T_e, ρ_e, u_e) from (p₀, T₀, ε, γ, R_s), composed from existing isentropic kernels + #3 | same |
| 5 | `srp_thrust_coefficient_kernel` | C_T = T/(q∞·S_ref) | Jarvinen & Adams (1970), NASA CR, NTRS 19720005324 |
| 6 | `momentum_flux_ratio_kernel` | J = ρ_j u_j²/(ρ_∞ u_∞²) | Cordell & Braun, JSR 50(4), 2013 |
| 7 | `srp_preserved_drag_fraction_kernel` | digitized central-nozzle preserved-drag vs C_T, validity Mach 0.4–2.0; carries the low-C_T drag-collapse structure ("collapses almost entirely by C_T ≈ 1") | Jarvinen & Adams (1970), digitized at build time |
| 8 | `jarvinen_adams_baseline_axial_coefficient_kernel` | unpowered C_A0(M) of the large-angle cone, digitized from the same report | Jarvinen & Adams (1970) |
| 9 | `srp_total_axial_force_coefficient_kernel` | C_A,total = C_T + preserved(C_T)·C_A0 | composition of #5/#7/#8 |
| 10 | `srp_stability_margin_kernel` | margin to the published C_T ≈ 3 bow-shock-instability onset | Jarvinen–Adams; Keyes–Hefner via the Korzun–Braun–Cruz survey |
| 11 | `cordell_braun_plume_boundary_kernel` | analytic effective-obstruction geometry (max plume radius, penetration length, terminal-shock standoff) from exit state + freestream, on-axis | Cordell & Braun (2013); Jarvinen–Adams flowfield construction (shadowgraphs within ~10%) |
| 12 | `stopping_distance_kernel` / `ignition_altitude_kernel` | d = v²/(2a_net), h_ign = v²/(2(a_T − g)) + margin; rejects a_net ≤ 0 (T/W ≤ 1 cannot stop) | closed-form kinematics; Klumpp (1974) / Açıkmeşe–Ploen (2007) cited as the guidance upgrade path |
| 13 | `suicide_burn_deceleration_kernel` | a_cmd = v²/(2h) + g; rejects h ≤ 0 | same |

## Goals / Non-Goals

**Goals:**

- Land the family under `src/kernels/propulsion/` following the hypersonic template exactly:
  topic files + `wrappers.rs`, quantities newtypes, constants with source-block comments, flat
  crate-root exports, mirrored tests, Bazel suite, papers/ PDFs.
- Every kernel pointwise-validated in isolation against published values before any solver
  integration (the gap-2 split's payoff, adopted verbatim by the note).
- Preserve the kernel/solver split: stateless pointwise relations here; discretization,
  marching, masks, and stages stay in `deep_causality_cfd`.

**Non-Goals:**

- No CFD-side work (stages, observables, verification targets, rank studies — Stages 2–4).
- No atmosphere kernel (the note keeps the atmosphere a hand-authored example table).
- No `thrust = throttle · T_max` kernel (trivial arithmetic; lives in the `RetroThrust` stage).
- No cleanup of the adjacent hand-rolls the audit surfaced (pressure-parameterized isentropics
  and critical star state in `duct_march_run.rs:123-141,247-248`; (p, ρ)-form sound speed at
  4 sites; the normal-shock ratio set in `fitting.rs:117-119`; the `isen(m)` duplicate in the
  nozzle example `model.rs:156-159`). Real findings, separate change — this diff stays scoped
  to the new family.
- No peripheral-nozzle SRP correlation (the note flies the central config; peripheral is named
  Tier-C context).
- No Lean/THEOREM_MAP witnesses (correlation and geometry kernels are empirical fits, not
  theorems).

## Decisions

- **Naming: `srp_thrust_coefficient` vs the duct `C_f`.** The duct solver's thrust coefficient
  (`duct_march_run.rs:305`) is nozzle-normalized (`p₀·A*`); the SRP coefficient is
  freestream-normalized (`q∞·S_ref`). Same words, different physics. The `srp_` prefix on every
  SRP-specific kernel keeps them apart; the docstring states the distinction explicitly.
- **Dimensionless outputs are raw `R`.** Matches the crate (Mach, Reynolds, isentropic ratios
  all return `Result<R>`); C_T, J, preserved fraction, and C_A follow. Dimensioned outputs get
  quantities: `MassFlowRate`, `PlumeGeometry` (new), `Speed`/`Length`/`Temperature`/`Pressure`
  /`Density` (existing). Missing quantities (audit could not confirm `Force`,
  `Acceleration`-scalar, `MassFlowRate`) are added under `quantities/propulsion/` per the
  hypersonic pattern; existing ones are reused — verify at build, never duplicate.
- **Digitized correlations live in `constants/propulsion.rs`** as `pub const` tables with
  source-block comments (publication, figure/table, units), exactly like the RP-1232 blocks in
  `constants/hypersonic.rs`; kernels interpolate with domain validation and reject
  out-of-domain inputs (`PhysicalInvariantBroken`). The interpolation rule mirrors the
  weather-table loader contract: bracket by value, clamp never — out-of-domain is an error
  here, because a silent clamp would fabricate physics.
- **The plume kernel returns geometry, never a mask.** `PlumeGeometry { max_radius,
  penetration_length, terminal_shock_standoff }` (all `Length`). Shaping the smoothed forcing
  region from it is the CFD stage's job (the split rule: kernels do not discretize space).
- **Composition kernels call sibling kernels.** `nozzle_exit_state_kernel` composes the
  existing isentropic ratios plus `inverse_area_mach_kernel`; `srp_total_axial_force` composes
  #5/#7/#8. No formula appears twice in the crate.
- **`g₀` (standard gravity, 9.80665 m/s²)** is checked against `src/constants/` first; added
  there with a typed accessor only if absent.
- **Paper PDFs**: NTRS and Georgia Tech repository documents (Jarvinen–Adams CR;
  Korzun–Braun–Cruz survey) are public and land in `papers/`. The Cordell–Braun JSR paper may
  be paywalled; the accessible fallback with the same model content is Cordell's Georgia Tech
  dissertation (SMARTech, public) — commit whichever is public, cite both in the docstring.
  Textbook relations (Sutton & Biblarz, Anderson) are cited by edition and equation number in
  the docstring without a PDF.

## Risks / Trade-offs

- [Digitization error in the Jarvinen–Adams curves → wrong sign-flip band, and gate (4b) of the
  future example would blame the wrong layer] → Digitize with figure/table pinpoints recorded
  beside each constant; pointwise tests pin every digitized point within a stated tolerance;
  the correlation's non-monotone structure is itself asserted by a test.
- [Cordell–Braun model equations span a paper section, not one formula; transcription risk] →
  The kernel validates against the paper's own published comparison cases (analytic boundary vs
  CFD/shadowgraph values) as pointwise tests; the on-axis validity envelope is enforced as
  input rejection, matching the note's §6 discipline pin.
- [Validity-domain creep: callers feeding Mach or C_T outside the wind-tunnel envelope] →
  Kernels reject out-of-domain inputs with typed errors rather than extrapolating; the domain
  constants are exported so the future envelope/classifier stages read the same bounds.
- [Quantity-type churn if `Force`/`MassFlowRate` exist under another name] → Build-time
  verification step before adding any newtype; reuse wins over addition.

## Migration Plan

Additive change to one crate; no callers exist yet. Rollback is reverting the single commit.

## Open Questions

None blocking. The exact Cordell–Braun equation subset (full barrel-shock contour vs the
effective-obstruction summary geometry) is resolved during digitization against what the
Stage-2 `PlumeObstruction` mask actually needs: the three `PlumeGeometry` outputs above are the
contract; internal intermediate quantities stay private to the kernel.

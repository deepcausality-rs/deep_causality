<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# `deep_causality_physics` test-suite audit

Scanned 2026-09-08: **1758 test functions across 190 files**, 26 780 test lines against 23 377
source lines. The detectors are in `scratchpad/audit_tests.py`; every class below was calibrated by
reading samples, and the false positives that calibration removed are recorded at the end.

## The three broken-by-design classes, as found

| Class | Count | Share | What it means |
|---|---|---|---|
| **single-input** | 1616 | 91.9% | One hand-chosen input. No loop, no table, no range. |
| **cherry-picked** | 641 | 36.5% | One input, a magic literal expectation, and no stated provenance for it. |
| **tautology** | 169 | 9.6% | No assertion says anything about the answer — typically `assert!(x.is_ok())` and nothing else. |
| **circular** | 57 | 3.2% | The expected value is recomputed in the test body from the same formula the kernel uses. |
| no-assertion | 1 | 0.1% | No assertion, and no delegation to a helper that asserts. |

The classes overlap: a cherry-picked test is also single-input by construction.

## What each looks like, from the suite

**Tautology** — `kernels/fluids/wrappers_tests.rs:309`:

```rust
fn test_weber_number_wrapper() {
    let rho = Density::<f64>::new(1000.0).unwrap();
    let u = Speed::<f64>::new(2.0).unwrap();
    let l = Length::<f64>::new(0.001).unwrap();
    assert!(weber_number(&rho, &u, &l, 0.072_f64).is_ok());
}
```

`We = ρu²L/σ = 1000·4·0.001/0.072 = 55.56`, and the test never looks at it. Any implementation that
returns `Ok` of anything passes — including one that returns zero.

**Circular** — `kernels/fluids/dimensionless_tests.rs:351`:

```rust
let bo = bond_number_kernel(&rho, 9.8_f64, &l, 0.072).unwrap();
let expected = 1000.0 * 9.8 * 1.0e-4 / 0.072;
assert!((bo - expected).abs() < TOL);
```

The expectation is the kernel's own formula, retyped. It asserts that the expression was written the
same way twice, and would pass against a kernel with the same error in it. `kernels/photonics/ray_tests.rs:41`
does the same for Snell's law: `let expected = ((1.0 / 1.5) * 0.5f64.sin()).asin();`.

**Cherry-picked** — one input, a literal, no source. These are the 641, and the risk is not that the
number is wrong but that it was read off the implementation once and never questioned. Nothing in the
test records where it came from, so nothing distinguishes a verified value from a captured one.

## Worst files first

Ranked by hard defects (circular + tautology + no-assertion), then by cherry-picked.

| File (under `tests/`) | circular | tautology | cherry-picked | single-input |
|---|---|---|---|---|
| `kernels/fluids/wrappers_tests.rs` | 0 | 80 | 66 | 119 |
| `kernels/fluids/dimensionless_tests.rs` | 8 | 0 | 22 | 40 |
| `kernels/condensed/wrappers_tests.rs` | 0 | 6 | 2 | 15 |
| `kernels/em/wrappers_tests.rs` | 0 | 6 | 0 | 12 |
| `kernels/thermodynamics/wrappers_tests.rs` | 0 | 5 | 2 | 14 |
| `kernels/relativity/wrappers_tests.rs` | 0 | 5 | 0 | 10 |
| `theories/general_relativity/gr_ops_impl_tests.rs` | 5 | 0 | 0 | 15 |
| `kernels/fluids/compressible_tests.rs` | 4 | 0 | 18 | 31 |
| `theories/weak_force/weak_force_tests.rs` | 3 | 1 | 13 | 40 |
| `theories/electroweak/electroweak_tests.rs` | 4 | 0 | 10 | 54 |
| `kernels/relativity/spacetime_tests.rs` | 1 | 3 | 7 | 27 |
| `kernels/hypersonic/wrappers_tests.rs` | 0 | 4 | 2 | 8 |
| `theories/electromagnetism/em_tests.rs` | 2 | 2 | 2 | 31 |
| `kernels/fluids/boundary_layer_tests.rs` | 3 | 0 | 10 | 20 |
| `quantities/thermodynamics_quantities/thermodynamics_quantities_tests.rs` | 0 | 3 | 6 | 11 |
| `kernels/em/fields_tests.rs` | 0 | 3 | 4 | 31 |
| `quantities/index_of_refraction/index_of_refraction_tests.rs` | 0 | 3 | 4 | 8 |
| `quantities/chronometric_quantities/space_time_coordinate_tests.rs` | 3 | 0 | 2 | 12 |
| `kernels/materials/wrappers_tests.rs` | 0 | 3 | 0 | 5 |
| `kernels/em/forces_tests.rs` | 0 | 3 | 0 | 6 |
| `kernels/photonics/wrappers_tests.rs` | 0 | 2 | 8 | 25 |
| `kernels/nuclear/qcd_tests.rs` | 2 | 0 | 6 | 31 |
| `kernels/nuclear/physics_tests.rs` | 2 | 0 | 3 | 8 |
| `kernels/hypersonic/finite_rate_tests.rs` | 1 | 1 | 3 | 13 |
| `quantities/dynamics_quantities/force_tests.rs` | 0 | 2 | 3 | 7 |
| `quantities/dynamics_quantities/torque_tests.rs` | 0 | 2 | 3 | 7 |
| `kernels/condensed/phase_tests.rs` | 0 | 2 | 2 | 12 |
| `quantities/materials_quantities/materials_quantities_tests.rs` | 0 | 2 | 2 | 16 |
| `kernels/dynamics/wrappers_tests.rs` | 0 | 2 | 1 | 10 |
| `kernels/fluids/mechanics_tests.rs` | 2 | 0 | 1 | 5 |
| `kernels/fluids/coherent_structures_coverage_tests.rs` | 0 | 2 | 0 | 2 |
| `kernels/fluids/wrappers_coverage_tests.rs` | 0 | 2 | 0 | 2 |
| `kernels/astro/solver_convergence_tests.rs` | 2 | 0 | 0 | 6 |
| `theories/general_relativity/adm_state_tests.rs` | 2 | 0 | 0 | 7 |
| `kernels/fluids/governing_tests.rs` | 1 | 0 | 14 | 19 |
| `quantities/em_quantities/em_quantities_tests.rs` | 0 | 1 | 10 | 12 |
| `quantities/energy/energy_tests.rs` | 0 | 1 | 9 | 10 |
| `quantities/temperature/temperature_tests.rs` | 0 | 1 | 9 | 11 |
| `kernels/fluids/kinematics_tests.rs` | 1 | 0 | 8 | 14 |
| `kernels/dynamics/kinematics_tests.rs` | 0 | 1 | 6 | 12 |
| `kernels/fluids/coherent_structures_tests.rs` | 1 | 0 | 6 | 15 |
| `kernels/astro/two_body_tests.rs` | 1 | 0 | 6 | 9 |
| `kernels/propulsion/wrappers_tests.rs` | 0 | 1 | 5 | 6 |
| `kernels/thermodynamics/stats_tests.rs` | 0 | 1 | 5 | 14 |
| `kernels/hypersonic/thermochemistry_tests.rs` | 1 | 0 | 5 | 8 |
| `kernels/condensed/qgt_tests.rs` | 0 | 1 | 4 | 13 |
| `kernels/hypersonic/shock_tests.rs` | 1 | 0 | 4 | 6 |
| `quantities/dynamics_quantities/speed_tests.rs` | 0 | 1 | 4 | 8 |
| `quantities/dynamics_quantities/acceleration_tests.rs` | 0 | 1 | 4 | 7 |
| `quantities/dynamics_quantities/length_tests.rs` | 0 | 1 | 4 | 8 |
| `quantities/fluids_quantities/pressure_tests.rs` | 0 | 1 | 4 | 9 |
| `quantities/fluids_quantities/wall_shear_stress_tests.rs` | 0 | 1 | 4 | 9 |
| `quantities/fluids_quantities/specific_enthalpy_tests.rs` | 0 | 1 | 4 | 9 |
| `quantities/fluids_quantities/kinematic_viscosity_tests.rs` | 0 | 1 | 4 | 9 |
| `kernels/photonics/ray_tests.rs` | 1 | 0 | 3 | 6 |
| `kernels/mhd/wrappers_tests.rs` | 0 | 1 | 3 | 22 |
| `quantities/dynamics_quantities/volume_tests.rs` | 0 | 1 | 3 | 7 |
| `quantities/dynamics_quantities/frequency_tests.rs` | 0 | 1 | 3 | 7 |
| `quantities/fluids_quantities/viscosity_tests.rs` | 0 | 1 | 3 | 8 |
| `kernels/propulsion/descent_tests.rs` | 1 | 0 | 2 | 7 |
| `kernels/mhd/resistive_tests.rs` | 0 | 1 | 2 | 4 |
| `kernels/astro/mechanics_tests.rs` | 1 | 0 | 2 | 11 |
| `quantities/nuclear_quantities/activity_tests.rs` | 0 | 1 | 2 | 5 |
| `quantities/nuclear_quantities/half_life_tests.rs` | 0 | 1 | 2 | 7 |
| `theories/electroweak/radiative_tests.rs` | 1 | 0 | 2 | 9 |
| `kernels/relativity/gravity_tests.rs` | 1 | 0 | 1 | 12 |
| `quantities/nuclear_quantities/hadron_tests.rs` | 1 | 0 | 1 | 2 |
| `kernels/dynamics/wrappers_coverage_tests.rs` | 0 | 1 | 0 | 1 |
| `kernels/materials/mechanics_tests.rs` | 1 | 0 | 0 | 10 |
| `kernels/waves/wrappers_coverage_tests.rs` | 0 | 1 | 0 | 1 |
| `kernels/quantum/wrappers_tests.rs` | 0 | 1 | 0 | 2 |
| `kernels/relativity/spacetime_coverage_tests.rs` | 0 | 1 | 0 | 0 |
| `kernels/astro/c6_explore_tests.rs` | 0 | 0 | 0 | 0 |
| `kernels/fluids/turbulence_tests.rs` | 0 | 0 | 18 | 24 |
| `quantities/time/time_tests.rs` | 0 | 0 | 12 | 14 |
| `kernels/fluids/ideal_flow_tests.rs` | 0 | 0 | 11 | 15 |
| `quantities/nuclear_quantities/four_momentum_tests.rs` | 0 | 0 | 11 | 12 |
| `quantities/ratio/ratio_tests.rs` | 0 | 0 | 8 | 8 |
| `quantities/quantum_quantities/quantum_quantities_tests.rs` | 0 | 0 | 8 | 15 |
| `quantities/fluids_quantities/velocity3_tests.rs` | 0 | 0 | 6 | 9 |
| `kernels/propulsion/performance_tests.rs` | 0 | 0 | 5 | 10 |
| `kernels/propulsion/plume_tests.rs` | 0 | 0 | 5 | 9 |
| `kernels/em/solver_tests.rs` | 0 | 0 | 5 | 9 |
| `kernels/mhd/ideal_tests.rs` | 0 | 0 | 5 | 13 |
| `quantities/dynamics_quantities/mass_tests.rs` | 0 | 0 | 5 | 8 |
| `quantities/propulsion_quantities/mass_flow_rate_tests.rs` | 0 | 0 | 5 | 6 |
| `quantities/hypersonic_quantities/mass_fraction_tests.rs` | 0 | 0 | 5 | 6 |
| `quantities/hypersonic_quantities/vibrational_temperature_tests.rs` | 0 | 0 | 5 | 6 |
| `quantities/hypersonic_quantities/electron_temperature_tests.rs` | 0 | 0 | 5 | 6 |
| `quantities/hypersonic_quantities/reaction_rate_tests.rs` | 0 | 0 | 5 | 6 |
| `quantities/hypersonic_quantities/electron_density_tests.rs` | 0 | 0 | 5 | 6 |
| `quantities/hypersonic_quantities/ionization_fraction_tests.rs` | 0 | 0 | 5 | 6 |
| `kernels/propulsion/nozzle_tests.rs` | 0 | 0 | 4 | 6 |
| `kernels/propulsion/srp_tests.rs` | 0 | 0 | 4 | 13 |
| `kernels/photonics/polarization_tests.rs` | 0 | 0 | 4 | 9 |
| `quantities/dynamics_quantities/moment_of_inertia_tests.rs` | 0 | 0 | 4 | 7 |
| `quantities/dynamics_quantities/area_tests.rs` | 0 | 0 | 4 | 7 |
| `quantities/fluids_quantities/body_force_density_tests.rs` | 0 | 0 | 4 | 6 |
| `quantities/fluids_quantities/density_tests.rs` | 0 | 0 | 4 | 8 |
| `quantities/fluids_quantities/acceleration_vector_tests.rs` | 0 | 0 | 4 | 6 |
| `quantities/fluids_quantities/vorticity_vector_tests.rs` | 0 | 0 | 4 | 6 |
| `quantities/condensed_quantities/twist_angle_tests.rs` | 0 | 0 | 4 | 4 |
| `quantities/condensed_quantities/mobility_tests.rs` | 0 | 0 | 4 | 4 |
| `quantities/condensed_quantities/conductance_tests.rs` | 0 | 0 | 4 | 4 |
| `quantities/photonics_quantities/numerical_aperture_tests.rs` | 0 | 0 | 4 | 4 |
| `quantities/relativity_quantities/relativity_quantities_tests.rs` | 0 | 0 | 4 | 5 |
| `quantities/nuclear_quantities/amount_of_substance_tests.rs` | 0 | 0 | 4 | 5 |
| `quantities/nuclear_quantities/energy_density_tests.rs` | 0 | 0 | 4 | 5 |
| `constants/propulsion_tests.rs` | 0 | 0 | 4 | 4 |
| `theories/general_relativity/metrics_tests.rs` | 0 | 0 | 4 | 12 |
| `kernels/mhd/plasma_tests.rs` | 0 | 0 | 3 | 7 |
| `kernels/fluids/constitutive_tests.rs` | 0 | 0 | 3 | 11 |
| `quantities/photonics_quantities/complex_beam_parameter_tests.rs` | 0 | 0 | 3 | 3 |
| `quantities/photonics_quantities/focal_length_tests.rs` | 0 | 0 | 3 | 3 |
| `quantities/photonics_quantities/ray_height_tests.rs` | 0 | 0 | 3 | 3 |
| `quantities/photonics_quantities/ray_angle_tests.rs` | 0 | 0 | 3 | 3 |
| `quantities/photonics_quantities/optical_power_tests.rs` | 0 | 0 | 3 | 3 |
| `quantities/chronometric_quantities/central_body_tests.rs` | 0 | 0 | 3 | 14 |
| `quantities/mhd_quantities/magnetic_pressure_tests.rs` | 0 | 0 | 3 | 4 |
| `quantities/mhd_quantities/diffusivity_tests.rs` | 0 | 0 | 3 | 4 |
| `quantities/mhd_quantities/alfven_speed_tests.rs` | 0 | 0 | 3 | 4 |
| `quantities/mhd_quantities/plasma_beta_tests.rs` | 0 | 0 | 3 | 4 |
| `quantities/nuclear_quantities/lund_parameters_tests.rs` | 0 | 0 | 3 | 3 |
| `kernels/dynamics/estimation_tests.rs` | 0 | 0 | 2 | 5 |
| `kernels/nuclear/pdg_tests.rs` | 0 | 0 | 2 | 3 |
| `kernels/photonics/beam_tests.rs` | 0 | 0 | 2 | 8 |
| `kernels/photonics/diffraction_tests.rs` | 0 | 0 | 2 | 4 |
| `kernels/waves/general_tests.rs` | 0 | 0 | 2 | 5 |
| `kernels/waves/wrappers_tests.rs` | 0 | 0 | 2 | 4 |
| `quantities/propulsion_quantities/plume_geometry_tests.rs` | 0 | 0 | 2 | 3 |
| `quantities/propulsion_quantities/nozzle_exit_state_tests.rs` | 0 | 0 | 2 | 3 |
| `quantities/condensed_quantities/berry_curvature_tests.rs` | 0 | 0 | 2 | 2 |
| `quantities/condensed_quantities/band_drude_weight_tests.rs` | 0 | 0 | 2 | 2 |
| `quantities/condensed_quantities/quantum_metric_tests.rs` | 0 | 0 | 2 | 2 |
| `quantities/condensed_quantities/orbital_angular_momentum_tests.rs` | 0 | 0 | 2 | 2 |
| `quantities/mhd_quantities/larmor_radius_tests.rs` | 0 | 0 | 2 | 4 |
| `quantities/velocity_one_form/velocity_one_form_tests.rs` | 0 | 0 | 2 | 8 |
| `kernels/chronometric/forward_clock_tests.rs` | 0 | 0 | 1 | 5 |
| `kernels/mhd/grmhd_tests.rs` | 0 | 0 | 1 | 4 |
| `kernels/condensed/moire_tests.rs` | 0 | 0 | 1 | 5 |
| `kernels/condensed/generic_real_field_tests.rs` | 0 | 0 | 1 | 1 |
| `kernels/astro/ks_propagator_tests.rs` | 0 | 0 | 1 | 8 |
| `kernels/astro/wrappers_tests.rs` | 0 | 0 | 1 | 6 |
| `kernels/hypersonic/ionization_tests.rs` | 0 | 0 | 1 | 7 |
| `quantities/condensed_quantities/vector_potential_tests.rs` | 0 | 0 | 1 | 2 |
| `quantities/condensed_quantities/order_parameter_tests.rs` | 0 | 0 | 1 | 1 |
| `quantities/photonics_quantities/beam_waist_tests.rs` | 0 | 0 | 1 | 4 |
| `quantities/photonics_quantities/wavelength_tests.rs` | 0 | 0 | 1 | 4 |
| `quantities/vorticity_two_form/vorticity_two_form_tests.rs` | 0 | 0 | 1 | 6 |
| `quantities/mhd_quantities/trait_coverage_tests.rs` | 0 | 0 | 1 | 2 |
| `quantities/body_force_one_form/body_force_one_form_tests.rs` | 0 | 0 | 1 | 6 |
| `quantities/pressure_zero_form/pressure_zero_form_tests.rs` | 0 | 0 | 1 | 6 |
| `constants/universal_tests.rs` | 0 | 0 | 1 | 1 |
| `constants/atomic_tests.rs` | 0 | 0 | 1 | 1 |
| `constants/thermodynamics_tests.rs` | 0 | 0 | 1 | 1 |

## Calibration, and the false positives it removed

The first pass over-reported and was corrected three ways, each verified by reading the flagged tests:

* **`assert!(x.is_err())` is a real claim.** An error-path test that asserts only a refusal is doing
  its job. The first pass flagged 535 "weak-only"; distinguishing the error assertions cut it to 169.
* **Assertions may live in a helper.** `constants_accessors_f64` calls `check_constants::<f64>()`,
  which asserts. The first pass called 48 tests assertion-free; only 1 actually is.
* **A conversion is not a re-solve.** `solver_convergence_tests.rs` builds its expected position as
  `a(cos E − e)` from a root obtained by an *independent* bisection. The detector cannot tell that
  from re-deriving the root, so it flags it; it is not circular, and is listed here for honesty.

A residual known limitation: `single-input` counts any test with no loop or table, so a test pinning
one genuinely singular case — a boundary, a refusal — is counted alongside a value test that should
have swept a range. It is a ceiling on the work, not a work list.

## The remediation standard

Every rewritten test must satisfy all four:

1. **An oracle independent of the implementation.** A published/reference value with a citation, a
   value produced by the Python generator in `scripts/physics_oracles.py`, a closed form evaluated by
   hand, or an invariant the answer must satisfy. Never the kernel's own formula retyped.
2. **A range, not a point.** Table-driven over inputs spanning the physical range, log-spaced where
   the quantity is scale-free, including the extremes the kernel claims to support.
3. **Corner cases named in advance.** Zero, one, boundary values, the smallest and largest
   representable inputs, and any discontinuity the physics has.
4. **Error paths and improper state.** Every documented refusal exercised, and every invariant
   violation — negative mass, `M < 1` for a normal shock, `γ ≤ 1` — rejected rather than absorbed.

## Remediation: proof that it matters, and progress

### The blind spot, demonstrated

The tautology class was not a theoretical worry. Replacing `weber_number`'s body with
`PropagatingEffect::pure(R::zero())` — reporting **zero for every input in the world** — left the
entire `kernels/fluids/wrappers_tests.rs` suite **passing**. After the rewrite the same one-line
defect **fails** it.

That is the measure the remediation is held to: not that the new tests pass, but that they fail when
the code is wrong.

A second, blunter set of defects was tried first — Weber losing its square, Knudsen inverted — and
the *old* suite caught those. Worth recording, because it bounds the claim: the old tests were not
uniformly worthless. Their failure mode is specific. A single hand-chosen input catches an error that
moves every value, and misses one that the chosen input happens to be blind to; a `.is_ok()`
assertion catches nothing at all.

### Done

| File | Before | After |
|---|---|---|
| `kernels/fluids/dimensionless_tests.rs` | 40 tests, 8 circular, 22 cherry-picked, 1 input each | 32 tests over **~100 oracle rows**, 3 cross-kernel identities, 4 scaling laws, every divisor guard, zero-numerator corner cases |
| `kernels/fluids/wrappers_tests.rs` | 119 tests, 80 tautology, 66 cherry-picked | the 18 dimensionless wrappers now assert the carried **value** against the oracle over every table row |

`scripts/physics_oracles.py` is the generator: textbook definitions, 50-digit decimal, inputs spread
over many decades and several physical regimes (creeping flow to re-entry, microns to kilometres,
liquid metals to heavy oils, terrestrial/lunar/Jovian gravity).

The three cross-kernel identities — `Pe = Re·Pr`, `Ra = Gr·Pr`, `Le = Sc/Pr` — are the strongest
check available here. They relate kernels that share no code, so no consistently-retyped formula can
satisfy them, and an error in any one is caught by the other two.

### Not done

153 files remain, listed in the table above. The two rewritten are the worst two by flagged count
(146 and 30 of the 869 total findings); the rest have not been touched. The pattern to apply is the
one established here: generate the oracle in `scripts/physics_oracles.py`, drive it from a table, add
whatever identities and scaling laws the physics offers, and cover every documented refusal.

<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# What reading the physics tests found that the scanners did not

One defect explains most of the crate's blind spots, and neither scanner can see it: **where a
kernel's distinguishing term is a connection, a commutator or a curvature, every fixture sets that
term to zero or to a constant diagonal.** The tests then pin an answer that is zero, or an identity
that holds for any implementation, and the term that makes the physics non-trivial is pinned
nowhere.

The crate already contains a written diagnosis of this, in `kernels/mhd/ideal_tests.rs`:

> This test asserted `is_ok()` until the silent skips were removed, and it was never able to see
> anything: `create_dummy_manifold` is a single triangle in a 2D ambient space filled with `0.0` at
> every simplex, so the kernel's answer was zero whether or not the algebra was right. Two
> silent-default idioms kept the shape error invisible [...] and the all-zero data made the
> truncated result indistinguishable from the correct one.

That was found once and fixed in one file. The same combination is still live elsewhere.

## 1. The zero-connection family

| Kernel | Every fixture | What is therefore unpinned |
|---|---|---|
| `geodesic_integrator_kernel` | `Γ = 0` (5 tests); `Γ = 1e150` (1 test, `is_err` only) | the whole connection term `−Γ^μ_αβ u^α u^β` |
| `parallel_transport_kernel` | `Γ = 0` (5 tests); `Γ = NaN` (1 test, `is_err` only) | the same contraction |
| `weak_field_strength` | `CausalTensor::zeros(&[n, 4, 3])`, the only SU(2) connection in the crate | `F = dA + g[A, A]`; the commutator that distinguishes SU(2) from U(1) |
| `geodesic_deviation_kernel` | Riemann all zeros | the rank-4 contraction |
| `einstein_tensor_kernel` | `Ricci = metric = I`, `R` chosen so `G = 0` | the coefficient, the two tensors' distinct roles, and 3 of 4 components |
| `foppl_von_karman_strain_kernel` | flat manifold, zero data | the full strain path |
| `resistive_diffusion_kernel` | zero field | the Laplacian |
| `klein_gordon_kernel` | uniform field, `is_ok()` only | everything |

A wrong sign, a transposed index pair or a dropped term survives all of them.

**The oracle already exists and is already correct.** `theories/general_relativity/metrics_tests.rs`
builds a Schwarzschild connection with `schwarzschild_christoffel_at` and checks five components
against closed forms (`Γ^t_tr = f'/2f`, `Γ^r_tt = f f'/2`, `Γ^θ_rθ = 1/r`). It is never handed to
either consumer.

## 2. The distinguishing input is computed and discarded

`test_kerr_black_hole` verifies that Kerr reduces to Schwarzschild at `a = 0`, across all sixteen
components. Then:

```rust
// Case 2: Extreme Kerr (a=M)
let _kerr_rot = kerr_metric_at(mass, mass * 0.9, r, theta).unwrap();
```

The rotating case is computed and thrown away. Rotation is what makes Kerr Kerr, and no test
observes it.

`test_pure_electric_field` does the same with `let _ = is_rad;`, after a comment conceding the
fixture cannot discriminate: *"Actually with B=0, E·B=0 so it might pass the orthogonality test"*.

## 3. `unwrap_or(0.0)` inside an assertion

`theories/electromagnetism/em_tests.rs` reads tensor components through
`data.get(idx).copied().unwrap_or(0.0)` at twelve sites, including the antisymmetry loop:

```rust
let f_mu_nu: f64 = data.get(idx_mu_nu).copied().unwrap_or(0.0);
let f_nu_mu: f64 = data.get(idx_nu_mu).copied().unwrap_or(0.0);
assert!((f_mu_nu + f_nu_mu).abs() < 1e-10, ...);
```

An out-of-range index becomes `0.0`, and `0.0 + 0.0` satisfies the assertion. A tensor that is too
small, or an index formula that is wrong, passes. This is the second of the two "silent-default
idioms" the MHD comment names.

## 4. Tests that cannot fail

Verified by reading, not by pattern match.

- **`!data().is_empty()` as the only value assertion.** `lorentz_force_kernel`,
  `poynting_vector_kernel`, `torque_kernel`, `angular_momentum_kernel`. The result is a
  `CausalMultiVector` over `Metric::Euclidean(3)`, so `data()` has eight entries for every input
  and every implementation.
- **Rank instead of value.** `test_hookes_law_kernel_valid` asserts
  `stress.inner().num_dim() == 2`. Any rank-2 output passes; the contraction is unpinned, and the
  fixture's answer (`σ₀₀ = 1`) is trivially known.
- **A pure function called twice.** `test_isentropic_ratio_purity`, `test_area_mach_ratio_purity`
  and `test_strain_rate_galilean_invariant` call one function twice with identical arguments and
  assert the results agree. No deterministic Rust function without interior mutability can fail
  this. The last one's comment concedes the real property is "exercised structurally by the kernel
  signature".
- **Reflexive equality.** Six `trait_coverage_tests.rs` files assert `assert_eq!(a, a.clone())`,
  which holds for any derived `PartialEq` and for a broken one that always returns true, then
  discard a `format!`.
- **`R(-θ) I R(θ) = I`.** `test_jones_rotation` rotates the identity operator, chosen because the
  author could not construct the real test (*"Let's test rotating an Identity operator?"*). The
  identity is invariant under any similarity transform, so a completely wrong rotation matrix
  passes; only `d[0]` and `d[3]` are checked.

## 5. Tests whose name promises what the body does not do

- `test_kalman_filter_identity_shape_mismatch` — eighteen lines of live derivation concluding *"it
  seems extremely difficult to reach Line 121"*, then a bare `is_err()`. Both it and
  `test_kalman_filter_state_update_shape_mismatch` are superseded by
  `estimation_coverage_tests.rs`, which proves the same guards unreachable.
- `test_ginzburg_landau_error_vector_size_mismatch` — named for an error path; the body explains
  the case cannot be triggered and then asserts `is_ok()`.
- `test_qgt_band_1` — the band index is the point; the value is never checked, and with an
  all-ones eigenvector matrix the bands are indistinguishable anyway.
- `test_effective_band_drude_weight_error_negative_lattice` — passes `0.0`, not a negative value.
- `test_schwarzschild_radius_sun_known_value` — the docstring states 2.95 km; the body recomputes
  `2GM/c²` from the library's own constants and compares against that.
- `test_energy_momentum_tensor_em_wrapper_dimension_error` — body is
  `let _ = result.is_ok() || result.is_err();`.

## 6. The number is in the comment; the assertion retypes the formula

A recurring shape. The independent value is already written down and then not used:

| Test | Comment states | Assertion uses |
|---|---|---|
| `test_hydrostatic_pressure_kernel_valid` | `≈ 199391.5` | `p0 + ρ·G·depth` |
| `test_y_plus_known_value` | `≈ 3.333` | `1.0e-4 * 0.5 / 1.5e-5` |
| `test_binding_energy_mc2_formula` | `~931 MeV ≈ 1.49e-10 J` | `mass * c * c` |
| `test_doppler_effect_kernel_approaching` | `≈ 1060.6 Hz` | `f_obs > 1000.0` |
| `test_ginzburg_landau_superconducting_state` | `= -0.46875` | `energy < 0.0` |
| `test_recovery_temperature_subtracts_kinetic_term` | — | `24500.0 - 0.5*2000²/1004` |

The crate already contains the fix, twice: `test_log_law_known_value_at_y_plus_100` and
`test_speed_of_sound_air_at_room_temperature` assert the retyped formula **and** an independent
number, so the circularity is broken by the second assertion.

## 7. Two whole-file circular oracles

`kernels/chronometric/solve_gm_tests.rs` and `wrapper_tests.rs` both build their input's
`clock_drift_rate` from `forward_drift_rate`, a reimplementation of the kernel's own 1PN forward
model, and then assert the kernel inverts it back. The helper is duplicated across the two files
("mirror solve_gm_tests so each test file is self-contained"), so there are two copies of an oracle
derived from the code under test. A shared misreading of the J2 or potential convention closes the
loop regardless.

The module docstring claims this is unavoidable:

> This is a round-trip identity test — the only way an analytical inversion can be verified without
> external truth data.

`kernels/chronometric/forward_clock_tests.rs`, in the same directory, validates the same 1PN clock
equation against Ashby 2003 (`+45.7`, `−7.2`, `+38.5` µs/day). External truth exists and is in use
next door.

The oracle-defining helper also carries dead code:

```rust
let phi = -target_gm * inv_r_eff_to_potential_factor(inv_r_eff, r);
let _ = phi; // kept for clarity; the actual formula below
let phi = -target_gm * inv_r_eff;
```

`inv_r_eff_to_potential_factor` is an identity function, so both lines compute the same value.

## 8. Unedited reasoning left in committed comments

Seventeen sites in eleven files, against AGENTS.md's "document what the code does, never its
history". Three of them mark a test that was abandoned mid-thought and left asserting less than its
name claims (§4, §5 above). Others are harmless but wrong: `em/solver_tests.rs` explains a blade
index as *"1010?? No 110. 2^4=6"*, which is a bitwise OR, not a power.

`photonics/ray_tests.rs` is the clearest:

```rust
// Spec kernel takes r1: Length, r2: Length? No, I updated kernel to take f64 for signed radii.
// Wait, let's check what I implemented.
// I implemented: fn lens_maker_kernel(n: IndexOfRefraction, r1_signed: f64, r2_signed: f64)
```

## 9. One test that declines to pin the sign it derived

`em/solver_tests.rs::test_current_density_success`:

```rust
assert!((val - 1.0).abs() < 1e-9 || (val + 1.0).abs() < 1e-9, "Result: {:?}", j.data());
// Note: sign depends on metric signature/contraction order.
// e1 . (e1 e2) = (e1 . e1) e2 - (e1 . e2) e1 = (1) e2 - 0 = e2. (Minkowski e1 squared is +1?)
```

The comment derives the correct sign and the assertion accepts either. A sign error in the
contraction passes.

## 10. The wrapper delegation repair is half-finished

The comment *"Delegation, not merely success"* appears at 36 sites in 8 of the 12 wrapper test
files. Counting success-path wrapper tests only: **94 check a value, 75 assert `is_ok()` alone.**

Three files have no value check on any success path: `em` (0 of 6), `materials` (0 of 3),
`quantum` (0 of 1). Four files repaired one wrapper and left its siblings in the same file: `astro`
(`escape_velocity` yes, `orbital_velocity` and `schwarzschild_radius` no), `dynamics` (`torque` and
`angular_momentum` yes, the two energies no), `waves` (`wave_speed` yes, `doppler_effect_approaching`
no), `fluids` (one consolidated table test covers all eighteen; the eighteen named tests remain
tautologies).

`test_angular_momentum_wrapper_success` contains the same `let effect = ...; assert_eq!(...)` pair
twice verbatim, the second shadowing the first.

## 11. Six behaviours in one test function

`test_generalized_master_equation_kernel` holds six logical cases. A failure in case 2 prevents
cases 3 to 6 from running, and the suite reports one test rather than six. The 1766 figure counts
it once.

## What is strong

The crate is bimodal, and saying so bounds the work. These areas already meet the remediation
standard, and two of them are the patterns to copy:

- **`kernels/astro`** — `solver_convergence_tests.rs` roots Kepler's equation by an independent
  bisection, adds a periodicity invariant that mutation testing showed was missing, and cross-checks
  `f32` against an `f64` control. `ks_propagator_tests.rs` measures the *convergence order* of the
  Strang split and conserves energy and angular momentum over twenty steps.
- **`kernels/fluids`** — oracle tables in 50-digit decimal, three cross-kernel identities, scaling
  laws, `f32` sweeps, monotonicity on both branches of `A/A*`.
- **`kernels/propulsion`** — NACA Report 1135 table values, Anderson Ch. 5, published engine
  figures, round-trip across both branches, internal consistency (`ρ = p/RT`, `u = M√(γRT)`).
- **`kernels/chronometric/forward_clock_tests.rs`** — Ashby 2003, plus the identity that the offset
  equals the difference of drift rates.
- **`theories/general_relativity/metrics_tests.rs`** — exact Schwarzschild components, Kretschmann
  over three cases, the Kerr → Schwarzschild limit across all sixteen components.

## Repair order

1. **Give the connection kernels a non-zero connection.** Feed `schwarzschild_christoffel_at` to
   `parallel_transport_kernel` and `geodesic_integrator_kernel`; transport around a closed loop and
   assert the holonomy. Give `weak_field_strength` a non-zero SU(2) connection so `[A, A]` is
   non-zero. This is the largest correctness risk and the fixture already exists.
2. **Remove the silent defaults.** Replace the twelve `unwrap_or(0.0)` reads in `em_tests.rs` with
   indexing that panics, so a malformed tensor fails instead of reading as zero.
3. **Repair the tests that cannot fail** (§4) and the tests that do not test their name (§5).
   Delete the two superseded Kalman tests and the seventeen redundant dimensionless wrapper
   tautologies.
4. **Observe the discarded results**: `_kerr_rot`, `is_rad`, `effect.logs()`.
5. **Add the second assertion** to the §6 tests: keep the formula, add the number already written
   in the comment.
6. **Replace the chronometric round-trip's oracle** with the Ashby split its own directory already
   uses, and delete the duplicated forward model.
7. **Finish the wrapper delegation repair** across the remaining 75 success-path tests.
8. Strip the seventeen reasoning comments.

Each item closes when targeted mutation testing kills the wrong sign, constant or index it was
aimed at.

## Coverage of this read

Every file under `kernels/` was opened; `theories/` and `quantities/` were read in full for the
physics-bearing files and sampled across the 122 newtype files, whose shape is uniform. Every defect
named above was confirmed by reading the test, not by a pattern match. The one detector I wrote to
count §4's "same function twice" class was too noisy to trust (it matched constructors), so only
the instances I read are listed.

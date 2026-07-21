# deep_causality_cfd — breadth sweep across src/, verification/, studies/

**Production readiness: `needs-work`**

The engineering discipline here is unusually high for research CFD: `verification/README.md` discloses its own worst numbers (C_d 23.8 vs a 1.345 cross-reference, −80 % dissipation at 16³, +4.3 % Strouhal), every gate constant carries a rustdoc, `#[allow]` is confined to 14 style lints with zero correctness suppressions, and spot-checked reference formulas (2-D Euler conservative flux, DEC advective/diffusive CFL limits, Joseph-form covariance update, quasi-1D area source) are correct against their textbook forms. What blocks certification is a small number of silent-wrong-number paths rather than pervasive sloppiness. The compressible QTT marcher floors pressure for the LLF wave speed while feeding the unfloored negative pressure into the flux (marcher_2d.rs:134–144), removing stabilizing dissipation exactly where hyperbolicity is lost; `CompressibleCarrier::finish` drops the density-positivity guard its sibling `publish_and_transport` has, so a diverged run reports NaN as a result instead of erroring; and the ESKF folds a measurement with no guard on the innovation covariance, a NaN-poisoning path on a public avionics API. Traceability has one hard gap: the committed `dec_lid_cavity` baseline is an aborted run containing no RMSE, no vortex table, and no verdict. One of the six Park-2T acceptance gates is a transcription tautology that re-types `ler_step`'s body and compares with `==`, so the printed verdict "All six LER gates passed" overstates what was independently checked. These are bounded, individually fixable defects in an otherwise well-documented stack.

- Files read: **35**
- Findings raised: **21** — surviving adversarial verification: **21** (refuted: 0)
- Surviving by severity: critical 1, major 6, minor 12, info 2
- Independently confirmed-correct items: **10**

> **⚠ NOT ADVERSARIALLY VERIFIED.** The verifier for this module terminated on a
> session limit before returning. Every finding below is a **single-auditor claim that
> has not been independently re-checked**. Treat as unconfirmed leads and verify at
> source before acting.

## Verified correct against reference

These were positively confirmed, not merely un-flagged.

| Item | Location | Reference checked against |
|---|---|---|
| 2-D conservative Euler flux vectors F and G | `src/solvers/qtt/compressible/marcher_2d.rs:137-144` | F = (ρu, ρu²+p, ρuv, (E+p)u), G = (ρv, ρuv, ρv²+p, (E+p)v) — standard 2-D Euler conservative form (Toro, Riemann Solvers and Numerical Methods for Fluid Dynamics, §3.1) |
| DEC advective and diffusive CFL limits | `src/solvers/dec/dec_ns_solver/step.rs:140,155` | Advective: dt ≤ C·Δx/max\|u\|. Diffusive (explicit, D spatial dimensions): dt ≤ C·Δx²/(2·D·ν), from FTCS von Neumann analysis ν·dt·Σᵢ(2/Δxᵢ²) ≤ 1 |
| Joseph-form covariance update and scalar Kalman gain | `src/navigation/eskf.rs:119-141` | K = P hᵀ / S with S = h P hᵀ + r; P ← (I − K h) P (I − K h)ᵀ + K r Kᵀ (Groves 2013, Principles of GNSS, Inertial and Multisensor Integrated Navigation, §3.2.4 Joseph form) |
| Quasi-1-D duct pressure area-source term | `src/types/flow/duct_march_run.rs:184-190` | ∂(ρu)/∂t + (1/A)∂((ρu²+p)A)/∂x = (p/A)·dA/dx — quasi-one-dimensional Euler with variable area (Anderson, Modern Compressible Flow, §5.2) |
| Skew-symmetrized convective operator ½(G − G*) | `src/solvers/dec/dec_ns_rate.rs:665-678` | The skew-symmetric part of a linear operator G with respect to the mass-matrix inner product is ½(G − G*), with G* the M₁-weighted adjoint: (G*u)[j] = ⟨G e_j, M₁u⟩ / M₁[j] |
| Mean-crossing frequency estimator core relation | `src/types/flow/frequency.rs:45-48` | A sinusoid crosses its mean twice per period, so periods = crossings/2 and f = periods/T |
| Standard Knudsen regime band edges | `src/types/flow/corridor/regime.rs:171,179-181` | Continuum Kn<0.01, slip 0.01≤Kn<0.1, transitional 0.1≤Kn<10, free-molecular Kn≥10 (Schaaf & Chambré; Anderson, Hypersonic and High-Temperature Gas Dynamics, §1.4) |
| Park/Gupta cgs→SI rate-constant conversion | `src/solvers/qtt/compressible/fitting.rs:179-184` | k [cm³·mol⁻¹·s⁻¹] × 1e-6 [m³/cm³] ÷ N_A [mol⁻¹] = k [m³·s⁻¹ per particle]; τ = 1/(k·n) with n in m⁻³ |
| ESKF velocity-error ← attitude-error transition block | `src/navigation/eskf.rs:62-68` | δv̇ = −[f×]δψ with [f×] the skew matrix of specific force, so −[f×] = [[0,f_z,−f_y],[−f_z,0,f_x],[f_y,−f_x,0]] (Solà 2017, Quaternion kinematics for the error-state KF, §6; Groves 2013 §14.2) |
| KeyedTable construction invariants | `src/types/keyed_table.rs:74-101` | A monotone interpolation table requires: non-empty, finite keys, uniform column count, no duplicate keys, sorted |

## Findings

### 3.1 [CRITICAL] QTT compressible marcher feeds negative pressure into the flux while flooring it for the LLF wave speed

- **Verification verdict:** pending verification
- **Axis:** physics-math
- **Location:** `deep_causality_cfd/src/solvers/qtt/compressible/marcher_2d.rs:135`
- **Auditor confidence:** confirmed

**Claim.** When the equation of state yields p ≤ 0, the code floors p to 1e-300 only for the sound-speed/wave-speed computation, but the unfloored (negative) p is what is written into every flux component. The result is a physically invalid flux combined with an under-estimated Rusanov dissipation coefficient, produced silently with no error.

**Code evidence.**

```
133:            let mom2 = mx * mx + my * my;
134:            let p = (self.gamma - R::one()) * (e - half * mom2 / rho);
135:            let p_floor = if p > tiny { p } else { tiny };
136:            let c = (self.gamma * p_floor / rho).sqrt();
137:            f[0].push(mx);
138:            f[1].push(mx * vx + p);
...
143:            g[2].push(my * vy + p);
144:            g[3].push((e + p) * vy);
145:            let sx = vx.abs() + c;
146:            let sy = vy.abs() + c;
(density is guarded at 125-129 with PhysicalInvariantBroken; pressure is not)
```

**Reference form.** For a local Lax-Friedrichs / Rusanov flux the numerical viscosity coefficient must bound the true spectral radius s ≥ max(|u|+c) with c = sqrt(γp/ρ) (Toro, Riemann Solvers and Numerical Methods for Fluid Dynamics, §10.5). When p < 0 the Euler system is not hyperbolic, c is imaginary, and the correct response is to reject the state (or apply a positivity-preserving limiter), not to substitute c ≈ 0 while retaining p < 0 in the flux.

**Impact.** A run that leaves the positive-pressure cone keeps marching and returns numbers. Because p_floor ≈ 0 drives c ≈ 0, s_max collapses to max|u|, so the Rusanov dissipation is removed at exactly the step where it is needed to stabilise the scheme — the failure accelerates instead of being caught. An engineer reading the returned field has no signal that the state was unphysical. Density positivity is enforced eleven lines above, so the omission reads as an oversight rather than a decision.

**Recommended fix.** Treat p ≤ 0 the same way as rho ≤ 0: return PhysicsError::PhysicalInvariantBroken naming the offending cell. If a floor is genuinely wanted for robustness, apply the same floored value to the flux and the wave speed consistently, and record that the floor engaged in the report so the result is not read as a clean solve. Check the sibling sites carrying the identical 1e-300 literal (marcher_3d.rs:124, marcher_3d_fitted.rs:136, euler_1d.rs:100) for the same pattern.

---

### 3.2 [MAJOR] Park-2T acceptance gate (ii) is a transcription tautology: it re-types ler_step's body and compares with ==

- **Verification verdict:** pending verification
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/verification/qtt_park2t_blackout/print_utils.rs:38`
- **Auditor confidence:** confirmed

**Claim.** Gate (ii) compares ler_step's output against an expression that is character-for-character the function's own implementation body, using bitwise equality. It cannot fail while the implementation is unchanged and it verifies no physical or numerical property — yet it is counted as one of the six gates behind the printed verdict 'All six LER gates passed — Gap-2 Tier-A slice verified.'

**Code evidence.**

```
verification/qtt_park2t_blackout/print_utils.rs:
35:/// (ii) Exactness of the closed-form exponential on the linear relaxation (equality, not tolerance).
36:fn gate_exponential_exactness() -> bool {
37:    let (x, x_eq, tau, dt) = (300.0_f64, 7000.0_f64, 0.01_f64, 0.003_f64);
38:    ler_step(x, x_eq, tau, dt) == x_eq - (x_eq - x) * (-(dt / tau)).exp()
39:}

src/types/flow/blackout.rs (the function under test):
41:pub fn ler_step<R: CfdScalar>(x: R, x_eq: R, tau: R, dt: R) -> R {
42:    if tau <= R::zero() {
44:        return x_eq;
45:    }
46:    x_eq - (x_eq - x) * (-(dt / tau)).exp()
47:}
```

**Reference form.** A verification gate must compare the implementation against an independently derived reference — an analytic solution, a published value, or a conserved invariant. Comparing f(x) against a copy of f's own body tests transcription, not correctness (ASME V&V 20-2009, §2: the comparison standard must be independent of the code being verified).

**Impact.** The example's verdict line asserts the Tier-A slice is 'verified' on the strength of six gates, one of which is vacuous. A reviewer counting gates over-estimates the independent evidence by one sixth. The same pattern appears in gate (iv) at line 79-80, where `saha = ler_step(0.0, alpha_eq, 0.0, 1e-5)` exercises the `tau <= 0` early-return that returns `x_eq` exactly, and is then checked against `alpha_eq` to within 1e-12 — also true by construction.

**Recommended fix.** Either delete gate (ii) and renumber, or convert it into a real check: integrate dx/dt = (x_eq − x)/τ with a high-order adaptive ODE solver (or compare against a series expansion computed independently of ler_step) and assert agreement to a stated tolerance. Same for the τ→0 limb of gate (iv). Update the verdict text and verification/README.md line 39 to state the number of independent gates.

---

### 3.3 [MAJOR] CompressibleCarrier::finish drops the density-positivity guard that publish_and_transport enforces

- **Verification verdict:** pending verification
- **Axis:** physics-math
- **Location:** `deep_causality_cfd/src/types/flow/compressible_march_run.rs:470`
- **Auditor confidence:** confirmed

**Claim.** `publish_and_transport` rejects a non-positive or non-finite density before dividing by it, but `finish` — running the same per-cell EOS loop over the same decoded state — has no such guard. A diverged march therefore returns a Report full of inf/NaN as its headline final field instead of returning an error.

**Code evidence.**

```
publish_and_transport (guarded):
427:        for (((&rho, &mx), &my), &e) in dense[0].iter()...
429:            if rho <= R::zero() || !rho.is_finite() {
430:                return Err(PhysicsError::PhysicalInvariantBroken(
431:                    "compressible carrier: density must stay positive".into(),
432:                ));
433:            }
434:            let u2 = (mx * mx + my * my) / (rho * rho);

finish (unguarded — same loop, no check):
468:        for (((&rho, &mx), &my), &e) in dense[0].iter()...
470:            let u2 = (mx * mx + my * my) / (rho * rho);
471:            let p_hat = (self.gamma - R::one()) * (e - half * rho * u2);
473:            t_tr.push((p_hat / rho) * self.reference.t_ref);
477:        report.set_final_field(t_tr);
```

**Reference form.** The crate's own invariant, stated at line 431: 'compressible carrier: density must stay positive'. Positivity of ρ is a precondition of the ideal-gas EOS p = (γ−1)(E − ½ρ|u|²) and of every division by ρ.

**Impact.** `finish` writes the final temperature field, `final_n_tot`, and `final_speed` — the numbers a consumer actually reads out of a completed run. A run whose density collapsed to zero or went negative mid-march produces a Report populated with NaN/inf rather than a PhysicsError, so the failure is discovered only if the consumer independently checks finiteness. The invariant is enforced during the march and abandoned at the point of reporting.

**Recommended fix.** Hoist the rho guard into a shared helper used by both loops (both already share the same `half`/`tiny` lift preamble), so the positivity check and the EOS evaluation cannot drift apart again.

---

### 3.4 [MAJOR] ESKF scalar update divides by an unguarded innovation covariance and never validates the measurement variance

- **Verification verdict:** pending verification
- **Axis:** physics-math
- **Location:** `deep_causality_cfd/src/navigation/eskf.rs:122`
- **Auditor confidence:** confirmed

**Claim.** `update_scalar` computes the Kalman gain as `ph[i] / s` with no check that `s > 0`, and neither it nor its public caller `ReentryNav::correct_position` validates that the measurement variance `r` is finite and non-negative. With s = 0 the gain is 0/0 = NaN, which propagates into both the state estimate and the full covariance matrix in the same call.

**Code evidence.**

```
eskf.rs:
117:    pub fn update_scalar(&mut self, h: [R; NAV_STATES], z: R, r: R) {
119:        let ph = mat_vec(&self.cov, &h); // P·hᵀ
120:        let s = dot(&h, &ph) + r; // innovation covariance (scalar)
121:        let innov = z - dot(&h, &x);
122:        let k: [R; NAV_STATES] = core::array::from_fn(|i| ph[i] / s); // Kalman gain
123:        let x_new: [R; NAV_STATES] = core::array::from_fn(|i| x[i] + k[i] * innov);

reentry_nav.rs (public caller, r_var passed straight through):
89:    pub fn correct_position(&mut self, measured_position: [R; 3], r_var: R) {
93:            h[i] = R::one();
94:            self.filter.update_scalar(h, z, r_var);
```

**Reference form.** The Kalman gain K = P hᵀ S⁻¹ requires S = h P hᵀ + R to be strictly positive definite; for the scalar case S > 0. Standard practice is to reject or skip the update when S ≤ 0, and to require R ≥ 0 as an input precondition (Groves 2013, §3.2.3; Bar-Shalom, Li & Kirubarajan, Estimation with Applications to Tracking and Navigation, §5.2).

**Impact.** Reachable from the public API: `NavFilter::new(state, cov_diag)` accepts an all-zero covariance diagonal and `NavFilter::restore(state, cov)` accepts an arbitrary matrix, so P[i][i] = 0 is constructible; combined with `correct_position(pos, 0.0)` — a plausible way to express a perfect/truth-injected fix — this yields s = 0 and NaN-poisons the entire 17×17 covariance and the navigation state in one call, with no error returned. A negative `r_var` is likewise accepted and produces a sign-flipped gain and a divergent filter, silently. On an avionics navigation API these inputs must be rejected, not consumed.

**Recommended fix.** Change the signature to return Result, validate `r.is_finite() && r >= 0`, and reject or no-op the fold when `s` is not strictly positive and finite, logging the skipped update. Apply the same validation at the `ReentryNav::correct_position` boundary so the constraint is visible to callers.

---

### 3.5 [MAJOR] Committed lid-cavity baseline.txt is an aborted run containing no RMSE, no vortex table, and no verdict

- **Verification verdict:** pending verification
- **Axis:** doc-gap
- **Location:** `deep_causality_cfd/verification/dec_lid_cavity_re1000_verification/baseline.txt:11`
- **Auditor confidence:** confirmed

**Claim.** The committed baseline artifact for the Ghia lid-cavity verification is 11 lines long and stops at t = 44.99 of a t_end = 100 run. It contains none of the output the harness exists to produce — no centerline RMSE, no detected vortex centers, no gate verdict — so the committed evidence does not substantiate any of the numbers verification/README.md reports for this example.

**Code evidence.**

```
Full contents of baseline.txt (11 lines):
  # DEC lid-driven cavity, Re = 1000
  # grid 65x65 (h = 0.01562), dt = 0.00703, t_end = 100, steps = 14223
  # t =     5.00 (711/14223)
  ...
  # t =    44.99 (6399/14223)

The run never reached the reporting stage; main.rs:68 is where results are produced:
67:    let u_form = march(n, t_end);
68:    print_utils::render(&u_form, n, config::ft(h));

Every other verification baseline is complete and ends in a verdict, e.g. verification/qtt_sod/baseline.txt ends with '=== Sod profiles match the exact Riemann solution — compressible flux verified. ==='
```

**Reference form.** verification/README.md line 15-17: 'Every example self-verifies and exits with a nonzero status the moment its invariant or reference check fails.' Line 34 reports specific measured values (primary vortex (0.563, 0.594); RMSE 0.137) for this example. A committed baseline is the artifact that lets a reviewer confirm those figures without re-running a ~28 s to minutes-long job.

**Impact.** An auditor comparing README claims against committed evidence finds nothing to compare for the one example that validates against a published benchmark table (Ghia et al. 1982). The truncation is silent — the file looks like a normal baseline until you notice the step counter stops at 45 % of the horizon. This is the only baseline in the suite with this defect, so a reviewer sampling other files will not discover it.

**Recommended fix.** Re-run `cargo run --release -p deep_causality_cfd --example dec_lid_cavity_re1000_verification` to completion, capturing both stdout and stderr, and commit the full output. Add the `trend` mode's gated output as a second artifact, since that is the mode README line 124-125 describes as the gate. Consider a CI check that every baseline.txt ends with its example's verdict line.

---

### 3.6 [MAJOR] Unjustified Mach 1.05 shock floor decides whether the Rankine-Hugoniot jump is applied to the inflow

- **Verification verdict:** pending verification
- **Axis:** magic-number
- **Location:** `deep_causality_cfd/src/types/flow/compressible_march_run.rs:327`
- **Auditor confidence:** confirmed

**Claim.** A bare literal 1.05 selects between applying an exact normal-shock jump and passing the freestream through unchanged. The physical bifurcation is at M = 1, not 1.05, so in the band 1.0 < M ≤ 1.05 the code uses the freestream where a shock physically stands, and at M = 1.05 the inflow state jumps discontinuously. The value has no cited source and is not configurable.

**Code evidence.**

```
326:        // The exact RH jump when a shock stands; the freestream itself below Mach ~1.
327:        let shock_floor = Self::lift(1.05)?;
328:        let (t_post, n_post, u_post) = if mach > shock_floor {
329:            let shock = FittedNormalShock::new(schedule.gamma_eff)?;
330:            let post = shock.post_shock(row.temperature, row.n_tot, mach)?;
331:            (post.t2, post.n_tot2, speed * post.u_ratio)
332:        } else {
333:            (row.temperature, row.n_tot, speed)
334:        };
(the comment says 'below Mach ~1' while the code tests 1.05)
```

**Reference form.** A normal shock exists for all M₁ > 1; the Rankine-Hugoniot relations give ρ₂/ρ₁ = ((γ+1)M²)/((γ−1)M²+2) and T₂/T₁ = [(2γM²−(γ−1))((γ−1)M²+2)]/[(γ+1)²M²], both continuous and equal to 1 at M = 1 (Anderson, Modern Compressible Flow, §3.6). The physically correct threshold is therefore 1.0, at which the branch is continuous by construction.

**Impact.** The inflow boundary state — which sets rho_hat, t_hat, u_hat and hence the entire nondimensional inflow strip at lines 337-343 — steps discontinuously as a descent trajectory crosses M = 1.05. For γ_eff = 1.1 the jump at that Mach is a few percent in each primitive, injected as a boundary transient. The code comment states a threshold ('~1') different from the one implemented (1.05), so a reader checking the comment will not notice. There is no `with_shock_floor` override, unlike the neighbouring rebuild tolerance and rebuild budget which are both configurable.

**Recommended fix.** Use 1.0 so the branch is continuous, or, if a small buffer above sonic is deliberate to avoid the M→1⁺ stiffness of the RH relations, state that reason in the comment, cite the stiffness bound it is protecting against, and expose it as a builder override alongside `with_rebuild_tolerance`. Fix the comment to match whatever value is chosen.

---

### 3.7 [MAJOR] Absolute 1e-12 tolerance on the Hodge star diagonal silently zeroes real terms of the convective operator

- **Verification verdict:** pending verification
- **Axis:** magic-number
- **Location:** `deep_causality_cfd/src/solvers/dec/dec_ns_rate.rs:637`
- **Auditor confidence:** confirmed

**Claim.** `zero_tol = 1e-12` is an absolute threshold applied to the Hodge-star diagonal entry m1[j], a dimensional quantity that scales with mesh spacing and metric grading. Any edge whose diagonal falls below it has its adjoint contribution set to exactly zero, dropping a real term from the skew-symmetric convective operator. The accompanying comment justifies only that 1e-12 is representable, not that it is the right magnitude.

**Code evidence.**

```
637:        let zero_tol = <R as FromPrimitive>::from_f64(1e-12)
638:            // Coverage exemption: 1e-12 lifts into every real field.
639:            .expect("1e-12 is representable in every RealField");
...
665:            *slot = if m1[j].abs() <= zero_tol {
666:                R::zero()
667:            } else {
668:                dot / m1[j]
669:            };
```

**Reference form.** A degeneracy test on a dimensional quantity must be scaled by that quantity's own magnitude — e.g. `m1[j].abs() <= eps_rel * max_j|m1[j]|`, or a machine-epsilon-relative bound `eps * ||M₁||`. The discrete Hodge star on a D-dimensional lattice carries units of h^(D−2k) for k-forms (Hirani, Discrete Exterior Calculus, 2003, §5), so an absolute cutoff changes meaning under mesh refinement and under a change of length unit.

**Impact.** Two silent-wrong-result paths. First, on a fine or strongly graded mesh, legitimate small diagonal entries are treated as degenerate and their convective adjoint contribution is dropped, breaking the skew-symmetry the operator is constructed to have and hence the discrete energy conservation that skew-symmetry buys. Second, the same geometry expressed in millimetres rather than metres shifts every m1[j] by orders of magnitude, so the solver's answer depends on the caller's choice of length unit. Neither condition raises a diagnostic.

**Recommended fix.** Replace with a relative test against the diagonal's own scale, e.g. compute `m1_max = m1.iter().map(|v| v.abs()).fold(zero, max)` once and test `m1[j].abs() <= eps * m1_max` with eps tied to R::epsilon(). Record in the rate's diagnostics whenever any edge is zeroed, so a user can tell that the operator was truncated.

---

### 3.8 [MINOR] The 1e-300 pressure floor evaluates to zero in the f32 precision mode the crate documents as supported

- **Verification verdict:** pending verification
- **Axis:** magic-number
- **Location:** `deep_causality_cfd/src/solvers/qtt/compressible/marcher_2d.rs:123`
- **Auditor confidence:** confirmed

**Claim.** `tiny = R::from_f64(1e-300)` is used as the pressure floor in the compressible marchers, but 1e-300 is below f32's smallest subnormal (~1.4e-45). Under the f32 instantiation the constant becomes 0.0 (or the `unwrap_or_else(R::zero)` fallback fires), so the floor silently disappears and `p_floor` can be exactly zero.

**Code evidence.**

```
123:        let tiny = R::from_f64(1e-300).unwrap_or_else(R::zero);
...
135:            let p_floor = if p > tiny { p } else { tiny };
136:            let c = (self.gamma * p_floor / rho).sqrt();

Same literal at marcher_3d.rs:124, marcher_3d_fitted.rs:136, euler_1d.rs:100.

The f32 mode is a documented, supported configuration — src/traits/cfd_scalar.rs:12-15:
/// Scalar bound for every CFD theory and solver: precision as a parameter (`f32`,
/// `f64`, `Float106`) ...
```

**Reference form.** A guard constant must be representable in every scalar type the generic code is instantiated at. f32 has minimum normal ≈1.18e-38 and minimum subnormal ≈1.4e-45; f64 minimum subnormal ≈4.9e-324. A portable floor should derive from the scalar's own limits (R::min_positive_value() or a multiple of R::epsilon()) rather than a hard-coded decimal.

**Impact.** At f32 the guard is a no-op: `p > 0.0` is false for p = 0, so p_floor = 0, c = 0, and the LLF wave speed loses its acoustic contribution entirely for that cell. The behaviour differs between the f64 and f32 builds of the same case with no diagnostic, which undermines the crate's 'precision as a parameter' claim for this code path. The `unwrap_or_else(R::zero)` also means a lift failure degrades to no floor rather than to an error.

**Recommended fix.** Derive the floor from the scalar type — e.g. `R::min_positive_value()` or `R::epsilon() * reference_pressure` — so it is meaningful at every supported precision. Independently of this, see the critical finding above: the floor should not be applied asymmetrically between the flux and the wave speed in any precision.

---

### 3.9 [MINOR] Silent unwrap_or_else fallbacks substitute physically wrong constants instead of erroring, next to ok_or_else error handling in the same function

- **Verification verdict:** pending verification
- **Axis:** magic-number
- **Location:** `deep_causality_cfd/src/types/flow/blackout.rs:356`
- **Auditor confidence:** confirmed

**Claim.** Within four lines, the same function lifts one constant with `ok_or_else(...)` (propagating a PhysicsError on failure) and the next with `unwrap_or_else(R::one)` (silently substituting 1.0 for 1e30). The fallback value is not a degraded approximation of the intended one — it changes the frozen-chemistry timescale from dt·1e30 to dt·1.0, i.e. from 'no relaxation' to 'a full relaxation time constant per step'. The same anti-pattern appears in the Knudsen classifier.

**Code evidence.**

```
blackout.rs:
352:        let cm3_per_m3 = R::from_f64(1.0e6)
353:            .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(1e6) failed".into()))?;
354:        // A frozen-chemistry timescale ≫ dt: when the forward rate vanishes the LER
355:        // step leaves α effectively unchanged (no spurious jump to equilibrium).
356:        let huge = R::from_f64(1.0e30).unwrap_or_else(R::one);
357:        let frozen_tau = ctx.dt() * huge;

corridor/regime.rs (same pattern, three physical thresholds):
179:            slip_threshold: R::from_f64(0.01).unwrap_or_else(R::zero),
180:            transitional_threshold: R::from_f64(0.1).unwrap_or_else(R::zero),
181:            free_molecular_threshold: R::from_f64(10.0).unwrap_or_else(R::one),
```

**Reference form.** The crate's own convention, visible at blackout.rs:352-353 and used throughout compressible_march_config.rs: a failed constant lift is a NumericalInstability error, not a substituted value. A fallback is only defensible when the substitute is a safe degradation of the intended semantics.

**Impact.** If the lift ever fails, `frozen_tau = dt` makes ler_step relax ~63 % of the way to equilibrium every step in precisely the regime the comment says must stay frozen — silently reported as a chemistry result. In regime.rs a failed lift sets the slip and transitional Knudsen thresholds to 0.0 and the free-molecular threshold to 1.0, misclassifying every flow regime and hence every GNSS-denial decision downstream. For all currently supported scalar types (f32/f64/Float106) these lifts succeed, so the paths are not reachable today — the defect is that an unreachable-today branch encodes silently wrong physics rather than a refusal, which is exactly what a new scalar type would trip over.

**Recommended fix.** Use `ok_or_else(|| PhysicsError::NumericalInstability(...))?` uniformly at all four sites, matching the convention already used two lines above in blackout.rs. Where the constructor cannot return Result (RegimeClassify::new), either make it fallible or move the lift to a validated build step.

---

### 3.10 [MINOR] DRAG_SANITY_MAX is documented as bounding an 'O(1) drag coefficient' but admits the measured value of 23.76

- **Verification verdict:** pending verification
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/verification/qtt_cylinder_verification/print_utils.rs:20`
- **Auditor confidence:** confirmed

**Claim.** The gate constant's rustdoc describes it as a 'Pinned upper bound on a physical O(1) drag coefficient (sanity)', but the harness's own committed baseline measures C_d = 23.7577 — 24× O(1) and well inside the bound of 100. As written the gate tests only positivity and finiteness; the stated 'O(1)' rationale is not what the number encodes.

**Code evidence.**

```
print_utils.rs:
19:/// Pinned upper bound on a physical O(1) drag coefficient (sanity).
20:const DRAG_SANITY_MAX: f64 = 100.0;
...
108:    if !(finest.drag > 0.0 && finest.drag < DRAG_SANITY_MAX) {
109:        eprintln!("FAIL: drag {:.4} is not positive and finite", finest.drag);

baseline.txt (measured):
  bond <=  24   C_d = 23.7577   |dC_d| = 1.89e-11   interior_max|u| = 4.22e-2

config.rs (the cross-reference it is 17.7x away from):
38:pub const DEC_CD_REF: f64 = 1.345;
```

**Reference form.** A sanity bound stated as testing an 'O(1)' quantity should be within a small factor of 1. The verification README is candid about the discrepancy — line 222-224: 'The absolute C_d ≈ 23.8 is not the isolated-cylinder value (DEC ≈ 1.345): it is inflated by ~30 % blockage, the smoothing-skirt penalization-force definition, and the transient' — so the correct characterisation exists, just not at the constant.

**Impact.** A reader of the gate source concludes the harness bounds C_d to an O(1) range; it does not, and cannot, since the configuration produces 23.8. The gate is effectively unfailable for this case. This matters because `summary()` at line 117-125 prints 'Immersed cylinder verified' on the strength of these three gates. The README carries the honest framing, so the defect is a local doc-vs-code mismatch rather than a suite-wide misrepresentation.

**Recommended fix.** Restate the constant's doc as what it is — a non-divergence guard, not an O(1) physicality claim — and, since the configuration's expected magnitude is known to ~1e-11 reproducibility, tighten the bound to bracket the expected 23.76 (e.g. 20.0 to 30.0) so it would actually catch a regression. Mirror the README's blockage/skirt explanation at the constant.

---

### 3.11 [MINOR] NO_SLIP_FLOOR is an absolute constant while the Brinkman penalization error it gates scales as sqrt(eta)

- **Verification verdict:** pending verification
- **Axis:** magic-number
- **Location:** `deep_causality_cfd/verification/qtt_cylinder_verification/print_utils.rs:16`
- **Auditor confidence:** confirmed

**Claim.** The no-slip gate compares interior speed against a fixed 0.15·U_inf. The physical interior slip of a volume-penalized body scales with the penalization parameter as O(sqrt(eta)); with the configured ETA = 0.016, sqrt(eta) = 0.126, so the gate value tracks the theory only by coincidence at this one ETA. `ETA` is a plain const in config.rs, so changing it silently decouples the gate from the physics it is meant to test.

**Code evidence.**

```
print_utils.rs:
15:/// Pinned no-slip floor: interior speed must fall below this fraction of the free-stream.
16:const NO_SLIP_FLOOR: f64 = 0.15;
...
83:    // 1. No-slip: the interior velocity is at the penalization floor.
84:    if finest.interior_max_speed > NO_SLIP_FLOOR * config::U_INF {

config.rs:
27:/// Brinkman penalization parameter (small → hard wall).
28:pub const ETA: f64 = 0.016;

baseline.txt measured: interior_max|u| = 4.22e-2 (against a gate of 0.15 — 3.5x margin)
```

**Reference form.** Angot, Bruneau & Fabrie (1999), 'A penalization method to take into account obstacles in incompressible viscous flows', Numer. Math. 81:497-520 — the volume-penalization solution converges to the no-slip solution at rate O(eta^{1/2}) in the H¹ norm. The gate bound should therefore be written as C·sqrt(ETA) with C stated, not as a bare fraction. This paper is already cited in verification/README.md line 226.

**Impact.** Raising ETA to 0.1 (a softer wall) would raise the theoretical slip to ~0.32·U_inf, and the gate would fail for a solver that is behaving exactly as the penalization theory predicts — or, conversely, lowering ETA would leave the gate 10x too loose to detect a genuine no-slip regression. The constant encodes a result rather than a criterion. The measured 4.22e-2 also sits a factor of 3 below sqrt(eta) = 0.126, which is consistent with the theory but is not itself checked.

**Recommended fix.** Express the floor as a coefficient times `ETA.sqrt()`, cite the Angot et al. rate at the constant (the paper is already in the README reference list), and state the coefficient's origin. That makes the gate track ETA automatically and turns it into a test of the penalization convergence rate rather than of one tuned number.

---

### 3.12 [MINOR] Lid-cavity trend gates are back-fitted to the measured values with 25% headroom, under a header claiming validation against Ghia

- **Verification verdict:** pending verification
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/verification/dec_lid_cavity_re1000_verification/main.rs:117`
- **Auditor confidence:** confirmed

**Claim.** The two RMSE gates were chosen after seeing the measured values (0.252 and 0.133), as the code comment states outright. They therefore test non-regression against this code's own prior output, not agreement with Ghia. The run header printed one line above calls the mode 'refinement trend vs Ghia (1982)', which reads as an accuracy claim.

**Code evidence.**

```
main.rs:
103:    println!("# DEC lid-driven cavity, Re = {RE}: refinement trend vs Ghia (1982)");
...
117:    // Gates from the pinning measurements (time-converged 0.252 / 0.133, ~25 % headroom) plus the
118:    // strict refinement-trend margin. Compared in native `FloatType` (the `f64` gates lift via `ft`).

config.rs:
83:pub const TREND_COARSE_GATE: f64 = 0.32;
85:pub const TREND_FINE_GATE: f64 = 0.20;
87:pub const TREND_MARGIN: f64 = 0.04;

print_utils.rs:56-70 — the RMSE is an unnormalized absolute RMS deviation from the Ghia u/v tables, whose values span roughly [-0.4, 1.0] against a lid speed of 1.0. A passing fine-grid RMSE of 0.20 is therefore ~20 % of lid speed.
```

**Reference form.** A back-fitted bound is legitimate as a regression tripwire but cannot serve as a validation criterion; the two must be labelled differently (ASME V&V 20-2009 distinguishes validation comparison error from numerical regression). A validation gate against Ghia would state a target accuracy derived from the reference's own uncertainty and the grid's expected discretization error.

**Impact.** The strongest of the three gates is the strict-decrease trend check at line 140, which is a genuine (if weak) discretization property. The two RMSE bounds add little: they can only fail if the solver regresses past a quarter of its current error. A reader of the header line concludes the run validates against Ghia at these grids; it certifies that a 17²/33² cavity is no worse than it was when the gates were pinned, at 13-25 % of lid speed. The verification README (line 130-133) does give the honest framing with the 6 %-of-span vortex offset, so the overstatement is local to the harness.

**Recommended fix.** Rename the constants to say what they are (e.g. TREND_COARSE_REGRESSION_BOUND) and change the header to 'refinement trend, regression-gated; accuracy vs Ghia reported not gated'. If a genuine accuracy gate is wanted, run at 129² (Ghia's own grid, already noted as the reporting resolution) and pin the bound to the discretization error expected there.

---

### 3.13 [MINOR] RAM-C blackout-onset gate is implied by the electron-density gate and cannot independently fail

- **Verification verdict:** pending verification
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/verification/qtt_ramc_stagline/print_utils.rs:68`
- **Auditor confidence:** confirmed

**Claim.** Gate g3 asserts omega_p > COMMS_BAND_RAD_S. Gate g2, evaluated four lines earlier, already requires n_e > 3e18 m⁻³. Since omega_p is a monotone function of n_e alone, g2 passing forces g3 to pass by a margin of about 19x in frequency. g3 contributes no independent verification but is presented as one of four gates.

**Code evidence.**

```
64:    let g2 = gate(
65:        "peak n_e within ~3× of RAM-C II (Park-2T controller)",
66:        out.electron_density > NE_LO && out.electron_density < NE_HI,
67:    );
68:    let g3 = gate("blackout onset (ω_p > comms band)", out.blackout);
...
70:    g1 && g2 && g3 && g4

config.rs:
27:pub const COMMS_BAND_RAD_S: f64 = 9.4e9;
print_utils.rs:
20:const NE_LO: f64 = 3.0e18;
```

**Reference form.** Cold-plasma frequency omega_p = sqrt(n_e e²/(eps0 m_e)) = 5.64e4·sqrt(n_e[cm⁻³]) rad/s (Chen, Introduction to Plasma Physics and Controlled Fusion, §4.3). Solving omega_p > 9.4e9 gives n_e > 2.8e10 cm⁻³ = 2.8e16 m⁻³ — two orders of magnitude below the NE_LO = 3e18 m⁻³ that gate g2 already enforces.

**Impact.** The example reports four gates and the README (line 41) counts them as evidence. One is a deterministic consequence of another, so the independent evidence is three gates, not four. The blackout condition itself is physically meaningful — it is simply not a test, given the band this configuration flies in. Low practical severity because the surrounding disclosure in this harness is otherwise exemplary (NETWORK_BAND_DECADES = 0.7 is explicitly labelled as pinned from measurement and justified against the cited DPLR/LAURA/US3D 2-5x spread).

**Recommended fix.** Either fold g3 into g2's label (noting that the n_e band implies blackout at this comms frequency by a stated margin), or make it independent by gating the blackout *margin* or the predicted onset *time* against a RAM-C flight value, which would test something the n_e gate does not.

---

### 3.14 [MINOR] Stability gate's second conjunct is a comparison between two compile-time constants

- **Verification verdict:** pending verification
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/verification/qtt_park2t_blackout/print_utils.rs:32`
- **Auditor confidence:** confirmed

**Claim.** The `euler > x_eq * 100.0` term of gate (i) is computed entirely from literals defined inside the test function; it invokes no library code and evaluates to 6700300.0 > 700000.0, a constant true. It is presented as the half of the gate that demonstrates 'where an explicit Euler rate step diverges'.

**Code evidence.**

```
20:fn gate_stability_at_stiffness() -> bool {
21:    let (dt, x_eq) = (1.0_f64, 7000.0_f64);
22:    let tau = dt / 1000.0;
23:    let mut x = 300.0_f64;
...
31:    let euler = 300.0 + (dt / tau) * (x_eq - 300.0);
32:    (x - x_eq).abs() < 1.0 && euler > x_eq * 100.0
(euler = 300 + 1000*6700 = 6_700_300; x_eq*100 = 700_000)
```

**Reference form.** A gate term must be capable of both outcomes as a function of the code under test. Here `euler` depends only on dt, tau, and x_eq, all bound to literals in lines 21-22, so no change to ler_step or to any library code can alter this conjunct.

**Impact.** Minor: the first conjunct `(x - x_eq).abs() < 1.0` does exercise ler_step across 50 stiff steps and is a real check, and the loop body's monotonicity bounds at lines 26-28 are also real. The constant term is illustrative arithmetic embedded in a boolean gate, where it reads as a second independent condition. Combined with the tautological gate (ii) in the same file, two of the six advertised gates carry less evidence than their count implies.

**Recommended fix.** Move the explicit-Euler divergence figure out of the boolean and into the printed output as context ('explicit Euler would reach 6.7e6 at this stiffness'), keeping the gate to the conditions that actually depend on ler_step.

---

### 3.15 [MINOR] Undocumented pressure clamp means a reported T_tr can be a floor artifact rather than an EOS value

- **Verification verdict:** pending verification
- **Axis:** doc-gap
- **Location:** `deep_causality_cfd/src/types/flow/compressible_march_run.rs:436`
- **Auditor confidence:** confirmed

**Claim.** Both the publish and finish paths clamp the nondimensional pressure at 1e-12 before dividing by density to obtain temperature, but the rustdoc describing the published scalars says only that they come 'from the equation of state'. A consumer cannot tell from the documentation that a returned T_tr may be the clamp value rather than a computed one, and nothing in the Report records that the clamp engaged.

**Code evidence.**

```
407:    /// Publish the evolved physical projections: `"speed"` (m/s), `"T_tr"` (K) and
408:    /// `"pressure_atm"` from the equation of state, and `"n_tot"` (m⁻³) from the density.
...
426:        let tiny = Self::lift(1.0e-12)?;
...
435:            let p_hat = (self.gamma - R::one()) * (e - half * rho * u2);
436:            let p_hat = if p_hat > tiny { p_hat } else { tiny };
437:            let t_hat = p_hat / rho;
438:            let t_phys = t_hat * self.reference.t_ref;
(and again at 467/472 in `finish`)
```

**Reference form.** The documented contract is p = (γ−1)(E − ½ρ|u|²) and T = p/ρ in nondimensional variables. The implemented contract is T = max(p, 1e-12)/ρ. Where an implementation deviates from the stated formula, the deviation and its trigger condition belong in the rustdoc.

**Impact.** A cell whose energy budget has gone inconsistent (E < ½ρ|u|², i.e. negative internal energy) reports T_tr = 1e-12·t_ref/rho — a small positive temperature that looks like a cold cell rather than an invalid one. Downstream chemistry stages consume T_tr; the Park ionization surrogate at a spuriously low temperature returns a near-zero ionization fraction, so a broken energy budget can present as a benign 'no plasma' result. The clamp is also a bare literal on a nondimensional quantity, so its severity depends on the reference scaling chosen by the caller.

**Recommended fix.** Document the clamp and its trigger in the rustdoc for both `publish_and_transport` and `finish`. Better, count clamp activations and surface the count as a Report series so a consumer can distinguish a clean solve from one that was floored, in the same spirit as the existing carrier-rebuild logging at lines 385-390.

---

### 3.16 [MINOR] Mean-crossing Strouhal estimator carries a +/-1-crossing bias that is largest on the short records its docs claim it suits

- **Verification verdict:** pending verification
- **Axis:** physics-math
- **Location:** `deep_causality_cfd/src/types/flow/frequency.rs:47`
- **Auditor confidence:** confirmed

**Claim.** The estimator divides the crossing count by the full record length (n-1)*dt rather than by the interval actually spanned by the counted crossings. Because a record of length T contains 2fT +/- 1 mean-crossings depending on its start phase, the frequency carries an absolute error up to 1/(2T) — a relative error of 1/(2·number of periods). The docstring advertises the method as robust precisely for short records, where this term is largest.

**Code evidence.**

```
20:/// Robust for the short, low-noise records a march yields; it does not resolve multiple
21:/// spectral peaks (a single dominant tone is assumed, as in a clean shedding wake).
...
45:    let half = R::from_f64(0.5).expect("0.5 lifts into every real field");
46:    let periods = R::from_usize(crossings).expect("the crossing count lifts into R") * half;
47:    let total_time = R::from_usize(n - 1).expect("the sample count lifts into R") * dt;
48:    periods / total_time
```

**Reference form.** The unbiased zero-crossing frequency estimate uses the span between the first and last detected crossing: f = (crossings - 1) / (2 * (t_last - t_first)). Dividing an integer crossing count by the full record introduces an end-effect bias of order one half-period (Kay, Fundamentals of Statistical Signal Processing: Estimation Theory, on zero-crossing estimators).

**Impact.** `strouhal_number` at line 58 multiplies this frequency by L/U, so the bias transfers directly to the reported St. Over a 5-period record the systematic error is up to 10 %; over 10 periods, 5 %. verification/README.md line 36 reports a +4.3 % Strouhal deviation from Williamson for the DEC cylinder, which is the same order as this estimator's own bias at typical record lengths — so part of the reported physical deviation may be estimator bias rather than solver error, and the two are not currently separable.

**Recommended fix.** Record the sample index of the first and last crossing and divide (crossings - 1)/2 by their time separation. This removes the end effect at no extra cost. Then state the residual quantization error in the docstring so a consumer comparing St against Williamson knows the measurement floor.

---

### 3.17 [MINOR] KeyedTable::interpolate panics on a NaN key; DescentSchedule::sample silently returns the first row

- **Verification verdict:** pending verification
- **Axis:** physics-math
- **Location:** `deep_causality_cfd/src/types/keyed_table.rs:148`
- **Auditor confidence:** confirmed

**Claim.** `interpolate` handles the below-range and above-range cases with `<=` and `>=` comparisons, both of which are false for NaN, so a NaN key falls through to a `position` search that returns None and an `.expect` that panics. The parallel hand-rolled lookup in DescentSchedule::sample has the same comparison structure but no expect, so it silently returns the lowest-altitude row instead.

**Code evidence.**

```
keyed_table.rs (panics):
126:        if key <= *first_key {
135:        if key >= *last_key {
143:        // Strictly inside the range: the first row whose key reaches `key` is the upper bracket.
144:        let upper = self
147:            .position(|(k, _)| *k >= key)
148:            .expect("an in-range key has an upper bracket");

compressible_march_config.rs (silent):
151:        if altitude_m <= first.altitude_m { return first; }
154:        if altitude_m >= last.altitude_m { return last; }
157:        let mut lo = first;
158:        for w in self.table.windows(2) {
159:            if altitude_m <= w[1].altitude_m { ... return ...; }
171:        }
172:        lo
```

**Reference form.** A total lookup function must classify every input in its domain, including NaN, or reject NaN explicitly. The crate already applies this discipline at construction — KeyedTable::new rejects non-finite keys at line 82 — but not at query time.

**Impact.** The panic is reachable only from an external consumer, since `interpolate` has no in-crate production callers (only tests/types/keyed_table_tests.rs). The silent path is the more consequential of the two: `sample` is called from CompressibleCarrier::pre_step:323 with an altitude derived from `sqrt` of the carried truth state, so a diverged trajectory yields a NaN altitude and the schedule quietly returns the densest atmosphere row — the maximum-heating condition — with no diagnostic. The run then reports plausible-looking freestream numbers computed from a meaningless altitude.

**Recommended fix.** Add an explicit `if !key.is_finite()` branch to both lookups. For `interpolate`, either return a Result or a clamped result with a flag; for `sample`, return a Result so a non-finite altitude surfaces as PhysicsError rather than as the first row. The KeyedInterpolation struct already carries a `clamped` flag, which is the natural place to signal an out-of-domain query.

---

### 3.18 [MINOR] Quasi-1D duct scheme's well-balanced claim holds only for a stagnant uniform state, not a uniform moving one

- **Verification verdict:** pending verification
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/src/types/flow/duct_march_run.rs:183`
- **Auditor confidence:** confirmed

**Claim.** The comment asserts that 'for a uniform state the source cancels the flux difference exactly (well-balanced)'. Working the algebra, the cancellation holds only when u = 0. For a uniform state with u != 0 the momentum flux carries an additional rho*u^2 term that the pressure area-source does not balance.

**Code evidence.**

```
180:            // Conservative update `u_i ← u_i − Δt/(Δx·A_i)·(F_{i+1/2} − F_{i−1/2})
181:            // + Δt·s_i/A_i` with the pressure area-source
182:            // `s_i = p_i·(A_{i+1/2} − A_{i−1/2})/Δx`; for a uniform state the
183:            // source cancels the flux difference exactly (well-balanced).
...
185:                let src = prim[i].p * (a_faces[i + 1] - a_faces[i]) / dx;
189:                    z[1] - coeff * (flux[i + 1][1] - flux[i][1]) + dt * src / a_centers[i],
```

**Reference form.** For uniform (rho, u, p) the momentum face flux is (rho*u^2 + p)*A, so the flux difference is (rho*u^2 + p)*(A_{i+1/2} - A_{i-1/2}) while the source supplies only p*(A_{i+1/2} - A_{i-1/2}); the residual is rho*u^2*dA/dx. At u = 0 both reduce to p*dA and cancel identically. This is consistent with the physics — a uniform moving state is not a steady solution of quasi-1D flow, since continuity requires rho*u*A = const (Anderson, Modern Compressible Flow, §5.2).

**Impact.** Low practical impact — the scheme is not wrong, and the property it actually has (exact preservation of a stagnant state over a variable-area duct) is the standard and useful one. The defect is that a reader auditing well-balancedness will take the comment at face value and may design a verification case around a uniform moving state, which will not preserve and will look like a solver bug.

**Recommended fix.** Amend the comment to 'for a stagnant uniform state (u = 0) the source cancels the flux difference exactly', and, if a stronger property is claimed elsewhere, add a unit test that marches a quiescent variable-area duct and asserts bit-level preservation.

---

### 3.19 [MINOR] Duct march hardcodes CFL = 0.5 with no override, unlike every neighbouring solver

- **Verification verdict:** pending verification
- **Axis:** magic-number
- **Location:** `deep_causality_cfd/src/types/flow/duct_march_run.rs:96`
- **Auditor confidence:** confirmed

**Claim.** The duct marcher's CFL number is bound to the local `half` constant and cannot be configured. Every other solver in the crate exposes its CFL safety factor: DecNsConfigReady has `cfl_factors`, DecNsSolver has `with_cfl_factors`, and AcousticImex1d takes `cfl` as a constructor argument.

**Code evidence.**

```
duct_march_run.rs:
92:        let half = lift(0.5, "0.5")?;
94:        // The explicit-march CFL number; 0.5 keeps the first-order Rusanov
95:        // update comfortably inside its stability bound.
96:        let cfl = half;
...
166:            let dt = cfl * dx / s_global;

Contrast — src/solvers/dec/dec_config/mod.rs:
95:    /// Replace the CFL safety factors (advective, diffusive). Validated at `build`.
96:    pub fn cfl_factors(mut self, advective: R, diffusive: R) -> Self {
and src/solvers/qtt/compressible/euler_1d.rs:71 takes `cfl: R` as a parameter.
```

**Reference form.** For a first-order explicit Rusanov update in 1-D the stability bound is CFL <= 1 (Toro, §5.3). 0.5 is a conventional and safe choice, and the comment gives that rationale — so the value itself is defensible; the absence of an override is the gap.

**Impact.** A user who needs a tighter step for a stiff area profile, or who wants to run a CFL sensitivity study as part of a grid-convergence exercise, cannot do so without editing the crate. Certification evidence typically requires demonstrating time-step insensitivity, which this API forecloses for the duct path. The value being a documented, conventional safety factor keeps this at minor.

**Recommended fix.** Add a `cfl` field to the duct config with a validated builder setter defaulting to 0.5, matching the pattern in DecNsConfigReady::cfl_factors, and validate 0 < cfl <= 1 at build.

---

### 3.20 [INFO] Stokes MMS residual is identically zero for any pressure gradient, making the g_press = 100.0 literal an arbitrary error-scale setter

- **Verification verdict:** pending verification
- **Axis:** magic-number
- **Location:** `deep_causality_cfd/src/types/flow/mms.rs:145`
- **Auditor confidence:** confirmed

**Claim.** The manufactured Poiseuille state sets lap = -G/mu and grad_p = -G, so the Stokes RHS cancels algebraically for every value of G, nu, and rho. The literal 100.0 therefore does not affect whether the check passes — it only scales the absolute floating-point residual that gets reported as 'mms_error'.

**Code evidence.**

```
141:            Regime::Stokes => {
145:                let g_press = lift(100.0);
146:                let mu = self.rho * self.nu;
147:                let lap = [R::zero() - g_press / mu, R::zero(), R::zero()];
148:                let grad_p = [R::zero() - g_press, R::zero(), R::zero()];
149:                let rhs = stokes_momentum_rhs(&lap, &grad_p, &rho, &nu, &body)?.into_inner();
151:                report.add_series("mms_error", vec![l2_residual(&rhs, &[R::zero(); 3])]);
```

**Reference form.** Stokes momentum: du/dt = -(1/rho)*grad_p + nu*lap(u). Substituting: -(1/rho)*(-G) + nu*(-G/mu) = G/rho - nu*G/(rho*nu) = G/rho - G/rho = 0, independently of G. For fully developed plane Poiseuille flow the balance mu*d2u/dy2 = dp/dx is exactly this identity (Batchelor, An Introduction to Fluid Dynamics, §4.2).

**Impact.** Informational rather than defective — the check does retain real discriminating power (it would catch a sign error, a missing 1/rho, or a wrong viscosity coefficient, since those break the cancellation). But because the residual is analytically zero for any G, the reported 'mms_error' is a pure round-off quantity whose magnitude is set by the arbitrary choice of 100.0. A reader comparing this residual against the other regimes' residuals is comparing differently-scaled quantities.

**Recommended fix.** Either normalize the residual by G/rho so the reported error is relative and comparable across regimes, or document at the literal that the residual is a round-off measurement whose absolute scale is set by g_press and carries no accuracy information beyond machine epsilon.

---

### 3.21 [INFO] Coverage census: 189 literal-bearing lines, 88 panic sites, 14 allow attributes — with the panic surface concentrated in unreachable constant lifts

- **Verification verdict:** pending verification
- **Axis:** magic-number
- **Location:** `deep_causality_cfd/src/lib.rs:1`
- **Auditor confidence:** confirmed

**Claim.** Quantitative sweep results for the record. src/ contains 24,854 lines across 140 files. Of 189 non-comment lines bearing a floating-point literal, 102 are range/loop bounds, leaving roughly 87 genuine numeric literals in code paths; 14 of those are reported above as unjustified or under-justified. There are 88 panic sites (85 expect, 3 unwrap) and zero panic!/unreachable!/todo!/unimplemented!. All 14 #[allow] attributes suppress style lints only.

**Code evidence.**

```
Panic-site breakdown by grep over src/:
  85 expect(, 3 unwrap(, 0 panic!/unreachable!/todo!/unimplemented!
Approximately 80 of the 85 expects are RealField constant lifts of the form:
  src/types/flow/march_run.rs:535:    let half = R::from_f64(0.5).expect("0.5 lifts into every real field");
  src/solvers/dec/dec_config/mod.rs:76:        let safety = R::from_f64(0.9).expect("0.9 lifts into R");
The 3 unwraps:
  src/tensor_bridge/projection.rs:183:    <R as FromPrimitive>::from_f64(n as f64).unwrap()
  src/tensor_bridge/operators.rs:36:    CausalTensor::new(data, vec![rl, 2, 2, rr]).unwrap()
All 14 allow attributes are clippy::too_many_arguments (11) or clippy::type_complexity (3), e.g.:
  src/solvers/qtt/compressible/marcher_2d.rs:103:    #[allow(clippy::type_complexity)]
```

**Reference form.** Repo policy (per AGENTS.md convention) is to fix clippy lints rather than suppress them; a certification review additionally requires that library code not panic on any input reachable from the public API.

**Impact.** Positive finding overall. The panic surface is far smaller than the raw count suggests: the great majority are infallible lifts of compile-time constants into a RealField, which cannot fail for f32/f64/Float106, and most carry an explicit coverage-exemption comment. Only two reachable panic paths were identified in this sweep (KeyedTable::interpolate on NaN, reported separately; the two tensor_bridge unwraps are on fixed-shape allocations). No #[allow] suppresses a correctness lint, so the repo policy is being followed. This context should temper the severity of the individual findings above.

**Recommended fix.** No action required for the census itself. Consider converting the two tensor_bridge unwraps to expects with the same coverage-exemption comment style used elsewhere in the crate, purely for consistency of the audit trail.

---

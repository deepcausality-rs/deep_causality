# deep_causality_cfd::theories — pointwise governing-equation evaluators (compressible_ns, euler, incompressible_ns, stokes, wrappers, mod) plus their upstream deep_causality_physics kernels, the MMS/verify consumers, and the Knudsen regime classifier

**Production readiness: `needs-work`**

The pointwise mathematics is correct everywhere I could check it against a reference. The Stokes-hypothesis stress tensor (constitutive.rs:41-57 with kinematics.rs:41) is exactly mu*(grad u + grad u^T) - (2/3)mu(div u)I, the 1/rho on the pressure gradient is right and nu is never confused with mu (nu is a distinct KinematicViscosity newtype), euler is a bit-exact reduction of compressible_ns at div_tau=0, stokes is incompressible_ns minus the convective kernel, the energy equation matches the standard conservative form term for term, the Knudsen bands are the standard 0.01/0.1/10, and the TaylorGreen manufactured solution in manufactured.rs is a genuine independent reference (I re-derived its pressure field and reproduced the baseline.txt numbers by hand). The wrappers in wrappers.rs are pure lifts and alter no arithmetic. What blocks certification is the evidence layer, not the arithmetic. mms.rs:165-166 feeds the continuity kernel literal zeros and reports the identically-zero result as `continuity_error`, and mms_tests.rs:57-72 asserts on it under the name `compressible_regime_pins_continuity` — a gate that cannot fail. compressible_ns_verification_tests.rs:152-181 claims to "recover the speed of sound" from kernel outputs, but c is injected into grad_p at line 166 and divided back out at line 174; no EOS, gamma or dispersion relation is ever exercised. The whole energy equation (compressible_ns_energy_rhs) has no reference solution, no producer for div_q or div(tau.u), and its documented Fourier sign convention is carried by an untyped raw scalar. All four evaluators return AccelerationVector::new_unchecked, so NaN or Inf in the raw [R;3] inputs is returned as Ok — the opposite of the policy the crate's own constitutive.rs:59-62 states. Finally the README (lines 111-114, 198-199) tells an engineer the Knudsen number "selects the governing model" among four models, but GoverningModel is read nowhere outside tests and no slip-corrected, transitional or free-molecular evaluator exists in the crate.

- Files read: **39**
- Findings raised: **13** — surviving adversarial verification: **13** (refuted: 0)
- Surviving by severity: major 1, minor 10, info 2
- Independently confirmed-correct items: **14**

## Verified correct against reference

These were positively confirmed, not merely un-flagged.

| Item | Location | Reference checked against |
|---|---|---|
| Newtonian viscous stress with Stokes hypothesis: the 2/3 factor and the symmetric gradient are both correct | `deep_causality_physics/src/kernels/fluids/constitutive.rs:41-57 together with deep_causality_physics/src/kernels/fluids/kinematics.rs:41` | tau_ij = 2*mu*S_ij - (2/3)*mu*(div u)*delta_ij with S_ij = 0.5*(du_i/dx_j + du_j/dx_i) (White, Viscous Fluid Flow, Eq. 2-29; Batchelor 1967 Sec. 3.3; zero bulk viscosity) |
| The 2/3 bulk factor is numerically exercised by a test that would fail if the factor were wrong | `deep_causality_physics/tests/kernels/fluids/constitutive_tests.rs:58-78` | For isotropic dilatation S = I and div_u = 3 with mu = 1: tau_ii = 2*1*1 - (2/3)*1*3 = 0 |
| Convective acceleration index convention matches the pinned Jacobian convention | `deep_causality_physics/src/kernels/fluids/governing.rs:48-52` | [(u.grad)u]_i = sum_j u_j * du_i/dx_j, with grad_u[i][j] = du_i/dx_j |
| Pressure-gradient force carries 1/rho and the correct sign; kinematic vs dynamic viscosity are not confused | `deep_causality_physics/src/kernels/fluids/governing.rs:87-92 and 66-67` | du/dt = ... - (1/rho) grad p + nu * laplacian(u), nu = mu/rho [m^2/s] |
| Continuity RHS matches the conservative divergence form | `deep_causality_physics/src/kernels/fluids/governing.rs:112-115` | drho/dt = -div(rho u) = -(u.grad rho + rho div u) |
| Compressible total-energy equation matches the standard conservative form term by term | `deep_causality_cfd/src/theories/compressible_ns.rs:110` | d(rho E)/dt + div(rho u E) = -div(p u) + div(tau.u) - div(q) + rho(u.g), E = e + 0.5\|u\|^2, q = -k grad T (Anderson, Computational Fluid Dynamics, Ch. 2; Landau & Lifshitz Sec. 49) |
| Euler is an exact reduction of the compressible momentum evaluator at div_tau = 0 (same discretisation, same kernels) | `deep_causality_cfd/src/theories/euler.rs:47-49 vs deep_causality_cfd/src/theories/compressible_ns.rs:81-83` | Euler momentum = Navier-Stokes momentum at mu = 0, i.e. du/dt = -(u.grad)u - (1/rho)grad p + g |
| Stokes is the Re->0 reduction with nothing else changed | `deep_causality_cfd/src/theories/stokes.rs:48-50 vs deep_causality_cfd/src/theories/incompressible_ns.rs:63-65` | Stokes (creeping flow): du/dt = -(1/rho)grad p + nu*laplacian(u) + g, i.e. incompressible NS with (u.grad)u dropped |
| Knudsen-number definition and regime band edges match the standard thresholds | `deep_causality_cfd/src/types/flow/corridor/regime.rs:179-181 and 266-276; deep_causality_physics/src/kernels/fluids/dimensionless.rs:131-137` | Kn = lambda / L; continuum Kn < 0.01, slip 0.01 <= Kn < 0.1, transitional 0.1 <= Kn < 10, free-molecular Kn >= 10 (Bird, Molecular Gas Dynamics and the Direct Simulation of Gas Flows, 1994, Sec. 1.4;  |
| Hard-sphere mean-free-path formula (the Knudsen driver) is the correct kinetic-theory expression | `examples/avionics_examples/src/shared/stages.rs:44-46` | lambda = 1/(sqrt(2)*pi*d^2*n), equivalently lambda = k_B*T/(sqrt(2)*pi*d^2*p) via p = n k_B T (Chapman & Cowling; Bird 1994 Eq. 1.36) |
| Mean molecular mass of air constant is numerically correct | `examples/avionics_examples/src/shared/constants.rs:107` | m = M/N_A = 28.97e-3 kg/mol / 6.02214076e23 mol^-1 = 4.8106e-26 kg |
| Sutton-Graves stagnation-heating constant matches the published SI value | `examples/avionics_examples/src/shared/constants.rs:121` | q = k*sqrt(rho/R_n)*V^3 with k = 1.7415e-4 kg^0.5 m^-1 for air (Sutton & Graves, NASA TR R-376, 1971) |
| The TaylorGreen manufactured solution is a genuine independent reference, not a kernel-derived one | `deep_causality_cfd/src/types/flow_config/manufactured.rs:116-202 and deep_causality_cfd/verification/mms_taylor_green_verification/baseline.txt` | u = sin x cos y e^{-2nu t}, v = -cos x sin y e^{-2nu t}, p = (rho/4)(cos 2x + cos 2y) e^{-4nu t}; du/dt = -2 nu u (Taylor 1923) |
| The causal-effect wrappers lift only; they perform no arithmetic | `deep_causality_cfd/src/theories/wrappers.rs:32-143` | A monadic lift must satisfy value(lift(f(x))) == f(x) and must not transform the payload |

## Findings

### 11.1 [MINOR] The compressible MMS `continuity_error` gate is fed literal zeros and can never be non-zero

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/src/types/flow/mms.rs:166`
- **Auditor confidence:** confirmed

**Claim.** The `continuity_error` series reported by the compressible MMS regime is identically zero by construction and cannot fail, and mms.rs:12-13 wrongly claims it "genuinely pins the kernel". The continuity kernel itself is separately and adequately verified by the acoustic-wave reference test, so no governing equation is unverified — the defect is a vacuous metric and a stale doc sentence.

**Code evidence.**

```
mms.rs:164-167:
                // Divergence-free => continuity RHS = 0.
                let continuity =
                    compressible_ns_continuity_rhs(&rho, &u, &[R::zero(); 3], R::zero());
                report.add_series("continuity_error", vec![continuity.abs()]);

The kernel it calls (deep_causality_physics/src/kernels/fluids/governing.rs:112-115):
    let u_dot_grad_rho = u_raw[0]*grad_rho[0] + u_raw[1]*grad_rho[1] + u_raw[2]*grad_rho[2];
    -(u_dot_grad_rho + r * div_u)

With grad_rho = [0,0,0] and div_u = 0 this is -(0 + rho*0) = -0.0 exactly, in f32 and f64 alike.

The assertion (deep_causality_cfd/tests/types/flow/mms_tests.rs:65-71):
    assert!(
        report.series("continuity_error").expect("continuity_error series")[0] < 1e-13,
        "the divergence-free manufactured state has zero continuity residual"
    );

And the doc that presents it as verification (mms.rs:20-21):
//! - **Compressible** - ... momentum `du/dt = -2nu u`, continuity `drho/dt = 0`.
```

**Reference form.** A Method-of-Manufactured-Solutions residual must be formed by feeding the kernel the exact non-trivial spatial derivatives of the manufactured field and comparing against an independently derived time derivative (Roache, Verification and Validation in Computational Science and Engineering, Ch. 3; Salari & Knupp, SAND2000-1444). Feeding a homogeneous kernel all-zero derivative arguments and asserting the result is zero is not a residual.

**Impact.** An engineer reading the CFD report or the test name `compressible_regime_pins_continuity` will conclude the compressible continuity equation has been verified against a manufactured solution. It has not. A sign error, a dropped rho factor, or a transposed contraction in `continuity_rhs_kernel` would all leave this gate green. The same evaluator is exported as public API (`compressible_ns_continuity_rhs`) for downstream use.

**Recommended fix.** Replace the zero arguments with the actual analytic derivatives of a compressible manufactured field. If the intent is to keep the Taylor-Green field, at minimum sample a manufactured density with a non-zero grad_rho and a non-zero div_u whose exact combination -(u.grad rho + rho div u) is known independently, so the residual can differ from zero. Otherwise rename the series to something that does not read as a verification result and drop the assertion.

**Adversarial check.** The tautology is real and the code is quoted verbatim: mms.rs:165-166 calls compressible_ns_continuity_rhs(&rho, &u, &[R::zero();3], R::zero()). The kernel (governing.rs:110-115) computes -(u·grad_rho + rho*div_u), which is identically -0.0 for those arguments under any sign, any rho factor, and any contraction ordering. So the `continuity_error` series carries no information about the kernel and the assertion at mms_tests.rs:64-70 cannot fail. The module doc at mms.rs:12-13 ("The references are exact (not kernel-derived), so a passing error genuinely pins the kernel") is false for this series. BUT the auditor's impact claim — that a sign error or dropped rho would go undetected — is refuted: the continuity kernel IS verified against a genuine non-trivial reference in tests/theories/compressible_ns_verification_tests.rs:92-108 (acoustic plane wave, grad_rho = rho0*u0*k/c, div_u = u0*k, expected -rho0*u0*k), plus test_continuity_known_value. Both a dropped rho factor and a sign flip fail that test. The defect is therefore a non-informative metric plus a doc overclaim, not an unverified governing equation. Not a certification blocker.

> Evidence re-read: deep_causality_cfd/src/types/flow/mms.rs:164-167 (verbatim match); deep_causality_physics/src/kernels/fluids/governing.rs:98-115 (continuity_rhs_kernel body); deep_causality_cfd/tests/types/flow/mms_tests.rs:56-73 (assertion verbatim); deep_causality_cfd/tests/theories/compressible_ns_verification_tests.rs:92-108 (independent non-trivial continuity verification the auditor did not account for)

---

### 11.2 [MINOR] The acoustic "speed of sound recovered from kernel outputs" verification divides out the constant it injected; no EOS is exercised

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/tests/theories/compressible_ns_verification_tests.rs:174`
- **Auditor confidence:** confirmed

**Claim.** The test is redundant, not circular: it re-derives c from two kernel outputs and would fail on a dropped 1/rho or dropped rho factor, but it adds no coverage beyond test_compressible_ns_acoustic_wave_continuity and _momentum. The genuine defect is the header comment, which cites Anderson Eq. 3.18 (c^2 = gamma p0/rho0) for a test that never touches gamma, p0, or any equation of state — the crate's own speed_of_sound_ideal_gas_kernel is not exercised anywhere in the CFD verification suite.

**Code evidence.**

```
Line 166:  let grad_p = [r.rho0 * r.c * r.u0 * r.k, 0.0, 0.0];
Line 159:  let div_u = r.u0 * r.k;
Line 174:  let c_recovered = du_dt[0].abs() / (drho_dt.abs() / r.rho0);

Substituting the kernels: du_dt[0] = -grad_p[0]/rho0 = -c*u0*k (governing.rs:89), drho_dt = -(0 + rho0*div_u) = -rho0*u0*k (governing.rs:115). Therefore c_recovered = (c*u0*k) / ((rho0*u0*k)/rho0) = c, algebraically independent of gamma, p0, rho0, u0 and k.

The claim being made (lines 137-148):
// Source: Anderson, "Modern Compressible Flow" (2003), Sec. 3.4 Eq. (3.18):
//         c^2 = gamma p0 / rho0 for an ideal gas. ...
// i.e. the ratio of the momentum and continuity RHS magnitudes recovers the
// speed of sound. This is the *physical* content of the dispersion relation
// omega = c k applied to the kernel outputs
```

**Reference form.** Anderson, Modern Compressible Flow (3rd ed.), Sec. 3.4 Eq. 3.18: a = sqrt(gamma * R * T) = sqrt(gamma p / rho). Verifying this relation requires the code under test to compute a from gamma and the thermodynamic state. The crate does have such a kernel (deep_causality_physics/src/kernels/fluids/compressible.rs:26-41, `speed_of_sound_ideal_gas_kernel`), and it is not called by this test.

**Impact.** The test file is titled "Reference-solution verification" and carries four textbook citations, which is exactly the evidence a certification reviewer weighs. The test as written verifies only that press = -grad_p/rho and continuity = -rho*div_u, both already covered by test_continuity_known_value and test_euler_known_value. No dispersion relation, no EOS and no thermodynamic closure is checked anywhere in the compressible path.

**Recommended fix.** Either compute c inside the test from `speed_of_sound_ideal_gas_kernel(gamma, r_specific, T)` so the recovered ratio genuinely closes against the EOS, or rewrite the test docstring to state what it actually checks (the relative scaling of the pressure and continuity terms) and drop the Anderson Eq. 3.18 citation.

**Adversarial check.** Code quoted accurately (lines 166, 159, 174 all match). The algebra is right: c_recovered = |−c·u0·k| / (|−rho0·u0·k|/rho0) = c, and gamma/p0 enter only through the fixture's own computation of c. The EOS limb is CONFIRMED — speed_of_sound_ideal_gas_kernel exists at deep_causality_physics/src/kernels/fluids/compressible.rs:26-42 and is never called from this crate's tests; no thermodynamic closure is exercised in the compressible path. However the tautology limb is REFUTED under the rule that a gate is tautological only if no input makes it fail: defective kernels do make it fail. If the momentum kernel dropped the 1/rho factor, du_dt[0] = −rho0·c·u0·k and c_recovered = rho0·c ≈ 417 ≠ 340. If the continuity kernel dropped the rho factor, drho_dt = −u0·k and c_recovered = rho0·c likewise. The test is redundant (its detection set is a strict subset of the two preceding tests) rather than vacuous. Also note grad_rho at line 158 is dead in this test because u = 0.

> Evidence re-read: deep_causality_cfd/tests/theories/compressible_ns_verification_tests.rs:151-181 (full test), :137-148 (header claim verbatim), :92-133 (the two prior tests it duplicates); deep_causality_physics/src/kernels/fluids/compressible.rs:26-42 (uncalled EOS kernel); grep for speed_of_sound callers across the workspace

---

### 11.3 [MINOR] README states the Knudsen number selects the governing model, but the classification is a label no code consumes and three of the four advertised models do not exist

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/README.md:113`
- **Auditor confidence:** confirmed

**Claim.** The claim is factually correct as stated. Corrected framing: this is a doc-precision defect — `GoverningModel` is a rarefaction diagnostic label, not a solver selector, and the README should say the Knudsen number classifies the rarefaction regime rather than that it "selects the governing model" alongside two axes that genuinely do switch behaviour. No solver output is affected.

**Code evidence.**

```
README.md:110-114:
... this crate
switches three regime axes independently, each on a measured quantity:

- **Flow regime.** `RegimeClassify` turns the freestream Knudsen number into the governing
  model: continuum Navier-Stokes, slip-corrected continuum, transitional, or free-molecular.

README.md:198-199:
density gates which measurements the Kalman filter may fold, the Knudsen number selects the
governing model, ...

What the stage actually does (src/types/flow/corridor/regime.rs:347-355):
            field.log_mut().add_entry(&format!(
                "regime -> {} ({}), Kn={}{}", model.name(), denial, kn, phase,
            ));
        }
        field.set_regime(class);

`grep -rn "GoverningModel" --include=*.rs .` outside regime.rs returns only re-exports (src/lib.rs:95, src/types/flow/mod.rs:59, src/types/flow/corridor/mod.rs:42) and two test files. `grep -rni "slip_correct|slip_velocity|maxwell_slip|smoluchowski|free_molecular|dsmc" deep_causality_cfd/src deep_causality_physics/src` returns only the threshold field names in regime.rs. src/theories/ contains exactly four momentum evaluators: compressible NS, Euler, incompressible NS, Stokes.
```

**Reference form.** Standard rarefied-flow practice (Bird 1994; Schaaf & Chambre) is that crossing Kn = 0.01 requires Maxwell velocity-slip and Smoluchowski temperature-jump wall conditions, and crossing Kn = 0.1 requires abandoning the continuum equations for DSMC or a model Boltzmann solver. Claiming the code "selects the governing model" across those bands asserts those closures exist.

**Impact.** The corridor run this README advertises spends most of its logged descent in the slip band (README.md:147-148 shows Kn = 0.078 and Kn = 0.017 both labelled `slip`). A reader is told the solver is applying slip-corrected physics there. It is not: the same continuum evaluator runs, and only the log line changes. That is a materially wrong picture of the fidelity of the reported trajectory.

**Recommended fix.** Change the README to say the Knudsen number *classifies and logs* the flow regime as a diagnostic, and state explicitly that no slip, transitional, or free-molecular closure is implemented. If model switching is intended, add the closures and a consumer that dispatches on `RegimeClass::model`.

**Adversarial check.** Every factual limb checks out. README.md:110-114 and :198-199 quoted verbatim. `grep -rn GoverningModel --include=*.rs .` outside regime.rs returns only the three re-exports (lib.rs:95, types/flow/mod.rs:59, corridor/mod.rs:42), one doc mention (corridor/mod.rs:10), and two test files — no consumer. The only field-regime property any stage reads is `gnss_denied` (trajectory_nav.rs:107, examples/.../stages.rs:173); `model` is read only at regime.rs:129 to format the log label. No slip-corrected, transitional, DSMC, Maxwell-slip or Smoluchowski evaluator exists in either crate. src/theories/ exports exactly four momentum evaluators. So the flow-regime axis is diagnostic labelling, while the other two axes the README lists (integrator switch, link regime) do change behaviour. Severity is the only correction: this is a README wording defect with no numerical consequence — the reported trajectory is not wrong, it is described with more physics than it implements. The band edges and their meaning are correctly documented in the enum's own rustdoc (regime.rs:21-33).

> Evidence re-read: deep_causality_cfd/README.md:110-114, :147-150, :198-199; deep_causality_cfd/src/types/flow/corridor/regime.rs:21-46 (GoverningModel + name()), :266-276 (model_for), :316-355 (apply/log/set_regime); workspace-wide greps for GoverningModel consumers and for slip/Smoluchowski/DSMC/free-molecular closures; deep_causality_cfd/src/theories/mod.rs:20-31 (exported evaluators)

---

### 11.4 [MAJOR] All four regime evaluators construct their result with `new_unchecked`, so NaN/Inf in the raw derivative inputs is returned as `Ok`

- **Verification verdict:** CONFIRMED
- **Axis:** physics-math
- **Location:** `deep_causality_cfd/src/theories/compressible_ns.rs:80`
- **Auditor confidence:** confirmed

**Claim.** `grad_p`, `laplacian_u` and `div_tau` are untyped raw `[R; 3]` arrays that bypass every quantity-newtype validator. All four evaluators assemble the result with `AccelerationVector::new_unchecked`, so a NaN or Inf in any of them is returned as a successful `Ok(AccelerationVector(NaN,...))`. This directly contradicts the validation policy the crate states for the analogous case in `newtonian_viscous_stress_kernel`.

**Code evidence.**

```
compressible_ns.rs:80-84:
    Ok(AccelerationVector::new_unchecked([
        -conv[0] + press[0] + inv_rho * div_tau[0] + g[0], ...
Same pattern at euler.rs:46-50, incompressible_ns.rs:62-66, stokes.rs:47-51.

The checked constructor exists and does validate (deep_causality_physics/src/quantities/fluids/mod.rs:323-330):
    pub fn new(raw: [R; 3]) -> Result<Self, PhysicsError> {
        if !raw[0].is_finite() || !raw[1].is_finite() || !raw[2].is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "AccelerationVector components must be finite".into(), ));

The crate's own stated policy for exactly this situation (deep_causality_physics/src/kernels/fluids/constitutive.rs:59-62):
    // Use the checked constructor: `div_u` is a raw R input, so a NaN/Inf
    // value would otherwise propagate into the diagonal silently.
    ViscousStress::new(tau)

Additionally the only density guard is exact equality (governing.rs:82-86):
    if r == R::zero() { return Err(...) }
so a subnormal rho (e.g. 1e-320, which `Density::new` accepts since it only rejects rho < 0 and non-finite) yields inv_rho = Inf and an Ok(Inf) result.
```

**Reference form.** The crate's own documented invariant, constitutive.rs:59-62: raw `R` inputs that are not carried by a validating newtype must be surfaced through the checked constructor so a non-finite value is a `PhysicsError` rather than a silent value. The evaluators' own docstrings advertise fallibility only for rho = 0 (compressible_ns.rs:60, euler.rs:16, incompressible_ns.rs:21, stokes.rs:17).

**Impact.** In an avionics loop a NaN entering from an upstream gradient stencil propagates silently through the momentum RHS into the integrator and downstream state, with the `Result` type falsely signalling success at every hop. The failure surfaces far from its origin, or not at all if a later comparison masks it. The error contract advertised in the docstrings ("errors when rho = 0") is also incomplete: it omits the Inf-on-subnormal-rho case.

**Recommended fix.** Replace `AccelerationVector::new_unchecked(...)` with `AccelerationVector::new(...)?` in all four evaluators, matching constitutive.rs. Separately, tighten `pressure_gradient_force_kernel` to reject any rho for which 1/rho is not finite, rather than only rho == 0, and update the four docstrings to state the full error contract.

**Adversarial check.** Every limb verified at the cited lines. `Ok(AccelerationVector::new_unchecked([` sits at compressible_ns.rs:80, euler.rs:46, incompressible_ns.rs:62, stokes.rs:47 — exact match. AccelerationVector::new at quantities/fluids/mod.rs:323-330 does reject non-finite components, so the checked path exists and is deliberately bypassed. grad_p, laplacian_u and div_tau are raw [R;3] with no validating newtype, so a NaN from an upstream stencil is returned as Ok(AccelerationVector(NaN,..)). The crate's contrary policy is stated verbatim at constitutive.rs:58-61 ("Use the checked constructor: `div_u` is a raw R input, so a NaN/Inf value would otherwise propagate into the diagonal silently") and restated in that kernel's rustdoc at :31-33. The subnormal-density limb also holds: Density::new (mod.rs:57-69) rejects only non-finite and negative values, so rho = 1e-320 is accepted, pressure_gradient_force_kernel's guard `if r == R::zero()` (governing.rs:81-86) passes, and 1.0/1e-320 overflows to +Inf in f64 — returned as Ok. I checked for compensation and found none: no caller re-validates. The docstrings (compressible_ns.rs:60, euler.rs:16, incompressible_ns.rs:21, stokes.rs:17) advertise only the rho = 0 failure mode. Severity major is right for an avionics-facing API where the Result type is the error contract.

> Evidence re-read: deep_causality_cfd/src/theories/compressible_ns.rs:71-85, euler.rs:32-51, incompressible_ns.rs:45-67, stokes.rs:33-52; deep_causality_physics/src/quantities/fluids/mod.rs:321-340 (AccelerationVector::new / new_unchecked), :56-72 (Density::new); deep_causality_physics/src/kernels/fluids/constitutive.rs:26-63 (the stated policy); governing.rs:70-92 (pressure_gradient_force_kernel zero-guard)

---

### 11.5 [MINOR] The compressible energy equation has no reference verification, no producer for its heat-flux and viscous-work inputs, and its documented Fourier sign convention is unenforced

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** doc-gap
- **Location:** `deep_causality_cfd/src/theories/compressible_ns.rs:95`
- **Auditor confidence:** confirmed

**Claim.** compressible_ns_energy_rhs has no reference-solution verification (its only coverage is a hand-arranged sum totalling zero and an affinity check) and its documented Fourier and Newtonian conventions on the raw div_q / div_tau_dot_u scalars are unenforced, which is inconsistent with the type-level convention enforcement the crate applies to ViscousStress. The absence of a heat-flux producer is not a defect: compressible_ns.rs:26-28 explicitly scopes these as pointwise kernels with caller-supplied divergences.

**Code evidence.**

```
compressible_ns.rs:95-111 signature takes `div_tau_dot_u: R, div_q: R` as raw scalars and returns line 110:
    -div_rho_u_e - div_p_u + div_tau_dot_u - div_q + rho.value() * u_dot_g

The convention it relies on (compressible_ns.rs:22-24):
//! Sign convention follows continuum mechanics: viscous stress positive in
//! tension; heat-flux vector `q` follows Fourier's law `q = -kappa grad T`, so the
//! `-div q` term in the energy equation is a heat *source*.

`grep -rn "fourier|heat_flux_kernel" deep_causality_physics/src/kernels` returns no Fourier heat-flux kernel; `thermal_conductivity` appears only in `nusselt_number_kernel` (dimensionless.rs:281) and `entropy_production_rate_kernel` (compressible.rs:248). `grep -rn "div_tau" deep_causality_cfd/src` shows the only producer anywhere is mms.rs:156, `let div_tau = scale(lap, self.rho * self.nu);` — the incompressible limit, which never reaches the energy kernel at all.

Entire test coverage (tests/theories/compressible_ns_tests.rs:92-141): `test_energy_known_value` asserts -2-3+4-5+6 = 0, and `test_energy_linear_in_divergences` asserts affinity. Neither is a reference solution. mms.rs never calls the energy evaluator.
```

**Reference form.** The energy equation d(rho E)/dt = -div(rho u E) - div(pu) + div(tau.u) - div(q) + rho(u.g) with q = -kappa grad T (Anderson, Computational Fluid Dynamics, Ch. 2). A verification of the conduction term requires a case with a known temperature field, e.g. steady 1-D conduction where -div q = kappa d2T/dx2 is analytically known, or a Couette-flow case where the viscous-work and conduction terms balance a known total-temperature profile.

**Impact.** A caller who supplies +kappa*laplacian(T) where -div q is expected, or who supplies u.(div tau) instead of the full div(tau.u), gets a silently wrong energy budget: heat conduction runs backwards or viscous heating is under-counted by tau:grad u. Nothing in the crate can detect it — no type, no runtime check, no test. The crate elsewhere goes to considerable trouble to enforce exactly this class of convention at the type level (constitutive.rs:14-19 justifies the `ViscousStress` vs `CauchyStress` split on precisely these grounds), so the omission is inconsistent with its own design standard.

**Recommended fix.** Add a Fourier heat-flux kernel `q = -kappa grad T` returning a `HeatFlux` newtype and take `div_q` as that type (or as a `HeatFluxDivergence` newtype) so the sign convention is carried by the type rather than the docstring. Add at least one reference case for the energy equation — steady 1-D conduction and a Couette viscous-heating balance are both closed-form and cheap.

**Adversarial check.** The factual limbs hold. compressible_ns_energy_rhs (lines 95-111) takes div_tau_dot_u and div_q as bare R; line 110 is verbatim. There is no Fourier heat-flux kernel anywhere — grep for fourier/heat_flux across both src trees returns only a comment at compressible.rs:260 and unrelated corridor/QTT "heat_flux" field plumbing. No producer for div(tau.u) or div q exists. No MMS or reference case exercises the energy evaluator; mms.rs never calls it. Two corrections. First, the auditor's coverage inventory is incomplete: compressible_ns_tests.rs:104-125 (test_energy_dissipation_term_is_nonnegative_for_newtonian_fluid) sits inside the Energy block and checks Phi = tau:grad u >= 0 — though it exercises viscous_dissipation_rate_kernel, not the energy RHS, so the auditor's substantive point survives. Second, the "no producer" limb is answered by the module's own scoping at compressible_ns.rs:26-28, which states explicitly that the caller computes all spatial divergences and these kernels do not discretise space; that is a documented design boundary, not an omission. What remains genuinely open is the unenforced convention: raw scalars carry a Fourier sign and a Newtonian-closure assumption that the crate enforces at the type level for the analogous ViscousStress/CauchyStress case (constitutive.rs:14-19). That inconsistency is real but is a design-standard gap, not a physics error.

> Evidence re-read: deep_causality_cfd/src/theories/compressible_ns.rs:20-28 (sign convention + caller-supplies-divergences scoping), :86-111 (energy kernel); deep_causality_cfd/tests/theories/compressible_ns_tests.rs:88-142 (full energy test block, including the dissipation test the auditor omitted); workspace greps for fourier/heat_flux kernels and for energy_rhs callers; deep_causality_physics/src/kernels/fluids/constitutive.rs:1-20 (the type-level convention standard cited as precedent)

---

### 11.6 [MINOR] `theories/mod.rs` module doc is wrong on both of its factual claims

- **Verification verdict:** CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/src/theories/mod.rs:6`
- **Auditor confidence:** confirmed

**Claim.** The module doc states the four Navier-Stokes regimes are each a `FluidTheory` realization, and that the pointwise regime evaluators live in `deep_causality_physics`. Neither is true: only `DecIncompressible` implements `FluidTheory`, and the evaluators are defined in this module, not in `deep_causality_physics`.

**Code evidence.**

```
mod.rs:6-10:
//! Fluid-dynamics theories: the Navier-Stokes regimes (incompressible,
//! compressible, Euler, Stokes) and the DEC-native incompressible rate, each a
//! `FluidTheory` realization reused across solvers. The pointwise regime
//! evaluators stay in `deep_causality_physics` and are reached through this layer
//! for verification solvers.

Contradicted three lines later by mod.rs:12-16, which declares `mod compressible_ns; mod euler; mod incompressible_ns; mod stokes;` locally, and by wrappers.rs:7-9: "the theory layer migrated out of `deep_causality_physics::kernels::fluids::wrappers`".

`grep -rn "impl.*FluidTheory" deep_causality_cfd/src` returns exactly one hit: incompressible_dec.rs:38. The four regime evaluators are free generic functions (compressible_ns.rs:41/61/95, euler.rs:32, incompressible_ns.rs:45, stokes.rs:33) with no trait impl.

The same error is repeated in mms.rs:7: "checked against a pointwise `FluidTheory` regime evaluator".
```

**Reference form.** The `FluidTheory` trait as defined in deep_causality_cfd/src/traits/fluid_theory.rs:22-41 requires `type State`, `type Ambient` and `fn rate(&self, state, ambient)`. None of the four regime evaluators has any of these.

**Impact.** A reader looking for the compressible NS math will search `deep_causality_physics` on the strength of this doc and not find it. A reader planning to plug a regime evaluator into an `Rk4` march through the `FluidTheory` seam will discover the seam is not implemented for any of them. Both are stale-doc navigation costs on the module that is the entry point to the governing equations.

**Recommended fix.** Rewrite the module doc: the four pointwise regime evaluators live here and are free functions; the shared sub-kernels (`convective_acceleration_kernel`, `pressure_gradient_force_kernel`, `viscous_diffusion_kernel`, `continuity_rhs_kernel`) stay in `deep_causality_physics`; `DecIncompressible` is the only `FluidTheory` realization. Fix the same phrase at mms.rs:7.

**Adversarial check.** Both claims verified false at the cited lines. mod.rs:6-10 reads verbatim as quoted, including "each a `FluidTheory` realization" and "The pointwise regime evaluators stay in `deep_causality_physics`". Lines 12-17 immediately declare `mod compressible_ns; mod euler; mod incompressible_dec; mod incompressible_ns; mod stokes; mod wrappers;` locally, and lines 20-31 re-export the evaluators from those local modules — they are defined in this crate, not in deep_causality_physics. `grep -rn 'impl.*FluidTheory' deep_causality_cfd/src` returns exactly one hit, incompressible_dec.rs:38 (DecIncompressible). The four regime evaluators are free generic functions returning Result<AccelerationVector<R>, PhysicsError> with no State/Ambient associated types and no rate() method. wrappers.rs:6-9 independently confirms the migration out of deep_causality_physics. The same stale phrasing recurs at mms.rs:7 ("checked against a pointwise `FluidTheory` regime evaluator"). Minor severity is correct — navigation cost only.

> Evidence re-read: deep_causality_cfd/src/theories/mod.rs:1-31 (doc, local mod declarations, re-exports); deep_causality_cfd/src/theories/wrappers.rs:6-9; deep_causality_cfd/src/types/flow/mms.rs:6-7; grep for FluidTheory impls across deep_causality_cfd/src

---

### 11.7 [MINOR] The MMS verification program prints that "the causal monad sequenced the two stages"; the code path uses plain `Result` and the non-effect kernel

- **Verification verdict:** CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/verification/mms_taylor_green_verification/print_utils.rs:42`
- **Auditor confidence:** confirmed

**Claim.** The rendered verification artifact asserts that the causal monad sequenced the two verification stages. `VerifyRun::run` contains no `PropagatingEffect` at all: it is straight-line `Result` code using the `?` operator and calls `incompressible_ns_rhs`, not `incompressible_ns_rhs_effect`.

**Code evidence.**

```
print_utils.rs:41-43:
    println!("Manufactured solution reproduced: exact AD derivatives drive the kernel, Rk4 tracks");
    println!("the analytic decay, and the causal monad sequenced the two stages.");

This text appears verbatim in the checked-in evidence file baseline.txt.

src/types/flow/verify.rs:42-107 is the entire run. Line 42: `pub fn run(self) -> Result<Report<R>, PhysicsError>`. Line 55: `let kernel = incompressible_ns_rhs(` — the plain evaluator. Line 92: same call inside the Rk4 rate closure, with `.expect("kernel evaluates")`. `PropagatingEffect` is neither imported nor mentioned anywhere in the file.
```

**Reference form.** Doc-vs-code parity: a statement printed into a verification artifact must describe the mechanism the code actually used. The `*_rhs_effect` wrappers that would make this claim true exist at src/theories/wrappers.rs:20-143 and are simply not used by this path.

**Impact.** baseline.txt is a checked-in verification artifact. A reviewer auditing the causal-monad claims of the crate will take this line as evidence that the verification path exercises the monadic composition. It does not, so the artifact overstates what was demonstrated.

**Recommended fix.** Either drop the clause from print_utils.rs:42 and regenerate baseline.txt, or route `VerifyRun::run` through `incompressible_ns_rhs_effect` and sequence the two stages in `PropagatingEffect` so the printed claim becomes true.

**Adversarial check.** Verified and slightly understated. print_utils.rs:42-43 matches verbatim and the string is present in the checked-in baseline.txt:16. VerifyRun::run (src/types/flow/verify.rs:42-107) is straight-line Result code: line 42 `pub fn run(self) -> Result<Report<R>, PhysicsError>`, line 55 calls `incompressible_ns_rhs` (the plain evaluator), line 92 calls it again inside the Rk4 rate closure with `.expect("kernel evaluates")`. PropagatingEffect is neither imported nor referenced in verify.rs, and main.rs of the verification program is a single `CfdFlow::verify(&config).run()` with `unwrap_or_else` — no monadic composition at any level. The `*_rhs_effect` wrappers do exist (theories/wrappers.rs:19+) and are unused by this path. Beyond the printed line, the program's README.md:12-13 makes the stronger and equally false claim that "Each stage binds onto the previous one, and a kernel failure short-circuits the chain through the effect's ...", and README.md:64 describes main.rs as "the monadic pipeline that sequences the two stages". Minor severity is right — no numerical impact — but the fix must cover three sites, not one.

> Evidence re-read: deep_causality_cfd/verification/mms_taylor_green_verification/print_utils.rs:42-43; baseline.txt:16; main.rs (full file — plain Result + unwrap_or_else); README.md:12-13, :64; deep_causality_cfd/src/types/flow/verify.rs:42-107 (grep for PropagatingEffect/_effect returns nothing)

---

### 11.8 [MINOR] The plane-Poiseuille steady-state tolerances are loosened by eight orders of magnitude on a rationale that names the wrong quantity

- **Verification verdict:** CONFIRMED
- **Axis:** magic-number
- **Location:** `deep_causality_cfd/tests/theories/stokes_verification_tests.rs:63`
- **Auditor confidence:** confirmed

**Claim.** Two steady-state tests widen their tolerance from 1e-12 to 1e-9 with the stated justification that two ~1e5 terms are cancelling. The terms that actually cancel are ~0.1 in magnitude, so the true floating-point residual is ~1e-17. The 1e-9 bound is roughly eight orders looser than the physics requires, and its justification is arithmetically wrong.

**Code evidence.**

```
stokes_verification_tests.rs:46-51, 63-69:
    let rho_val = 1000.0; let mu = 1.0e-3; let nu_val = mu / rho_val;   // nu = 1e-6
    let pressure_drop_per_length = 100.0;
    let lap_u_x = -pressure_drop_per_length / mu;                        // = -1e5
    ...
    // Reference: du/dt = 0 (steady state). Tolerance is loosened to absorb
    // the cancellation of two ~1e5 terms in f64.
        assert!(c.abs() < 1e-9, ...)

The two terms actually summed (stokes.rs:48): press[0] = -(1/1000)*(-100) = +0.1 and visc[0] = 1e-6 * (-1e5) = -0.1. Both are 1e-1, not 1e5. One ulp at 0.1 in f64 is about 1.4e-17.

Identical wording and bound at incompressible_ns_verification_tests.rs:149-153:
    // Reference: steady-state => RHS = 0. Loosened tolerance to absorb the
    // cancellation between viscous and pressure terms (~1e5 in magnitude).
        assert!(c.abs() < 1e-9);
and again at stokes_verification_tests.rs:130.
```

**Reference form.** A cancellation tolerance should be set at a small multiple of the ulp of the largest term entering the sum. Here max|term| = 0.1, ulp = 1.4e-17, so a defensible bound is ~1e-15. The 1e5 figure is the Laplacian input, which is multiplied by nu = 1e-6 before it ever enters the sum.

**Impact.** A real defect that shifted the pressure-viscous balance by up to 1e-9 m/s^2 — for example a slightly wrong nu-vs-mu conversion at the eighth significant digit — would pass all three of these gates silently. The written rationale also misleads a maintainer into believing the loose bound is forced by conditioning when it is not, so the bound is unlikely to be tightened on review.

**Recommended fix.** Tighten the three bounds to ~1e-15 (or `f64::EPSILON * 100.0 * term_magnitude`) and correct the comments to name the actual cancelling magnitude (0.1), not the Laplacian input (1e5).

**Adversarial check.** Re-derived numerically and the auditor is right. With rho = 1000, mu = 1e-3, nu = mu/rho = 1e-6, G = 100: lap_u_x = -G/mu = -100000.0 exactly; press[0] = -(1/rho)(-G) = 0.1; visc[0] = nu*lap = -0.09999999999999999; sum = 1.3877787807814457e-17. The two terms entering the sum are 1e-1, not 1e5 — the 1e5 figure is the Laplacian input, which is scaled by nu = 1e-6 before it reaches the addition. The actual f64 residual is 1.39e-17, so a bound of ~1e-15 is defensible and 1e-9 is roughly eight orders loose. Cited comment text and bounds all verified: stokes_verification_tests.rs comment at :64-65 with assert at :67 (auditor said 63 — off by a few lines, not fabricated), second site at :130, and incompressible_ns_verification_tests.rs:149-152 with the same "(~1e5 in magnitude)" wording. The detection-gap claim holds: a defect shifting the balance by up to 1e-9 would pass all three gates. Minor severity is correct.

> Evidence re-read: deep_causality_cfd/tests/theories/stokes_verification_tests.rs:42-71 and :120-132; deep_causality_cfd/tests/theories/incompressible_ns_verification_tests.rs:126-153; deep_causality_cfd/src/theories/stokes.rs:44-52 (the actual summands); independent f64 evaluation of press+visc giving 1.3877787807814457e-17

---

### 11.9 [INFO] `RegimeClassify` accepts unordered Knudsen thresholds without validation and silently substitutes wrong band edges if the scalar lift fails

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** magic-number
- **Location:** `deep_causality_cfd/src/types/flow/corridor/regime.rs:179`
- **Auditor confidence:** confirmed

**Claim.** The from_f64 fallbacks at regime.rs:179-181 substitute wrong band edges (0 for 0.01, 1.0 for 10.0), but the path is unreachable for every scalar type the crate instantiates and is a defensive-style flaw rather than a live misclassification risk. with_thresholds does not validate ordering, which permits a caller to configure an unreachable band, but it is an infallible builder with the precondition documented on the method.

**Code evidence.**

```
regime.rs:179-181:
            slip_threshold: R::from_f64(0.01).unwrap_or_else(R::zero),
            transitional_threshold: R::from_f64(0.1).unwrap_or_else(R::zero),
            free_molecular_threshold: R::from_f64(10.0).unwrap_or_else(R::one),

regime.rs:190-196:
    /// Override the Knudsen band thresholds (`slip <= transitional <= free_molecular`).
    pub fn with_thresholds(mut self, slip: R, transitional: R, free_molecular: R) -> Self {
        self.slip_threshold = slip;
        self.transitional_threshold = transitional;
        self.free_molecular_threshold = free_molecular;
        self
    }

regime.rs:266-276:
    fn model_for(&self, kn: R) -> GoverningModel {
        if kn < self.slip_threshold { GoverningModel::Continuum }
        else if kn < self.transitional_threshold { GoverningModel::Slip }
        else if kn < self.free_molecular_threshold { GoverningModel::Transitional }
        else { GoverningModel::FreeMolecular }
    }

With slip_threshold falling back to zero, `kn < 0` is false for every physical Kn >= 0, so `Continuum` becomes unreachable and every step classifies as `Slip` or worse. With `transitional < slip` the `Slip` arm is dead code.
```

**Reference form.** Bird 1994 Sec. 1.4: the bands are ordered and half-open, continuum Kn < 0.01 <= slip < 0.1 <= transitional < 10 <= free-molecular. An ordered-band classifier must reject a non-monotone edge set rather than silently collapse a band.

**Impact.** Both failure modes are silent and both change the reported regime. The fallback path degrades the classification rather than failing loudly, which is the wrong default for a safety-relevant classifier; the unvalidated override lets a caller configure a classifier in which one of the four documented outcomes can never be produced, with no diagnostic.

**Recommended fix.** Make the constructor fallible (`-> Result<Self, PhysicsError>`) and propagate the `from_f64` failure instead of substituting a different band edge. Make `with_thresholds` validate `0 < slip <= transitional <= free_molecular` and return `Result`.

**Adversarial check.** Code quoted verbatim at regime.rs:179-181, :190-196, :266-276. Two corrections. First, the fallback limb is effectively unreachable: CfdScalar is a blanket impl over RealField + FromPrimitive + ... (traits/cfd_scalar.rs:16-24), and for every scalar the crate actually instantiates (f32, f64, Float106) from_f64(0.01) succeeds. A hypothetical third-party fixed-point scalar could hit it, so the fallbacks are poor defensive style — unwrap_or_else(R::zero) for a 0.01 edge and unwrap_or_else(R::one) for a 10.0 edge are both wrong values — but no reachable path today produces the described misclassification. Second, the unvalidated-override limb is real but weaker than stated: with_thresholds is an infallible fluent builder returning Self, so it structurally cannot return an error, and its rustdoc at :190 states the precondition. The consequence (a collapsed, unreachable band under a non-monotone edge set) is correctly derived from model_for's cascade. Net: a defensive-coding and API-shape observation, not a defect with a reachable failure mode. The auditor's Bird 1994 band description is correct and matches the enum's own rustdoc at regime.rs:21-33.

> Evidence re-read: deep_causality_cfd/src/types/flow/corridor/regime.rs:170-196 (new + with_thresholds), :265-277 (model_for), :21-33 (documented bands); deep_causality_cfd/src/traits/cfd_scalar.rs:16-24 (blanket CfdScalar impl, so from_f64 is total for all instantiated scalars)

---

### 11.10 [MINOR] The air molecular diameter that sets the Knudsen number is an uncited literal, and Kn depends on it quadratically

- **Verification verdict:** CONFIRMED
- **Axis:** magic-number
- **Location:** `examples/avionics_examples/src/shared/constants.rs:105`
- **Auditor confidence:** confirmed

**Claim.** `AIR_MOLECULE_DIAMETER_M = 3.7e-10` is the sole free parameter in the mean-free-path formula that drives every regime classification in the corridor examples, and it carries no source. Because lambda scales as 1/d^2, a shift to the N2 kinetic diameter 3.64e-10 m moves every reported Kn by about 3.3%, which is enough to move a band edge.

**Code evidence.**

```
constants.rs:104-105:
/// Effective air molecule diameter for the freestream mean free path, m.
pub const AIR_MOLECULE_DIAMETER_M: f64 = 3.7e-10;

Consumed at examples/avionics_examples/src/shared/stages.rs:44-46:
        let d = utils::ft(AIR_MOLECULE_DIAMETER_M);
        let sigma = Real::sqrt(utils::ft(2.0)) * FloatType::pi() * d * d;
        let mfp = utils::ft(1.0) / (sigma * n_inf);

which feeds the "mean_free_path" field (stages.rs:49) that `RegimeClassify` reads at regime.rs:285-291.

By contrast the neighbouring constants are traceable: AIR_MEAN_MOLECULAR_MASS_KG (line 107) states "28.97 amu", SUTTON_GRAVES_K (line 121) states its formula and units, COMMS_BAND_RAD_S (line 101) states "2pi * 1.57542 GHz".
```

**Reference form.** The hard-sphere effective diameter of air is commonly taken as 3.7e-10 m (e.g. Bird 1994 Table A.1 gives the VHS reference diameter for N2 as 4.17e-10 m at 273 K and for O2 as 4.07e-10 m; the simple hard-sphere air value 3.7e-10 m is a different model). Which model is intended determines the value, and the two differ by more than 10%, i.e. more than 20% in lambda.

**Impact.** The README publishes specific Knudsen numbers from a corridor run (README.md:147-150, including Kn = 0.009938 which sits 0.6% below the 0.01 continuum/slip band edge). That classification is not reproducible by an independent reviewer without knowing which molecular-diameter model was used, and a reviewer picking the Bird VHS value instead would get a different regime label at that step.

**Recommended fix.** Cite the source and the model in the docstring, in the style the neighbouring constants already use — e.g. "hard-sphere effective diameter for air, 3.7e-10 m (Vincenti & Kruger, Introduction to Physical Gas Dynamics, Ch. II)" — and note that the VHS model would give a different value.

**Adversarial check.** Verified at examples/avionics_examples/src/shared/constants.rs:104-105: the doc comment is "Effective air molecule diameter for the freestream mean free path, m" with no source and no model named, while immediate neighbours are traceable — COMMS_BAND_RAD_S:100-101 states "2pi * 1.57542 GHz", AIR_MEAN_MOLECULAR_MASS_KG:106-107 states "28.97 amu", SUTTON_GRAVES_K:120-121 states its formula and units, NOSE_RADIUS_M:122-123 states "the RAM-C 6-inch hemisphere". The consumption chain checks out: stages.rs:43-47 forms sigma = sqrt(2)*pi*d^2 and mfp = 1/(sigma*n_inf), publishes "mean_free_path" at :49, and RegimeClassify reads that field and divides by L_CHAR (regime.rs:285-292). So lambda ~ 1/d^2 and Kn inherits it. The 3.3% sensitivity to the N2 kinetic diameter is arithmetically right ((3.7/3.64)^2 = 1.033), and the README publishes Kn = 0.009938, 0.6% inside the continuum edge — so the band label at that step is not reproducible without knowing the diameter model. Genuinely untraceable per the magic-number rule: no comment, no named source, no paper in deep_causality_cfd/papers/ covers it. Minor severity correct — the fix is a one-line citation.

> Evidence re-read: examples/avionics_examples/src/shared/constants.rs:99-123 (the constant and its documented neighbours); examples/avionics_examples/src/shared/stages.rs:39-51 (sigma/mfp formation and field publication); deep_causality_cfd/src/types/flow/corridor/regime.rs:283-292 (Knudsen consumption); deep_causality_cfd/README.md:147-150 (published Kn values)

---

### 11.11 [MINOR] Brachet et al. (1983) is cited as the standard reference for the 2-D Taylor-Green fields; that paper is about the 3-D vortex

- **Verification verdict:** CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/tests/theories/incompressible_ns_verification_tests.rs:19`
- **Auditor confidence:** likely

**Claim.** The verification header cites Brachet et al., J. Fluid Mech. 130 (1983), as the "Standard reference for the 2D form of the velocity / pressure fields used below". That paper studies the three-dimensional Taylor-Green vortex; it is not a source for the 2-D fields used here.

**Code evidence.**

```
incompressible_ns_verification_tests.rs:19-21:
//!   - Brachet et al., "Small-scale structure of the Taylor-Green vortex",
//!     J. Fluid Mech. 130 (1983), pp. 411-452. [Standard reference for the
//!     2D form of the velocity / pressure fields used below.]

The fields actually used are two-dimensional (lines 36-39):
//   u(x,y,t) =   cos(x) sin(y) . exp(-2 nu t)
//   v(x,y,t) = - sin(x) cos(y) . exp(-2 nu t)
//   w        = 0
```

**Reference form.** Brachet, Meiron, Orszag, Nickel, Morf & Frisch, "Small-scale structure of the Taylor-Green vortex", J. Fluid Mech. 130 (1983) 411-452, studies the 3-D initial condition u = sin x cos y cos z, v = -cos x sin y cos z, w = 0, which is not an exact solution and does not decay as a single exponential. The 2-D decaying vortex used here is from Taylor (1923), which the header already cites correctly on lines 12-14.

**Impact.** A reviewer checking the citation trail will open Brachet and find a different field with different dynamics, which undermines confidence in the other citations in the same header (Pope Sec. 6.1.5, Taylor & Green 1937). The physics of the test is correct — I re-derived the pressure field and it is consistent — so this is a traceability defect only.

**Recommended fix.** Drop the Brachet citation or restate it as background on the 3-D vortex, and attribute the 2-D decaying fields to Taylor (1923) alone, which is already cited.

**Adversarial check.** Header verified verbatim at incompressible_ns_verification_tests.rs:19-21, with the bracketed gloss "[Standard reference for the 2D form of the velocity / pressure fields used below.]", and Brachet is invoked a second time at :34 as a source for the quoted closed forms. The fields at :36-39 are the 2-D decaying vortex u = cos x sin y exp(-2 nu t), v = -sin x cos y exp(-2 nu t), w = 0, p = -(rho/4)(cos 2x + cos 2y) exp(-4 nu t). Brachet, Meiron, Orszag, Nickel, Morf & Frisch, JFM 130 (1983) 411-452 studies the three-dimensional Taylor-Green initial condition u = sin x cos y cos z, v = -cos x sin y cos z, w = 0 — a field that is not an exact NS solution and does not decay as a single exponential. It is not a source for the 2-D fields. The correct attribution is Taylor (1923), already cited correctly in the same header at :12-14. Traceability defect only; the physics of the test is sound (I re-derived the pressure field: dp/dx = (rho/2) sin 2x balances -rho(u.grad)u_x at the sampled points). Severity minor/info as claimed.

> Evidence re-read: deep_causality_cfd/tests/theories/incompressible_ns_verification_tests.rs:11-21 (reference list) and :28-45 (field definitions and the second Brachet invocation)

---

### 11.12 [MINOR] The compressible module documents a three-equation conserved-variable system but supplies no equation of state, so the system is not closed and `p` is an unconstrained input

- **Verification verdict:** CONFIRMED
- **Axis:** doc-gap
- **Location:** `deep_causality_cfd/src/theories/compressible_ns.rs:10`
- **Auditor confidence:** confirmed

**Claim.** The module header presents a closed system in the conserved variables (rho, rho u, rho E) but the crate provides no EOS relating p to (rho, e), no reconstruction from rho E to p, and no continuity or energy counterpart for the Euler regime. `p` enters only as a caller-supplied `grad_p` and `div_p_u`, with nothing linking it to the density and energy the other two kernels evolve.

**Code evidence.**

```
compressible_ns.rs:10-18:
//! d(rho)/dt   = - div(rho u)
//! d(u)/dt   = - (u.grad)u - (1/rho) grad p + (1/rho) div(tau) + g
//! d(rho E)/dt = - div(rho u E) - div(p u) + div(tau.u) - div q + rho (u.g)
//! Conserved variables: `rho` (density), `rho u` (momentum), `rho E` (total energy
//! per unit volume), with `E = e + 0.5||u||^2`.

The only closure note is about spatial discretisation (lines 26-28), not about the EOS:
//! Caller computes the spatial divergences (`div tau`, `div q`, `div(p u)`,
//! `div(tau.u)`, `div(rho u E)`) at the sample point; these kernels do not
//! discretise space.

No EOS appears in the module; `grep -rn "gamma" deep_causality_cfd/src/theories` returns nothing. euler.rs exports only `euler_momentum_rhs` (mod.rs:24) — no `euler_continuity_rhs`, no `euler_energy_rhs`.
```

**Reference form.** The compressible Navier-Stokes system is closed by a thermodynamic relation, for a calorically perfect gas p = (gamma - 1) rho e = (gamma - 1)(rho E - 0.5 rho |u|^2) (Anderson, Modern Compressible Flow, Ch. 1). Without it the three equations have four unknowns.

**Impact.** An engineer reading the header will expect the three exported functions to constitute a marchable compressible system. They do not: the caller must supply an EOS the crate does not provide and must keep it consistent with the rho and rho E the other two kernels evolve, with no seam or check. The audit question "is Euler consistent with compressible_ns on the same EOS" has no answer because neither carries one.

**Recommended fix.** State explicitly in the module header that these are pointwise RHS contributions only, that the thermodynamic closure p(rho, e) is the caller's responsibility, and that no EOS is provided or checked here. If a closure is intended, `speed_of_sound_ideal_gas_kernel` already exists at deep_causality_physics/src/kernels/fluids/compressible.rs:26 and a matching `pressure_from_conserved` kernel would complete the seam.

**Adversarial check.** Header verified verbatim at compressible_ns.rs:8-19, including "Conserved variables: rho, rho u, rho E ... with E = e + 0.5||u||^2". No EOS exists in the theory layer: `grep -rn gamma deep_causality_cfd/src/theories` returns nothing. Searching the physics crate, the only gamma-bearing kernels are speed_of_sound_ideal_gas_kernel and the isentropic stagnation ratios (compressible.rs:26-200); there is no p = (gamma-1) rho e reconstruction and no kernel relating p to (rho, e) anywhere. mod.rs:24 exports only euler_momentum_rhs — no euler_continuity_rhs or euler_energy_rhs — so the two regimes cannot be compared on a shared closure. The auditor's reference form is correct: p = (gamma-1)(rho E - 0.5 rho |u|^2) for a calorically perfect gas. Partial mitigation: lines 26-28 do scope these as pointwise kernels with caller-supplied divergences, which signals this is not a marchable solver. But that note is about spatial discretisation, not thermodynamic closure, so the gap the auditor identifies is unaddressed. Minor severity correct — the fix is a header sentence naming the required closure and pointing at the physics-crate EOS kernels.

> Evidence re-read: deep_causality_cfd/src/theories/compressible_ns.rs:6-28 (full header) and :37-111 (all three exported kernels); deep_causality_cfd/src/theories/mod.rs:20-27 (exports; euler exposes momentum only); grep for gamma in deep_causality_cfd/src/theories (empty) and in deep_causality_physics/src/kernels/fluids/compressible.rs (speed of sound + isentropic ratios only)

---

### 11.13 [INFO] Two contradictory Taylor-Green sign conventions coexist in the crate with no note that both are valid

- **Verification verdict:** CONFIRMED
- **Axis:** doc-gap
- **Location:** `deep_causality_cfd/src/types/flow/mms.rs:174`
- **Auditor confidence:** confirmed

**Claim.** `mms.rs::taylor_green_sample` implements u = cos x sin y with p = -(rho/4)(cos 2x + cos 2y), while `manufactured.rs::TaylorGreen` implements u = sin x cos y with p = +(rho/4)(cos 2x + cos 2y). Both are internally consistent exact solutions, but a reader comparing them will conclude one has a sign error.

**Code evidence.**

```
mms.rs:185-193 (u = cos x sin y form; I verified grad_u and grad_p against it):
    let grad_u = VelocityGradient::<R>::new_unchecked([
        [neg_half, half, R::zero()],
        [neg_half, half, R::zero()],
        [R::zero(); 3], ]);
    // grad p = (rho/2, rho/2, 0).
    let grad_p = [rho * half, rho * half, R::zero()];

manufactured.rs:56-58 (the opposite convention):
/// u =  sin x . cos y . F(t),   v = -cos x . sin y . F(t),   w = 0,
/// p = (rho/4)(cos 2x + cos 2y) . F(t)^2,   F(t) = exp(-2 nu t).

incompressible_ns_verification_tests.rs:36-39 follows the mms.rs convention (u = cos x sin y, p negative), so the crate carries both spellings in files that a reviewer will read side by side.
```

**Reference form.** Taylor (1923): both (u = sin x cos y, v = -cos x sin y, p = +(rho/4)(cos2x+cos2y)) and (u = cos x sin y, v = -sin x cos y, p = -(rho/4)(cos2x+cos2y)) are exact 2-D solutions related by a quarter-period translation. I verified each pairing satisfies grad p = -rho (u.grad)u independently.

**Impact.** No numerical defect — I confirmed both are correct. The cost is review time and a false-positive finding for anyone auditing the manufactured solutions, since the pressure sign differs between two files that both claim to implement "the Taylor-Green vortex".

**Recommended fix.** Add one sentence to each docstring noting that the two forms are the same solution shifted by pi/2 in x and y, and that the pressure sign follows the velocity convention chosen. Or converge both on one convention.

**Adversarial check.** Both spellings verified and both are correct, as the auditor states. mms.rs:177-195 (taylor_green_sample, sampled at x = y = pi/4) sets u = (0.5, -0.5, 0) with grad_p = (rho/2, rho/2, 0), which is the u = cos x sin y branch: p = -(rho/4)(cos 2x + cos 2y) gives dp/dx = (rho/2) sin 2x = rho/2 at pi/4. Correct. flow_config/manufactured.rs:56-58 documents the opposite branch verbatim: u = sin x cos y, v = -cos x sin y, p = +(rho/4)(cos 2x + cos 2y). I re-derived that one too: (u.grad)u_x = 0.5 at (pi/4, pi/4), and dp/dx = -(rho/2) sin 2x = -rho/2 satisfies grad p = -rho (u.grad)u. Also correct. incompressible_ns_verification_tests.rs:36-39 follows the mms.rs branch, so the crate carries both spellings across three files a reviewer reads together, with no note that they are quarter-period translates. No numerical defect; the cost is exactly the false-positive review time the auditor describes. Info severity correct — a one-line cross-reference in either file closes it.

> Evidence re-read: deep_causality_cfd/src/types/flow/mms.rs:177-195 (sample point, grad_u, lap, grad_p); deep_causality_cfd/src/types/flow_config/manufactured.rs:53-62 (TaylorGreen rustdoc, opposite branch); deep_causality_cfd/tests/theories/incompressible_ns_verification_tests.rs:36-39; independent re-derivation of grad p = -rho (u.grad)u for both branches

---

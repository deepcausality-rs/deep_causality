# deep_causality_cfd — GNSS-denial navigation and estimation layer (src/navigation/*, types/flow/flight_sensors.rs, types/flow/corridor/trajectory_nav.rs)

**Production readiness: `not-ready`**

The core ESKF linear algebra is correct and I confirmed it line by line: `nav_transition_matrix` reproduces `InsErrorState::propagate` exactly, the −[f×] skew block has the right sign and index placement, and `update_scalar` implements the textbook sequential-scalar Joseph update (S=hPhᵀ+r, K=Phᵀ/S, P←APAᵀ+rKKᵀ) with an explicit re-symmetrization. Against that, the covariance path — which feeds a safety gate, `ThrottleGuidance::corridor_holds` commits ignition on `sqrt(nav_position_variance) ≤ margin_m` (throttle_guidance.rs:321-328) — is wrong in two independently demonstrated ways: Q is added un-scaled by dt (I measured a 16× position-variance difference for the same 10 s horizon at dt=1.0 vs dt=0.1), and `correct_position` collapses the attitude-error covariance (P[8][8] 26.0 → 3.33 in a run I executed) while discarding the corresponding mean correction, because the engine carries no attitude to inject into. Traceability is also weak: the Q diagonal is a hand-set per-step magic vector documented as deriving from "this IMU's grade" with no derivation from any IMU spec, and gravity is pure point-mass while the omitted J2 acceleration at 90 km (≈1.5e-2 m/s²) is the same order as the accelerometer bias (2e-2 m/s²) the whole demonstration is built on. Finally, several load-bearing verification claims are circular: the "coast follows the exact Kepler orbit" test compares `ks_strang_step` at zero force against `KsPropagator::propagate`, which is the identical code path; `is_on_orbit_manifold` is an energy<0 predicate presented as the spec's "Sp(2,R)/KS constraint projection" (no projection is implemented); and the corridor's truth propagator and estimator share the same integrator, gm and aero kick, so the demonstrated GNSS-denial drift can only ever be the configured bias constant. The README additionally advertises a Cowell integrator that does not exist anywhere in `src/`. None of these are unfixable, but an avionics R&D consumer cannot today trust the published nav uncertainty or the advertised integrator capability.

- Files read: **28**
- Findings raised: **18** — surviving adversarial verification: **18** (refuted: 0)
- Surviving by severity: major 4, minor 14
- Independently confirmed-correct items: **7**

## Verified correct against reference

These were positively confirmed, not merely un-flagged.

| Item | Location | Reference checked against |
|---|---|---|
| nav_transition_matrix F reproduces InsErrorState::propagate exactly, including the −[f×] attitude→velocity block sign and index placement | `deep_causality_cfd/src/navigation/eskf.rs:53-80 vs ins_error_state.rs:77-95` | Standard strapdown INS error dynamics (Groves, Principles of GNSS, Inertial and Multisensor Integrated Navigation, 2nd ed. 2013, §14.2): δṗ=δv; δv̇=−[f×]δψ−δb_a; δψ̇=−δb_g; with [f×]=[[0,−fz,fy],[fz,0 |
| Sequential scalar Kalman update: innovation covariance, gain, state update, and Joseph covariance form with re-symmetrization | `deep_causality_cfd/src/navigation/eskf.rs:117-142` | S = hPhᵀ + r (scalar); K = Phᵀ/S; x ← x + K(z − hx); Joseph: P ← (I−Kh)P(I−Kh)ᵀ + K r Kᵀ (Groves 2013 §3.3, numerically-stable covariance update). |
| GNSS measurement is genuinely excluded under gnss_denied and the exclusion does not perturb the propagation | `deep_causality_cfd/src/types/flow/corridor/trajectory_nav.rs:93-116` | Measurement gating must skip the update step only; the predict step must be identical in aided and denied modes. |
| aero_gravity_ratio computes ε = \|a_aero\|/(GM/r²) from real accelerations, with no embedded threshold constants | `deep_causality_cfd/src/navigation/regime_switch.rs:35-51` | ε = a_perturbation / a_central = \|a_aero\| / (μ/r²) — the standard Encke/Cowell perturbation-magnitude indicator. |
| Earth constants used by the navigation layer | `deep_causality_physics/src/constants/earth.rs:9,25,28` | μ_Earth = 3.986004418e14 m³/s² (IERS 2010); J2 = 1.08263e-3 (JGM-3); R_e,equatorial = 6378137 m (WGS-84). |
| Relativistic clock-rate kernel formula and its anchoring to an external reference | `deep_causality_physics/src/kernels/chronometric/forward_clock.rs:73-77` | dτ/dt − 1 = Φ/c² − v²/(2c²) with Φ = −GM/r (1PN, Ashby, Living Reviews in Relativity 6:1, 2003). |
| FlightSensors scalar derivations | `deep_causality_cfd/src/types/flow/flight_sensors.rs:130,150,161-164` | q∞ = ½ρV² with ρ = n·m̄; ideal-gas p = n·k_B·T; radial descent rate ḣ = d\|r\|/dt = (r·v)/\|r\|, positive-downward convention ⇒ −(r·v)/\|r\|. |

## Findings

### 6.1 [MAJOR] ESKF process noise Q is added un-scaled by dt: covariance growth depends on step count, not elapsed time

- **Verification verdict:** CONFIRMED
- **Axis:** physics-math
- **Location:** `deep_causality_cfd/src/navigation/eskf.rs:105`
- **Auditor confidence:** confirmed

**Claim.** Q is added un-scaled by dt, so covariance growth is per-step rather than per-second and the filter is not dt-invariant. The crate documents Q as a per-step diagonal, so this is an API/portability defect rather than a doc-code contradiction; at the shipped DT_FLIGHT and Q magnitudes the numeric effect on the ignition gate is small but unbounded under any re-timing.

**Code evidence.**

```
eskf.rs:100-106 —
    pub fn predict(&mut self, dt: R, specific_force: [R; 3], process_noise_diag: [R; NAV_STATES]) {
        self.state = self.state.propagate(dt, specific_force);
        let f = nav_transition_matrix(dt, specific_force);
        let fp = mat_mul(&f, &self.cov);
        let fpft = mat_mul(&fp, &mat_transpose(&f));
        self.cov = mat_add(&fpft, &diag(&process_noise_diag));
    }
`dt` is consumed only by propagate and nav_transition_matrix; it never touches process_noise_diag.

Measured (probe I compiled and ran against the shipped crate, Q = [1e-4;17], P0 = 0, 10 s horizon):
  PROBE-Q dt=1.0/10steps pos_var=9.561e-1   dt=0.1/100steps pos_var=1.5277475100000004e1   ratio=15.979
A correctly discretised Q would give the same variance for both.
```

**Reference form.** Continuous-to-discrete process-noise discretisation: Q_d = ∫₀^{τ} F(t)·Q_c·F(t)ᵀ dt ≈ Q_c·τ_s to first order (Van Loan, IEEE TAC 23(3), 1978; Groves, *Principles of GNSS, Inertial, and Multisensor Integrated Navigation Systems*, 2nd ed. 2013, §3.2.3 — 'Q_k ≈ Q_c τ_s' for a short propagation interval). A random-walk noise is specified as a PSD and MUST be multiplied by the interval.

**Impact.** nav_position_variance is not an uncertainty in any dt-independent sense. ThrottleGuidance::corridor_holds gates the one-way ignition commit on sigma_m = sqrt(nav_position_variance) ≤ margin_m (throttle_guidance.rs:321-328), so re-timing the coupling loop (DT_FLIGHT = 0.1 s today, constants.rs:54) silently moves a safety commit decision. Any run at a different step size is not comparable to a run at the shipped step size, and no gate detects this.

**Recommended fix.** Scale Q inside predict: self.cov = mat_add(&fpft, &diag(&core::array::from_fn(|i| process_noise_diag[i] * dt))) and redefine the parameter as a continuous-time PSD diagonal (units m²/s, (m/s)²/s, rad²/s, …), or use the Van Loan form. Update the rustdoc on eskf.rs:98-99, nav_sensors.rs:32,55 and the Q_DIAG doc at constants.rs:170-171 to state the units, and rescale Q_DIAG by 1/DT_FLIGHT so the shipped corridor result is preserved.

**Adversarial check.** Code is exactly as quoted. `predict` consumes `dt` only in `self.state.propagate(dt, ...)` and `nav_transition_matrix(dt, ...)`; `diag(&process_noise_diag)` is added with no dt factor, so P grows per *call*, not per elapsed second. The auditor's reference form is correct: for a random-walk/PSD specification Q_d ≈ Q_c·τ_s to first order (Van Loan 1978; Groves 2013 §3.2.3). Two mitigations that do NOT refute the mechanism but do bear on severity: (a) the shipped Q is documented as a per-step quantity, not a PSD — constants.rs:170 'ESKF process-noise diagonal **per coupled step**' — so there is no doc/code contradiction, only an API that has no dt-invariant meaning; (b) the shipped magnitudes are tiny against the prior (position Q 1e-4 m²/step × 300 steps ≈ 3e-2 m² vs P0 position 2500 m²), so the numeric effect on the shipped ignition gate at DT_FLIGHT=0.1 s is small. The defect is real: any consumer re-timing the loop gets an incomparable covariance and a moved safety commit, with nothing detecting it. Downgraded from critical to major on the two mitigations above.

> Evidence re-read: deep_causality_cfd/src/navigation/eskf.rs:100-106 — verbatim match, `self.cov = mat_add(&fpft, &diag(&process_noise_diag));` at line 105, no dt term; examples/avionics_examples/src/shared/constants.rs:170-171 doc reads 'per coupled step'; deep_causality_cfd/src/types/flow/throttle_guidance.rs:320-328 confirms sigma_m = sqrt(variance) gates the one-way commit

---

### 6.2 [MAJOR] correct_position collapses the attitude-error covariance while discarding the attitude-error mean, making the filter permanently overconfident

- **Verification verdict:** CONFIRMED
- **Axis:** physics-math
- **Location:** `deep_causality_cfd/src/navigation/reentry_nav.rs:96`
- **Auditor confidence:** confirmed

**Claim.** The filter estimates and covariance-shrinks an attitude error that is then zeroed without ever being injected, leaving mean and covariance inconsistent and biasing nav_position_variance optimistic. The root cause is that ReentryNavEngine carries no nominal attitude, so no injection is possible — the fix is to model attitude or to stop estimating it, not merely to add an injection line.

**Code evidence.**

```
reentry_nav.rs:96-102 —
        let est = *self.filter.state();
        let dp = est.position_error();
        let dv = est.velocity_error();
        self.position = core::array::from_fn(|i| self.position[i] + dp[i]);
        self.velocity = core::array::from_fn(|i| self.velocity[i] + dv[i]);
        self.filter.reset_navigation_error();
(`est.attitude_error()` is never read; ReentryNavEngine has no attitude field at all — see the struct at reentry_nav.rs:33-42.)

ins_error_state.rs:166-173 — reset_navigation() zeroes position, velocity AND attitude.
eskf.rs:164 — '(The covariance is unchanged — the reset moves the *mean*, not the spread.)'

Measured (probe I compiled and ran; 50 predicts at dt=0.1 under f=[9.81,0,0], then one 5 m x-axis fix at r=0.01):
  PROBE att_before=[0.0, 0.0, 0.0]  P88_before=2.600045424999998e1
  PROBE att_after_correct=[0.0, 0.0, 0.0]  P88_after=3.32501104904165e0
At filter level (before the engine's reset) the same sequence yields an attitude estimate of 0.1048 rad — a real, non-trivial correction that is thrown away.
```

**Reference form.** ESKF feedback-reset discipline: after the measurement update, the *entire* estimated error state must be injected into the nominal (p ← p+δp, v ← v+δv, q ← q ⊗ δq(δψ)), and only then may δx be reset to zero; the reset Jacobian G = I − ½[δψ×] (or I) is applied to P (Sola, 'Quaternion kinematics for the error-state Kalman filter', arXiv:1711.02508, §6; Groves 2013 §14.1 closed-loop correction). Zeroing a mean component that was not injected while keeping the shrunk covariance violates the mean/covariance consistency the filter's optimality rests on.

**Impact.** Every fix ratchets P's attitude block down (26.0 → 3.33, a factor 7.8, on a single fix) while the true attitude error is untouched. Repeated fixes drive the attitude variance toward zero, the −[f×]δψ coupling then stops contributing to velocity/position variance growth, and the published nav_position_variance becomes an unboundedly optimistic understatement of the dead-reckoning error. Because ThrottleGuidance commits ignition on sqrt(nav_position_variance) ≤ margin_m, this biases a one-way safety latch toward committing. The corridor example hides this only because IMU_GYRO_BIAS = [0,0,0] (constants.rs:191), so no true attitude error is ever generated.

**Recommended fix.** Either (a) give ReentryNavEngine a nominal attitude (a DCM/quaternion) and inject δψ into it before the reset, or (b) if the engine deliberately carries no attitude, stop resetting it: change reset_navigation() to zero only position and velocity, and document that δψ is an un-fed-back state. Option (b) is the minimal correct change for the current architecture. Add a regression test asserting that the attitude block of P is not reduced by a position-only fix in the no-attitude configuration.

**Adversarial check.** Verified end to end. `correct_position` (reentry_nav.rs:89-102) folds three scalar position updates; `update_scalar` (eskf.rs:117-142) applies the full 17-vector gain K = P·hᵀ/S to `x`, so every state including attitude (indices 6-8) receives a correction via the P position↔attitude cross-covariance, and the Joseph update shrinks the attitude block. The engine then reads only `position_error()` and `velocity_error()` (lines 97-98) and calls `reset_navigation_error()`, which routes to `InsErrorState::reset_navigation` (ins_error_state.rs:166-173) zeroing position, velocity AND attitude. The attitude mean is discarded; the shrunk covariance is kept. eskf.rs:164 confirms the reset does not touch P. `ReentryNavEngine` (reentry_nav.rs:33-42) has no attitude field, so there is nothing to inject δψ into. The cross-covariance is genuinely non-zero: nav_transition_matrix couples δv←δψ (m[3][7], m[3][8], m[4][6], m[4][8], m[5][6], m[5][7]) and δp←δv (m[0][3..]), so P[0][6..8] becomes populated within two steps. Reference discipline cited (Sola arXiv:1711.02508 §6; Groves §14.1) is stated correctly. Downgraded from critical: the correct remedy is architectural (the nominal has no attitude to inject into at all, so this is a missing model element rather than a forgotten line), and in the shipped corridor the attitude prior is 1e-4 rad² with IMU_GYRO_BIAS = [0,0,0], bounding the realised error.

> Evidence re-read: deep_causality_cfd/src/navigation/reentry_nav.rs:89-102 verbatim; ins_error_state.rs:163-173 reset_navigation zeroes attitude; eskf.rs:117-142 update_scalar corrects all 17 states; eskf.rs:53-80 confirms non-zero δv←δψ coupling generating the cross-covariance; examples/avionics_examples/src/shared/constants.rs:191 IMU_GYRO_BIAS all zero

---

### 6.3 [MAJOR] README advertises runtime Encke↔Cowell integrator switching; no Cowell integrator exists and RegimeSwitch is never called outside tests

- **Verification verdict:** CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/README.md:115`
- **Auditor confidence:** confirmed

**Claim.** The crate README states as a shipped capability that the navigation engine switches integrators at runtime on ε = a_aero/a_grav. ReentryNavEngine::predict unconditionally calls ks_strang_step; there is no direct/Cowell integrator anywhere in src/; RegimeSwitch::select is never invoked by any production or example code path.

**Code evidence.**

```
README.md:115-119 —
- **Dynamics regime.** The navigation engine switches integrators on the force ratio
  `ε = a_aero/a_grav`: while gravity dominates, the trajectory advances on the exact
  KS-conformal core with aero as a between-step kick (Encke); once aero dominates, it switches
  to direct Cowell integration.

reentry_nav.rs:70-76 (the only integrator call) —
        let (r1, v1) = ks_strang_step(self.position, self.velocity, self.gm, dt, |_r, _v| {
            aero_accel
        })?;
No branch, no regime read.

`grep -rn "aero_gravity_ratio|RegimeSwitch|IntegratorRegime" --include=*.rs` over the whole workspace returns hits only in src/navigation/regime_switch.rs, the two `pub use` lines in src/lib.rs:66-67, and deep_causality_cfd/tests/navigation/regime_switch_tests.rs. `grep -rni "cowell"` over all *.rs finds the word only in doc comments and in a test-local RK4 helper (regime_switch_tests.rs:61-90).

The repo's own notes concede this: openspec/notes/archive/cfd-plasma-retropulsion/plasma-retropulsion-descent.md:545 — 'its Gap-3 Encke↔Cowell regime switch (`RegimeSwitch`, built but unwired)'.
```

**Reference form.** Docs-vs-code parity: a README capability claim must be satisfied by a reachable production code path. The corresponding requirement, openspec/specs/trajectory-nav-engine/spec.md:33 'Encke↔Cowell regime-switched integrator', says the engine SHALL 'select the perturbed-conformal ... integrator when ε < ε_switch and direct integration when ε ≥ ε_switch'.

**Impact.** An engineer reading the README will believe the trajectory integrator adapts at peak dynamic pressure — the exact regime where the KS-Strang split's accuracy claim is weakest. It does not. The IntegratorRegime::Direct variant is inert: selecting it changes no computation anywhere. The archived spec is therefore recorded as satisfied while the requirement is unimplemented.

**Recommended fix.** Either wire the switch (add a direct integrator, call aero_gravity_ratio from ReentryNavEngine::predict, and branch on RegimeSwitch::select), or restate README.md:115-119, src/navigation/mod.rs:9-10, src/lib.rs:64 and regime_switch.rs:6-17 as 'a regime *detector* is provided; only the perturbed-conformal integrator is implemented', and re-open the spec requirement as unmet.

**Adversarial check.** README.md:115 is exactly as quoted ('- **Dynamics regime.** The navigation engine switches integrators on the force ratio ... once aero dominates, it switches to direct Cowell integration.'). ReentryNavEngine::predict (reentry_nav.rs:70-72) unconditionally calls ks_strang_step with no branch and no regime read. My own workspace grep for `aero_gravity_ratio|RegimeSwitch|IntegratorRegime` returns hits only in src/navigation/regime_switch.rs, the two pub use lines at src/lib.rs:66-67, and tests/navigation/regime_switch_tests.rs — no production or example caller. A grep for 'cowell' over all *.rs finds it only in doc comments plus the test-local RK4 helper at regime_switch_tests.rs:61. openspec/specs/trajectory-nav-engine/spec.md:33-37 states the requirement as SHALL, and the repo's own archived note (plasma-retropulsion-descent.md:545) concedes 'RegimeSwitch, built but unwired'. IntegratorRegime::Direct selects nothing. The README states this as a shipped capability of the crate, not as a roadmap item.

> Evidence re-read: deep_causality_cfd/README.md:115-119; deep_causality_cfd/src/navigation/reentry_nav.rs:70-72 (sole integrator call, no branch); workspace-wide grep for RegimeSwitch/IntegratorRegime/aero_gravity_ratio and for 'cowell'; openspec/specs/trajectory-nav-engine/spec.md:33-42; openspec/notes/archive/cfd-plasma-retropulsion/plasma-retropulsion-descent.md:545

---

### 6.4 [MINOR] is_on_orbit_manifold is a boundedness predicate that cannot fail for realistic corrections, presented as the spec's KS constraint projection

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/src/navigation/reentry_nav.rs:159`
- **Auditor confidence:** confirmed

**Claim.** is_on_orbit_manifold is a bound-orbit predicate that cannot fail for any correction the corridor can produce, and it is cited (in the test assertion and via the spec) as evidence for a constraint projection. A KS re-lift does occur every predict step inside ks_strang_step, but it is an identity for any bound Cartesian state and therefore enforces nothing on a corrected state.

**Code evidence.**

```
reentry_nav.rs:157-161 —
    /// Whether the current nominal is a bound Kepler orbit — i.e. it lifts back onto the KS constraint
    /// manifold (the B2 projection invariant: a valid re-lift means the bilinear gauge is satisfied).
    pub fn is_on_orbit_manifold(&self) -> bool {
        KsPropagator::from_state(self.position, self.velocity, self.gm).is_ok()
    }

ks_propagator.rs:71-79 (all that from_state can reject) —
        let energy = v2 / two - gm / radius;
        if energy >= R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "State is not a bound orbit (energy >= 0); only ellipses are supported".into(),

reentry_nav_tests.rs:117-134 — the correction under test is `[p[0] + 10.0, p[1], p[2]]` on a state at |r| ≈ 7.35e6 m, then
    assert!(
        eng.is_on_orbit_manifold(),
        "the corrected state is still a valid bound orbit (B2 projection holds)"
    );
```

**Reference form.** The KS bilinear relation u₁u₄ − u₂u₃ + ... = 0 is satisfied by construction of the lift for any 3-vector, so it is not a falsifiable invariant of a corrected state. The spec (openspec/specs/trajectory-nav-engine/spec.md:12) requires 'correct = a 17-state tightly-coupled ESKF ... followed by the Sp(2,R)/KS constraint projection' — i.e. an actual projection operator applied to the corrected state, not a boolean.

**Impact.** The crate presents a gate that cannot fail as evidence for a constraint-projection step that is not implemented. A reviewer checking 'is the KS constraint enforced after correction?' finds a green assertion and stops. In the shipped corridor no correction can plausibly unbind a 7e6 m orbit, so the check has zero discriminating power over the operating envelope.

**Recommended fix.** Either implement the projection the spec calls for and test it against a deliberately off-manifold state, or rename is_on_orbit_manifold to is_bound_orbit, restate its rustdoc as 'the corrected state still has negative specific energy so the KS lift is defined', drop the 'B2 projection holds' phrasing from reentry_nav_tests.rs:133 and reentry_nav.rs:157-158, and mark the spec's projection clause unimplemented.

**Adversarial check.** The core claim holds. reentry_nav.rs:157-161 is verbatim as quoted. KsPropagator::from_state (ks_propagator.rs:59-80) can reject on exactly three conditions: gm ≤ 0, radius ≤ 0, and energy = v²/2 − gm/r ≥ 0. At |r| ≈ 7.35e6 m the specific energy is ≈ −2.4e7 J/kg, so a 10 m position nudge (test at reentry_nav_tests.rs:117-134) cannot flip the sign — the assertion has no discriminating power over the shipped envelope. The auditor's reference reasoning is also right: the KS lift satisfies the bilinear gauge by construction for any 3-vector, so 'gauge satisfied' is not falsifiable. Corrected on one detail: it is not true that 'no projection is performed anywhere' — ks_strang_step (ks_propagator.rs:275) calls KsPropagator::from_state every step, which is the documented re-lift (reentry_nav.rs:11-15). That re-lift is real code; it is simply an identity in Cartesian coordinates for any bound state, so it constrains nothing. Severity reduced: the rustdoc on is_on_orbit_manifold is honest about being a bound-orbit predicate; the overclaim lives in the test's assertion message and in the archived spec's 'Sp(2,R)/KS constraint projection' being recorded as satisfied.

> Evidence re-read: deep_causality_cfd/src/navigation/reentry_nav.rs:157-161; deep_causality_physics/src/kernels/astro/ks_propagator.rs:59-80 (only three rejection paths, energy ≥ 0 the operative one); deep_causality_physics/src/kernels/astro/ks_propagator.rs:267-285 (ks_strang_step re-lifts each step); deep_causality_cfd/tests/navigation/reentry_nav_tests.rs:117-134

---

### 6.5 [MINOR] 'Coast follows the exact Kepler orbit' test compares ks_strang_step against the same KsPropagator it calls internally

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/tests/navigation/reentry_nav_tests.rs:29`
- **Auditor confidence:** confirmed

**Claim.** The engine-level coast test is a semigroup self-consistency check rather than an accuracy check, and its assertion message ('coast follows the exact Kepler orbit') overstates what it verifies. The claimed consequence — that a systematic KS error would go undetected — does not hold: deep_causality_physics/tests/kernels/astro/ks_propagator_tests.rs independently anchors the propagator against a separate two-body implementation and against energy/angular-momentum conservation.

**Code evidence.**

```
reentry_nav_tests.rs:32-50 —
    let reference = KsPropagator::from_state(r0, v0, EARTH_GM).unwrap();
    ...
        eng.predict(dt, [0.0; 3], q).unwrap();
    ...
    let (rp, _vp) = reference.propagate(dt * steps as f64).unwrap();
    assert!(n3(eng.position(), rp) < 1e-3, "coast follows the exact Kepler orbit: {}", ...);

ks_propagator.rs:267-285 — with accel returning [0,0,0]:
    let a0 = accel(position, velocity);            // [0,0,0]
    let v_half = [velocity[0] + a0[0]*half_dt, ...] // == velocity
    let (r1, v1) = KsPropagator::from_state(position, v_half, gm)?.propagate(dt)?;
    let a1 = accel(r1, v1);                        // [0,0,0]
    ... v_out == v1
Both sides of the assertion are KsPropagator::propagate.
```

**Reference form.** An accuracy claim against 'the exact Kepler orbit' requires an independent reference — a closed-form Kepler-equation solve, a high-order independent integrator, or a conserved-quantity check (energy, angular momentum, Laplace-Runge-Lenz vector) computed outside the propagator.

**Impact.** A systematic error in the KS propagator (a wrong ω₀, a bad Newton inversion of t(s), a sign in the lift) would cancel identically on both sides and the test would still pass at 1e-3 m. The module docstring (reentry_nav_tests.rs:6-7) and the spec scenario 'the trajectory matches the analytic Kepler orbit to round-off' rest on this test.

**Recommended fix.** Keep the semigroup test but rename it accordingly, and add a genuinely independent coast check: assert conservation of specific energy, specific angular momentum and the LRL vector across the coast to round-off, and/or compare against the test-local RK4 already written at regime_switch_tests.rs:62-90 with k=0.

**Adversarial check.** The circularity is real and confirmed by reading ks_strang_step (ks_propagator.rs:267-285): with accel ≡ [0,0,0] both half-kicks are additions of zero, so the step is exactly KsPropagator::from_state(r,v,gm).propagate(dt), and the test compares 40 compositions against one call. The test's own inline comment at reentry_nav_tests.rs:30-31 says exactly this ('N steps of dt compose (semigroup) to propagate(N·dt) to round-off'), so the code is candid; the overclaim is in the assertion message and the module docstring. But the IMPACT claim is refuted: the KS propagator IS anchored independently. deep_causality_physics/tests/kernels/astro/ks_propagator_tests.rs:35 `coast_matches_independent_planar_kepler_to_round_off` checks the 3-D KS propagator against a separate TwoBodyPropagator implementation to 1e-6 m over 6000 s; :83 `conserves_energy_and_angular_momentum` checks both invariants to 1e-12 relative; :100 checks Kepler's third law. So a wrong ω₀, a bad Newton inversion of t(s), or a sign in the lift would NOT pass silently — it would fail those tests. The engine-level test is redundant, not load-bearing, and severity drops accordingly.

> Evidence re-read: deep_causality_cfd/tests/navigation/reentry_nav_tests.rs:28-51 (assertion and its own semigroup comment); deep_causality_physics/src/kernels/astro/ks_propagator.rs:267-285 (zero-accel reduction); deep_causality_physics/tests/kernels/astro/ks_propagator_tests.rs:35-52, :83-96, :100 (independent planar-Kepler cross-check, energy/|L| conservation, third law)

---

### 6.6 [MAJOR] The corridor's truth propagator and the navigation estimator share the identical dynamics model, integrator and step, so the demonstrated GNSS-denial drift is an analytic restatement of one configured constant

- **Verification verdict:** CONFIRMED
- **Axis:** tautology-circular
- **Location:** `examples/avionics_examples/src/shared/stages.rs:126`
- **Auditor confidence:** confirmed

**Claim.** TruthGnss advances truth with ks_strang_step(r, v, EARTH_GM, ctx.dt(), |_,_| kick) and TrajectoryNav advances the nominal with ks_strang_step(position, velocity, gm, dt, |_,_| aero_accel) using the same gm, the same dt and the same constant aero vector. The only difference is IMU_ACCEL_BIAS added to the nominal's kick. Therefore the corridor's dead-reckoning error is exactly the propagated initial error plus ½·b·t² — no gravity-model error, atmosphere-model error, integrator error or timing error can appear.

**Code evidence.**

```
examples/avionics_examples/src/shared/stages.rs:123-126 (truth) —
        let kick = field.aero_force().unwrap_or([utils::ft(0.0); 3]);
        let (r1, v1) = ks_strang_step(r, v, utils::ft(EARTH_GM), ctx.dt(), |_r, _v| kick)?;

deep_causality_cfd/src/types/flow/corridor/trajectory_nav.rs:88-93 (nav) —
        let aero = field.aero_force().unwrap_or([R::zero(); 3]);
        let sensed = match &self.imu { Some(imu) => imu.sense_specific_force(aero), None => aero };
        ...engine.predict(ctx.dt(), sensed, self.process_noise)

deep_causality_cfd/src/navigation/reentry_nav.rs:70-72 —
        let (r1, v1) = ks_strang_step(self.position, self.velocity, self.gm, dt, |_r, _v| aero_accel)?;

nav_sensors.rs:43-45 — sense_specific_force adds only accel_bias.
world.rs:236-241 seeds the engine with utils::ft(EARTH_GM) — the same gm TruthGnss uses.
```

**Reference form.** A navigation-performance demonstration must inject at least one error source the estimator does not know about (gravity-model truncation, atmosphere-density error, integrator mismatch, timing skew), otherwise the reported drift is a closed-form function of the configuration rather than a measurement.

**Impact.** The corridor's headline result — INS-only drift through blackout — carries no information beyond IMU_ACCEL_BIAS = [2.0e-2, -1.4e-2, 1.0e-2] (constants.rs:189) and NAV_INIT_ERR (constants.rs:157). It cannot expose an error in the gravity model, in the integrator, or in the aero force, because those are shared byte-for-byte between truth and estimate. An avionics reader will reasonably read it as an end-to-end navigation validation; it is not one.

**Recommended fix.** Perturb the truth model relative to the navigation model: give TruthGnss a J2 term (or a different gm/integrator/substep count) that the nav engine's point-mass KS core does not have, and report the resulting drift decomposition (bias-driven vs model-driven). At minimum, state explicitly in the example README and in trajectory_nav.rs's rustdoc that truth and nav share the dynamics model and that the demonstrated drift is bias-only by construction.

**Adversarial check.** All four code sites verified verbatim. TruthGnss (stages.rs:123-126) calls ks_strang_step(r, v, ft(EARTH_GM), ctx.dt(), |_,_| kick) where kick = field.aero_force(). TrajectoryNav (trajectory_nav.rs:88-93) reads the same field.aero_force(), passes it through sense_specific_force (nav_sensors.rs:43-45, which adds only accel_bias), and hands it to engine.predict(ctx.dt(), ...) which calls the identical ks_strang_step with the identical gm (world.rs:236-241 seeds ft(EARTH_GM)). Same integrator, same μ, same dt, same aero vector. The only divergence sources are NAV_INIT_ERR (constants.rs:157) and IMU_ACCEL_BIAS (constants.rs:189). No gravity-model, atmosphere-model, integrator or timing-skew error can appear by construction. The auditor's reference standard — a navigation demonstration must inject at least one error the estimator does not know about — is the right standard.

> Evidence re-read: examples/avionics_examples/src/shared/stages.rs:123-126; deep_causality_cfd/src/types/flow/corridor/trajectory_nav.rs:86-93; deep_causality_cfd/src/navigation/nav_sensors.rs:42-45; deep_causality_cfd/src/navigation/reentry_nav.rs:70-72; examples/avionics_examples/src/shared/world.rs:230-241

---

### 6.7 [MINOR] Q_DIAG is an untraceable magic vector: not derived from any IMU specification, includes direct position-state noise, and stands in for the omitted gravity-gradient term

- **Verification verdict:** CONFIRMED
- **Axis:** magic-number
- **Location:** `examples/avionics_examples/src/shared/constants.rs:172`
- **Auditor confidence:** confirmed

**Claim.** Q_DIAG's seventeen values carry a stated rationale but no derivation, units, or sensor provenance, and the 'gravity-gradient mismatch' justification names a term F omits entirely. They are example-level tuning literals rather than a library constant claiming physical provenance, which bounds the severity.

**Code evidence.**

```
constants.rs:170-182 —
/// ESKF process-noise diagonal per coupled step, per block: modest position/velocity random
/// walk (integrator and gravity-gradient mismatch), near-constant biases.
pub const Q_DIAG: [f64; 17] = [
    1.0e-4, 1.0e-4, 1.0e-4, // position
    1.0e-4, 1.0e-4, 1.0e-4, // velocity
    1.0e-12, 1.0e-12, 1.0e-12, // attitude
    1.0e-12, 1.0e-12, 1.0e-12, // accelerometer bias
    1.0e-14, 1.0e-14, 1.0e-14, // gyro bias
    1.0e-12, 1.0e-14, // clock bias, drift
];

nav_sensors.rs:55-58 —
    /// The ESKF process-noise diagonal `Q` implied by this IMU's grade.
    pub fn process_noise(&self) -> [R; 17] { self.process_noise_diag }
(ImuModel stores whatever the caller passes; nothing derives Q from accel_bias or gyro_bias.)

eskf.rs:53-80 — nav_transition_matrix has no δv←δp entry: rows 3,4,5 have no column 0,1,2 term, i.e. ∂g/∂r is absent.
```

**Reference form.** Standard INS error dynamics carry a gravity-gradient block: δv̇ = −[f×]δψ − δb_a + G(r)δp with G(r) = −(μ/r³)(I − 3 r̂ r̂ᵀ) (Groves 2013 eq. 14.72; Titterton & Weston, *Strapdown Inertial Navigation Technology*, 2nd ed., §12.3). Process noise for an inertial-grade IMU is specified as velocity random walk (m/s/√h) and angle random walk (°/√h) and enters as Q_c = diag(0, σ_vrw², σ_arw², σ_ba², σ_bg², …)·dt — the position block has no independent driving noise.

**Impact.** None of the seventeen values can be traced to a sensor datasheet, a physical argument, or a stated unit. Since these values (together with P0_DIAG) determine nav_position_variance, and nav_position_variance gates the ignition commit, the safety margin is set by tuned literals. At r ≈ 6.46e6 m the omitted gravity-gradient scale 2μ/r³ ≈ 2.96e-6 s⁻² is real but small over a ~30 s blackout (≈0.07 m for a 50 m position error), so the 1e-4 m²/step position noise is not derived from it either — it is a free parameter.

**Recommended fix.** Add the G(r)δp block to nav_transition_matrix (it is three more entries per row and removes the need for the stand-in). Re-derive Q from a named IMU grade: state the velocity random walk and angle random walk in datasheet units, show the conversion to PSD, and set the position block to zero. Give every entry a unit in the doc comment. Cite the datasheet or the grade class (e.g. 'tactical-grade, 0.05 °/√h ARW').

**Adversarial check.** Verified. constants.rs:170-182 matches verbatim; the rustdoc gives a rationale ('integrator and gravity-gradient mismatch') but no units, no derivation, and no datasheet. Nothing derives Q from IMU_ACCEL_BIAS: ImuModel::new (nav_sensors.rs:33-40) stores whatever the caller passes, and process_noise() (nav_sensors.rs:55-58) returns it while its doc calls it 'the ESKF process-noise diagonal Q implied by this IMU's grade' — nothing is implied by anything. nav_transition_matrix (eskf.rs:53-80) confirmed to have no δv←δp entry (rows 3-5 have no column 0-2 term), so the named gravity-gradient term is genuinely absent from F, and the auditor's own arithmetic (2μ/r³ ≈ 2.96e-6 s⁻², ≈0.07 m over 30 s for a 50 m error) shows the 1e-4 position noise is not sized from it either. The reference INS error dynamics cited (Groves 2013 eq. 14.72; Titterton & Weston §12.3) are stated correctly, including that the position block carries no independent driving noise. Severity reduced to minor: these are example-crate tuning constants in examples/avionics_examples/, and the spec itself scopes the ESKF as 'example-level' (openspec/specs/trajectory-nav-engine/spec.md:12), so they are not a library-surface constant claiming physical provenance.

> Evidence re-read: examples/avionics_examples/src/shared/constants.rs:170-182 verbatim; deep_causality_cfd/src/navigation/nav_sensors.rs:33-40 and :55-58 (ImuModel stores, derives nothing); deep_causality_cfd/src/navigation/eskf.rs:53-80 (no gravity-gradient block in F); openspec/specs/trajectory-nav-engine/spec.md:12 ('The ESKF is example-level')

---

### 6.8 [MINOR] Gravity is point-mass only; the omitted J2 acceleration at reentry altitude is the same order as the accelerometer bias the demonstration is built on

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** physics-math
- **Location:** `deep_causality_cfd/src/navigation/reentry_nav.rs:70`
- **Auditor confidence:** confirmed

**Claim.** The navigation layer models point-mass gravity only, which IS explicitly documented in several places. What is undocumented is that the omitted J2 acceleration at the corridor's 90 km start (≈1.5e-2 m/s², independently re-derived) is the same order as the modelled accelerometer bias (2.0e-2 m/s²) — so an error budget read off this layer would wrongly conclude the bias dominates. The gap is a missing magnitude disclosure, not an undisclosed model class.

**Code evidence.**

```
reentry_nav.rs:70-72 — ks_strang_step(self.position, self.velocity, self.gm, dt, |_r,_v| aero_accel) — the only gravity is KsPropagator's monopole core.
ks_propagator.rs:15-19 — 'In `s` the **bound** motion is the constant 4-D harmonic oscillator u'' + ω₀² u = 0' — a pure two-body model by construction.
regime_switch.rs:31 — '`ε = a_aero / a_grav = |a_aero| / (GM/r²)`' — point-mass.
eskf.rs:53-80 — F contains no gravity block of any kind.
constants.rs:189 — pub const IMU_ACCEL_BIAS: [f64; 3] = [2.0e-2, -1.4e-2, 1.0e-2];
constants.rs:152 — pub const TRUTH_ALTITUDE_0: f64 = 90_000.0;
EARTH_J2 = 1.082_63e-3 exists in the workspace (deep_causality_physics/src/constants/earth.rs:25) but is never used by the navigation layer.
```

**Reference form.** J2 zonal acceleration magnitude: |a_J2| ≈ (3/2)·J2·(μ/r²)·(R_eq/r)². At r = R_e + 90 km ≈ 6.461e6 m: μ/r² = 3.986004418e14/4.174e13 = 9.55 m/s²; (R_eq/r)² = (6378137/6461000)² = 0.9745; so |a_J2| ≈ 1.5·1.08263e-3·9.55·0.9745 ≈ 1.51e-2 m/s². (Vallado, *Fundamentals of Astrodynamics and Applications*, 4th ed., §8.6.)

**Impact.** An engineer sizing an INS error budget from this layer would conclude the accelerometer bias dominates. In real flight the unmodelled J2 gravity error is of the same magnitude and, unlike a bias, is not estimable from position fixes with this F matrix. The omission is disclosed for the *clock* kernel (forward_clock.rs:37-38 explicitly says J2 is omitted and justifies it) but nowhere in src/navigation/. It is invisible in the corridor only because truth uses the same point-mass model (see the shared-dynamics finding).

**Recommended fix.** State the gravity model explicitly in src/navigation/mod.rs and reentry_nav.rs rustdoc, with the numeric size of the omitted J2 term at the corridor's altitude, as forward_clock.rs already does for the clock. If the layer is to be used for anything flight-adjacent, add J2 as a between-step kick alongside the aero kick (ks_strang_step already accepts an arbitrary acceleration closure) and add the corresponding gravity-gradient block to F.

**Adversarial check.** The physics is right and I re-derived it independently. |a_J2| ≈ (3/2)·J2·(μ/r²)·(R_eq/r)²; at r = R_eq + 90 km = 6.468e6 m: μ/r² = 3.986004418e14/4.1836e13 = 9.528 m/s²; (6378137/6468137)² = 0.9724; a_J2 ≈ 1.5 × 1.08263e-3 × 9.528 × 0.9724 = 1.50e-2 m/s². IMU_ACCEL_BIAS max component is 2.0e-2 m/s² (constants.rs:189) — same order, confirmed. The code claims are all accurate: reentry_nav.rs:70-72 uses only the KS monopole core, eskf.rs:53-80 has no gravity block, and EARTH_J2 exists in deep_causality_physics but is unused by src/navigation/. REFUTED on the disclosure claim: the point-mass model class is disclosed repeatedly and explicitly in the navigation layer — reentry_nav.rs:10-11 names the KsPropagator conformal core, regime_switch.rs:31 writes a_grav = GM/r², ks_propagator.rs:15-19 states the two-body harmonic form, and the spec scenario says 'monopole gravity'. What is undisclosed is the *magnitude comparison*: nowhere is it stated that the omitted J2 term is comparable to the modelled bias. That is a real documentation gap, but it is a narrower one than 'never disclosed', and it is an example-level model-fidelity choice, not a formula error.

> Evidence re-read: deep_causality_cfd/src/navigation/reentry_nav.rs:70-72 and module doc lines 10-11; deep_causality_cfd/src/navigation/eskf.rs:53-80; deep_causality_cfd/src/navigation/regime_switch.rs:31; deep_causality_physics/src/kernels/astro/ks_propagator.rs:15-19; examples/avionics_examples/src/shared/constants.rs:152, :189; deep_causality_physics/src/kernels/.../forward_clock.rs:36-38 (J2 omission justified there); independent re-derivation of a_J2 = 1.50e-2 m/s²

---

### 6.9 [MINOR] Rustdoc claims Q is 'inflated during buffet'; no code path anywhere modifies Q

- **Verification verdict:** CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/src/navigation/eskf.rs:99`
- **Auditor confidence:** confirmed

**Claim.** NavFilter::predict's doc comment describes an adaptive process-noise behaviour that does not exist. Q is a fixed field on TrajectoryNav/ImuModel, set once at construction and passed through unchanged on every step.

**Code evidence.**

```
eskf.rs:98-99 —
    /// Predict one step: propagate the error state and `P ← F·P·Fᵀ + Q` (`Q` = the process-noise
    /// diagonal — IMU random walk + clock noise, inflated during buffet).

`grep -rni "buffet" --include=*.rs deep_causality_cfd/src examples/avionics_examples/src` returns exactly one hit: this doc line.

trajectory_nav.rs:40-45 — process_noise: [R; 17] is a plain struct field; trajectory_nav.rs:93 passes `self.process_noise` verbatim on every step. No regime, dynamic pressure or g-load reads it.
```

**Reference form.** Docs-vs-code parity: a documented adaptive-filtering behaviour must have a code path that implements it.

**Impact.** A reader reasonably concludes the filter adapts Q under high dynamic-pressure buffet — a standard and load-bearing practice for reentry INS. It does not, so a real buffet episode produces an unadapted, over-confident covariance with no indication.

**Recommended fix.** Delete 'inflated during buffet' from eskf.rs:99, or implement regime-conditional Q inflation in TrajectoryNav (the RegimeClass is already on the field at trajectory_nav.rs:107) and add a test.

**Adversarial check.** eskf.rs:98-99 matches verbatim: 'Q = the process-noise diagonal — IMU random walk + clock noise, inflated during buffet'. My own grep -rni 'buffet' over deep_causality_cfd/src and examples/avionics_examples/src returns exactly one hit — that doc line. Q reaches predict as a plain field: TrajectoryNav::process_noise is a struct field (trajectory_nav.rs:40-45), set once in new() or from imu.process_noise() in with_imu(), and passed verbatim as self.process_noise at trajectory_nav.rs:93. No regime, dynamic pressure or g-load value is read on that path. The documented adaptive behaviour has no implementation.

> Evidence re-read: deep_causality_cfd/src/navigation/eskf.rs:98-99 verbatim; workspace grep for 'buffet' (single hit, the doc line); deep_causality_cfd/src/types/flow/corridor/trajectory_nav.rs:39-66 (plain field, no adaptation) and :93 (verbatim pass-through)

---

### 6.10 [MINOR] Spec lists the 17-state ordering as gyro-bias-before-accel-bias; the code uses accel-bias-before-gyro-bias, and the two blocks' priors differ by six orders of magnitude

- **Verification verdict:** CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `openspec/specs/trajectory-nav-engine/spec.md:11`
- **Auditor confidence:** confirmed

**Claim.** The archived requirement states the state ordering as '(position 3, velocity 3, attitude-error 3, gyro bias 3, accel bias 3, clock bias + drift 2)'. InsErrorState::to_array puts accel_bias at indices 9-11 and gyro_bias at 12-14. Anyone building a P0 or Q diagonal from the spec text would swap them.

**Code evidence.**

```
openspec/specs/trajectory-nav-engine/spec.md:11-12 —
correct = a **17-state tightly-coupled ESKF** (position 3, velocity 3, attitude-error 3, gyro bias 3,
accel bias 3, clock bias + drift 2)

ins_error_state.rs:126-148 —
    /// Pack the error state into its 17-vector form (the order the covariance filter uses):
    /// `[pos(3), vel(3), att(3), accel_bias(3), gyro_bias(3), clock_bias, clock_drift]`.
    ... self.accel_bias[0], self.accel_bias[1], self.accel_bias[2],
        self.gyro_bias[0], self.gyro_bias[1], self.gyro_bias[2],

constants.rs:165-169 (follows the code order, not the spec order) —
    1.0e-2, 1.0e-2, 1.0e-2, // accelerometer bias
    1.0e-8, 1.0e-8, 1.0e-8, // gyro bias
```

**Reference form.** Docs-vs-code parity for an index convention. The state ordering is the interface contract for every [R;17] argument in the public API (NavFilter::new cov_diag, predict's process_noise_diag, update_scalar's h, InsErrorState::from_array, and the snapshot format at state_snapshot.rs:335-341).

**Impact.** P0_DIAG's accel-bias prior is 1.0e-2 (m/s²)² and its gyro-bias prior is 1.0e-8; transposing them per the spec text gives the accelerometer bias a 1e-8 prior, which would make the filter refuse to estimate the very error the corridor exists to demonstrate, silently and with no error. Nothing in the code validates the ordering of a caller-supplied [R;17].

**Recommended fix.** Correct the spec text at spec.md:11-12 (and the identical text in the archived change at openspec/changes/archive/2026-07-02-add-plasma-blackout-corridor/specs/trajectory-nav-engine/spec.md) to match the implementation, and repeat the canonical ordering in the rustdoc of NavFilter::new, NavFilter::predict and NavFilter::update_scalar so callers see it at each [R;17] boundary.

**Adversarial check.** Both texts read and quoted. openspec/specs/trajectory-nav-engine/spec.md:11-12: '17-state tightly-coupled ESKF (position 3, velocity 3, attitude-error 3, gyro bias 3, accel bias 3, clock bias + drift 2)'. InsErrorState::to_array (ins_error_state.rs:128-148) packs accel_bias at 9-11 and gyro_bias at 12-14, and from_array (:151-161) unpacks the same way; the method's own doc at :127 states the correct order. constants.rs:165-169 follows the code order with accel-bias prior 1.0e-2 and gyro-bias prior 1.0e-8 — six orders apart, so a transposition is not numerically benign. Nothing validates a caller-supplied [R;17] ordering, and the ordering is the interface contract for NavFilter::new, predict, update_scalar's h, from_array, and the snapshot format. The spec text is the one that is wrong.

> Evidence re-read: openspec/specs/trajectory-nav-engine/spec.md:11-12; deep_causality_cfd/src/navigation/ins_error_state.rs:126-161; examples/avionics_examples/src/shared/constants.rs:163-169; deep_causality_cfd/src/types/flow/state_snapshot.rs:334-342 (snapshot reads the array positionally)

---

### 6.11 [MINOR] covariance_trace() sums states with incompatible physical units and is documented as 'total filter uncertainty'

- **Verification verdict:** CONFIRMED
- **Axis:** physics-math
- **Location:** `deep_causality_cfd/src/navigation/eskf.rs:175`
- **Auditor confidence:** confirmed

**Claim.** The public covariance_trace() adds m² (position), (m/s)² (velocity), rad² (attitude), (m/s²)² (accel bias), (rad/s)² (gyro bias), s² (clock bias) and s⁻²·s² (clock drift) into one scalar, and presents the result as a physical uncertainty measure.

**Code evidence.**

```
eskf.rs:174-177 —
    /// The full covariance trace (total filter uncertainty).
    pub fn covariance_trace(&self) -> R {
        (0..NAV_STATES).fold(R::zero(), |s, i| s + self.cov[i][i])
    }
```

**Reference form.** A trace is only a meaningful scalar uncertainty measure when the states share units, or after normalising by a reference covariance (e.g. tr(P·P₀⁻¹), or the D-optimality scalar det(P)^{1/n}). Summing variances of different physical dimensions is not a physical quantity.

**Impact.** The value is dominated by whichever block happens to have the largest numeric variance in the chosen unit system, and would change if any state were re-expressed (rad → mrad, s → ns). It is exported publicly (src/lib.rs:66) and used as a filter-health readout. Currently exercised only as a monotone diagnostic in eskf_tests.rs:70-96, so no shipped number depends on it — but a consumer treating it as 'total uncertainty' would be misled.

**Recommended fix.** Either restrict the doc to 'a unitless monotone diagnostic, not a physical uncertainty — the states have different units', or replace it with a normalised measure such as tr(P·P₀⁻¹) against the initial covariance.

**Adversarial check.** eskf.rs:174-177 matches verbatim. The sum runs over all 17 diagonal entries, which per the state ordering are m² (0-2), (m/s)² (3-5), rad² (6-8), (m/s²)² (9-11), (rad/s)² (12-14), s² (15) and dimensionless-rate² (16). The doc calls the result 'total filter uncertainty'. The auditor's reference point is correct: a trace is a meaningful scalar only for commensurate states or after normalisation (tr(P·P0⁻¹), or det(P)^{1/n}). The mitigation the auditor already states is accurate and I verified it: no shipped number uses it — production reads position_variance() (the position-block trace, dimensionally sound in m²) via reentry_nav.rs:121-123 and trajectory_nav.rs:134, and throttle_guidance.rs:52 correctly documents that the published scalar is an m² trace compared against a squared margin. covariance_trace is exercised only as a monotone diagnostic in tests. Severity minor as claimed.

> Evidence re-read: deep_causality_cfd/src/navigation/eskf.rs:169-177 (both position_variance and covariance_trace); deep_causality_cfd/src/navigation/ins_error_state.rs:128-148 (unit assignment per index); deep_causality_cfd/src/types/flow/corridor/trajectory_nav.rs:134 and throttle_guidance.rs:52, :320-328 (production reads position_variance only)

---

### 6.12 [MINOR] update_scalar performs an unguarded division by the innovation covariance S

- **Verification verdict:** CONFIRMED
- **Axis:** magic-number
- **Location:** `deep_causality_cfd/src/navigation/eskf.rs:122`
- **Auditor confidence:** confirmed

**Claim.** The Kalman gain is computed as ph[i]/s with no check that s is positive or non-zero. r is entirely caller-supplied (r_var flows from TrajectoryNav's gnss_variance/optical_variance, which are public constructor parameters), so r = 0 with a zero P entry yields 0/0 = NaN, which then propagates into the state and the entire covariance with no error returned.

**Code evidence.**

```
eskf.rs:119-124 —
        let ph = mat_vec(&self.cov, &h); // P·hᵀ
        let s = dot(&h, &ph) + r; // innovation covariance (scalar)
        let innov = z - dot(&h, &x);
        let k: [R; NAV_STATES] = core::array::from_fn(|i| ph[i] / s); // Kalman gain

(the function returns `()`, so there is no channel to report a bad update)

trajectory_nav.rs:51-57 — gnss_variance and optical_variance are unvalidated constructor arguments passed straight to correct_position (reentry_nav.rs:94).
```

**Reference form.** A numerically defensive Kalman update rejects or skips a measurement whose innovation covariance is non-positive: S = hPhᵀ + r must satisfy S > 0 for the gain to exist (Bar-Shalom, Li & Kirubarajan, *Estimation with Applications to Tracking and Navigation*, §5.2).

**Impact.** A single NaN silently poisons all 17 states and the full 17×17 covariance; nav_position_variance becomes NaN, and every downstream comparison (including ThrottleGuidance's `sigma_m > c.margin_m` at throttle_guidance.rs:326) evaluates false, so the guard does not fire and the failure passes as a normal run. Reachable through the public API with a zero or negative measurement variance.

**Recommended fix.** Return Result from update_scalar (or add a debug-checked early return) rejecting s ≤ 0, and validate gnss_variance/optical_variance > 0 in TrajectoryNav::new and r_var > 0 in correct_position.

**Adversarial check.** eskf.rs:119-124 matches verbatim: s = dot(&h,&ph) + r, then k[i] = ph[i]/s with no positivity or non-zero test, and the function returns (), so there is no channel to report a rejected measurement. r is unvalidated all the way from the public surface: TrajectoryNav::new (trajectory_nav.rs:49-57) takes gnss_variance/optical_variance as bare parameters and passes them to correct_position (reentry_nav.rs:94) unchecked. The NaN-propagation path is real and I verified the downstream sink: throttle_guidance.rs:321-328 tests `variance < R::zero()` (false for NaN), then `sigma_m > c.margin_m` (false for NaN), so a NaN variance passes the navigation-quality gate rather than blocking it — the fail-open direction on a one-way ignition latch. Bar-Shalom et al. §5.2 is cited correctly for S > 0. Severity minor is right given it requires a caller to supply r ≤ 0, but the fail-open behaviour is worth noting in the report.

> Evidence re-read: deep_causality_cfd/src/navigation/eskf.rs:117-124; deep_causality_cfd/src/types/flow/corridor/trajectory_nav.rs:49-57 (unvalidated constructor params); deep_causality_cfd/src/navigation/reentry_nav.rs:89-95; deep_causality_cfd/src/types/flow/throttle_guidance.rs:320-328 (NaN passes both comparisons)

---

### 6.13 [MINOR] The carried relativistic clock offset is integrated with a first-order right-endpoint rule and gated by a bound ~370x looser than the true value

- **Verification verdict:** CONFIRMED
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/src/navigation/reentry_nav.rs:78`
- **Auditor confidence:** confirmed

**Claim.** tau_offset accumulates rate(r1, v1)·dt using the post-step state (a right-endpoint rectangle rule, O(dt) local error, not the O(dt²) midpoint the Strang-split nominal is), and the only test on the engine's carried clock admits any magnitude below 1e-4 s while the true accumulation is ~2.7e-7 s.

**Code evidence.**

```
reentry_nav.rs:77-82 —
        // Carried clock: dτ/dt − 1 at the current geometry, integrated on proper time (s ≠ τ).
        let radius = norm(r1);
        let speed = norm(v1);
        let rate = relativistic_clock_drift_rate_kernel(radius, speed, self.gm)?;
        self.tau_offset += rate * dt;

reentry_nav_tests.rs:57-68 (300 steps of dt = 1.0 from r ≈ 7.35e6 m, v ≈ 7.2e3 m/s) —
    assert!(tau.abs() > 0.0, "a relativistic offset accumulates");
    assert!(
        tau.abs() < 1e-4 && tau.abs() < 1e-6 * t.max(1.0) * 1.0e3,
        ...
(the second conjunct is 1e-3·t = 0.3 at t = 300, so the binding bound is 1e-4)
```

**Reference form.** τ − t = ∫₀^T (dτ/dt − 1) dt. At r = 7.348e6 m: (GM/r)/c² = (3.986004418e14/7.348e6)/8.9875e16 = 6.04e-10; v²/(2c²) = 5.225e7/(2·8.9875e16) = 2.91e-10; total rate ≈ −8.95e-10, so over 300 s |τ−t| ≈ 2.68e-7 s. The gate at 1e-4 is ~370× above this.

**Impact.** A sign error, a factor-of-2 error in the kinematic term, or a dropped potential term would all still pass this gate, and the lower bound `tau.abs() > 0.0` cannot fail for any non-zero output. The engine's clock carry is therefore effectively unverified. Mitigating: the underlying kernel *is* anchored externally — forward_clock_tests.rs:19-53 checks the GPS split against Ashby (2003) +45.7/−7.2/+38.5 µs/day — so this is a gate-strength defect, not a formula defect.

**Recommended fix.** Replace the bound with a quantitative one: assert |tau − tau_expected| / |tau_expected| < 1e-3 where tau_expected is computed from the analytic 1PN expression at the mean geometry. Separately, use the midpoint (or averaged endpoint) rate so the clock integration matches the second-order accuracy of the trajectory it rides on.

**Adversarial check.** reentry_nav.rs:77-82 matches verbatim: radius and speed are taken from the post-step (r1, v1), so tau_offset += rate(r1,v1)·dt is a right-endpoint rectangle rule, O(dt) local error, against an O(dt²) Strang-split nominal. The test at reentry_nav_tests.rs:57-68 is as quoted, with the binding conjunct 1e-4 s (the second conjunct evaluates to 0.3 s at t = 300). I re-derived the true magnitude independently: |r| = sqrt(49+1+4)e12 = 7.348e6 m; (GM/r)/c² = 5.425e7/8.98755e16 = 6.04e-10; v² = 52.25e6, v²/(2c²) = 2.91e-10; |rate| ≈ 8.95e-10, so over 300 s |τ−t| ≈ 2.68e-7 s — the gate is ~373× loose. The lower bound `tau.abs() > 0.0` cannot fail for any non-zero output. The auditor's own mitigation is accurate and I confirm the kernel is externally anchored (forward_clock docstring cites Ashby 2003, Living Reviews in Relativity 6:1). This is a gate-strength defect, correctly classified as minor.

> Evidence re-read: deep_causality_cfd/src/navigation/reentry_nav.rs:77-82; deep_causality_cfd/tests/navigation/reentry_nav_tests.rs:53-68 (state_3d seed at :15 gives the r and v I used); deep_causality_physics forward_clock.rs:30-45 (Ashby 2003 reference block); independent re-derivation of |τ−t| ≈ 2.68e-7 s

---

### 6.14 [MINOR] RegimeSwitch documents a precondition it never enforces, and its threshold comparison contradicts the spec at the boundary

- **Verification verdict:** CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/src/navigation/regime_switch.rs:65`
- **Auditor confidence:** confirmed

**Claim.** Two defects in one type: (a) the constructor documents 'Requires exit_direct ≤ enter_direct' but performs no validation, so an inverted band silently produces per-step chatter — the exact failure the hysteresis exists to prevent; (b) the module doc and spec say direct is selected for ε ≥ ε_switch, while select() uses a strict `>`.

**Code evidence.**

```
regime_switch.rs:64-72 —
    /// A switch with the lower (`exit_direct`) and upper (`enter_direct`) g-load thresholds and an initial
    /// regime. Requires `exit_direct ≤ enter_direct` (the hysteresis band).
    pub fn new(exit_direct: R, enter_direct: R, initial: IntegratorRegime) -> Self {
        Self { exit_direct, enter_direct, regime: initial }
    }
(no check, no Result)

regime_switch.rs:12 — '* `ε ≥ ε_switch` — **direct** (Cowell) integration'
regime_switch.rs:78 —
                if epsilon > self.enter_direct {

openspec/specs/trajectory-nav-engine/spec.md:37 — 'select ... direct integration when ε ≥ ε_switch'
```

**Reference form.** A Schmitt trigger requires low-threshold ≤ high-threshold; violating it degenerates to a comparator that flips on every sample in the inverted band. Documented preconditions on a public constructor that cannot fail should be enforced or the type should be made non-constructible in the invalid state.

**Impact.** (a) A caller who transposes the two arguments — plausible, since both are bare R with no newtype — gets a chattering switch with no diagnostic; the hysteresis test at regime_switch_tests.rs:37-59 only covers the correctly-ordered case. (b) At exactly ε = enter_direct the code stays PerturbedConformal while the spec says Direct. Both are currently latent because the type is never used in production (see the Encke↔Cowell overclaim finding).

**Recommended fix.** Make RegimeSwitch::new return Result<Self, PhysicsError> and reject exit_direct > enter_direct, or swap-and-warn. Change the comparison to `>=` to match the spec, or amend regime_switch.rs:12 and spec.md:37 to say 'ε > ε_switch'.

**Adversarial check.** Both sub-claims verified. (a) regime_switch.rs:64-72 matches verbatim — the doc says 'Requires exit_direct ≤ enter_direct (the hysteresis band)' and new() performs no check and returns Self, not Result. With the arguments transposed the Schmitt trigger degenerates: for ε in the inverted band both `epsilon > enter_direct` and `epsilon < exit_direct` hold, so the regime flips every sample — precisely the chatter the hysteresis exists to prevent. Both parameters are bare R with no newtype, so transposition is plausible. (b) The module doc at regime_switch.rs:12 says 'ε ≥ ε_switch — direct (Cowell) integration' and spec.md:37 says 'direct integration when ε ≥ ε_switch', while select() at :78 uses strict `if epsilon > self.enter_direct` — at exactly ε = enter_direct the code stays PerturbedConformal. The hysteresis test (regime_switch_tests.rs:29-33, :38-54) covers only the correctly-ordered case. Both defects are latent because the type has no production caller, consistent with Finding 3.

> Evidence re-read: deep_causality_cfd/src/navigation/regime_switch.rs:64-72 (constructor, no validation), :12 (module doc '≥'), :75-92 (select, strict '>'); openspec/specs/trajectory-nav-engine/spec.md:36-37; deep_causality_cfd/tests/navigation/regime_switch_tests.rs:29, :38

---

### 6.15 [MINOR] The 'overlap band' integrator-agreement test runs at ε < 0.01, two orders below any switch threshold it is cited to justify

- **Verification verdict:** CONFIRMED
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/tests/navigation/regime_switch_tests.rs:93`
- **Auditor confidence:** confirmed

**Claim.** The module doc and the spec justify the switch handover by claiming the two integrators agree 'in the overlap band'. The test asserts the test aero is at ε < 0.01, while the hysteresis band exercised in the sibling tests is [0.1, 0.5]. Agreement is therefore demonstrated only where nobody would ever switch.

**Code evidence.**

```
regime_switch_tests.rs:97-104 —
    let k = 1.0e-6;
    ...
    let eps = aero_gravity_ratio(a_aero, radius, EARTH_GM).unwrap();
    assert!(
        eps < 0.01,
        "the test aero is in the perturbative band: ε = {eps}"
    );

regime_switch_tests.rs:29 and :38 (the band the switch actually uses) —
    let mut sw = RegimeSwitch::new(0.1, 0.5, IntegratorRegime::PerturbedConformal);

regime_switch.rs:16-17 (the claim) — 'In the overlap band the two integrators agree (the KS Strang split is 2nd-order against a direct solve), so the handover is seamless.'
spec.md:40-42 — 'THEN the perturbed-conformal and direct integrators agree within tolerance in an overlap band'
```

**Reference form.** A handover-agreement claim must be verified at the handover point. The relevant test is agreement at ε ∈ [exit_direct, enter_direct], where the Strang split's O(dt²) splitting error scales with the perturbation magnitude and is largest.

**Impact.** The 'seamless handover' claim is unsupported at the ε where a handover would occur. The Strang splitting error grows with ε, so agreement at ε = 0.001 says little about agreement at ε = 0.5 — which is where accuracy matters and where the crate claims the switch earns its keep. Mitigating: the RK4 reference at regime_switch_tests.rs:62-90 is genuinely independent of the KS path, so the test itself is not circular — only mis-scoped.

**Recommended fix.** Sweep ε across the actual band (0.1 → 0.5) and report the KS-Strang vs RK4 disagreement as a function of ε, with the tolerance derived from the expected O(dt²·ε) splitting error rather than a flat 1.0 m. If disagreement at ε = 0.5 exceeds the navigation error budget, that is the justification for the threshold value.

**Adversarial check.** regime_switch_tests.rs:93-104 matches verbatim, including `assert!(eps < 0.01, ...)`. I computed the actual ε: k = 1e-6, |v0| = sqrt(1+42.25+9)e3 = 7.23e3 so |a_aero| = 7.23e-3 m/s²; |r0| = 7.348e6 so a_grav = 3.986e14/5.40e13 = 7.38 m/s²; ε ≈ 9.8e-4 — three orders below the 0.5 enter threshold used in the sibling hysteresis tests (regime_switch_tests.rs:29, :38), and two orders below the 0.1 exit threshold. The module doc claim (regime_switch.rs:16-17, 'In the overlap band the two integrators agree ... so the handover is seamless') and the spec scenario (spec.md:40-42) are both as quoted. The Strang splitting error scales with the perturbation magnitude, so agreement at ε ≈ 1e-3 does not evidence agreement at ε ≈ 0.5. The auditor's mitigation is also correct: rk4_direct (regime_switch_tests.rs:61-90) is a genuinely independent RK4 of ẍ = −μx/r³ − kv, so the test is well-constructed, just mis-scoped.

> Evidence re-read: deep_causality_cfd/tests/navigation/regime_switch_tests.rs:92-115 (test body, k = 1.0e-6), :61-90 (independent RK4), :29 and :38 ([0.1, 0.5] band); deep_causality_cfd/src/navigation/regime_switch.rs:16-17; openspec/specs/trajectory-nav-engine/spec.md:40-42; independent computation of ε ≈ 9.8e-4

---

### 6.16 [MINOR] GNSS 'receiver noise' is a low-discrepancy sequence, not white noise, so it violates the assumption under which the filter's R is defined

- **Verification verdict:** CONFIRMED
- **Axis:** physics-math
- **Location:** `examples/avionics_examples/src/shared/stages.rs:83`
- **Auditor confidence:** confirmed

**Claim.** The corridor's measurement noise is a deterministic golden-ratio equidistributed sequence documented as 'consistent with the filter's R'. Its marginal variance is indeed GNSS_VAR, but a Kalman filter's R is defined for temporally white noise; a golden-ratio sequence is maximally anti-correlated in time.

**Code evidence.**

```
stages.rs:83-100 —
/// Deterministic receiver noise for the published fix: a golden-ratio low-discrepancy sequence
/// per axis, scaled so the per-axis variance is exactly `GNSS_VAR` (uniform on `±σ√3`).
/// Reproducible on every run, with no RNG dependency, consistent with the filter's `R`, ...
pub fn fix_noise(step: usize, draw: usize) -> [FloatType; 3] {
    const PHI: f64 = 0.618_033_988_749_894_9;
    ...
    let amplitude = Real::sqrt(utils::ft(GNSS_VAR) * utils::ft(3.0));
    ...
        let u = x - x.floor();
        amplitude * (utils::ft(2.0) * u - utils::ft(1.0))

constants.rs:180-183 —
/// GNSS fix variance, m²: a precise code-phase receiver at 1 m 1σ. The published fixes carry
/// deterministic receiver noise with exactly this variance, so the filter's `R` matches the
/// sensor.
pub const GNSS_VAR: f64 = 1.0;
```

**Reference form.** The Kalman measurement model requires E[v_k v_jᵀ] = R·δ_kj (white, zero-mean, uncorrelated across epochs). Variance of a uniform on [−a, a] is a²/3, so a = σ√3 does give marginal variance σ² — that part is correct. Serial independence is the part that fails.

**Impact.** Sequential fixes carry systematically anti-correlated errors, so repeated folds average down faster than R predicts — the filter's actual estimation error is better than its covariance claims, i.e. conservative in this instance, but the reported covariance is not the covariance of the realised error process either way. The doc claim 'the filter's R matches the sensor' overstates what holds (marginal variance only). Combined with the Q-scaling and attitude-reset defects, the direction of the net covariance error is not established by any single one of these.

**Recommended fix.** Either restate the doc as 'marginal per-axis variance equals GNSS_VAR; the sequence is deliberately low-discrepancy and is not temporally white, so R is matched in variance but not in whiteness', or switch to a seeded PRNG draw so the noise is genuinely white and R is exactly the sensor model. Keep determinism via a fixed seed rather than a quasi-random sequence.

**Adversarial check.** stages.rs:83-100 matches verbatim, including the doc phrase 'consistent with the filter's R'. The sequence is u = frac((step+1)·stride + phase) mapped to amplitude·(2u−1) with amplitude = sqrt(GNSS_VAR·3) — a golden-ratio equidistributed (Weyl) sequence, deterministic and by construction low-discrepancy, i.e. negatively serially correlated rather than independent. constants.rs:180-183 matches verbatim including 'so the filter's R matches the sensor'. The auditor's reference reasoning is exact and I verified both halves: Var(U[−a,a]) = a²/3, so a = σ√3 does give the correct marginal variance (the auditor concedes this rather than overclaiming), while E[v_k v_jᵀ] = R·δ_kj fails. The auditor also correctly declines to assert a net direction for the covariance error. This is a bounded doc-precision defect; minor is the right severity.

> Evidence re-read: examples/avionics_examples/src/shared/stages.rs:83-100 verbatim; examples/avionics_examples/src/shared/constants.rs:180-183 verbatim; stages.rs:133-138 (the fix is truth + fix_noise, so the sequence is the entire measurement-noise process)

---

### 6.17 [MINOR] The documented t^3 gyro-bias drift law has no path to affect the navigation engine

- **Verification verdict:** CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/src/navigation/ins_error_state.rs:13`
- **Auditor confidence:** confirmed

**Claim.** The module doc calls the t³ gyro-driven position drift 'the load-bearing property — and what the closed-loop navigation gate rests on'. ImuModel::sense_specific_force adds only the accelerometer bias, and ReentryNavEngine carries no attitude, so a gyro bias cannot perturb the engine's nominal trajectory at all. The law is exercised only by InsErrorState::propagate in isolation.

**Code evidence.**

```
ins_error_state.rs:12-15 —
//! (3), plus the two carried clock states (bias, drift). The load-bearing property — and what the
//! closed-loop navigation gate rests on — is the **inertial drift growth law through the blackout**: a
//! constant accelerometer bias grows the position error as `t²`, a constant gyro bias as `t³` (the bias
//! tilts the attitude, which mis-projects the specific force).

nav_sensors.rs:43-45 (the only sensing path) —
    pub fn sense_specific_force(&self, true_specific_force: [R; 3]) -> [R; 3] {
        core::array::from_fn(|i| true_specific_force[i] + self.accel_bias[i])
    }
(gyro_bias is stored and returned by a getter, never applied)

reentry_nav.rs:33-42 — ReentryNavEngine has fields gm, position, velocity, filter, tau_offset, elapsed. No attitude.
constants.rs:190-191 — '/// Gyro bias (unused by the translational corridor; carried for the IMU's grade).' pub const IMU_GYRO_BIAS: [f64; 3] = [0.0, 0.0, 0.0];
The only t³ test operates on InsErrorState directly: ins_error_state_tests.rs:37-49.
```

**Reference form.** Docs-vs-code parity: a property described as load-bearing for the closed-loop gate must be reachable from the closed-loop path. The t³ law itself is standard and correctly implemented in InsErrorState::propagate (Groves 2013 §14.2).

**Impact.** A reader concludes the engine models gyro-driven attitude drift end to end. It does not; the corridor's gyro bias is identically zero and could not matter if it were not. This interacts with the attitude-reset finding: the one mechanism that would generate a real δψ is absent, which is why that defect is currently dormant.

**Recommended fix.** Restate ins_error_state.rs:12-15 to say the t³ law is a property of the error-dynamics model and is not exercised by the current translational engine, and note in nav_sensors.rs:23-24 that ImuModel::gyro_bias is carried but not applied to the sensed specific force. If the law is meant to be load-bearing, give ReentryNavEngine a nominal attitude and rotate the sensed specific force through it.

**Adversarial check.** ins_error_state.rs:11-15 matches verbatim, including 'The load-bearing property — and what the closed-loop navigation gate rests on'. The break in the chain is confirmed at three points: ImuModel::sense_specific_force (nav_sensors.rs:42-45) adds only accel_bias, with gyro_bias stored and exposed by a getter (:52-54) but never applied; ReentryNavEngine (reentry_nav.rs:33-42) has fields gm, position, velocity, filter, tau_offset, elapsed and no attitude; and the corridor seeds the filter with InsErrorState::zero() (world.rs:232), not from_biases, so the filter's own gyro-bias state starts at zero as well, on top of IMU_GYRO_BIAS = [0,0,0] (constants.rs:191, whose own comment concedes 'unused by the translational corridor'). The t³ law is correctly implemented in InsErrorState::propagate (ins_error_state.rs:88-90) and exercised only in isolation. The interaction the auditor notes with Finding 2 is accurate: the absence of any δψ-generating mechanism is why that defect is dormant in the shipped corridor.

> Evidence re-read: deep_causality_cfd/src/navigation/ins_error_state.rs:11-15 and :77-95; deep_causality_cfd/src/navigation/nav_sensors.rs:42-54; deep_causality_cfd/src/navigation/reentry_nav.rs:33-42; examples/avionics_examples/src/shared/world.rs:232-236; examples/avionics_examples/src/shared/constants.rs:190-191

---

### 6.18 [MINOR] NavFilter::restore and ReentryNavEngine::restore accept an arbitrary covariance from the snapshot stream with no symmetry or positive-definiteness validation

- **Verification verdict:** CONFIRMED
- **Axis:** doc-gap
- **Location:** `deep_causality_cfd/src/navigation/eskf.rs:153`
- **Auditor confidence:** confirmed

**Claim.** The snapshot resume path reads a full 17×17 matrix from bytes and installs it as the filter covariance without checking symmetry, positive semi-definiteness, or non-negative diagonal. Neither restore's rustdoc nor the snapshot documentation mentions that the caller owns this invariant.

**Code evidence.**

```
eskf.rs:150-155 —
    /// Rebuild a filter from snapshotted state and covariance: the exact inverse of reading
    /// [`state`](Self::state) and [`covariance`](Self::covariance). Exists for the
    /// state-snapshot resume path.
    pub fn restore(state: InsErrorState<R>, cov: [[R; NAV_STATES]; NAV_STATES]) -> Self {
        Self { state, cov }
    }

state_snapshot.rs:335-342 —
        let mut cov = [[R::zero(); NAV_STATES]; NAV_STATES];
        for row in cov.iter_mut() {
            let r: Vec<R> = read_values(bytes, &mut o, NAV_STATES, "nav")?;
            row.copy_from_slice(&r);
        }
        let filter = NavFilter::restore(crate::navigation::InsErrorState::from_array(state), cov);
(read_values checks only length, not content)
```

**Reference form.** A Kalman covariance must be symmetric positive semi-definite. Every entry point that can set P — including deserialisation — must either validate or document the invariant it assumes (Bar-Shalom et al., §5; standard practice for checkpointed filters).

**Impact.** A truncated, reordered, or hand-edited snapshot resumes a filter with an indefinite P. Joseph form preserves PSD but cannot restore it; a negative diagonal entry yields a negative innovation covariance S and a sign-flipped Kalman gain, which drives the estimate away from every measurement — silently, since neither restore nor update_scalar returns an error. The published nav_position_variance could go negative, and ThrottleGuidance's `if variance < R::zero() { return None }` (throttle_guidance.rs:322-324) would then silently withhold the ignition commit rather than reporting corruption.

**Recommended fix.** Either validate in NavFilter::restore (symmetry to a stated tolerance and non-negative diagonal, returning Result), or state the invariant explicitly in the rustdoc of restore and in the snapshot format documentation, and validate at the deserialisation boundary in state_snapshot.rs where a PhysicsError is already returnable.

**Adversarial check.** eskf.rs:150-155 matches verbatim — restore is `Self { state, cov }` with no validation and no Result, and its rustdoc describes only the round-trip contract, never the PSD invariant the caller must uphold. state_snapshot.rs:334-342 matches: 17 rows are read positionally via read_values, which the surrounding deserialiser uses only for length/short-buffer checking, and the result is handed straight to NavFilter::restore and then ReentryNavEngine::restore. The failure chain is sound: a negative diagonal entry yields S = hPhᵀ + r possibly ≤ 0, and update_scalar (eskf.rs:117-124) divides by it unguarded (see Finding 12), producing a sign-flipped gain that drives the estimate away from measurements. Joseph form is PSD-preserving but cannot restore PSD from an indefinite start. I confirmed the downstream behaviour the auditor predicts: throttle_guidance.rs:321-324 does `if variance < R::zero() { return None }`, so a negative published variance silently withholds the commit with no diagnostic. Bar-Shalom §5 is cited appropriately. Minor is right — it requires a corrupted or hand-edited snapshot.

> Evidence re-read: deep_causality_cfd/src/navigation/eskf.rs:150-155; deep_causality_cfd/src/types/flow/state_snapshot.rs:334-345 (positional read, no content validation, direct hand-off to restore); deep_causality_cfd/src/types/flow/throttle_guidance.rs:321-324

---

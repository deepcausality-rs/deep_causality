# CfdFlow DSL semantics — counterfactual machinery (src/types/flow/**, src/types/flow_config/**)

**Production readiness: `needs-work`**

The counterfactual core is genuinely sound: the fork shares the marched tensor by `Arc` and never copies it (carrier.rs:625-626, 834-837, 862), the fan-out is an order-preserving `scoped_map` with no shared mutable state, no RNG, and no cross-thread floating-point reduction (scoped_map.rs:57-72), and there is a real bit-identity regression test comparing a 2-branch fan-out against the manual sequential fork (fork_tests.rs:65-79). `Err` short-circuiting (coupling.rs:332-333), the `!!ContextAlternation!!` marker naming the baseline (carrier.rs:616-621), and `verdict()` returning data with no print or exit (reduce.rs:171-174) all hold as documented. What blocks certification is a cluster of silent-wrong-answer paths rather than the DSL skeleton: the compressible-regime `continuity_error` is computed by feeding the kernel two zeros, so it is identically zero for any implementation (mms.rs:165-167); the observed convergence order divides by `ln 2` for arbitrary user-supplied resolutions (operator_study.rs:110-120); a DEC march built without `march_for` silently marches zero steps and reports the seed (march_builder.rs:48); the compressible carrier clamps non-positive pressure to 1e-12 instead of failing (compressible_march_run.rs:436); a counterfactual world whose name equals the baseline's is silently discarded and the baseline flies instead (origin_fork.rs:212-214); and `save_log` is silently dropped on the event-fork path the README uses as its headline example (event_fork.rs:36-41). Three uncited tuned constants (1.2x acoustic re-pin, 20% rebuild tolerance, Mach 1.05 shock floor) directly set the marched result. None of these are architectural; each is a bounded fix, but every one of them can hand an engineer a wrong number with no error and no log line.

- Files read: **56**
- Findings raised: **18** — surviving adversarial verification: **18** (refuted: 0)
- Surviving by severity: major 3, minor 10, info 5
- Independently confirmed-correct items: **12**

## Verified correct against reference

These were positively confirmed, not merely un-flagged.

| Item | Location | Reference checked against |
|---|---|---|
| MMS incompressible/Euler references are analytically independent of the kernel under test — not a self-referential source term | `deep_causality_cfd/src/types/flow/mms.rs:126-139, 174-195` | 2D Taylor-Green at (pi/4,pi/4): u=(1/2,-1/2,0); grad_u rows (-1/2,1/2,0),(-1/2,1/2,0); lap=-2u=(-1,1,0); grad_p=(rho/2,rho/2,0). Reference du/dt=-2*nu*u (Taylor 1923). Hand-derived: (u.grad)u=(-1/2,-1 |
| Branch fan-out is deterministic and order-preserving; no shared mutable state, no RNG, no cross-thread FP reduction | `deep_causality_par/src/functions/scoped_map.rs:57-72; deep_causality_cfd/src/types/flow/carrier.rs:557-574` | Bit-identity to a sequential run requires (i) per-element results independent of thread assignment, (ii) results written to index-addressed slots, (iii) no floating-point accumulation whose order vari |
| The paused marched tensor is genuinely never copied on fork (the O(1) half of the CoW claim) | `deep_causality_cfd/src/types/flow/carrier.rs:625, 834-837, 855-862` | Copy-on-write sharing means the child holds an Arc clone and the underlying buffer is only duplicated when a mutable path is taken. |
| An Err from any coupling stage short-circuits the whole step (README claim c) | `deep_causality_cfd/src/types/flow/coupling.rs:327-334; carrier.rs:139-146` | Sequential composition must propagate the head's error without running the tail, and the step driver must not continue past a stage failure. |
| Every branch stamps a !!ContextAlternation!! marker naming its baseline (README claim e) | `deep_causality_cfd/src/types/flow/carrier.rs:615-621, 688-693` | The marker must be written into the branch's provenance log and must identify the baseline world it departs from. |
| verdict() returns data; the study path never prints or exits | `deep_causality_cfd/src/types/flow/study/reduce.rs:171-174; verdict.rs:103-123` | README: "verdict() returns data", "the DSL never exits or prints". |
| Duct inlet-ghost isentropic relations and thrust coefficient in stagnation-scaled variables | `deep_causality_cfd/src/types/flow/duct_march_run.rs:141, 247-248, 305` | Anderson, Modern Compressible Flow: T/T0 = 1 - (gamma-1)u^2/(2*gamma*R*T0); p/p0 = (T/T0)^(gamma/(gamma-1)); T*/T0 = 2/(gamma+1) so u_hat* = sqrt(gamma*T_hat*) = sqrt(2*gamma/(gamma+1)). C_f = (mdot*u |
| Converging-diverging duct area profile has zero area slope at the throat | `deep_causality_cfd/src/types/flow_config/duct_config.rs:110-118` | Quasi-1D area-Mach relation requires dA/dx = 0 where M = 1, i.e. at the throat. |
| Seed::TaylorGreenVortex is the classic divergence-free 3D initial condition | `deep_causality_cfd/src/types/flow_config/seed.rs:64-84` | Taylor & Green (1937): u = sin(kx)cos(ky)cos(kz), v = -cos(kx)sin(ky)cos(kz), w = 0. |
| Cosine metric grading preserves total axis length | `deep_causality_cfd/src/types/flow_config/mesh.rs:155-172` | Doc claim (mesh.rs:43-44): l(pos) = h*(1 + amp*cos(2*pi*pos/N)) "sums to N*h, so the wavenumber is unchanged". |
| Hodge-Laplacian eigenvalue used as the operator-study reference is correct | `deep_causality_cfd/src/types/flow/operator_study.rs:12-15, 143-153, 157-174` | On a flat torus, Delta = d*delta + delta*d and Delta = -nabla^2 on components. For divergence-free u (delta u = 0), Delta u = delta d u. For the k=1 TG field u = (cos x sin y, -sin x cos y), nabla^2 u |
| continue_branches error semantics match their documented contract | `deep_causality_cfd/src/types/flow/carrier.rs:554-574` | Doc: "The first failing branch's error, in world order. Every branch runs to completion first; a failure does not cancel its siblings." |

## Findings

### 14.1 [MINOR] MMS compressible continuity check feeds the kernel two zeros, so continuity_error is identically zero for any implementation

- **Verification verdict:** CONFIRMED
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/src/types/flow/mms.rs:165`
- **Auditor confidence:** confirmed

**Claim.** The compressible `continuity_error` observable is evaluated on a degenerate input (grad_rho = 0, div_u = 0) at which every term of the continuity kernel vanishes, so it cannot discriminate a sign or coefficient error in that kernel. The inline comment declares the degeneracy honestly; the module-level claim at mms.rs:11-13 that a passing error 'genuinely pins the kernel' does not hold for this observable.

**Code evidence.**

```
mms.rs:164-167:
                // Divergence-free => continuity RHS = 0.
                let continuity =
                    compressible_ns_continuity_rhs(&rho, &u, &[R::zero(); 3], R::zero());
                report.add_series("continuity_error", vec![continuity.abs()]);

deep_causality_physics/src/kernels/fluids/governing.rs:112-115:
    let r = rho.value();
    let u_raw = u.value();
    let u_dot_grad_rho = u_raw[0]*grad_rho[0] + u_raw[1]*grad_rho[1] + u_raw[2]*grad_rho[2];
    -(u_dot_grad_rho + r * div_u)
```

**Reference form.** MMS requires substituting a manufactured field into the PDE and comparing the kernel residual against an independently derived exact rate. For continuity: d(rho)/dt = -(u.grad(rho) + rho*div(u)), with grad(rho) and div(u) evaluated from the manufactured field, not set to zero. The kernel's own rustdoc (governing.rs:101-102) states it "Reduces to 0 for incompressible flow when grad_rho = 0 and div_u = 0" — the code supplies exactly that degenerate input.

**Impact.** An engineer reading `continuity_error ~ 0` in a verification report concludes the continuity closure is pinned. It is not: a sign flip, a factor of 2, or swapping grad_rho with div_u would all still report zero. mms.rs's own module doc (lines 11-13) asserts "The references are exact (not kernel-derived), so a passing error genuinely pins the kernel" — that assertion is false for this one case.

**Recommended fix.** Manufacture a compressible field with non-zero density variation (e.g. rho(x) = rho0*(1 + eps*sin x) advected by the TG velocity), compute grad(rho) via the same autodiff tangent functor `manufactured.rs` already uses, and compare the kernel output against the analytically differentiated d(rho)/dt. If a genuinely compressible manufactured solution is out of scope, delete the `continuity_error` series rather than reporting a number that cannot move.

**Adversarial check.** Code is verbatim at the cited location. `compressible_ns_continuity_rhs(&rho, &u, &[R::zero();3], R::zero())` is called with grad_rho = 0 and div_u = 0; the kernel body (`-(u·grad_rho + rho*div_u)`) is a sum of products in which every term carries one of the two zeroed arguments, so the result is exactly 0 for any sign, coefficient, or term-ordering variation of that form. The observable therefore has no discriminating power. The module doc at mms.rs:11-13 does assert 'The references are exact (not kernel-derived), so a passing error genuinely pins the kernel', which is false for this one observable. Two corrections to the auditor: (a) 'regardless of any ... error' is slightly overstated — a kernel returning a term not multiplied by grad_rho or div_u (e.g. bare rho) would still be caught; (b) the inline comment at mms.rs:164 honestly declares 'Divergence-free ⇒ continuity RHS = 0', so the degeneracy is stated, and the compressible *momentum* check in the same arm (mms.rs:154-162) is non-degenerate. Severity downgraded to minor: this is a weak-test / doc-overclaim issue, not a wrong computed number.

> Evidence re-read: mms.rs:164-167 (verbatim as cited); mms.rs:11-13 (module doc claim); deep_causality_physics/src/kernels/fluids/governing.rs:99-115 — `continuity_rhs_kernel` = `-(u_dot_grad_rho + r * div_u)`, rustdoc: 'Reduces to 0 for incompressible flow when grad_rho = 0 and div_u = 0'

---

### 14.2 [MAJOR] CompressiblePause::state() claims the resumed march is bit-identical to continuing the pause, but the resumed leg discards the evolved fluid state

- **Verification verdict:** CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/src/types/flow/carrier.rs:502`
- **Auditor confidence:** confirmed

**Claim.** The rustdoc on `CarrierPause::state()` states the exported `MarchState` resumes "bit-identically to continuing this pause directly". It does not. `MarchState` carries only the `CoupledField` and the step index; the incoming leg rebuilds its carrier and re-quantizes the world's uniform seed, so the evolved conserved state is thrown away. The crate documents this correctly in a different file.

**Code evidence.**

```
carrier.rs:499-504:
    /// Export this pause as a resumable [`MarchState`]: the carried field plus the step reached.
    /// The state resumes a continued march (in memory now, or from disk later after
    /// [`save`](MarchState::save)) bit-identically to continuing this pause directly.
    pub fn state(&self) -> MarchState<R> {
        MarchState::at((*self.field).clone(), self.step)
    }

coupled_march.rs:88-95 (the method that consumes it):
    /// **A leg boundary carries the coupled field, not the marched fluid layer.** `MarchState` is
    /// the field plus a step index; the incoming leg rebuilds its carrier and re-quantizes the
    /// world's uniform seed, so the evolved conserved state, the inflow strip, any acoustic-envelope
    /// drift the previous leg earned, its rebuild count, and its plume-imprint budget are all
    /// discarded.
```

**Reference form.** "Bit-identical resume" means the resumed run reproduces, bit for bit, the state trajectory the un-suspended run would have produced. `CarrierPause::continue_with` resumes `Arc<M::State>` (the evolved tensor); `CoupledMarch::from(MarchState)` calls `carrier.seed_state(cfg)` on the config's uniform seed. These are different initial conditions, so the claim is falsified by construction.

**Impact.** An engineer suspending a descent leg at 61 km and resuming from disk believes the trajectory is continuous. It is not: the shock layer restarts from the world's uniform seed and re-converges over some number of steps. Any quantity integrated across the seam (heat load, blackout dwell, delta-v) is wrong by the re-convergence transient, silently and with no error.

**Recommended fix.** Replace the `state()` rustdoc with the coupled_march.rs wording verbatim: the field is carried, the marched fluid layer is re-seeded. If bit-identical disk resume is actually wanted, extend `pack_resume` (state_snapshot.rs:140-250) to serialize the four conserved tensor trains alongside the field, mirroring `pack_tt_fields`.

**Adversarial check.** Both quoted doc blocks are verbatim and they contradict each other. carrier.rs:499-504 claims the exported MarchState 'resumes a continued march ... bit-identically to continuing this pause directly' — the comparator named is `continue_with`. coupled_march.rs (the `from(MarchState)` consumer) states the opposite in bold: 'A leg boundary carries the coupled field, not the marched fluid layer ... the evolved conserved state, the inflow strip, any acoustic-envelope drift the previous leg earned, its rebuild count, and its plume-imprint budget are all discarded. That is the design's accepted quasi-steady defense.' Confirmed by construction: `MarchState::at((*self.field).clone(), self.step)` carries only the CoupledField and step index, while `run_continued_segment` (carrier.rs:834-837) resumes the shared `Arc<M::State>` — genuinely different initial conditions. The crate documents the correct behaviour in the consumer and the incorrect claim on the producer.

> Evidence re-read: carrier.rs:499-504 (state(), verbatim); carrier.rs:829-864 (run_continued_segment resumes Arc<M::State>); coupled_march.rs `from()` rustdoc, ~lines 85-95 (the 'evolved conserved state ... discarded' paragraph, verbatim)

---

### 14.3 [MINOR] Observed convergence order hard-codes refinement ratio r = 2 while resolutions() accepts arbitrary sweeps

- **Verification verdict:** CONFIRMED
- **Axis:** physics-math
- **Location:** `deep_causality_cfd/src/types/flow/operator_study.rs:116`
- **Auditor confidence:** confirmed

**Claim.** The observed-order computation assumes a doubling sweep (r = 2), an assumption documented in both the module doc and the inline comment but not enforced: `OperatorStudyBuilder::resolutions` accepts any sweep and `run` validates only that at least two resolutions are given, so a non-doubling sweep silently reports a wrong order.

**Code evidence.**

```
operator_study.rs:109-120:
        // Observed order between consecutive resolutions: log2(e_n / e_2n).
        let two = R::from_f64(2.0).expect("2.0 lifts into every real field");
        let ln2 = two.ln();
        let mut orders: Vec<R> = Vec::with_capacity(errors.len() - 1);
        for w in errors.windows(2) {
            let order = if w[1] > R::zero() {
                (w[0] / w[1]).ln() / ln2
            } else {
                R::zero()
            };

operator_study.rs:60-63 (unvalidated public setter):
    pub fn resolutions(mut self, resolutions: impl Into<Vec<usize>>) -> Self {
        self.resolutions = resolutions.into();
        self
    }
```

**Reference form.** Roache, Verification and Validation in Computational Science: p = ln(e_coarse/e_fine) / ln(r), with r = h_coarse/h_fine. Here h = 2*pi/n, so r = n_fine/n_coarse. The code substitutes the constant 2 for r.

**Impact.** A [16, 24, 48] sweep of a genuinely second-order operator reports orders of 2*ln(1.5)/ln(2) = 1.17 and 2.0 instead of 2.0 and 2.0. A certification gate asserting `order > 1.9` would fail a correct operator; a gate asserting `order > 1.0` would pass a first-order operator swept at r = 4 (true p = 1 reports 2). Both directions mislead.

**Recommended fix.** Compute `r` per pair from the resolutions: `let r = R::from_usize(res[i+1])? / R::from_usize(res[i])?; let order = (w[0]/w[1]).ln() / r.ln();`. Alternatively validate in `run()` that every consecutive pair satisfies `res[i+1] == 2*res[i]` and return `DimensionMismatch` otherwise — the same style as the existing `resolutions.len() < 2` guard at line 97.

**Adversarial check.** Code verbatim at operator_study.rs:109-120 (ln2 divisor) and 60-63 (unvalidated public `resolutions`). `run` validates only `len() >= 2`; nothing checks that consecutive resolutions double. The auditor's reference form is correct: p = ln(e_coarse/e_fine)/ln(r) with r = h_coarse/h_fine = n_fine/n_coarse (operator_error uses h = 2π/n at line 133, so r = n_fine/n_coarse exactly as claimed). Their worked example is also right: for a true p=2 operator on [16,24,48], the first pair reports 2·ln(1.5)/ln2 = 1.17. Mitigating and worth recording: the assumption is *documented* — the module doc at line 11 and the inline comment at line 109 both write the formula as log₂(eₙ/e₂ₙ), and the default sweep [16,32,64] doubles. The defect is the unenforced precondition, not a hidden formula error, hence minor rather than major.

> Evidence re-read: operator_study.rs:60-63 (resolutions setter, no validation); :96-102 (build-time check is len>=2 only); :109-120 (ln2 hard-coded); :131-134 (h = 2π/n, so r = n_fine/n_coarse); :11 and :109 (documented log₂ form)

---

### 14.4 [MINOR] MarchConfigBuilder defaults to MarchStop::Fixed(0), so a config missing march_for silently marches zero steps and reports the seed

- **Verification verdict:** CONFIRMED
- **Axis:** magic-number
- **Location:** `deep_causality_cfd/src/types/flow_config/march_builder.rs:48`
- **Auditor confidence:** confirmed

**Claim.** `MarchConfigBuilder` defaults `stop` to `MarchStop::Fixed(0)` and `build()` does not validate it, so omitting both `march_for` and `march_until_steady` yields a successful config whose run executes zero steps and returns a Report of the projected seed with single-element series. The sibling compressible builder rejects a missing `stop`.

**Code evidence.**

```
march_builder.rs:41-54:
            seed: Seed::Rest,
            stop: MarchStop::Fixed(0),

march_builder.rs:156-164 (build validates only mesh + solver):
        let mesh = self.mesh.ok_or_else(|| { ... "a mesh is required" ... })?;
        let solver = self.solver.ok_or_else(|| { ... "a solver config is required" ... })?;

march_run.rs:227-241 (the zero-trip loop):
        match stop {
            MarchStop::Fixed(n) => {
                for s in 0..n {
```

**Reference form.** A configuration default must either be a physically meaningful value or be rejected at build time. Zero steps is neither: it is a no-op disguised as a completed run. The sibling builders reject their required sections — `CompressibleMarchConfigBuilder::build` returns `missing("stop")` (compressible_march_config.rs:515) rather than defaulting.

**Impact.** `ctx.sample` runs once before the loop (march_run.rs:226), so the Report carries a one-element kinetic_energy/divergence series and a final_field equal to the projected seed. A study gate reading `report.final_field()` or the last series element sees plausible, finite, wrong numbers — a lid-driven cavity that never moved reported as a converged cavity.

**Recommended fix.** Make `stop` an `Option<MarchStop<R>>` in the builder and have `build()` return `PhysicsError::PhysicalInvariantBroken("a march stop is required")` when unset, matching `CompressibleMarchConfigBuilder::build`. At minimum, reject `MarchStop::Fixed(0)` in `build()`.

**Adversarial check.** All three code sites are verbatim. `MarchConfigBuilder::new` sets `stop: MarchStop::Fixed(0)` (march_builder.rs:48); `build()` (156-164) validates only mesh and solver and passes `self.stop` through unchecked; `march_run.rs:227-241` runs `for s in 0..n`, a zero-trip loop at n = 0. `ctx.sample(&state, &mut series)` at march_run.rs:226 runs before the loop and `report.set_final_field(...)` at 271 emits the projected seed, so the resulting Report is well-formed with one-element series — exactly as claimed. The sibling asymmetry is also real: `CompressibleMarchConfigBuilder::build` returns `missing("stop")` for the same field. Downgraded to minor: the failure requires the author to omit the horizon verb entirely, and the resulting Report is visibly degenerate (single-element series) rather than a plausible converged result — the auditor's 'converged cavity' framing overstates how convincing the artifact looks.

> Evidence re-read: march_builder.rs:41-54 (defaults, incl. stop: MarchStop::Fixed(0)); :156-177 (build validates mesh + solver only, `self.stop` unvalidated); march_run.rs:226-242 (pre-loop sample then `for s in 0..n`); :266-272 (report assembled, final_field = seed); compressible_march_config.rs:508-518 (sibling builder rejects missing `stop`, `flight_dt`, `seed_fn`, `reference`)

---

### 14.5 [MAJOR] Compressible carrier clamps non-positive pressure to 1e-12 instead of failing, silently reporting near-zero temperature from a broken state

- **Verification verdict:** CONFIRMED
- **Axis:** magic-number
- **Location:** `deep_causality_cfd/src/types/flow/compressible_march_run.rs:436`
- **Auditor confidence:** confirmed

**Claim.** `publish_and_transport` guards density positivity with a hard error but silently floors a non-positive nondimensional pressure at 1e-12, then divides by density to publish `T_tr`. A marched state with negative internal energy — a genuine solver failure — is published as a physically valid field at T ~ 1e-12 * t_ref instead of raising an error.

**Code evidence.**

```
compressible_march_run.rs:426, 429-438:
        let tiny = Self::lift(1.0e-12)?;
        ...
            if rho <= R::zero() || !rho.is_finite() {
                return Err(PhysicsError::PhysicalInvariantBroken(
                    "compressible carrier: density must stay positive".into(),
                ));
            }
            let u2 = (mx * mx + my * my) / (rho * rho);
            let p_hat = (self.gamma - R::one()) * (e - half * rho * u2);
            let p_hat = if p_hat > tiny { p_hat } else { tiny };
            let t_hat = p_hat / rho;

Same clamp repeated in finish(), compressible_march_run.rs:467, 471-473.
```

**Reference form.** For an ideal gas p = (gamma-1)*(E - 0.5*rho*|u|^2) must be strictly positive; p <= 0 means the conserved state has left the physically admissible set and the march is invalid. The crate's own duct driver applies the correct discipline: duct_march_run.rs:365-369 returns `PhysicalInvariantBroken("duct march: pressure must stay positive and finite")`. The 1e-12 value has no cited source and no documented justification.

**Impact.** A negative-pressure cell propagates as T_tr ~ 0 K into `IonizationStage`, `RegimeClassify` (Knudsen number), and the blackout trigger, so an unstable march can report a plausible GNSS-availability corridor. The failure is invisible: no log entry, no error, no series flag. The floor's value also directly sets the published temperature in that cell, making it a result-determining constant.

**Recommended fix.** Mirror the duct driver: replace the clamp with `if !(p_hat > R::zero() && p_hat.is_finite()) { return Err(PhysicsError::PhysicalInvariantBroken("compressible carrier: pressure must stay positive")); }` in both `publish_and_transport` and `finish`. If a floor is genuinely needed for a documented reason, make it a config field with a cited rationale rather than a literal.

**Adversarial check.** Verbatim at compressible_march_run.rs:426, 429-438 and repeated at 467, 471-473. The asymmetry is real: `rho <= 0 || !rho.is_finite()` raises PhysicalInvariantBroken, while `p_hat <= tiny` is silently floored, then divided by rho to publish T_tr into 'T_tr' — the field IonizationStage / RegimeClassify / the blackout trigger read. No log entry, no counter, no series flag; 1e-12 has no nearby comment, no named constant, and no citation. The reference form is correct (p = (γ−1)(E − ½ρ|u|²) > 0 is the admissibility condition). One additional defect the auditor missed, which strengthens the finding: `finish()` (line 468) has no density guard at all, so a non-positive or non-finite rho there divides straight into `report.set_final_field(t_tr)` as inf/NaN. A positivity floor is defensible as a limiter for genuine near-vacuum cells, but a silent, uncounted, unlogged one that also sets the published temperature is not.

> Evidence re-read: compressible_march_run.rs:423-449 (publish_and_transport: tiny lift at 426, hard density error at 429-433, silent p_hat floor at 436, t_hat = p_hat/rho at 437, published at 445-448); :460-481 (finish: same floor at 472, and no density guard at all); grep for the duct discipline confirms duct_march_run raises on non-physical wave speed rather than flooring

---

### 14.6 [MINOR] Uncited tuned constants (1.2x acoustic re-pin, 20% rebuild tolerance) determine the marched result's numerical dissipation

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** magic-number
- **Location:** `deep_causality_cfd/src/types/flow/compressible_march_run.rs:380`
- **Auditor confidence:** confirmed

**Claim.** The 1.2× acoustic re-pin multiplier (compressible_march_run.rs:380) is a hard-coded, unconfigurable literal with no derivation or citation, and it raises the implicit acoustic dissipation ν̄ = ½·s_ref·Δx by 20% above the signal-speed bound at each rebuild. The companion 0.2 rebuild tolerance is *not* untraceable: it is a documented default with a public override (`with_rebuild_tolerance`), and the ratchet behaviour of the pair is described at length in the comment at lines 355-362.

**Code evidence.**

```
compressible_march_run.rs:363-391:
        let s_needed = u_hat + (self.gamma * t_hat).sqrt();
        if s_needed > self.s_ref * (R::one() + schedule.rebuild_tol) {
            ...
            let s_new = s_needed * Self::lift(1.2)?;
            ...
            self.s_ref = s_new;

compressible_march_config.rs:103-110 (the 0.2 default):
        let tol = R::from_f64(0.2)
            .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(0.2) failed".into()))?;
        ...
            rebuild_tol: tol,

compressible_march_config.rs:62-64 (doc states the value, gives no justification):
    /// a 20% rebuild tolerance.
```

**Reference form.** For a Rusanov/LLF-family scheme the dissipation coefficient must bound the maximum signal speed |u| + c; the standard choice is s = max(|u| + c) over the domain, with no discretionary multiplier. Any margin above 1.0 adds numerical dissipation proportional to (s - s_max), which smears the solution. A 1.2x margin is a 20% dissipation surcharge with no stated derivation.

**Impact.** The pair sets a hysteresis ratchet (each rebuild needs ~1.44x further growth at the default tolerance, as the code comment at lines 357-360 notes) and therefore controls how much excess dissipation the marched layer carries between rebuilds. Two runs with different `rebuild_tol` produce different temperature and electron-density fields, hence different blackout onset steps — but neither number is traceable and neither appears in the config's public surface as a documented physical choice.

**Recommended fix.** Either cite the source (a stability analysis, a paper in papers/) for the 1.2 margin and the 0.2 gate, or expose the re-pin factor as a `DescentSchedule` field beside `rebuild_tol` and record a sensitivity study in studies/ showing the marched result's dependence on both. At minimum, add the 1.2 to the doc comment where 0.2 is already named.

**Adversarial check.** The physics reasoning checks out and I verified the causal link the auditor asserted: marcher_2d.rs:53 and :63-64 document `ν̄ = ½·s_ref·Δx` as the implicit acoustic dissipation, so re-pinning s_ref to 1.2·s_needed does raise the dissipation coefficient 20% above the signal-speed bound. The 1.2 literal at line 380 is hard-coded, uncited, and not configurable — that half of the finding stands. But the claim 'neither appears in the config's public surface' is wrong: `rebuild_tol` has a public setter, `CompressibleMarchConfig::with_rebuild_tolerance` (compressible_march_config.rs:128-130), and its 0.2 default is stated in the `DescentSchedule::new` rustdoc ('a 20% rebuild tolerance'). The comment at compressible_march_run.rs:355-362 also documents the ratchet behaviour (~1.44× per rebuild), the one-sidedness, the density-independence, and the per-leg reset — far more traceability than 'untraceable'. What remains is that neither literal has a derivation or a cited source. Downgraded to minor.

> Evidence re-read: compressible_march_run.rs:355-392 (comment documenting the ratchet; `s_new = s_needed * lift(1.2)` at 380); compressible_march_config.rs:103-110 (0.2 default) and :62-64 (doc) and :128-130 (`with_rebuild_tolerance` public override); solvers/qtt/compressible/marcher_2d.rs:53, 63-64, 89 (`ν̄ = ½·s_ref·Δx`, `beta = dt*half*s_ref*h` — confirms s_ref sets dissipation)

---

### 14.7 [MINOR] Origin-fork alternate() identifies worlds by name string; a differently-parameterized world sharing the baseline's name silently flies the baseline

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** physics-math
- **Location:** `deep_causality_cfd/src/types/flow/study/origin_fork.rs:212`
- **Auditor confidence:** confirmed

**Claim.** `march_for` decides alternation by display-name inequality rather than by configuration identity, so a case world that carries different physics but the baseline's name flies the baseline and its config is discarded. The by-name contract is explicitly documented at origin_fork.rs:103-105 and the outcome is recorded as 'baseline' in the rejoin audit line, so the defect is a weak identity model reachable through a naming mistake, not an undocumented guarantee violation.

**Code evidence.**

```
origin_fork.rs:209-214:
                let world = &c.worlds[i];
                let stack = (c.coupling)(&c.cases[i], draw);
                let mut run = CfdFlow::march(&c.baseline);
                if world.name() != c.baseline.name() {
                    run = run.alternate_context(world);
                }

origin_fork.rs:106-108 (the documented guarantee, qualified "by name"):
    /// Bind each case to a world alternated from the baseline. Guarantee: every case whose world
    /// differs from the baseline (by name) flies alternated and carries the alternation marker; the
    /// baseline case flies unmarked.
```

**Reference form.** A counterfactual world is identified by the configuration it carries (grid, gamma, dt_flight, published constants, descent schedule), not by its display name. `CompressibleMarchConfig` carries `constants: Vec<(&'static str, R)>` (compressible_march_config.rs:230) — the commanded inputs that distinguish counterfactuals — none of which participate in this equality test.

**Impact.** In the weather-dispersion campaign the README describes ("six atmospheres alternated from one baseline"), a model author who names two worlds identically, or who names an alternate world after the baseline, gets a case that reports as a distinct atmosphere but flew the standard day. The resulting row is indistinguishable from a real result and the rejoin narration at origin_fork.rs:230-234 labels it "baseline", which is correct but only visible in the audit file.

**Recommended fix.** Either compare configurations structurally (a fingerprint over grid, gamma, dt_flight, constants, schedule — the crate already has `fingerprint64` in deep_causality_file, used by state_snapshot.rs:240), or always call `alternate_context(world)` and let the carrier record the no-op, or reject duplicate world names in `alternate()` with `StudyError::in_stage("alternate", ..)`.

**Adversarial check.** Code is verbatim at origin_fork.rs:208-213: `if world.name() != c.baseline.name() { run = run.alternate_context(world); }`. The hazard is real — a case world with distinct physics but a colliding name is marched in the baseline world, its config discarded, with no error. But the claim that the guarantee is misrepresented is not supported: the rustdoc at :103-105 explicitly writes 'every case whose world differs from the baseline (by name)' — the by-name identity is the stated contract, not an implicit one, and the auditor concedes this in their own quote. The rejoin narration at :230-234 additionally labels such a run 'baseline' in the audit file, so it is not fully silent. Axis 'physics-math' is also wrong: this is an API-robustness / identity-modelling issue, not a formula defect. Triggering it requires an authoring mistake (two worlds named alike), and the cheap fix is to alternate unconditionally.

> Evidence re-read: origin_fork.rs:206-219 (sweep body, name comparison at 211, save_log branch at 215-217); :103-105 (rustdoc, '(by name)' qualifier verbatim); :220-243 (rejoin narration records 'alternated'/'baseline' per branch from the !!ContextAlternation!! marker)

---

### 14.8 [MAJOR] save_log is silently dropped on the event-fork path the README uses as its headline counterfactual example

- **Verification verdict:** CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/src/types/flow/study/event_fork.rs:36`
- **Auditor confidence:** confirmed

**Claim.** `StudyDef::save_log` stores an audit base path in `Cases.audit`. `StudyEffect::fork` drops that field on the floor when building `ForkStudy`, so a study written as `.save_log(p).cases(..).fork(&onset).branch(..).continue_for(..)` writes no audit files at all — with no error, no warning, and no diagnostic. The README advertises the audit sink immediately after presenting exactly this fork/branch/continue_for chain.

**Code evidence.**

```
event_fork.rs:36-41 (audit silently discarded):
        self.map(|c| ForkStudy {
            title: c.title,
            cases: c.cases,
            pause,
        })

study/mod.rs:126-131 (Cases carries the field):
pub struct Cases<T> {
    title: String,
    cases: Vec<T>,
    /// The campaign-level audit base path (`save_log`), carried to the coupled `march_for`. Only
    /// the origin-counterfactual coupled path consumes it; the other binders drop it.
    audit: Option<PathBuf>,
}

README.md:83-105 (the fork/branch/continue_for example, then):
an optional `save_log(path)` flushes each run's provenance to disk, one file per branch under a fan-out.
```

**Reference form.** An audit facility that is silently inert on a subset of the paths it is offered on is not an audit facility. Either the verb must apply on the path, or requesting it there must be a compile error or a runtime error. The `Cases` rustdoc ("the other binders drop it") documents the behaviour but the README, which is what an evaluator reads, does not.

**Impact.** An avionics lab that adds `.save_log("corridor")` to the plasma-blackout corridor study to obtain a per-branch provenance trail for a certification package gets zero files and no indication of failure. The same call on the weather-dispersion `march_for` study works, so the omission looks like a filesystem problem rather than an unsupported path.

**Recommended fix.** Thread `audit` through `ForkStudy`/`Branched` into `continue_for`, lowering onto a per-branch `save_log` the way `march_for` does at origin_fork.rs:215-217; `CarrierPause::continue_branches` already has the per-branch seam. If threading it is deferred, make `fork()` return `StudyEffect::from_result(Err(StudyError::in_stage("fork", ..)))` when `audit.is_some()`, and correct the README sentence to name `march_for` explicitly.

**Adversarial check.** Verbatim and complete. `StudyEffect::<Cases<T>>::fork` (event_fork.rs:36-41) constructs `ForkStudy { title, cases, pause }` — `ForkStudy` has no audit field, so `Cases::audit` is dropped with no error path. Grep of `save_log` across src/ confirms `Cases::audit` is consumed at exactly one site, origin_fork.rs:216 (the coupled `march_for` path); the event-fork `branch`/`continue_for` chain never reads it and `run_continued_segment` (carrier.rs:829-864) attaches no AuditFlush sink, so the branches run under the default `NoAudit`. The README at lines 83-105 presents the fork/branch/continue_for chain and then advertises 'an optional save_log(path) flushes each run's provenance to disk, one file per branch under a fan-out' with no scoping. The `Cases` rustdoc (study/mod.rs:126-131) does document the drop internally, which is where the crate's honesty lives — but a silently inert audit verb on a certification path is the defect, and the README actively invites it.

> Evidence re-read: event_fork.rs:29-41 (fork drops audit); study/mod.rs:124-131 (Cases.audit + 'the other binders drop it'); grep save_log across src/ — only origin_fork.rs:216 consumes the campaign base path; carrier.rs:279-300 (AuditFlush seam, NoAudit default) and :829-864 (continued segment attaches no sink); README.md:83-105

---

### 14.9 [MINOR] Duct quasi-steady residual is a per-step delta, making the convergence gate resolution-dependent

- **Verification verdict:** CONFIRMED
- **Axis:** physics-math
- **Location:** `deep_causality_cfd/src/types/flow/duct_march_run.rs:209`
- **Auditor confidence:** likely

**Claim.** The duct quasi-steady gate compares the per-step relative change against residual_tol without dividing by dt. Since dt = cfl·dx/s_global, the measured quantity scales with dx, so the same physical level of unsteadiness reports a smaller residual on a finer grid and the convergence criterion loosens under refinement. The code names the quantity honestly ('maximum relative change per step') and raises a hard error when the budget is exhausted; the defect is that 'settled' is not grid-independent, so runs at different resolutions are not on equal footing.

**Code evidence.**

```
duct_march_run.rs:166 (dt proportional to dx):
            let dt = cfl * dx / s_global;

duct_march_run.rs:192-201, 209-221 (residual is the raw per-step delta, not divided by dt):
                for k in 0..3 {
                    let d = (znew[k] - z[k]).abs();
                    if d > max_delta[k] { max_delta[k] = d; }
                    let s = z[k].abs();
                    if s > max_scale[k] { max_scale[k] = s; }
                }
            ...
            residual = R::zero();
            for k in 0..3 {
                if max_scale[k] > R::zero() {
                    let r = max_delta[k] / max_scale[k];
                    if r > residual { residual = r; }
                }
            }
            if residual < cfg.residual_tol { settled = true; break; }
```

**Reference form.** The standard steady-state residual is the discrete time derivative, ||dU/dt||, i.e. the per-step change divided by dt, or the L2/Linf norm of the spatial operator R(U) = -(1/A)*d(fA)/dx + s. Normalizing by dt (or by the initial residual, R_n/R_0) makes the gate independent of the grid and of the CFL number. Ferziger & Peric, Computational Methods for Fluid Dynamics, section on convergence criteria.

**Impact.** With dx = L/n, doubling the cell count halves the reported residual at identical physical convergence. A case that passes `residual_tol = 1e-8` at n = 200 may be materially less converged than the same case passing at n = 100, and the reported Mach profile, shock position, and thrust coefficient are then compared across resolutions on unequal footing. The error is silent: `settled = true` and a full Report is returned.

**Recommended fix.** Divide by `dt` before comparing: `let r = max_delta[k] / (max_scale[k] * dt);` and restate `DuctConfig::residual_tol`'s rustdoc (duct_config.rs:162-164) as a rate. Alternatively normalize against the first step's residual and gate on the reduction factor. Either change requires re-tuning the tolerances used by the duct verification programs.

**Adversarial check.** Code verbatim at duct_march_run.rs:166 (`let dt = cfl * dx / s_global;`) and :180-221 (max_delta accumulated as the raw `|znew − z|`, normalized only by max_scale, compared against residual_tol at 218). I re-derived the scaling independently: Δz = (dt/(dx·A))·ΔF, and ΔF ≈ dx·∂F/∂x, so Δz ≈ dt·∂F/∂x ∝ dx·∂F/∂x at fixed CFL. Halving dx therefore halves the reported residual at identical physical unsteadiness — the gate does get easier under refinement, as claimed. The auditor's reference form is correct (normalize by dt to get ‖dU/dt‖, or by the initial residual R_n/R_0). Downgraded to minor for two reasons the auditor omits: the code names what it measures honestly at :207-208 ('maximum relative change per step'), and non-convergence is a hard error (:223-230), not a silent pass — the failure mode is an over-optimistic 'settled', not a fabricated result.

> Evidence re-read: duct_march_run.rs:147-166 (CFL step, dt = cfl·dx/s_global); :180-204 (update and max_delta/max_scale accumulation); :206-221 (residual = max_k max_delta[k]/max_scale[k], gate at 218); :223-230 (non-convergence raises CalculationError)

---

### 14.10 [MINOR] The coupling stack runs before the advance on the DEC host and after it on the carrier hosts, with no warning at the PhysicsStage seam

- **Verification verdict:** CONFIRMED
- **Axis:** doc-gap
- **Location:** `deep_causality_cfd/src/types/flow/coupling.rs:13`
- **Auditor confidence:** confirmed

**Claim.** The between-step coupling is applied before the advance on the DEC host and after the advance and publish on the carrier hosts, so a stage keyed on ctx.step() or reading a published scalar sees a state one advance apart between the two. Each host documents its own ordering at its call site, but neither the PhysicsStage trait nor the coupling module doc records that the ordering is host-dependent, so a stack validated on one host changes physics on the other with no compile error and no runtime signal.

**Code evidence.**

```
march_run.rs:361-374 (DEC: coupling, then advance):
/// Run the between-step coupling, then advance one projected step under the resulting ambient.
fn advance_coupled<...>(...) -> Result<SolenoidalField<R>, PhysicsError> {
    let ctx = StepContext::new(manifold, state, dt, step);
    coupling.apply(&ctx, field)?;
    Ok(solver.advance(state, field.ambient())?.into_state())
}

carrier.rs:139-146 (carrier: advance, publish, then coupling):
        self.pre_step(field, step)?;
        let next = self.advance(state)?;
        self.publish_and_transport(&next, field, kappa)?;
        let ctx = StepContext::<D, R>::qtt(self.dt(), step);
        coupling.apply(&ctx, field)?;

coupling.rs:6-8 (the uniform claim):
//! The `.couple` multi-physics seam (design D4): a statically-composed, between-step physics
//! pipeline run once per timestep *around* the CFD step.
```

**Reference form.** In a partitioned multi-physics coupling the ordering of the fluid solve relative to the auxiliary physics defines the scheme (fluid-first vs. physics-first Gauss-Seidel staggering) and changes the accuracy and lag structure of the coupled system. A trait that is portable across hosts must fix that ordering or state that it differs per host.

**Impact.** A stage keyed on `ctx.step()` — `AeroBlackoutStub`'s window `[start, end)` (coupling.rs:508) is one — fires at a different physical instant on the two hosts. A stage reading `field.scalar("speed")` gets the pre-advance value on the DEC host and the post-advance value on the carrier. Moving a validated coupling stack between hosts therefore changes the physics with no compile error and no runtime signal. The project clearly knows the ordering matters locally (throttle_guidance.rs:152-157 documents a deliberate one-step actuation lag from the force-channel ordering contract) but the general contract is silent.

**Recommended fix.** State the per-host ordering in the `PhysicsStage` trait rustdoc and in the coupling.rs module header, and document the resulting stale-read rule explicitly: a stage reading a named field written by a later stage in the same `.then` chain reads the previous step's value, because `CoupledField` persists across steps. Consider exposing the phase (pre-advance vs post-advance) on `StepContext` so a stage can assert which host it is running under.

**Adversarial check.** Both orderings are verbatim and they do differ. march_run.rs:361-374: `coupling.apply(&ctx, field)?;` then `solver.advance(...)`, with `ctx` built from the *pre*-advance state and step index s+1. carrier.rs:132-146 `coupled_step`: `pre_step`, `advance`, `publish_and_transport`, then `coupling.apply` with step index s+1 — the field carries post-advance published scalars. So a stage keyed on `ctx.step()` fires at a different physical instant on the two hosts, and a stage reading a published scalar sees state from opposite sides of the advance. Confirmed also that the seam is silent about it: the coupling.rs module doc (:6-17) says only 'run once per timestep *around* the CFD step' — 'around' fixes nothing — and `AeroBlackoutStub` (coupling.rs:506-517) is indeed step-keyed on a half-open `[window_start, window_end)`. Downgraded to minor: each host documents its own ordering at its call site (march_run.rs:361 'Run the between-step coupling, then advance'; carrier.rs:125-127 'the pre-step hook, advance, publish/transport, then apply the between-step coupling'), so the gap is a missing portability warning at the trait, not an undocumented behaviour.

> Evidence re-read: march_run.rs:361-374 (advance_coupled: apply then advance, verbatim); carrier.rs:125-146 (coupled_step doc + body: advance, publish, then apply, verbatim); coupling.rs:6-17 (module doc, 'around the CFD step'); coupling.rs:43-97 (StepContext::new vs ::qtt, step index); coupling.rs:506-517 (AeroBlackoutStub step-keyed window)

---

### 14.11 [MINOR] MMS sample point is unvalidated: at a zero of the manufactured field the residual is identically zero and the amplitude march divides by zero

- **Verification verdict:** CONFIRMED
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/src/types/flow/verify.rs:84`
- **Auditor confidence:** confirmed

**Claim.** `VerifyConfigBuilder::sample_at` accepts any point with no non-degeneracy check. At p = (0,0,0) the Taylor-Green velocity, its Laplacian, and its pressure gradient all vanish, so `mms_error` is exactly zero for any kernel that returns zero on zero inputs; and `u0_sq` is zero, so the amplitude march divides by zero and reports NaN with no error.

**Code evidence.**

```
verify.rs:84, 95:
            let u0_sq = u0[0] * u0[0] + u0[1] * u0[1] + u0[2] * u0[2];
            ...
                (accel[0] * u0[0] + accel[1] * u0[1] + accel[2] * u0[2]) / u0_sq

manufactured.rs:118-123 (u = sin x cos y, v = -cos x sin y; both zero at the origin):
    match comp {
        0 => p[0].sin() * p[1].cos() * f,
        1 => -(p[0].cos() * p[1].sin() * f),

manufactured.rs:241-245 (no validation of the point):
    pub fn sample_at(mut self, point: [R; 3], t: f64) -> Self {
        self.point = Some(point);
        self.t = t;
        self
    }
```

**Reference form.** MMS requires the residual to be evaluated where the manufactured solution and its derivatives are non-degenerate; a sample point at which every kernel term vanishes provides no discrimination. Concretely at p=(0,0,0): u=(0,0,0); grad_p = (rho/2)(sin 2x, sin 2y) = (0,0,0); lap = -2u = (0,0,0); the convective term u_j*du_i/dx_j = 0 since u = 0. So rhs = 0 and reference = -2*nu*0 = 0, giving mms_error = 0 for any implementation.

**Impact.** The crate's headline verification observable can be made to pass vacuously by a one-line config change, with no diagnostic distinguishing it from a genuine pass. Separately, `amplitude_final` silently becomes NaN (0/0) rather than returning an error, and NaN compared against `amplitude_exact` with a tolerance yields false — a confusing failure mode rather than a named one.

**Recommended fix.** In `VerifyConfigBuilder::build`, sample the manufactured solution at the requested point and reject it when the velocity norm or the residual scale is below a documented threshold, returning `PhysicsError::PhysicalInvariantBroken("MMS sample point is a zero of the manufactured field")`. Separately, guard `u0_sq` in verify.rs:84 and return an error rather than producing NaN.

**Adversarial check.** Code verbatim: verify.rs:84 (`u0_sq`) and :95 (division by it); manufactured.rs:241-245 (`sample_at` stores the point with no check) and :118-123 (u = sin x cos y, v = −cos x sin y). `build()` (manufactured.rs:258-264) validates only that a point was *set*, never that it is non-degenerate. I re-derived the origin case: u(0,0,0) = (0,0,0); ∇²u = −2u = 0; ∇p = −(ρ/2)(sin 2x, sin 2y)·f² = 0 (the auditor's sign is off but the value at the origin is zero either way); the convective term vanishes with u; so kernel = 0 and exact_time_derivative = −2ν·0 = 0, giving mms_error = 0 for any kernel of this form. And u0_sq = 0 makes the RK4 rate 0/0 = NaN, so `amplitude_final` is NaN with no error. Downgraded to minor: reaching the degenerate state requires a deliberate one-line config choice, not a default (`point` has no default and is required), so this is a missing input precondition rather than a defect in a shipped result.

> Evidence re-read: verify.rs:73-104 (amplitude march, u0_sq at 84, division at 95, no guard); manufactured.rs:113-123 (tg_velocity, zero at the origin); :127-134 (tg_pressure = (ρ/4)(cos2x+cos2y)f² ⇒ ∇p = 0 at the origin); :239-245 (sample_at, no validation); :255-265 (build validates presence only)

---

### 14.12 [INFO] README claims the fork shares the navigation engine and provenance log in O(1); both are unconditionally deep-cloned before the branch's first step

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/src/types/flow/carrier.rs:842`
- **Auditor confidence:** confirmed

**Claim.** The fork operation itself is O(1) (two Arc::clones) and the README's 'through copy-on-write' correctly signals that a write costs a clone. In practice the field clone is unavoidable rather than conditional: run_continued_segment always appends a branch audit entry before the first step, forcing `Arc::make_mut` on the CoupledField and so an O(cells) copy of the scalar vectors, nav engine, and log per branch. The marched tensor state is never cloned. A reader sizing a large fan-out from the README's O(1) phrase would under-predict per-branch memory.

**Code evidence.**

```
carrier.rs:839-850:
    // Field: the branch's one CoW clone happens at the first write (merging the audit log).
    let mut field_arc = field;
    {
        let field = Arc::make_mut(&mut field_arc);
        field.log_mut().append(&mut branch_log);
        field.log_mut().add_entry(&alloc::format!(
            "march resumed at step {} for {} steps in world '{}'",

coupling.rs:138-147 (what the field holds):
pub struct CoupledField<R: CfdScalar> {
    ambient: Ambient<R>,
    scalars: Vec<(String, Vec<R>)>,
    ...
    nav: Option<ReentryNavEngine<R>>,
    log: EffectLog,
}

compressible_march_run.rs:445-448 (per-cell scalars sized to the grid):
        field.set_scalar("speed", speed);
        field.set_scalar("T_tr", t_tr);
        field.set_scalar("n_tot", n_tot);
        field.set_scalar("pressure_atm", p_atm);
```

**Reference form.** O(1) fork means per-branch cost independent of state size. The compressible carrier publishes four dense per-cell scalars each step, so on a 2^6 x 2^6 grid the field carries ~16k scalars plus a 17x17 covariance; cloning that per branch is O(cells), not O(1). The marched tensor half of the claim is correct (verified separately).

**Impact.** An engineer sizing a large fan-out (hundreds of branch worlds, or a refinement round) from the README's O(1) claim under-predicts memory and per-branch setup time by a factor proportional to the grid. It does not affect any computed number.

**Recommended fix.** Amend the README sentence to say the marched tensor state is shared in O(1) and the coupled field is copy-on-write with the branch's clone taken at the resume-log write. The precise rustdoc already exists at carrier.rs:776-777 ("The first field write performs the branch's single copy-on-write clone") — the README should not extend it to O(1) for the field's contents.

**Adversarial check.** The mechanism is verbatim: run_continued_segment (carrier.rs:839-850) calls `Arc::make_mut(&mut field_arc)` before the step loop to append the branch log entry, and since the pause still holds the Arc the strong count is >1, so the clone always happens. CoupledField does carry the per-cell scalar vectors, the nav engine, and the log (coupling.rs:137-147), and the compressible carrier publishes four dense per-cell scalars each step (compressible_march_run.rs:445-448), so the clone is O(cells). But the finding misreads the README. README.md:68 reads 'A fork shares the paused state in O(1) through copy-on-write (tensor fields, navigation engine, and provenance log included)' — the O(1) is attributed to the *fork*, which is literally two `Arc::clone`s (carrier.rs:522-533, 625-626), and 'through copy-on-write' is the explicit caveat that a write costs a clone. Appending the mandatory branch-log entry *is* that first write. The code comment at carrier.rs:839 says exactly this. So the README is accurate as written; what is true and worth noting is that the field's CoW clone is unavoidable in practice because the audit entry always writes, so per-branch cost is O(cells) even though the fork itself is O(1). The marched tensor state genuinely is never cloned.

> Evidence re-read: carrier.rs:520-533 (fork = two Arc::clone, O(1)); :622-631 (same in continue_with_sampled); :839-850 (unconditional Arc::make_mut before the loop, comment 'the branch's one CoW clone happens at the first write'); :853-864 (loop reuses field_arc, state Arc replaced not cloned); coupling.rs:137-147 (CoupledField fields); compressible_march_run.rs:445-448 (four dense per-cell scalars); README.md:68

---

### 14.13 [INFO] ForkEconomics::is_o1() is a gate that cannot fail at runtime, and carrier.rs documents it as if it could

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/src/types/flow/carrier.rs:622`
- **Auditor confidence:** confirmed

**Claim.** `ForkEconomics::is_o1()` is a compile-time tautology usable only as a source-change tripwire, not a runtime gate — accurately documented as such on the method itself. The claimed contradiction between the carrier.rs construction-site comment and report.rs does not exist: both describe it as a guard against a future source change, so the only residual risk is a study author treating is_o1() as evidence.

**Code evidence.**

```
carrier.rs:622-631:
        // The O(1) fork, and the record of it. Taken from the clones actually handed to the branch,
        // not asserted about them: if this path ever deep-copies instead of sharing, the recorded
        // economics say so and a study gating on `is_o1` fails rather than passing on a stale claim.
        let branch_state = Arc::clone(&self.state);
        let branch_field = Arc::clone(&self.field);
        let economics = ForkEconomics::new(
            Arc::ptr_eq(&branch_state, &self.state),
            Arc::ptr_eq(&branch_field, &self.field),
            sample,
        );

report.rs:80-86 (the honest statement):
    /// This is a guard against a source change, not a run-time measurement. Both conjuncts compare a
    /// clone against the `Arc` it was cloned from, so no input can falsify them
```

**Reference form.** A verification gate must be capable of failing on some admissible input. `Arc::ptr_eq(&Arc::clone(&x), &x)` is a tautology at the type level.

**Impact.** A study whose acceptance gate reads `report.fork_economics().unwrap().is_o1()` reports PASS unconditionally and contributes no evidence. The value as a source-change tripwire is real but requires the source to change and the test to be recompiled, which is a lint, not a runtime gate. The contradiction between the two comments will mislead the next reader of carrier.rs.

**Recommended fix.** Replace the carrier.rs:622-624 comment with report.rs's wording, or delete `is_o1` from the public gate surface and keep the genuine measurements (`fork_peak_bond`, `Report::bond_growth`) that do vary with the run. If a runtime O(1) check is wanted, compare `fluid_refs` before and after the fan-out, or add a debug assertion on total allocated bytes.

**Adversarial check.** The tautology is real and I confirm it: `is_o1()` is `shares_fluid && shares_field`, both set from `Arc::ptr_eq(&Arc::clone(&x), &x)` at carrier.rs:625-631, which no input can falsify. But the second half of the title — that carrier.rs 'documents it as if it could' fail, contradicting report.rs — is REFUTED. The carrier.rs comment reads '**if this path ever deep-copies instead of sharing**, the recorded economics say so and a study gating on is_o1 fails' — conditioned on a source change, which is precisely what report.rs:80-86 says ('a guard against a source change, not a run-time measurement ... a future edit that materializes the state instead of sharing it flips them'). The two comments agree; there is no contradiction to mislead the next reader. What remains is the accurate observation that a study using is_o1() as an acceptance gate contributes no runtime evidence — already stated on the method itself.

> Evidence re-read: carrier.rs:622-631 (construction site comment, verbatim — note the conditional 'if this path ever deep-copies instead of sharing'); report.rs:78-91 (is_o1 rustdoc, verbatim, same source-change-tripwire framing, and points to fork_peak_bond / bond_growth as the measurements that do vary)

---

### 14.14 [MINOR] Verdict::passed() returns true for an empty gate set, so a study whose gates were never attached reports PASS

- **Verification verdict:** CONFIRMED
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/src/types/flow/study/verdict.rs:90`
- **Auditor confidence:** confirmed

**Claim.** `Verdict::passed()` is `outcomes.iter().all(GateOutcome::passed)`, which is true for an empty slice. A `GateSeq::new("x")` with no `.gate(..)` calls, or a `gates()` call whose sequence was built but never populated, produces a verdict that reports PASS with no outcome lines.

**Code evidence.**

```
verdict.rs:88-91:
    /// Whether every gate passed. An empty gate set passes vacuously.
    pub fn passed(&self) -> bool {
        self.outcomes.iter().all(GateOutcome::passed)
    }

gate_seq.rs:29-35 (an empty sequence is constructible and insertable):
    pub fn new(title: &str) -> Self {
        Self { title: title.to_string(), gates: Vec::new() }
    }
```

**Reference form.** In a certification context a verdict with zero evidence must not be indistinguishable from a verdict with satisfied evidence. The sibling `Gates::finish` at least prints "no gates registered" (gates.rs:53-54); `Verdict::Display` prints only the title and the pass banner.

**Impact.** A refactor that drops a gate from a `GateSeq`, or a `model::corridor_gates()` that returns an unpopulated sequence, turns a certification study into an unconditional PASS with output that looks like a successful run. The behaviour is documented on the method, but the failure is silent at the call site.

**Recommended fix.** Either make `passed()` return false for an empty outcome set, or have `Display` and `verdict()` surface the vacuity — e.g. push a `StudyWarning` when `seq.check` produces zero outcomes, since the warning channel already exists (study_effect/mod.rs:79-82) and renders in the verdict (verdict.rs:110-112).

**Adversarial check.** Verbatim at verdict.rs:88-91 (`self.outcomes.iter().all(GateOutcome::passed)`, vacuously true on an empty slice) and gate_seq.rs:29-35 (`GateSeq::new` starts with an empty `gates` Vec; `check` maps over it into `Verdict::new`, so an empty sequence yields an outcome-free Verdict). The asymmetry with the sibling is real: `Gates::finish` prints '=== {title}: no gates registered. ===' (gates.rs:53-55) whereas `Verdict`'s Display (verdict.rs:103-121) writes only the title, no outcome lines, then '=== All gates passed ===' — indistinguishable at a glance from a satisfied verdict. The behaviour is documented on the method ('An empty gate set passes vacuously'), which is why this stays minor rather than rising.

> Evidence re-read: verdict.rs:88-91 (passed()); :103-122 (Display — no empty-set branch, prints the pass banner); gate_seq.rs:29-35 (empty construction) and :50-60 (check maps gates into Verdict::new); gates.rs:53-55 (sibling prints 'no gates registered')

---

### 14.15 [INFO] README claims the DSL never prints, but Gates::finish writes to stdout and is re-exported at the crate root

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/src/types/flow/gates.rs:44`
- **Auditor confidence:** confirmed

**Claim.** The README's 'never exits or prints' is written about the study grammar's verdict path, which does hold: GateSeq/Verdict return data and Display is opt-in. `Gates`, a separate helper whose own module doc announces that it prints, is the crate's only stdout writer (5 println! lines) and is re-exported at the crate root, so the sentence could be read as a crate-wide purity claim it does not make. Scoping the sentence would remove the ambiguity.

**Code evidence.**

```
gates.rs:43-62:
    pub fn finish(self) -> bool {
        println!("--- {} ---", self.title);
        ...
            println!("  [{}] {label}: {detail}", if *pass { "PASS" } else { "FAIL" });
        ...
            println!("=== {}: no gates registered. ===", self.title);
        } else if all {
            println!("=== All gates passed: {}. ===", self.title);

src/lib.rs:95 (crate-root re-export):
    ForkEconomics, Gates, GoverningModel, IGNITION_COMMIT_AIDED_FIELD, ...

README.md:104:
the DSL never exits or prints (`verdict()` returns data)
```

**Reference form.** A library-purity claim must hold for the whole exported surface, or be scoped to the part it holds for. Grep of src/ for println!/eprintln!/process::exit returns exactly these five lines and nothing else, so the claim is accurate for every path except this type.

**Impact.** Minor: an integrator embedding the crate in a service and relying on the no-print guarantee finds one type that violates it. The exit half of the claim is fully correct.

**Recommended fix.** Scope the README sentence to the study path ("the study grammar never exits or prints; `verdict()` returns data, and the printing helper `Gates` is opt-in"), or move `Gates` behind a `std`-gated `reporting` module so the no-print surface is structurally identifiable.

**Adversarial check.** The facts are right: gates.rs:43-62 contains five println! calls, `Gates` is re-exported at src/lib.rs:95, and my grep of src/ for println!/eprintln!/process::exit returns exactly those five lines and nothing else — so the auditor's survey is accurate and complete. But the README sentence is scoped, not global. README.md:104 reads 'The gating sequence is a named value the study inserts whole (`GateSeq<Row>`), the DSL never exits or prints (`verdict()` returns data)' — the subject is the study grammar and the parenthetical names `verdict()` as the thing that returns data instead of printing. That path (GateSeq → Verdict) genuinely never prints; Display is opt-in. `Gates` is a separate, explicitly-documented helper whose module doc opens 'the `[PASS]`/`[FAIL]` block every self-verifying program prints, as one type' — it announces printing as its purpose and is not part of the verdict path. A reader could still take the sentence as a crate-wide purity claim, which is the residual defect.

> Evidence re-read: README.md:100-105 (full sentence in context of the GateSeq/verdict paragraph); gates.rs:6-12 (module doc: 'the block every self-verifying program prints'); :43-62 (five println!); src/lib.rs:90-100 (Gates in the crate-root re-export list); grep println!/eprintln!/process::exit over src/ — 5 hits, all in gates.rs

---

### 14.16 [INFO] README and flow_config module doc claim "type-state builders"; every required config field is a runtime-checked Option

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/src/types/flow_config/march_builder.rs:156`
- **Auditor confidence:** confirmed

**Claim.** The flow_config builders carry real type-state on their composition axes — `zones()` and `couple()` transition the builder's type parameters — so the README's 'type-state builders' label is accurate for what it names. It does not extend to the required scalar sections (mesh, solver, grid, seed, reference, flight_dt, stop), which remain Options validated at build() and reported as a Result. The label is incomplete rather than wrong.

**Code evidence.**

```
march_builder.rs:29-30, 156-164:
    mesh: Option<Mesh<D, R>>,
    solver: Option<DecNsConfig<R>>,
    ...
        let mesh = self.mesh.ok_or_else(|| { PhysicsError::DimensionMismatch("CfdConfigBuilder::march: a mesh is required".into()) })?;
        let solver = self.solver.ok_or_else(|| { ... })?;

compressible_march_config.rs:498-518:
        let (lx, ly, dx, dy) = self.grid.ok_or_else(|| missing("grid"))?;
        let (dt_solver, s_ref, gamma, trunc) = self.solver.ok_or_else(|| missing("solver"))?;
        ... dt_flight: self.dt_flight.ok_or_else(|| missing("flight_dt"))?,
            seed: self.seed.ok_or_else(|| missing("seed_fn"))?,

qtt_march_config.rs:303-311: same Option-and-ok_or_else pattern.
```

**Reference form.** A type-state builder makes an invalid configuration unrepresentable: the terminal `build()` is only defined on a state reached by setting every required field, so it returns the config directly rather than a Result. `CfdConfigBuilder::dec_ns()` returning `DecNsConfigNeedsViscosity` (cfd_config_builder.rs:25-27) is the real thing; the flow_config builders are not.

**Impact.** An evaluator reading the README expects the compile-time guarantee the study phases actually deliver and assumes it extends to configuration. It does not: a forgotten `.mesh(..)` or `.flight_dt(..)` is a runtime error, and in the `MarchStop::Fixed(0)` case (reported separately) not even that.

**Recommended fix.** Restate the README row as "owned config containers, fluent builders validated at build(), and type-state phase transitions for the zone and coupling tuples". Alternatively convert the required fields to real type-states following the `DecNsConfigNeedsViscosity` pattern already in src/solvers/.

**Adversarial check.** The code facts hold: march_builder.rs:29-30 and :156-164 use Option + ok_or_else for mesh and solver, and compressible_march_config.rs:498-518 does the same for grid, solver, flight_dt, seed_fn, stop, and reference. But the README label is not false. README.md:253 says 'owned config containers and type-state builders', and MarchConfigBuilder *is* a type-state builder in the precise sense: it is parameterized on `Z: BoundaryZone` and `C: PhysicsStage`, and `zones()`/`couple()` return a builder of a *different type* (march_builder.rs:62-93). The auditor concedes this, plus the genuine study-phase typestates with compile_fail doctests. So the claim is that a correct label is incomplete rather than wrong — the type-state applies to the composition axes, not to the required scalars. That is a documentation-precision point, not a defect.

> Evidence re-read: march_builder.rs:22-38 (builder generic over Z and C); :62-93 (zones/couple return MarchConfigBuilder<D,R,Z2,C> / <D,R,Z,C2> — genuine state transitions); :156-164 (mesh/solver Option validation); compressible_march_config.rs:498-518 (missing("grid"/"solver"/"flight_dt"/"seed_fn"/"stop"/"reference")); README.md:253

---

### 14.17 [MINOR] Grading::cosine accepts an unvalidated amplitude and axis, admitting a degenerate metric or an index panic

- **Verification verdict:** CONFIRMED
- **Axis:** physics-math
- **Location:** `deep_causality_cfd/src/types/flow_config/mesh.rs:50`
- **Auditor confidence:** confirmed

**Claim.** `Grading::cosine(axis, amp)` performs no validation. An `amp >= 1` produces edge lengths `h*(1 + amp*cos(..))` that are zero or negative at `cos = -1`, and `CubicalReggeGeometry::from_edge_lengths` stores them without a positivity check, yielding a degenerate or sign-flipped metric with no error. An `axis >= D` panics with an index-out-of-bounds inside `base_geometry`.

**Code evidence.**

```
mesh.rs:49-52:
    /// A periodic cosine grading on `axis` with relative amplitude `amp`.
    pub fn cosine(axis: usize, amp: R) -> Self {
        Grading::Cosine { axis, amp }
    }

mesh.rs:157-166 (unchecked axis index, unchecked amp):
                let n =
                    R::from_usize(self.shape[*graded_axis]).expect("a lattice extent lifts into R");
                ...
                            self.spacing * (R::one() + *amp * (two_pi * pos / n).cos())

deep_causality_topology/src/types/cubical_regge_geometry/mod.rs:199-207 (no validation):
    pub fn from_edge_lengths(lengths: Vec<R>) -> Self {
        Self { edge_lengths: EdgeLengths::PerEdge { lengths }, ... }
    }
```

**Reference form.** A Regge metric requires strictly positive edge lengths; a zero or negative edge length makes the Hodge star singular or orientation-reversing, so the discrete Laplacian and the Leray projection are no longer well defined. The admissible amplitude range is |amp| < 1.

**Impact.** A grading study run with amp = 1.0 produces a manifold whose diagonal Hodge star divides by zero, propagating inf/NaN into the marched field with no error at configuration time. `axis >= D` is a library panic reachable from public input, which is a hard failure mode for an embedded consumer.

**Recommended fix.** Return `Result<Grading<R>, PhysicsError>` from `cosine`, rejecting `axis >= D` and `!(amp.abs() < R::one())`. If the signature must stay infallible, move both checks into `Mesh::materialize`, which already returns `Result`, and name them in the `# Errors` section of `Mesh::manifold`.

**Adversarial check.** Verbatim at mesh.rs:48-53 (no validation of either argument; the constructor is infallible and returns Self, not Result) and mesh.rs:149-169 (`self.shape[*graded_axis]` at :157-158 is an unchecked index — a panic for axis >= D, reachable from public input via `Mesh::graded`; edge length `spacing·(1 + amp·cos(2π·pos/N))` at :166 with no positivity check). I verified the degeneracy is exactly attainable rather than asymptotic: cos(2π·pos/N) = −1 at pos = N/2, an integer lattice position for even N, so amp = 1.0 produces an edge length of exactly zero and amp > 1 produces negative lengths. The reference form is correct — a Regge metric needs strictly positive edge lengths, admissible range |amp| < 1 — and `CubicalReggeGeometry::from_edge_lengths` stores them unvalidated as cited. The `axis >= D` panic in a `forbid(unsafe_code)` library reachable from public input is the sharper half of this.

> Evidence re-read: mesh.rs:37-53 (Grading enum + infallible cosine constructor); :77-81 (Mesh::graded, no validation); :147-169 (base_geometry: unchecked shape[*graded_axis] at 157-158, edge length formula at 166); deep_causality_topology/src/types/cubical_regge_geometry/mod.rs from_edge_lengths (no positivity check)

---

### 14.18 [INFO] Mach 1.05 shock floor selecting between the Rankine-Hugoniot jump and the freestream is an uncited threshold

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** magic-number
- **Location:** `deep_causality_cfd/src/types/flow/compressible_march_run.rs:327`
- **Auditor confidence:** confirmed

**Claim.** The Mach 1.05 shock-fitting floor is a bare, uncited, unconfigurable literal that selects the inflow-strip state; a note giving its rationale (a numerical guard above the M = 1 singular limit) and a named constant would close the gap. The comment says 'below Mach ~1', an approximation consistent with a small guard band, so the auditor's claim that the comment names a different number than the code is not supported.

**Code evidence.**

```
compressible_march_run.rs:326-334:
        // The exact RH jump when a shock stands; the freestream itself below Mach ~1.
        let shock_floor = Self::lift(1.05)?;
        let (t_post, n_post, u_post) = if mach > shock_floor {
            let shock = FittedNormalShock::new(schedule.gamma_eff)?;
            let post = shock.post_shock(row.temperature, row.n_tot, mach)?;
            (post.t2, post.n_tot2, speed * post.u_ratio)
        } else {
            (row.temperature, row.n_tot, speed)
        };
```

**Reference form.** A normal shock exists for M > 1; the Rankine-Hugoniot relations reduce to the identity as M -> 1+ (Anderson, Modern Compressible Flow, ch. 3). The physically motivated threshold is 1.0. Any margin above it is a numerical guard whose value should be stated and justified, and the comment should name it.

**Impact.** Between M = 1.0 and M = 1.05 the carrier enforces the undisturbed freestream on the inflow strip rather than the (weak) post-shock state, so the marched layer's inflow is wrong in that band. The physical error is small because the jump is weak there, but the threshold is undocumented, unconfigurable, and its comment states 1.0 while the code uses 1.05.

**Recommended fix.** Either use 1.0 and rely on the RH relations' own degeneracy at M = 1, or keep the guard, correct the comment to name 1.05, state why 5% is the right margin (e.g. conditioning of `FittedNormalShock` near the sonic point), and expose it as a `DescentSchedule` field alongside `rebuild_tol` and `strip_cols`.

**Adversarial check.** The code is verbatim at compressible_march_run.rs:326-334 and the substantive half stands: 1.05 is a bare literal with no named constant, no citation, no configuration hook, and it selects which state the shock-fitted inflow strip enforces in the 1.0 < M <= 1.05 band. The auditor's physics is right (the RH relations reduce to the identity as M → 1+, so the physically motivated threshold is 1.0 and any margin is a numerical guard). But the specific charge that 'the comment states 1.0 while the code uses 1.05' is not supportable: the comment reads 'the freestream itself below Mach ~1' — a tilde-qualified approximation, consistent with a small guard band, not a competing numeric claim. Since the jump is vanishingly weak there, the physical error in the band is second-order; this is a traceability gap, not a wrong result.

> Evidence re-read: compressible_march_run.rs:324-334 (mach computed at 324; comment at 326 reads 'the freestream itself below Mach ~1'; `let shock_floor = Self::lift(1.05)?;` at 327; branch at 328-334); grep confirms no named constant or config setter for the value

---

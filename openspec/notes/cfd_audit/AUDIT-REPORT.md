<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# `deep_causality_cfd` — Pre-Certification Audit

**Date:** 2026-07-21
**Scope:** `deep_causality_cfd/` (src, tests, verification, studies, benches) and `examples/avionics_examples/cfd/`
**Purpose:** Establish whether the crate can be certified for avionics **R&D** use — whether an engineer
can trust that each marcher and kernel computes what the specification and the reference formula say it
computes.

> **Status: NOT YET CERTIFIABLE. Phase 1 complete; Phase 2 outstanding.**
>
> The crate is not broken. Its numerical core is, in every place checkable against a closed-form
> reference, *exactly* right — including a lid-driven-cavity primary vortex matching Ghia (1982) to four
> decimal places, and Rankine–Hugoniot relations exact to displayed precision.
>
> What blocked certification was the **assurance layer**, not the mathematics: a large fraction of the
> gates advertised as proving the mathematics could not fail or could not discriminate, and none of them
> ran in CI.

> ### Remediation status — updated 2026-07-21
>
> **Phase 1 is implemented and archived** as
> [`openspec/changes/archive/2026-07-21-make-cfd-evidence-enforceable/`](../../changes/archive/2026-07-21-make-cfd-evidence-enforceable/),
> 43/43 tasks. Its four capability specs are synced into `openspec/specs/`.
>
> | Blocker | Status |
> |---|---|
> | **B-1** Millikan–White reduced mass | **open** — Phase 2 |
> | **B-2** No CI executes the verification suite | **RESOLVED** |
> | **B-3** `dec_cylinder_verification` has no gate | **RESOLVED** |
> | **B-4** `BlendedMap` documents an absent fold check | **open** — Phase 2 |
>
> What Phase 1 changed, and what it found beyond this report:
>
> - CI now runs the suite: nine fast harnesses per PR (12.7 s), four nightly, with a completeness
>   check so a new harness cannot escape both lists.
> - **Two further instances of B-3 surfaced.** `mms_taylor_green_verification` had only a setup-error
>   exit and never checked its measured residual; `dec_graded_mms_verification` had **zero**
>   `process::exit` calls at all. Both were on the per-PR list — CI would have run them every PR and
>   learned nothing. Both now gate against their analytic references.
> - Seven unfalsifiable gates repaired, each demonstrated failing by fault injection rather than
>   asserted. Every gate now carries a `[reference]` / `[tripwire]` evidence class: **38 gate lines,
>   0 unlabelled.**
> - **Baselines were worse than §3.2 reported.** Auditing for truncation found only the lid cavity
>   broken; the stricter "carries a verdict" check found **6 of 12** baselines carried none. All 13
>   harnesses now have a baseline and every one carries a verdict.
> - **One new physics finding, from a gate that did not previously exist** — see §5b.
> - **One finding in this report is refuted** — see §5.

---

## 1. Bottom line

**1. The physics core is sound, and in places excellent.** Independently re-derived: Rankine–Hugoniot
jump relations, plasma frequency, Sod exact-Riemann star state, RK4 tableau, the Hodge–de Rham Laplacian
and its adjoint structure, and graded-mesh convergence orders — all correct. The DEC lid-driven cavity
reproduces Ghia's primary and both corner vortices. 813 unit tests pass. Committed outputs reproduce.

**2. The verification layer does not verify what it claims to verify.** Of 294 findings, **72 are
tautology/circular-reasoning defects**. Several headline gates are algebraically incapable of failing;
others can fail in principle but are set orders of magnitude above the phenomenon they measure. For the
QTT immersed-cylinder harness specifically, **no gate constrains drag correctness** — one gate cannot
fail, one is provably invariant under the parameter that moves the answer 6×, and one has eleven orders
of margin.

**3. The evidence never runs.** No CI job, script, or Bazel target executes any of the 13 verification
programs. `cargo test` compiles them and never runs them. Every quantitative accuracy claim in the crate
README and `verification/README.md` is unenforced and can silently rot.

This is a **strong research instrument with a weak assurance case**. The remediation list is long but
almost entirely mechanical: make claims match reality and make gates bite. No solver rewrites.

---

## 2. Method, and its limits

### 2.1 What was done

| Activity | Extent |
|---|---|
| Independent module audits | **16** parallel auditors, one per subsystem |
| Adversarial re-check | **16 / 16 modules** (two re-run after their first verifiers hit a session limit) |
| Distinct source files read | **412** |
| Findings raised | **294** |
| Findings adversarially re-checked | **294 (100 %)** |
| Items positively confirmed correct against a reference | **176** |
| Unit tests executed | 813 pass / 0 fail / 2 ignored |
| Verification harnesses executed | 13 / 13, all exit 0 |
| Avionics examples executed | 7 / 7, all exit 0 |
| Findings independently re-verified at source by the lead auditor | 10 (9 upheld, **1 corrected — see B-6**) |

### 2.2 Verification outcome

| Verdict | Count |
|---|---|
| CONFIRMED | 190 |
| PARTIALLY_CONFIRMED (real, but severity or mechanism corrected) | 100 |
| REFUTED (not a defect) | 4 |

**Severity before vs after adversarial review:**

| Severity | As raised | After review |
|---|---|---|
| critical | 26 | **4** |
| major | 131 | **72** |
| minor | 126 | 179 |
| info | 11 | 35 |

The adversarial stage did substantial work. Outright refutation stayed rare (4 of 294), but **severity
correction was extensive** — criticals fell 85 % and majors 45 %, as overstated findings were
reclassified rather than rejected. Two of the re-run verifiers went further and **executed probes**
against the shipped code to reproduce or disprove the auditors' numbers, which is how the strongest
refutation in this audit was obtained (B-6). All numbers in this report are **post-review**.

### 2.3 Limitations — read before acting

- **Severity labels were not normalized across modules.** Compare within a module, not across.
- **Runtimes are not comparable to the README.** These ran alongside 16 concurrent agents; the lid cavity
  took 1407 s against a documented ~28 s at a different grid. Contention, not regression.
- **The 4 criticals in §4 are asserted as certain.** The 72 majors are verified but should be triaged;
  each carries its adversarial verdict and evidence in the per-module reports.
- **One lead-auditor blocker was itself corrected** during re-verification (B-6). That is recorded rather
  than quietly dropped, because it is the clearest evidence that the adversarial stage was functioning.

---

## 3. Evidence base (first-hand)

### 3.1 Test suite

```
cargo test -p deep_causality_cfd --release
→ 813 passed; 0 failed; 2 ignored
```

### 3.2 Execution ledger

All 13 verification harnesses and 7 avionics examples were built in `--release` and executed; every one
exited 0. Studies were executed separately. Full ledger: [`RUN-LEDGER.md`](RUN-LEDGER.md).

**That result is weaker than it looks.** Several exit-0 outcomes come from gates that cannot fail, and
`dec_cylinder_verification` has no gate at all and exits 0 even after a solver error.

### 3.3 Reproducibility of committed outputs — PASS

| Example | Fresh run vs committed `output.txt` |
|---|---|
| `viv_resonance_margin` | byte-identical |
| `flight_envelope_placard` | identical but for trailing newline |
| `nozzle_operating_map` | identical but for trailing newline |
| `plasma_blackout_corridor` | identical but for the wall-clock line (40.9 s committed vs 48.2 s under load) |

Committed evidence is **fresh and deterministic**. A genuine and uncommon strength.

### 3.4 Reference formulas re-derived by hand — PASS

`qtt_ramc_stagline`, M=25, γ=1.1, T₁=250 K:

| Quantity | Reference (re-derived) | Code | Verdict |
|---|---|---|---|
| ρ₂/ρ₁ = ((γ+1)M²)/((γ−1)M²+2) | 20.3488 | 20.349 | exact |
| p₂/p₁ = (2γM²−(γ−1))/(γ+1) | 654.7 | 6.547e2 | exact |
| u₂/u₁ = ρ₁/ρ₂ | 0.0491 | 0.049 | exact |
| T₂ = T₁·(p₂/p₁)(ρ₁/ρ₂) | 8043.6 K | 8044 K | exact |
| ωₚ = √(nₑe²/ε₀mₑ), nₑ=1.085e19 | 1.858e11 rad/s | 1.858e11 | exact |
| Sod p\* (γ=1.4, standard IC) | 0.30313 | 0.3031 | exact |

**The Rankine–Hugoniot implementation — the crate's central compressible claim — is exactly correct.**

### 3.5 DEC lid-driven cavity vs Ghia (1982) — PASS, better than documented

Default configuration (65², t=100), measured this run:

| Quantity | Measured | Ghia (1982) | README summary row (33², t=40) |
|---|---|---|---|
| Primary vortex | (0.5312, 0.5625) | (0.5313, 0.5625) | (0.563, 0.594) |
| Bottom-right eddy | (0.8594, 0.1094) | (0.8594, 0.1094) | — |
| Bottom-left eddy | (0.0781, 0.0781) | (0.0859, 0.0781) | — |
| Centerline RMSE | **0.0617** | — | 0.137 |

Primary vortex matches to 1e-4 in x and exactly in y; bottom-right eddy matches exactly. The README's
headline row reports a coarse 33² *trend* configuration while other rows report defaults — an
inconsistent basis that **understates** the solver. Fix the table, not the solver.

---

## 4. Certification blockers

Four findings survived review at critical severity. All four are CONFIRMED and were additionally
re-read at source by the lead auditor.

### B-1 — Millikan–White reduced mass is wrong for the documented collision pair
`verification/qtt_ramc_stagline/config.rs:35-37`, `src/solvers/qtt/compressible/fitting.rs:72-73`

```rust
/// Reduced mass `μ_sr` of the dominant relaxing collision pair (N₂–N₂ ≈ 14·14/28 = 7), in amu
pub const REDUCED_MASS_AMU: f64 = 7.0;
```

The derivation in the comment is arithmetically wrong. N₂ has mass **28** amu, not 14. The reduced mass
of an N₂–N₂ pair is 28·28/56 = **14 amu**. The value 7 amu is the **N–N atom** pair, mislabelled.

Since `A_sr = 1.16e-3·μ^(1/2)·θ_v^(4/3)` and μ enters inside the exponential, a factor-2 error in μ
shifts τ_vt substantially — moving T_ve, hence T_a = √(T_tr·T_ve), hence peak electron density.

**The constant is duplicated and propagates beyond the harness.** Defined twice, both at 7.0, both
labelled N₂–N₂:

- `deep_causality_cfd/verification/qtt_ramc_stagline/config.rs:37`
- `examples/avionics_examples/src/shared/constants.rs:128`

and reaching the flagship plasma-blackout examples through
`examples/avionics_examples/src/shared/world.rs:165,345`. Corridor, weather and retropulsion all inherit
it.

**This is the most consequential finding in the audit.** The module report records that re-running at the
physically correct μ = 14 moves the prediction to roughly **−1.27 decades** against the RAM-C anchor —
i.e. correcting the constant *removes* the headline agreement rather than preserving it. (That figure is
from the module auditor and was not independently re-computed; re-measuring it is step one.)

*Action:* decide the intended pair. If N₂–N₂, set 14.0 at **both** sites, correct all four docstrings, and
re-baseline the RAM-C chain **and** the three plasma-blackout examples. If N–N was intended, correct the
label and justify why atom–atom dominates. Derive the gates from the corrected physics — **do not re-tune
the gates to restore the previous agreement.**

### B-2 — No CI or Bazel target executes the verification suite
`.github/workflows/run_tests.yml` — no `cargo run --example` anywhere

Every external-reference comparison in the crate (exact Riemann, Ghia cavity, Williamson cylinder, RAM-C
flight nₑ) lives in `verification/` binaries that CI compiles and never runs.

*Action:* run the fast harnesses (10 of 13 complete in ≤5 s) on every PR and the three slow ones nightly.
**This single change converts the entire README table from prose into a gate, and is the highest
value-per-effort item in this report.**

### B-3 — `dec_cylinder_verification` has no gate and cannot fail
`verification/dec_cylinder_verification/main.rs` — **0** `process::exit`, **0** assertions

On a solver `Err` it prints, breaks the march, then reports Strouhal and drag from the *truncated* series
and returns 0. `verification/README.md` states the opposite convention. This is the crate's most
expensive harness (~510 s) and its only isolated-cylinder validation.

*Action:* gate St and C_d against the stated Williamson / Dröge–Verstappen bands; exit nonzero on solver
error.

### B-4 — `BlendedMap` documents a fold check it does not perform
`src/coordinate/blended.rs:17,171`

The module doc states the constructor rejects a fold and that det J one-signedness holds "by
construction". No such check exists in `BlendedMap::new`. The inverse metric divides by `det_at(ξ,η)`
with no guard, so degenerate geometries yield inf/NaN or ~1e15-magnitude metric entries while the
constructor still returns `Ok`.

*Action:* implement the documented check, or remove the guarantee from the doc. Guard the division.

---

## 4b. Major findings worth blocker-level attention

These were downgraded from critical but are, in aggregate, the reason two modules remain `not-ready`.

**The QTT immersed-cylinder harness has no gate that constrains drag correctness.** Three verified
findings compose:
- `SMOOTH_CELLS = 2.0` is untraceable and moves the reported C_d **6.1×** (measured sweep: 7.70 → 12.33
  → 23.76 → 35.81 → 47.27 for 0.5 → 4 cells) while the no-slip gate is **provably invariant** across
  that entire range.
- `ETA = 0.016` is set by the explicit-stability ratio (`dt/eta = 0.25`), not by physics, and the
  measured C_d is **non-monotone** in it (17.4 → 26.2 → 21.4) with no sign of an η → 0 limit — so the
  Angot convergence theorem's precondition is never demonstrated.
- The Brinkman layer `√(ην) = 0.144·dx` is **unresolved by 48×** against the criterion `η ≥ dx²/ν`, and
  neither the code nor the README states the constraint.
- The bond-convergence gate — on which the README rests the entire verification claim — has a bound
  eleven orders of magnitude above the measured difference.

**Two ESKF defects compound.** `Q` is added with no `dt` scaling (`eskf.rs:105`), so covariance growth
tracks step count rather than elapsed time; and `update_scalar` divides by an unguarded innovation
covariance with no validation of the measurement variance, reachable from the public API
(`P[i][i]=0, r_var=0` → `k = NaN` written into state and covariance). To the code's credit the same
function uses a correct Joseph-form update citing Groves (2013) §3.4.3.

**`wall_heat_flux` is not a flux** — it is a temperature-weighted volumetric rate with no gradient,
conductivity or surface normal, and the production path hardcodes `t_wall = 0` with no way to configure
it. For a re-entry TPS consumer this is the safety-critical quantity.

**The QTT convection gate tests a re-implementation, not the shipped solver.** The harness open-codes the
convection assembly; the shipped `rate` path is never invoked by any gate, and the only tests touching it
pass `u = v = 0`. A sign flip in the shipped convection would be invisible.

**The committed lid-cavity `baseline.txt` is a truncated run** — 11 lines, stopping at t=44.99 of 14223
steps, no RMSE, no vortex table, no verdict. The only truncated baseline of the 12.

---

## 5. What the adversarial stage overturned

Recording these protects the fix list from wasted effort.

**B-6 (withdrawn) — "39.6 % of penalization drag comes from cells outside the body."** The lead auditor
endorsed this in an earlier draft. **It is refuted.** A re-run probe reproduced the band shares exactly
(8.3 / 17.9 / 34.2 / 39.6 %) but sub-decomposition showed the true far field (χ < 1e-4, 411 cells)
contributes **0.1 %**, not a domain-wide bias; 99.9 % of the "outside" contribution is the 1–3 cells of
the smoothing skirt. Concentration of the penalization integrand in the interface layer is *expected*
Brinkman behaviour, and `F = (1/η)∫χ(u−u_b)dV` over the whole domain **is** the correct smoothed-χ
realization of the Angot form. The real content — the force is dominated by a numerically-set skirt
width — is the `SMOOTH_CELLS` finding in §4b, and should be merged there rather than pursued separately.

**`CompressibleCarrier::finish` "drops the density-positivity guard."** Refuted — `finish` is reachable
only via `finish_report`, which runs only after every `coupled_step` has already called
`publish_and_transport` with `?`. A diverged march errors out rather than returning a NaN report.

**`ideal_gas_pressure_2d`'s `unwrap_or_else` is "inconsistent with surrounding code."** Refuted on its
premise — `unwrap_or_else(R::one)` is the crate's dominant numeric-lift idiom (17 sites, two more in the
same file); the `.expect` form is the minority. Reframe as crate-wide policy, not a local defect.

**The bond-convergence gate is "circular."** Misclassified — it can fail; the bond-4 row (24.05 vs 23.76)
shows the quantity is genuinely bond-sensitive. It is a *weak* gate, not a tautological one.

**The energy-budget gate is a tautology.** *(Overturned during Phase 1 implementation, 2026-07-21.)*
**Refuted — no change made.** The finding claimed the skew-symmetrisation makes
`tests/solvers/dec/energy_budget_tests.rs:141`'s asserted quantity algebraically zero for every input.
It does not: `energy_budget` computes `convective = −⟨u, Cu⟩_M` from the **actual operator output**,
not from a formula that is zero for any `C`. Tested by flipping the adjoint sign in
`fill_convective_skew_fused` so the operator is no longer skew — the assert fails with convective power
**0.0338** against ~0. The gate is a genuine guard that the shipped convective operator remains
skew-symmetric, which is the discrete analogue of convection redistributing but not creating kinetic
energy. The auditor conflated *"zero given the correct operator"* with *"zero regardless of the
operator"*; only the latter would be a tautology. This was the only Phase-1 item where the correct
action was to change nothing.

---

## 5b. New finding, from a gate that did not previously exist

*Added 2026-07-21 during Phase 1 implementation.*

Re-founding the `qtt_cylinder_verification` gate set (§4b) added an η ladder and a mask-smoothing
ladder. **Both fail**, and the failure is a real measurement about the configuration:

| Ladder | Measured `C_d` | Verdict |
|---|---|---|
| η: 0.128 → 0.008 | 17.39, 24.02, 26.25, 23.76, 21.40 | rises, peaks, falls — **no η→0 limit** |
| smoothing: 0.5 → 4 cells | 7.70, 12.33, 23.76, 35.81, 47.27 | **6.1×** on a purely numerical parameter |

The η→0 limit is what licenses calling the penalization integral a drag at all (Angot, Bruneau &
Fabrie 1999, `O(η^{3/4})`). Without it, `C_d ≈ 23.8` is a property of this configuration's blur width,
not of a cylinder. Root cause is this report's own §4b finding: the physical Brinkman layer
`√(ην) = 0.144·dx` is ~7× thinner than a cell, and `η ≥ dx²/ν = 0.771` is violated 48×, so the grid
resolves the smoothing skirt rather than the penalization layer.

**No bound was widened.** Per owner decision the gate is kept and the harness moved to the nightly
cadence, so a known-open physics finding does not block merges; its committed `baseline.txt` records
`exit 1` with both `NOT CONVERGING` verdicts rather than a stale passing artifact. Documented under
`KNOWN-FAILING` in `.github/workflows/cfd_verification.yml` and routed to §9 Phase 2 item 10.

This is the clearest vindication of the audit's premise: the harness passed for years while gating
three things — no-slip (provably invariant across the whole smoothing sweep), bond convergence (bound
eleven orders above the measurement), and `0 < C_d < 100` — none of which constrained the drag.

---

## 6. Production readiness ranking

Ranking reflects **assurance**, not elegance. Critical counts are post-review.

| Rank | Module | Readiness | Crit | Basis |
|---|---|---|---|---|
| 1 | Pointwise theories | `needs-work` | 0 | Governing equations check out; issues are doc parity and constant traceability |
| 2 | DEC NS rate kernel | `needs-work` | 0 | RK4 tableau exact; Δ = dδ+δd correctly ordered; δ = M⁻¹BM is the true discrete adjoint, so −νΔ provably dissipates; Leray projection inside every RK4 stage. Docs describe a different convective operator than the code marches |
| 3 | DEC solver driver | `needs-work` | 0 | Projection consistent; issues in diagnostics and tolerance justification |
| 4 | QTT compressible marchers | `needs-work` | 0 | RH exact (hand-verified); Sod matches exact Riemann; unfloored negative pressure can reach the flux |
| 5 | CfdFlow DSL | `needs-work` | 0 | CoW fork and determinism largely as advertised; doc overclaims |
| 6 | Docs-vs-code parity | `needs-work` | 0 | 87 doc-overclaim + 39 doc-gap findings crate-wide |
| 7 | DEC boundary zones | `needs-work` | 0 | Documented `collect_constrained_edges` hook is never called |
| 8 | Examples | `needs-work` | 0 | Reproducible and well-structured; two gates cannot fail |
| 9 | Crate-wide constants sweep | `needs-work` | 0 | Negative-pressure flux, Mach-1.05 shock floor, f32 pressure-floor collapse |
| 10 | Studies | `needs-work` | 0 | Several headline findings not supported by what the code measures |
| 11 | Test suite & build health | `needs-work` | 0 | Change-detector tests; verification suite absent from CI (rolled into B-2) |
| 12 | Coordinate + tensor bridge | `needs-work` | 1 | B-4 |
| 13 | Verification harnesses | `needs-work` | 1 | B-3; the layer that must be strongest is among the weakest |
| 14 | Plasma / blackout physics | **`not-ready`** | 1 | Headline result rests on B-1 |
| 15 | Navigation / ESKF | **`not-ready`** | 0 | Two compounding filter defects (§4b); unguarded public API |
| 16 | QTT incompressible / immersed | **`not-ready`** | 0 | No gate constrains drag correctness (§4b); Brinkman layer unresolved 48× |

Modules 15 and 16 carry **zero criticals** yet remain `not-ready`. That is deliberate: their defects are
individually major but jointly remove the basis for trusting the module's headline output.

---

## 7. Systemic themes

| Axis | Count | Pattern |
|---|---|---|
| doc-overclaim | 87 | Prose describes intended design; code implements a subset. Recurring shape: a doc asserts a property holds "by construction" where no check exists (B-4 is the sharpest case). |
| tautology-circular | 72 | Gates restating the implementation, folding over hardcoded constants, or comparing a value to a bound pinned from that same value. |
| physics-math | 50 | Genuine formula/constant defects — concentrated in plasma chemistry, navigation and penalization, **not** in the core DEC/QTT marchers. |
| magic-number | 46 | Load-bearing literals without traceable provenance. |
| doc-gap | 39 | Real capabilities (`DuctMarchRun`, `IgnitionCorridor`, snapshot/resume, `AcousticCoreInverse`) absent from docs. |

Three structural observations:

- **Back-fitted bounds are disclosed, not hidden.** `qtt_ramc_stagline` states in its own gate text that
  its ±0.70-decade band is "pinned from the measurement". Preserve that honesty — but such gates must be
  **labelled regression tests**, not presented as validation against flight data.
- **`Gates` is dead, printing, duplicate API.** Exported from `lib.rs`, documented as "the `[PASS]`/`[FAIL]`
  block every self-verifying program prints" — yet **no** program uses it; every harness goes through
  `Verdict`'s `Display` impl. It holds the only 5 `println!` in `src/`, making it the sole violator of the
  README's "the DSL never exits or prints", and `Gates::finish()` returns `true` for an empty gate set.
- **The DEC family validates its envelope; the QTT family does not.** `dec_ns_solver/step.rs` rejects
  CFL and diffusive-limit violations with `PhysicalInvariantBroken`; `QttImmersed2d::new` and
  `QttIncompressible2d::new` validate nothing, at any layer including the builder. One family refuses an
  out-of-envelope configuration; its sibling returns numbers.

---

## 8. What is verified correct

176 items were positively confirmed against a reference. The load-bearing ones:

- **Rankine–Hugoniot normal-shock relations** — exact (hand-verified, §3.4).
- **RK4** — the exact classical Butcher tableau; coefficients 2 and 6 are tableau weights, not tuning.
- **Hodge–de Rham Laplacian** — Δ = dδ + δd correctly ordered, with δ = M⁻¹BM the true discrete
  M-adjoint of d, forcing Δ positive semi-definite so `−νΔ` provably dissipates energy.
- **Interior product** — Hirani's `(−1)^{k(D−k)}⋆(⋆ω∧X♭)`, giving the correct Lamb-vector sign.
- **Leray projection applied to the rate inside every RK4 stage** — matching the README claim exactly.
- **Spectral projector consistency** — `div(project(u)) = 0` holds identically by construction; the
  eigenvalue `−sin²(2πk/N)/dx²` is the consistent operator, correctly implemented (two *comments*
  misstate it; the code is right).
- **Plasma frequency** ωₚ = √(nₑe²/ε₀mₑ) — exact, correct CODATA constants; `comms_band` is correctly a
  parameter, documented as "GPS L-band ≈ 1.5 GHz".
- **Sod shock tube vs exact Riemann** — L1 0.0175 / 0.0274 / 0.0151 against a 0.03 bound, on a genuinely
  analytic reference. **The crate's strongest gate.**
- **DEC lid-driven cavity vs Ghia (1982)** — primary vortex to 1e-4, bottom-right eddy exact (§3.5).
- **Graded-mesh operator convergence** — second order at every grading amplitude; finest-pair 1.98–2.00.
- **MMS Taylor–Green** — residual 1.11e-16, amplitude error 6.66e-16, at the f64 floor.
- **ESKF `update_scalar` covariance form** — correct Joseph form with re-symmetrization, cited.
- **Brinkman force law** — `F = (1/η)∫χ(u−u_b)dV`, `C_d = F_x/(½ρU²D)` is the correct formulation
  (confirmed by probe during re-verification; the defect is parameter choice, not the force law).
- **Safety and lint posture** — `unsafe_code = "forbid"`; zero `panic!`/`process::exit` in `src/`; all 14
  `#[allow]` are `too_many_arguments`/`type_complexity`, never correctness-lint suppression.
- **`benches/PERFORMANCE.md`** — full methodology, honest that `parallel` is *slower* at bench scale.
- **Determinism and reproducibility** of committed example outputs.

---

## 9. Path to certification

**Phase 1 — Make the evidence bite (highest value, lowest effort) — ✅ COMPLETE (2026-07-21)**

Implemented and archived as `2026-07-21-make-cfd-evidence-enforceable`, 43/43 tasks; four capability
specs synced into `openspec/specs/`.

1. ✅ Add CI execution of the verification suite (fast on PR, slow nightly). — **B-2 resolved.**
   `.github/workflows/cfd_verification.yml`: 9 fast per PR (12.7 s), 4 nightly, plus a
   `verification-suite-complete` job that fails if a declared harness is in neither list.
2. ✅ Add a real gate to `dec_cylinder_verification`. — **B-3 resolved.** Four gates; a solver error
   now exits 1 instead of reporting `St`/`C_d` from the truncated series.
   **Two further instances found:** `mms_taylor_green_verification` (setup-error exit only, residual
   never checked) and `dec_graded_mms_verification` (**zero** `process::exit` calls). Both now gate.
3. ✅ Remove or repair every gate that cannot fail — seven repaired, each demonstrated failing by fault
   injection. **Exception: the energy-budget gate was refuted, not repaired** (§5) — it genuinely
   guards operator skew-symmetry.
4. ✅ Re-found the QTT cylinder gate set. `CONVERGENCE_BOUND` tightened `0.10 → 1e-6`.
   **Both new ladders fail** — a real finding, not a defect in the change; see §5b.
5. ✅ Audit every gate bound for back-fitting; relabel pinned bounds. Implemented as a two-value
   `EvidenceClass` (`reference` / `tripwire`) carried at the call site and rendered in output —
   **38 gate lines, 0 unlabelled** — plus convention sections in `verification/README.md`.
6. ✅ Regenerate the truncated lid-cavity `baseline.txt`. **Scope was larger than this item assumed:**
   the stricter "carries a verdict" check found **6 of 12** baselines carried none (stdout-only capture,
   or predating their gates). All 13 harnesses now have a baseline, all carrying a verdict.

*Docs corrected in passing:* the crate README's "validate … against RAM-C II flight data" is now an
order-of-magnitude framing naming the ±0.70-decade band as pinned; the lid-cavity summary row now
reports the 65² default (RMSE **0.0617**, primary vortex matching Ghia to 1e-4) instead of the coarse
33² trend rung that understated the solver by more than 2×.

**Phase 2 — Close the physics defects — ⬅ NEXT**

Phase 1 established that these are now *detectable*: item 10's condition is already failing a gate
nightly. Nothing here is blocked on further analysis.

7. Resolve `REDUCED_MASS_AMU` at both sites and re-baseline the RAM-C chain and the three examples. — **B-1**
8. Implement the fold/singularity check `BlendedMap` documents; guard ÷det J. — **B-4**
9. Scale ESKF `Q` by `dt`; guard the innovation covariance and validate measurement variance; add a
   horizon-invariance test; fix `correct_position` attitude handling. — §4b
10. Resolve the Brinkman envelope: choose η from a wall-error target, document the `η ≥ dx²/ν` resolution
    constraint, and state that it is currently violated 48×. — §4b, §5b.
    **This is now the nightly red build** — `qtt_cylinder_verification` fails on exactly this condition,
    so the fix has a ready-made acceptance test: the η ladder must converge.
11. Rename or re-derive `wall_heat_flux`; make `t_wall` configurable. — §4b
12. Reject non-positive pressure in `marcher_2d` instead of flooring it only for the wave speed.
13. Add numerical-envelope validation to the QTT constructors, matching the DEC family.
14. Clamp the body mask to [0,1] after quantization, or validate in `QttImmersed2d::new`.
15. Wire or remove `BoundaryZone::collect_constrained_edges`.

**Phase 3 — Documentation truth-up (bidirectional)**
16. Reconcile the 87 doc-overclaims. Where prose describes intent, mark it intent.
17. Correct the DEC kernel docs to describe the skew-symmetrized operator the code actually marches, and
    the two spectral-projector comments that contradict correct code.
18. Document the 39 undocumented capabilities.
19. Restate the RAM-C claim consistently — the crate README says "validate against flight data"; the
    verification README correctly says order-of-magnitude. Use the accurate phrasing in both.
20. Fix the lid-cavity summary row to report the default configuration (§3.5).
21. Qualify "clean 2nd-order convergence" as second-order in space, first-order in time, and document the
    TG ladder's temporal-error floor and maximum usable length.
22. Resolve `Gates`: adopt it everywhere, or retire it. *(Owner decision — no deletion without approval.)*

**Phase 4 — Traceability**
23. Give every load-bearing constant a source, units, and a `papers/` entry — starting with
    `SMOOTH_CELLS`, `ETA`, and the Mach-1.05 shock floor. Add an Angot/Kevlahan penalization reference to
    `papers/`, which currently has none.
24. Replace change-detector tests with independent-reference tests in load-bearing modules; add a test
    that exercises the shipped QTT convection path with `u, v ≠ 0`.

---

## 10. Recommendation

**Do not certify yet.** Certify after Phases 1 and 2 — a bounded, mechanical workstream with no solver
rewrites.

The distinction that matters for an avionics R&D lab: this crate's **numerics** are in materially better
shape than its **assurance case**. The risk is not that an engineer gets a wrong answer from the DEC or
QTT compressible marchers — on every closed-form reference available, they get the right one, sometimes
to four decimal places. The risk is that an engineer reads "all gates passed" and concludes something was
checked that was not.

Phase 1 removes that risk directly.

---

## Appendices

- [`MODULE-INDEX.md`](MODULE-INDEX.md) — per-module readiness table
- [`ACTION-LIST.md`](ACTION-LIST.md) — all 290 surviving findings, severity-ordered and actionable
- [`RUN-LEDGER.md`](RUN-LEDGER.md) — execution ledger with exit codes and gate counts
- [`modules/`](modules/) — 16 per-module reports with code evidence and adversarial verdicts

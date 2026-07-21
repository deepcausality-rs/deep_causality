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

> **Status: NOT YET CERTIFIABLE. Remediation required.**
>
> The crate is not broken. Its numerical core is, in every place checkable against a closed-form
> reference, *exactly* right — including a lid-driven-cavity primary vortex matching Ghia (1982) to four
> decimal places, and Rankine–Hugoniot relations exact to displayed precision.
>
> What blocks certification is the **assurance layer**, not the mathematics: a large fraction of the
> gates advertised as proving the mathematics cannot fail, and none of them run in CI.

---

## 1. Bottom line

**1. The physics core is sound, and in places excellent.** Independently re-derived by the lead auditor:
Rankine–Hugoniot jump relations, plasma frequency, Sod exact-Riemann star state, and graded-mesh operator
convergence orders — all correct to displayed precision. The DEC lid-driven cavity reproduces Ghia's
primary and both corner vortices. 813 unit tests pass. Committed `output.txt` files reproduce.

**2. The verification layer does not verify what it claims to verify.** Of 294 findings, **72 are
tautology/circular-reasoning defects**. Several headline gates are algebraically incapable of failing —
ten of which the lead auditor confirmed by hand at source. A suite advertised as "self-verifying, exit
nonzero on break" is substantially a set of statements true by construction.

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
| Adversarial re-check | 16 verifiers instructed to refute (14 returned; 2 died on a session limit) |
| Distinct source files read | **412** |
| Findings raised | **294** |
| Findings adversarially re-checked | **253** |
| Findings left unverified (verifier died) | **41** |
| Items positively confirmed correct against a reference | **176** |
| Unit tests executed | 813 pass / 0 fail / 2 ignored |
| Verification harnesses executed | 13 / 13, all exit 0 |
| Avionics examples executed | 7 / 7, all exit 0 |
| Findings independently re-verified at source by the lead auditor | **10 / 10 confirmed real** |

### 2.2 Verification outcome

| | Count |
|---|---|
| CONFIRMED | 168 |
| PARTIALLY_CONFIRMED (real, but severity or detail corrected) | 83 |
| REFUTED (not a defect) | 2 |
| UNVERIFIED (verifier terminated) | 41 |

**Severity before vs after adversarial review:**

| Severity | As raised | After review |
|---|---|---|
| critical | 26 | **10** |
| major | 131 | **76** |
| minor | 126 | 175 |
| info | 11 | 31 |

The verification stage did real work. Outright refutation was rare (2 of 253), but **severity correction
was extensive** — it cut criticals by 62 % and majors by 42 %, reclassifying overstated findings downward
rather than rejecting them outright. The numbers used throughout this report are the **post-review**
ones.

### 2.3 Limitations — read before acting

- **Two modules were never adversarially verified.** The verifiers for *QTT incompressible/immersed* and
  the *crate-wide constants sweep* terminated on a session limit. Their 41 findings — including 6 of the
  10 criticals — are **single-auditor claims**. Those module reports carry a warning banner.
  *Mitigation:* the lead auditor independently confirmed the two most consequential QTT-immersed
  criticals at source (§4, B-9 and B-10).
- **Severity labels were not normalized across modules.** Compare within a module, not across.
- **Runtimes are not comparable to the README.** These runs executed alongside 16 concurrent agents; the
  lid cavity took 1407 s against a documented ~28 s at a different grid. Contention, not regression.
- **Only §4 items are asserted as certain.** The remaining findings are credible, located, triage-ready
  leads. They should be worked, not assumed.

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

**That result is weaker than it looks.** Several exit-0 outcomes come from gates that cannot return
false, and `dec_cylinder_verification` has no gate at all and exits 0 even after a solver error.

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

## 4. Certification blockers — independently confirmed at source

Each item was re-read at source by the lead auditor and is asserted as fact. Where the automated
verifier assigned a different severity, that is noted. Remaining criticals are in
[`ACTION-LIST.md`](ACTION-LIST.md).

### B-1 — Millikan–White reduced mass is wrong for the documented collision pair
`verification/qtt_ramc_stagline/config.rs:35-37`, `src/solvers/qtt/compressible/fitting.rs:72-73`
*(verifier: CONFIRMED critical)*

```rust
/// Reduced mass `μ_sr` of the dominant relaxing collision pair (N₂–N₂ ≈ 14·14/28 = 7), in amu
pub const REDUCED_MASS_AMU: f64 = 7.0;
```

The derivation in the comment is arithmetically wrong. N₂ has mass **28** amu, not 14. The reduced mass
of an N₂–N₂ pair is 28·28/56 = **14 amu**. The value 7 amu is the **N–N atom** pair, mislabelled.

Since `A_sr = 1.16e-3·μ^(1/2)·θ_v^(4/3)` and μ enters inside the exponential, a factor-2 error in μ
shifts τ_vt substantially — moving T_ve, hence T_a = √(T_tr·T_ve), hence peak electron density. **The
crate's headline result ("peak nₑ = 1.08e19, +0.0 decades vs the RAM-C II anchor") rests on this
constant.**

**The constant is duplicated and propagates beyond the harness.** It is defined twice, both at 7.0 and
both labelled N₂–N₂:

- `deep_causality_cfd/verification/qtt_ramc_stagline/config.rs:37`
- `examples/avionics_examples/src/shared/constants.rs:128`

and reaches the flagship plasma-blackout examples through `examples/avionics_examples/src/shared/world.rs`
at lines 165 and 345. So the corridor, weather and retropulsion examples inherit it, not just the
verification harness.

**This is the most consequential finding in the audit**, because the module report records that
re-running at the physically correct μ = 14 moves the prediction to roughly **−1.27 decades** against the
RAM-C anchor — i.e. correcting the constant *removes* the headline agreement rather than preserving it.
(That figure comes from the module auditor and was not independently re-computed here; it must be
re-measured as the first step.)

*Action:* decide the intended pair. If N₂–N₂, set 14.0 at **both** definition sites, correct all four
docstrings, and re-baseline the whole RAM-C chain **and** the three plasma-blackout examples. If N–N was
intended, correct the label and justify why atom–atom dominates at these conditions. Either way, derive
the acceptance gates from the corrected physics — **do not re-tune the gates to restore the previous
agreement.**

### B-2 — No CI or Bazel target executes the verification suite
`.github/workflows/run_tests.yml` — no `cargo run --example` anywhere
*(verifier: CONFIRMED critical)*

Every external-reference comparison in the crate (exact Riemann, Ghia cavity, Williamson cylinder, RAM-C
flight nₑ) lives in `verification/` binaries that CI compiles and never runs.

*Action:* run the fast harnesses (10 of 13 complete in ≤5 s) on every PR and the three slow ones nightly.
**This single change converts the entire README table from prose into a gate, and is the highest
value-per-effort item in this report.**

### B-3 — `dec_cylinder_verification` has no gate and cannot fail
`verification/dec_cylinder_verification/main.rs` — **0** `process::exit`, **0** assertions
*(verifier: CONFIRMED critical)*

On a solver `Err` it prints, breaks the march, then reports Strouhal and drag from the *truncated* series
and returns 0. `verification/README.md` states the opposite convention. This is the crate's most
expensive harness (~510 s) and its only isolated-cylinder validation.

*Action:* gate St and C_d against the stated Williamson / Dröge–Verstappen bands; exit nonzero on solver
error.

### B-4 — `BlendedMap` documents a fold check it does not perform
`src/coordinate/blended.rs:17,171` *(verifier: CONFIRMED critical)*

The module doc states the constructor rejects a fold and that det J one-signedness holds "by
construction". No such check exists in `BlendedMap::new`. The inverse metric divides by `det_at(ξ,η)`
with no guard, so degenerate geometries yield inf/NaN or ~1e15-magnitude metric entries while the
constructor still returns `Ok`.

*Action:* implement the documented fold/singularity check, or remove the guarantee from the doc. Guard
the division.

### B-5 — `qtt_rank_dynamic` gate G2 is unreachable
`studies/qtt_rank_dynamic/main.rs:108`, with `:164,169` *(verifier: downgraded; lead auditor confirms)*

```rust
let mut peak = init;                          // :164
peak = peak.max(b);                           // :169
if b_peak < b0 { failures.push("G2: ...") }   // :108
```

`peak ≥ init` holds identically and `b0` *is* `init`. G2 can never fire. Verified by reading all three
lines.

*Action:* replace with a gate that can fail — assert the peak exceeds the smooth-encode floor by a stated
margin, which is what the comment says the gate is for.

### B-6 — Weather example gate "(0) table integrity" cannot fail
`examples/avionics_examples/cfd/plasma_blackout/weather/model.rs:223,264`
*(verifier: downgraded; lead auditor confirms)*

`errored: false` is a literal at row construction; the gate is `rows().all(|r| !r.errored)`.

*Action:* populate `errored` from a real per-row outcome, or remove the gate. A gate that cannot fail is
worse than no gate — it reads as assurance.

### B-7 — QTT Taylor–Green convection "amplitude" gate is computed from the reference alone
`verification/qtt_taylor_green_verification/main.rs:136-145` *(unverified by automation; lead auditor confirms)*

```rust
let analytic = -0.5 * (2.0 * (i as f64 * dx)).sin();
max_err = max_err.max((cs[i*n+j] - analytic).abs());
amp     = amp.max(analytic.abs());          // never reads `cs`
```

`amp` derives solely from the closed form, so the "convection is non-zero" gate tests only that the
*analytic reference* is non-zero. It cannot detect a zero or wrong computed convection. (`max_err` does
read `cs`, so the error gate itself is real.)

*Action:* compute the amplitude from `cs` and gate the computed-to-analytic amplitude ratio.

### B-8 — QTT cylinder is cross-referenced against a DEC case at a different Reynolds number
`verification/qtt_cylinder_verification/config.rs:21,30,32,36` *(lead auditor confirms)*

`D = 2·RADIUS_FRAC·2π = 1.885`, `ν = 0.05`, `U = 1` ⇒ **Re = 37.7**. Line 36 cross-references the
committed DEC isolated-cylinder drag at **Re 100**. C_d is strongly Reynolds-dependent in this range.

*Action:* match the Reynolds numbers, or state the cross-reference is qualitative only.

### B-9 — Penalization drag integrates a mask whose skirt rivals the body in area
`src/solvers/qtt/observe.rs:23-41`, `verification/qtt_cylinder_verification/config.rs:34`
*(unverified by automation; lead auditor confirms the geometry)*

The force definition itself is **correct**: `F = (1/η)∫χ(u−u_body)dV`, normalized `C_d = F_x/(½ρU²D)`.
The problem is the mask support. With `SMOOTH_CELLS = 2.0` on a 32² grid of side 2π, `dx = 0.196`, so the
skirt is 0.393 against a cylinder radius of 0.943 — **42 % of the radius**. The skirt annulus therefore
carries roughly the same area as the body, and in those cells `u → U∞`, contributing a large spurious
force. This is why the reported `C_d ≈ 23.8` is not comparable to the DEC `1.345`.

*Action:* restrict the force integral to the geometric body, or report C_d with the skirt contribution
separated. Document that the reported quantity is smoothing-width dependent.

### B-10 — The penalization parameter η is set by numerical stability, not by a wall-error target
`verification/qtt_cylinder_verification/config.rs:22-28` *(unverified by automation; lead auditor confirms)*

```rust
/// Explicit-Euler time step (`dt/η = 0.25`, explicit-stable).
pub const DT: f64 = 0.004;
pub const ETA: f64 = 0.016;
```

η is chosen to satisfy the explicit-stability ratio. In Brinkman penalization the no-slip error scales
as √(ην) (Angot et al. 1999), so η directly sets the wall error the harness then *gates on*. Choosing it
from stability makes the no-slip gate a function of the time step.

*Action:* choose η from a target no-slip error and adapt `dt` to it, or move to an implicit penalization
treatment. Document the resulting wall-error bound.

---

## 5. Production readiness ranking

Ranking reflects **assurance**, not elegance. Correct math with unfalsifiable gates ranks below correct
math with honest gates. Critical counts are post-review.

| Rank | Module | Readiness | Crit | Basis |
|---|---|---|---|---|
| 1 | Pointwise theories (`src/theories/`) | `needs-work` | 0 | Governing equations check out; issues are doc parity and constant traceability |
| 2 | DEC NS rate kernel | `needs-work` | 0 | RK4 tableau exact; Δ=dδ+δd correctly ordered; δ is the true discrete M-adjoint, so −νΔ provably dissipates; Leray projection inside every RK4 stage. Docs describe a different convective operator than the code marches |
| 3 | DEC solver driver | `needs-work` | 0 | Projection consistent; issues in diagnostics and tolerance justification |
| 4 | QTT compressible marchers | `needs-work` | 0 | RH exact (hand-verified); Sod matches exact Riemann; conservation-under-truncation undocumented |
| 5 | CfdFlow DSL | `needs-work` | 0 | CoW fork and determinism largely as advertised; doc overclaims |
| 6 | Docs-vs-code parity | `needs-work` | 0 | 87 doc-overclaim + 39 doc-gap findings crate-wide |
| 7 | DEC boundary zones | `needs-work` | 0 | Documented `collect_constrained_edges` hook is never called |
| 8 | Examples | `needs-work` | 0 | Reproducible and well-structured; two gates cannot fail |
| 9 | Test suite & build health | `needs-work` | 1 | Change-detector tests; verification suite absent from CI |
| 10 | Coordinate + tensor bridge | `needs-work` | 1 | B-4 |
| 11 | Verification harnesses | `needs-work` | 1 | B-3; the layer that must be strongest is among the weakest |
| 12 | Studies | `needs-work` | 0 | Several headline findings not supported by what the code measures |
| 13 | Crate-wide constants sweep | `needs-work` | 1 | ⚠ unverified. Negative pressure reaches the flux while floored for the LLF wave speed |
| 14 | Plasma / blackout physics | **`not-ready`** | 1 | Headline result rests on B-1 |
| 15 | Navigation / ESKF | **`not-ready`** | 0 | ESKF `Q` unscaled by `dt`; covariance/mean inconsistency in `correct_position` |
| 16 | QTT incompressible / immersed | **`not-ready`** | 5 | ⚠ unverified. B-9, B-10: penalization drag is definition- and stability-parameter dependent |

⚠ = module's findings were not adversarially re-checked (verifier terminated).

Per-module detail: [`MODULE-INDEX.md`](MODULE-INDEX.md) and [`modules/`](modules/).

---

## 6. Systemic themes

Five patterns account for nearly all 294 findings. Fixing the pattern is cheaper than fixing the
instances.

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
  **labelled regression tests**, not presented in the README as validation against flight data.
- **`Gates` is dead, printing, duplicate API.** `src/types/flow/gates.rs` is exported from `lib.rs` and
  documented as "the `[PASS]`/`[FAIL]` block every self-verifying program prints" — yet **no** program
  uses it; every harness goes through `Verdict`'s `Display` impl. It holds the only 5 `println!` in
  `src/`, making it the sole violator of the README's "the DSL never exits or prints", and
  `Gates::finish()` returns `true` for an empty gate set.
- **The doc/code divergence in the DEC kernel is systematic.** Module doc, type doc, method docs, the
  theory, and the README all state the convective term is `−i_u(du♭)`, while the code marches the
  skew-symmetrized `−½(G_ω u − G\*_ω u)`. The implementation choice is defensible; the documentation is
  simply describing a different solver.

---

## 7. What is verified correct

176 items were positively confirmed against a reference. The load-bearing ones:

- **Rankine–Hugoniot normal-shock relations** — exact (hand-verified, §3.4).
- **RK4** — the exact classical Butcher tableau, coefficients 2 and 6 being tableau weights, not tuning.
- **Hodge–de Rham Laplacian** — Δ = dδ + δd correctly ordered, with δ = M⁻¹BM the true discrete
  M-adjoint of d, which forces Δ positive semi-definite so `−νΔ` provably dissipates energy.
- **Interior product** — Hirani's `(−1)^{k(D−k)}⋆(⋆ω∧X♭)`, giving the correct Lamb-vector sign.
- **Leray projection applied to the rate inside every RK4 stage** — matching the README claim exactly.
- **Plasma frequency** ωₚ = √(nₑe²/ε₀mₑ) — exact, correct CODATA constants. `comms_band` is correctly a
  *parameter*, documented as "GPS L-band ≈ 1.5 GHz" — traceable, not magic.
- **Sod shock tube vs exact Riemann** — L1 0.0175 / 0.0274 / 0.0151 against a 0.03 bound, on a genuinely
  analytic reference. **The crate's strongest gate.**
- **DEC lid-driven cavity vs Ghia (1982)** — primary vortex to 1e-4, bottom-right eddy exact (§3.5).
- **Graded-mesh operator convergence** — second order confirmed for both operators at every grading
  amplitude; finest-pair 1.98–2.00.
- **MMS Taylor–Green** — residual 1.11e-16, amplitude error 6.66e-16, at the f64 floor.
- **ESKF `update_scalar`** — correct Joseph-form covariance update with re-symmetrization, citing Groves
  (2013) §3.4.3.
- **Brinkman force definition** — `F = (1/η)∫χ(u−u_body)dV` with `C_d = F_x/(½ρU²D)` is the correct
  formulation; the defect (B-9) is the mask support, not the force law.
- **Safety and lint posture** — `unsafe_code = "forbid"` applies; zero `panic!`/`process::exit` in
  `src/`; all 14 `#[allow]` are `too_many_arguments`/`type_complexity`, never correctness-lint suppression.
- **`benches/PERFORMANCE.md`** — full methodology, honest that `parallel` is *slower* at bench scale.
- **Determinism and reproducibility** of committed example outputs.

---

## 8. Path to certification

Four phases. Phase 1 alone changes the assurance posture more than the other three combined.

**Phase 0 — Close the audit gap**
1. Re-run adversarial verification for the two unverified modules (QTT incompressible/immersed; the
   crate-wide constants sweep) — 41 findings including 6 criticals currently rest on a single auditor.

**Phase 1 — Make the evidence bite (highest value, lowest effort)**
2. Add CI execution of the verification suite (fast on PR, slow nightly). — **B-2**
3. Add a real gate to `dec_cylinder_verification`. — **B-3**
4. Remove or repair every gate that cannot fail: **B-5**, **B-6**, **B-7**, the `qtt_park2t_blackout`
   gates (ii) and (iv), the MMS `continuity_error` gate, and the energy-budget CI gate.
5. Audit every gate bound for back-fitting; relabel any bound pinned from its own measurement as a
   **regression test**, in code and in `verification/README.md`.

**Phase 2 — Close the physics defects**
6. Resolve `REDUCED_MASS_AMU` and re-baseline the RAM-C chain. — **B-1**
7. Implement the fold/singularity check `BlendedMap` documents; guard ÷det J. — **B-4**
8. Scale ESKF `Q` by `dt`; add a horizon-invariance test. Fix `correct_position` attitude handling.
9. Re-found the penalization drag observable: exclude the skirt or report it separately; choose η from a
   wall-error target; match or drop the DEC cross-reference. — **B-8, B-9, B-10**
10. Stop negative pressure reaching the flux in `marcher_2d`.
11. Wire or remove `BoundaryZone::collect_constrained_edges`.

**Phase 3 — Documentation truth-up (bidirectional)**
12. Reconcile the 87 doc-overclaims. Where prose describes intent, mark it intent.
13. Correct the DEC kernel docs to describe the skew-symmetrized operator the code actually marches.
14. Document the 39 undocumented capabilities.
15. Restate the RAM-C claim consistently — the crate README says "validate against flight data"; the
    verification README correctly says order-of-magnitude. Use the accurate phrasing in both.
16. Fix the lid-cavity summary row to report the default configuration (§3.5).
17. Resolve `Gates`: adopt it everywhere, or retire it. *(Owner decision — no deletion without approval.)*

**Phase 4 — Traceability**
18. Give every load-bearing constant a source, units, and a `papers/` entry.
19. Replace change-detector tests with independent-reference tests in load-bearing modules.

---

## 9. Recommendation

**Do not certify yet.** Certify after Phases 0–2 — a bounded, mechanical workstream with no solver
rewrites.

The distinction that matters for an avionics R&D lab: this crate's **numerics** are in materially better
shape than its **assurance case**. The risk is not that an engineer gets a wrong answer from the DEC or
QTT marchers — on every closed-form reference available, it gets the right one, sometimes to four decimal
places. The risk is that an engineer reads "all gates passed" and concludes something was checked that
was not.

Phase 1 removes that risk directly.

---

## Appendices

- [`MODULE-INDEX.md`](MODULE-INDEX.md) — per-module readiness table
- [`ACTION-LIST.md`](ACTION-LIST.md) — all findings, severity-ordered and actionable
- [`RUN-LEDGER.md`](RUN-LEDGER.md) — execution ledger with exit codes and gate counts
- [`modules/`](modules/) — 16 per-module reports with code evidence and adversarial verdicts

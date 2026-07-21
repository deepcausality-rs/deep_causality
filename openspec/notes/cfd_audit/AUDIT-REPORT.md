<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# `deep_causality_cfd` — Pre-Certification Audit

**Date:** 2026-07-21
**Scope:** `deep_causality_cfd/` (src, tests, verification, studies, benches) and `examples/avionics_examples/cfd/`
**Purpose:** Establish whether the crate can be certified for avionics **R&D** use — i.e. whether an
engineer can trust that each marcher and kernel computes what the specification and the reference
formula say it computes.

> **Status: NOT YET CERTIFIABLE. Remediation required.**
> The crate is not broken. Its numerical core is, in every place checkable against a closed-form
> reference, *exactly* right — including a lid-driven-cavity primary vortex that matches Ghia (1982) to
> four decimal places. What blocks certification is not the mathematics. It is the **assurance layer**:
> a large fraction of the gates advertised as proving the mathematics cannot fail, and none of them run
> in CI.

---

## 1. Bottom line

**1. The physics core is sound, and in places excellent.** Independently re-derived by the lead auditor:
the Rankine–Hugoniot relations, the plasma frequency, the Sod exact-Riemann star state, and the
graded-mesh operator convergence orders are correct to displayed precision. The DEC lid-driven cavity
reproduces Ghia's primary and both corner vortices. 813 unit tests pass. Every executed harness, study
and example exits 0, and committed `output.txt` files reproduce byte-for-byte.

**2. The verification layer does not verify what it claims to verify.** Of 294 findings, **72 are
tautology/circular-reasoning defects**. Multiple headline gates are algebraically incapable of failing —
eight of which the lead auditor confirmed by hand. A suite advertised as "self-verifying, exit nonzero on
break" is substantially a suite of statements true by construction.

**3. The evidence never runs.** No CI job, script, or Bazel target executes any of the 13 verification
programs. `cargo test` compiles them and never runs them. Every quantitative accuracy claim in the crate
README and `verification/README.md` is unenforced and can silently rot.

This is a **strong research instrument with a weak assurance case**. The fix list is large but almost
entirely mechanical: making claims match reality and making gates bite, not rewriting solvers.

---

## 2. Method, and its limits

### 2.1 What was done

| Activity | Extent |
|---|---|
| Independent module audits | **16** parallel auditors, one per subsystem |
| Adversarial re-check of findings | 16 verifiers, instructed to refute |
| Distinct source files read | **412** |
| Findings raised | **294** (26 critical, 131 major, 126 minor, 11 info) |
| Items positively confirmed correct against a reference | **176** |
| Test suite executed | `cargo test -p deep_causality_cfd --release` → 813 pass / 0 fail |
| Verification harnesses executed | 13 / 13 |
| Studies executed | 14 / 14 |
| Avionics examples executed | 7 / 7 |
| Reference formulas re-derived by hand | RH jumps, plasma frequency, Sod star state, Millikan–White reduced mass, Reynolds number, Ghia comparison |
| Findings independently re-verified by the lead auditor | **8 / 8 confirmed real** |

### 2.2 Limitations — read before acting on the numbers

Stated because a certification report that hides its own weaknesses is worthless.

- **The adversarial verification stage was weak.** Across 171 verdicts it returned 116 CONFIRMED,
  54 PARTIALLY_CONFIRMED and **1 REFUTED**. A ~0.6 % refutation rate is not credible evidence that
  essentially every finding is real; it indicates the refutation stage under-performed.
- **Mitigation, and what it showed.** The lead auditor independently re-read the source for eight
  high-severity findings spanning five modules. **All eight were confirmed real** (§4). The findings are
  specific, located, and cheaply checkable, and the sample did not degrade. This raises confidence in the
  wider set — but the wider set is still best treated as **credible, triage-ready leads**, not as
  established defects. Only the items in §4 are asserted as certain.
- **Severity labels are auditor-assigned** and were not normalized across modules. Compare within a
  module, not across.
- **Runtime figures were measured under heavy load** (16 concurrent agents) and are *not* comparable to
  the README's quoted timings. The lid-cavity run took 1407 s against a documented ~28 s at a different
  grid; this is contention, not a regression.

---

## 3. Evidence base (first-hand)

### 3.1 Test suite

```
cargo test -p deep_causality_cfd --release
→ 813 passed; 0 failed; 2 ignored
```

### 3.2 Execution ledger

All 13 verification harnesses, 14 studies and 7 avionics examples were built and run — full ledger in
[`RUN-LEDGER.md`](RUN-LEDGER.md). Every one exited 0.

**That result is weaker than it appears.** Several exit-0 outcomes are produced by gates that cannot
return false (§4), and `dec_cylinder_verification` contains no gate at all and exits 0 even after a
solver error.

### 3.3 Reproducibility of committed outputs — PASS

| Example | Fresh run vs committed `output.txt` |
|---|---|
| `viv_resonance_margin` | byte-identical |
| `flight_envelope_placard` | identical but for trailing newline |
| `nozzle_operating_map` | identical but for trailing newline |
| `plasma_blackout_corridor` | identical but for the wall-clock line (40.9 s committed vs 48.2 s under load) |

Committed evidence is **fresh and deterministic**. This is a genuine strength and it is uncommon.

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

### 3.5 DEC lid-driven cavity vs Ghia (1982) — PASS, and better than documented

Default configuration (65², t=100), measured this run:

| Quantity | Measured | Ghia (1982) | README summary row (33², t=40) |
|---|---|---|---|
| Primary vortex | (0.5312, 0.5625) | (0.5313, 0.5625) | (0.563, 0.594) |
| Bottom-right eddy | (0.8594, 0.1094) | (0.8594, 0.1094) | — |
| Bottom-left eddy | (0.0781, 0.0781) | (0.0859, 0.0781) | — |
| Centerline RMSE | **0.0617** | — | 0.137 |

The primary vortex matches to 1e-4 in x and exactly in y; the bottom-right eddy matches exactly. The
README's headline row reports a coarse 33² *trend* configuration while other rows report defaults —
an inconsistent basis that **understates** the solver. Fix the table, not the solver.

---

## 4. Certification blockers — independently confirmed

Each item below was re-read at source by the lead auditor and is asserted as fact. The remaining 18
criticals are in [`ACTION-LIST.md`](ACTION-LIST.md) and require triage.

### B-1 — Millikan–White reduced mass is wrong for the documented collision pair
`verification/qtt_ramc_stagline/config.rs:35-37`, `src/solvers/qtt/compressible/fitting.rs:72-73`

```rust
/// Reduced mass `μ_sr` of the dominant relaxing collision pair (N₂–N₂ ≈ 14·14/28 = 7), in amu
pub const REDUCED_MASS_AMU: f64 = 7.0;
```

The derivation in the comment is arithmetically wrong. N₂ has mass **28** amu, not 14. The reduced mass
of an N₂–N₂ pair is 28·28/56 = **14 amu**. The value 7 amu is the **N–N atom** pair, mislabelled.

Since `A_sr = 1.16e-3·μ^(1/2)·θ_v^(4/3)` and μ enters inside the exponential, a factor-2 error in μ
changes `A` by √2 and shifts τ_vt substantially — moving T_ve, hence T_a = √(T_tr·T_ve), hence peak
electron density. **The crate's headline result ("peak nₑ = 1.08e19, +0.0 decades vs the RAM-C II
anchor") rests on this constant.**

*Action:* decide the intended pair. If N₂–N₂, set 14.0 and re-baseline the whole RAM-C chain. If N–N was
intended, correct the label and justify why atom–atom dominates. Either way the headline agreement must
be **re-measured, not re-asserted**.

### B-2 — ESKF process noise is not scaled by `dt`
`src/navigation/eskf.rs:105`

```rust
self.cov = mat_add(&fpft, &diag(&process_noise_diag));
```

`Q` is added once per call with no `dt` factor, so `P` growth is proportional to **step count, not
elapsed time**. The supplied diagonal is therefore not a discretisation of any continuous-time noise
density: halving `dt` over a fixed horizon doubles accumulated process noise, and tuning is implicitly
bound to one step size.

*Action:* use `Q_d = Q_c·dt` (or Van Loan), document the diagonal's units as a spectral density, and add
a test that fixes the horizon, halves `dt`, and asserts terminal covariance is invariant.

*To the code's credit:* `update_scalar` uses the **Joseph form** with re-symmetrization, citing Groves
(2013) §3.4.3. Correct, and better than common practice.

### B-3 — `dec_cylinder_verification` has no gate and cannot fail
`verification/dec_cylinder_verification/main.rs` — **0** `process::exit`, **0** assertions

On a solver `Err` it prints, breaks the march, then reports Strouhal and drag from the *truncated* series
and returns 0. `verification/README.md` states the opposite convention. This is the crate's most
expensive harness (~510 s) and its only isolated-cylinder validation.

*Action:* gate St and C_d against the stated Williamson / Dröge–Verstappen bands; exit nonzero on solver
error.

### B-4 — `qtt_rank_dynamic` gate G2 is unreachable
`studies/qtt_rank_dynamic/main.rs:108` with `:164,169`

```rust
let mut peak = init;                          // :164
peak = peak.max(b);                           // :169
if b_peak < b0 { failures.push("G2: ...") }   // :108
```

`peak ≥ init` holds identically and `b0` *is* `init`. G2 can never fire.

*Action:* replace with a gate that can fail — e.g. assert the peak exceeds the smooth-encode floor by a
stated margin, which is what the comment says the gate is for.

### B-5 — Weather example gate "(0) table integrity" cannot fail
`examples/avionics_examples/cfd/plasma_blackout/weather/model.rs:223,264`

`errored: false` is a literal at row construction; the gate is `rows().all(|r| !r.errored)`.

*Action:* populate `errored` from a real per-row outcome, or remove the gate. A gate that cannot fail is
worse than no gate — it reads as assurance.

### B-6 — No CI or Bazel target executes the verification suite
`.github/workflows/` — no `cargo run --example` anywhere

Every external-reference comparison in the crate (exact Riemann, Ghia cavity, Williamson cylinder, RAM-C
flight nₑ) lives in `verification/` binaries that CI compiles and never runs.

*Action:* run the fast harnesses (10 of 13 complete in ≤5 s) on every PR and the three slow ones nightly.
**This single change converts the entire README table from prose into a gate, and is the highest
value-per-effort item in this report.**

### B-7 — QTT Taylor–Green convection "amplitude" gate is computed from the reference alone
`verification/qtt_taylor_green_verification/main.rs:136-145`

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
`verification/qtt_cylinder_verification/config.rs:21,30,32,36`

`D = 2·RADIUS_FRAC·2π = 1.885`, `ν = 0.05`, `U = 1` ⇒ **Re = 37.7**. Line 36 cross-references the
committed DEC isolated-cylinder drag at **Re 100**. Drag coefficient is strongly Reynolds-dependent in
this range, so the comparison is not meaningful as stated.

*Action:* match the Reynolds numbers, or state explicitly that the cross-reference is qualitative only.

---

## 5. Production readiness ranking

Ranking reflects **assurance**, not elegance. Correct math with unfalsifiable gates ranks below correct
math with honest gates.

| Rank | Module | Readiness | Critical | Basis |
|---|---|---|---|---|
| 1 | Pointwise theories (`src/theories/`) | `needs-work` | 1 | Governing equations check out; issues are doc parity, constant traceability, and an MMS continuity gate fed literal zeros |
| 2 | DEC NS rate kernel | `needs-work` | 1 | Cartan/Hodge structure sound; MMS at machine ε; graded-mesh order 2 measured. Doc describes `−i_u(du♭)`; code marches the skew-symmetrized form |
| 3 | DEC solver driver | `needs-work` | 0 | Projection consistent; issues in diagnostics and tolerance justification |
| 4 | QTT compressible marchers | `needs-work` | 0 | RH exact (hand-verified); Sod matches exact Riemann; conservation-under-truncation undocumented |
| 5 | CfdFlow DSL | `needs-work` | 0 | CoW fork and determinism largely as advertised; doc overclaims |
| 6 | Docs-vs-code parity | `needs-work` | 0 | 87 doc-overclaim + 39 doc-gap findings crate-wide |
| 7 | DEC boundary zones | `needs-work` | 1 | Documented `collect_constrained_edges` hook is never called |
| 8 | Test suite & build health | `needs-work` | 1 | Change-detector tests; verification suite absent from CI |
| 9 | Crate-wide constants sweep | `needs-work` | 1 | Negative pressure reaches the flux while floored for the LLF wave speed |
| 10 | Coordinate + tensor bridge | `needs-work` | 1 | `BlendedMap` documents a fold check it does not perform; unguarded ÷det J |
| 11 | Examples | `needs-work` | 2 | Reproducible and well-structured, but two gates cannot fail |
| 12 | Verification harnesses | `needs-work` | 4 | The layer that must be strongest is among the weakest |
| 13 | Studies | `needs-work` | 4 | Several headline findings not supported by what the code measures |
| 14 | Plasma / blackout physics | **`not-ready`** | 2 | Headline result rests on B-1 |
| 15 | Navigation / ESKF | **`not-ready`** | 2 | B-2 plus a covariance/mean inconsistency in `correct_position` |
| 16 | QTT incompressible / immersed | **`not-ready`** | 5 | Penalization drag is definition-dependent: η set by stability not physics, C_d scales with mask smoothing width, ~40 % of reported drag from cells outside the body |

Per-module detail: [`MODULE-INDEX.md`](MODULE-INDEX.md) and [`modules/`](modules/).

---

## 6. Systemic themes

Findings cluster into five patterns. Fixing the pattern is cheaper than fixing 294 instances.

| Axis | Count | Pattern |
|---|---|---|
| doc-overclaim | 87 | Prose describes intended design; code implements a subset. Recurring shape: a doc asserts a property holds "by construction" where no check exists. |
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
- **The definition problem in penalization drag.** The QTT immersed C_d ≈ 23.8 vs DEC ≈ 1.345 gap is
  attributed in the README to blockage, smoothing skirt and transient. That attribution is incomplete:
  the reported quantity depends on the numerical smoothing width and includes cells outside the body, so
  it is not the same observable as the DEC drag. The bond-convergence trend remains valid; the
  cross-reference does not.

---

## 7. What is verified correct

176 items were positively confirmed against a reference. The load-bearing ones:

- **Rankine–Hugoniot normal-shock relations** — exact (hand-verified, §3.4).
- **Plasma frequency** ωₚ = √(nₑe²/ε₀mₑ) — exact, correct CODATA constants. `comms_band` is correctly a
  *parameter*, documented as "GPS L-band ≈ 1.5 GHz" — traceable, not magic.
- **Sod shock tube vs exact Riemann** — L1 0.0175 / 0.0274 / 0.0151 against a 0.03 bound, on a genuinely
  analytic reference. **The crate's strongest gate.**
- **DEC lid-driven cavity vs Ghia (1982)** — primary vortex to 1e-4, bottom-right eddy exact (§3.5).
- **Graded-mesh operator convergence** — second order confirmed for both the convective interior product
  and the viscous Hodge Laplacian at every grading amplitude; finest-pair 1.98–2.00.
- **MMS Taylor–Green** — residual 1.11e-16, amplitude error 6.66e-16, at the f64 floor.
- **ESKF `update_scalar`** — correct Joseph-form covariance update with re-symmetrization, cited.
- **Safety and lint posture** — `unsafe_code = "forbid"` applies; zero `panic!`/`process::exit` in `src/`;
  all 14 `#[allow]` are `too_many_arguments`/`type_complexity`, never correctness-lint suppression.
- **`benches/PERFORMANCE.md`** — full methodology, and honest that `parallel` is *slower* at bench scale.
- **Determinism and reproducibility** of committed example outputs.

---

## 8. Path to certification

Four phases. Phase 1 alone changes the assurance posture more than the other three combined.

**Phase 1 — Make the evidence bite (highest value, lowest effort)**
1. Add CI execution of the verification suite (fast on PR, slow nightly). — **B-6**
2. Add a real gate to `dec_cylinder_verification`. — **B-3**
3. Remove or repair every gate that cannot fail: **B-4**, **B-5**, **B-7**, the `qtt_park2t_blackout`
   gates (ii) and (iv), the MMS `continuity_error` gate, and the energy-budget CI gate.
4. Audit every gate bound for back-fitting; relabel any bound pinned from its own measurement as a
   **regression test**, in the code and in `verification/README.md`.

**Phase 2 — Close the physics defects**
5. Resolve `REDUCED_MASS_AMU` and re-baseline the RAM-C chain. — **B-1**
6. Scale ESKF `Q` by `dt`; add the horizon-invariance test. — **B-2**
7. Fix `correct_position` attitude-error handling.
8. Add the fold/singularity guard `BlendedMap` documents; guard ÷det J.
9. Wire or remove `BoundaryZone::collect_constrained_edges`.
10. Stop negative pressure reaching the flux in `marcher_2d`.
11. Re-found the penalization drag observable: exclude exterior cells, justify η physically, and either
    match Reynolds numbers with DEC or drop the cross-reference. — **B-8**

**Phase 3 — Documentation truth-up (bidirectional)**
12. Reconcile all 87 doc-overclaims. Where prose describes intent, mark it intent.
13. Document the 39 undocumented capabilities.
14. Restate the RAM-C claim consistently — the crate README says "validate against flight data"; the
    verification README correctly says order-of-magnitude. Use the accurate phrasing in both.
15. Fix the lid-cavity summary row to report the default configuration (§3.5).
16. Resolve `Gates`: adopt it everywhere, or retire it. *(Owner decision — no deletion without approval.)*

**Phase 4 — Traceability**
17. Give every load-bearing constant a source, units, and a `papers/` entry.
18. Replace change-detector tests with independent-reference tests in load-bearing modules.

---

## 9. Recommendation

**Do not certify yet.** Certify after Phases 1 and 2 — together a bounded, mechanical workstream with no
solver rewrites.

The distinction that matters for an avionics R&D lab: this crate's **numerics** are in materially better
shape than its **assurance case**. The risk is not that an engineer gets a wrong answer from the DEC or
QTT marchers — on every closed-form reference available, it gets the right one, sometimes to four
decimal places. The risk is that an engineer reads "all gates passed" and concludes something was
checked that was not.

Phase 1 removes that risk directly.

---

## Appendices

- [`MODULE-INDEX.md`](MODULE-INDEX.md) — per-module readiness table
- [`ACTION-LIST.md`](ACTION-LIST.md) — all 294 findings, severity-ordered and actionable
- [`RUN-LEDGER.md`](RUN-LEDGER.md) — execution ledger with exit codes and timings
- [`modules/`](modules/) — full per-module reports with code evidence and adversarial verdicts

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

> **Status: NOT YET CERTIFIABLE. Phase 1 complete; Phase 2 in progress (3 of 4 changes landed,
> plus one follow-up capability). All four certification blockers are resolved; B-1 closed with
> `fix-ramc-vibrational-relaxation-pair`. The remaining Phase-2 change is
> `fix-navigation-filter-correctness` (item 9).**
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
> | **B-1** Millikan–White reduced mass | **RESOLVED** — Phase 2 change `fix-ramc-vibrational-relaxation-pair` |
> | **B-2** No CI executes the verification suite | **RESOLVED** |
> | **B-3** `dec_cylinder_verification` has no gate | **RESOLVED** |
> | **B-4** `BlendedMap` documents an absent fold check | **RESOLVED** — Phase 2 change 4 |
>
> **Phase 2, change 4 is implemented and archived** as
> [`openspec/changes/archive/2026-07-21-resolve-cfd-contract-gaps/`](../../changes/archive/2026-07-21-resolve-cfd-contract-gaps/),
> closing items **8, 11 and 15**; its three capability specs are synced into `openspec/specs/`.
> One task is recorded as **skipped, not done** — see the note at the end of this block. What the
> change found beyond this report:
>
> - **B-4 was worse than "an absent check".** The module doc argued a fold was impossible — two
>   orientation-compatible charts, so `det J_λ` keeps one sign. That reasoning is false for a linear
>   blend: a parameter sweep found **275 accepted configurations that fold**. The claim was not merely
>   unenforced, it was wrong, and the audit had recorded it as the milder defect.
> - **The first implementation of the check was insufficient, and the falsifiability test is what
>   caught it.** Scanning `det` over the lattice the metric trains sample misses `ξ = 1` and `η = 1`;
>   a map degenerating on the fan's outer boundary passed, and its sampled minimum falls off only as
>   `~1/nx`, so refinement would not have caught it until `lx ≈ 20`. The scan now covers the closed
>   domain. This is the concrete return on Phase 1's rule that a gate must be shown to bite.
> - **Gate BM-A had the magic-number defect the audit catalogued, and a worse one underneath.** The
>   gate tested `min|detJ| <= 1e-6` — an absolute bound on an area ratio. But it also ran its own
>   `jacobian_scan`, a line-for-line copy of the constructor's Jacobian algebra, so BM-A verified the
>   copy: a green gate said nothing about the map the crate builds. BM-A now constructs a real
>   `BlendedMap` and reads the shipped scan's measured margin; the duplicate scan is deleted.
>
>   Underneath that sat a **live trap**. The study defined the chart's transverse width as
>   `2·RSHOCK·sin(½dθ)` (fan width at the *standoff* radius) where `BlendedMap` uses
>   `2·(r0+½dr)·sin(½dθ)` (the chord at *mid* radius). Both equal 2.121320 **only because
>   `r0+½dr = 1.5` happens to coincide with `RSHOCK = 1.5`**. The shock standoff is physically
>   independent of the fan geometry — moving it to 1.6 would have silently aimed both gates at a
>   chart the crate never constructs, 6.7% wide of it, with nothing to flag it.
>
>   Neither gate's number moved (`min|detJ| = 1.506`; bonds `114 → 5`), which is the evidence the copy
>   *did* agree at the shipped geometry. BM-A is now falsifiable against shipped code: inflating
>   `DET_FLOOR_FRACTION` makes it exit 1 naming the constructor's refusal.
> - **Item 15's hook was one of two documentation gaps, in opposite directions.** `BoundaryZone`
>   documented five hooks and declared six — `collect_slip_edges` was wired but undocumented, while
>   `collect_constrained_edges` was documented but unwired. Both are now correct. Zone-supplied
>   constrained edges compose by **union** with the structural set, folded after the free-slip un-pin
>   so an explicit constraint outranks a structural relaxation.
> - **The first attempt at the anti-drift test was itself an instance of this report's central
>   defect, and was rejected by the owner.** It scraped the trait and solver sources with
>   `include_str!` and compared the two lists of `collect_*` names. That is a test of *text*: it
>   passes on a hook named only in a comment, fails on reformatting, and cannot detect a hook that is
>   folded but ignored — in a change whose entire purpose was replacing "documented" with
>   "demonstrated". It also broke `bazel test`, which sandboxes each target so a test crate cannot
>   read `../../../src/**`; `cargo test` passed and hid it.
>
>   Replaced with behavioural coverage through the public API: a probe zone implements one hook at a
>   time and each must **change a marched result**. All six fold sites were then demonstrated
>   falsifiable by fault injection. Two flaws in the probe surfaced that a source scan would have
>   concealed — dropping *one* prescribed edge gives `Δ = 0` exactly (that edge is a corner already
>   no-slip pinned), and a probe supplying edges without values pins edges already at zero, making
>   every delta ~1e-18 round-off. **Standing rule adopted: no test may assert on source text.**
>
> **A follow-up change filled the name item 11 reserved.**
> [`2026-07-22-add-dec-scalar-transport-wall-heat-flux`](../../changes/archive/2026-07-22-add-dec-scalar-transport-wall-heat-flux/),
> 37/37 tasks, two capability specs synced. `wall_heat_flux` is now a genuine Fourier surface
> integral `q = −k ∮_S ∇T·n dA` over cut-cell fragments, so the crate has an honest wall heat flux —
> the quantity §4b calls safety-critical for a re-entry TPS consumer, and which it previously had
> nowhere.
>
> - **Change 4's design named the wrong blocker, and this is worth generalising.** It deferred a real
>   flux to "the Gap-2 reacting energy equation". The cut-cell surface was already shipped and in use:
>   `CutFaceFragment` carries a `(D−1)`-area, an outward unit normal and a centroid, and
>   `viscous_surface_force` already integrates `∮ μ(∇u+∇uᵀ)·n dA` over it. What was missing was a
>   *temperature field* — the DEC solver marched velocity only. **A deferral is a claim, and this
>   audit's own habit of accepting one unexamined is what left the name reserved and empty.** The
>   remaining Phase 2 and 3 deferrals deserve the same check.
> - **The flux is not computed on the QTT penalized path, deliberately.** Volume penalization has no
>   wall surface, only a mask smoothed over `SMOOTH_CELLS·dx`; the interface gradient scales as
>   `k·ΔT/w`, inversely with a purely numerical parameter. §5b measured the *drag* moving 6.1× across
>   that sweep, and drag is a volume integral that averages where a wall-normal derivative amplifies.
>   Computing the wall exchange from the penalization source — what `penalization_heat_integral` does
>   — is the standard volume-penalization answer for exactly this reason, so that quantity is
>   defensible as it stands. Only its old name was wrong.
> - **The reference is exact, not tolerated.** A linear profile makes the one-sided reconstruction
>   exact, so `q = −k·G·A` holds to the f64 floor at every rung of a spacing ladder
>   (`|q − analytic| = 0.000e0` at h = 1, 0.5, 0.25, 0.125, 0.0625).
>
> - **Item 11's `preserved_drag_fraction` was never at risk.** The audit reasoned the rename was safe
>   because the quantity is used comparatively; in fact it is computed from the thrust coefficient
>   (`srp_preserved_drag_fraction_kernel(C_T)`) and never reads the heat observable at all.
>>
> **One task skipped, by owner decision.** Change 4's task 5.9 — re-run the full verification suite and
> confirm no harness result moved — was **not discharged**. Two harnesses ran green
> (`mms_taylor_green_verification`, `dec_taylor_green_re1600_verification`); the compressible set (Sod,
> RAMC, cylinder, Park-2T, QTT Taylor-Green, blunt-body) was never run against the diff. It is recorded
> as skipped rather than marked done. The mitigation it rests on is the Phase-1 CI workflow, which runs
> those harnesses on the next PR — so **that CI run is now the only evidence behind "no result moved",
> and is worth watching.** What *is* established without it: the DEC path is byte-identical by
> construction (no shipped zone supplies constrained edges, asserted by test), all three `BlendedMap`
> consumers pass with no geometry refused, and the observable rename left the formula untouched by diff.
>
> **Phase 2, change `close-qtt-solver-envelope` is implemented (commit `cbe5ccd28`) and archived**,
> closing items **12, 13, 14** and resolving item **10** in configuration. It gave the QTT solver family
> the envelope contract the DEC family already had (§7's last theme). What it found beyond this report:
>
> - **Item 12 was a precision trap, not just an unfloored pressure.** The `1e-300` wave-speed floor is
>   exactly `0.0` at `f32` (`num-traits`' `f64→f32` is an infallible cast returning `Some(0.0)`, so the
>   `unwrap_or_else` never fires). Rejecting non-positive pressure instead of flooring it removes the
>   literal entirely, fixing the trap by construction and making the guard identical at every precision.
> - **Item 14's stated fix was infeasible.** D4 said "establish `χ ∈ [0,1]` after quantization by
>   clamping". A fixed-rank tensor train cannot hold pointwise `[0,1]`: clamp+re-quantize only *reduces*
>   the excursion (`−1.78e-3 → −1.21e-3` at bond 4). The contract became clamp-the-bulk + reject-gross +
>   accept a residual bounded by truncation tolerance; the spec was corrected to match.
> - **Item 10 is resolvable but not runnable.** The envelope resolves at `L = 8` (η from a 2.5 %
>   wall-error target), and every config change is forced by a guard this change added — the new
>   diffusive limit refuses the old `dt`, the mask guard rejects bond 4 at `L = 8` as a `−0.15` mask. But
>   a single march is ~17 min and the acceptance harness ~4-9 h. The field is low-rank at every `L`
>   (achieved bond saturates at 24), so this is **not** the tensor-train `O(χ²·L)` cost — that predicts
>   ~1.6× — but a superlinear bottleneck *elsewhere* (projection CG / a dense step) that QTT does not
>   accelerate. The gate is reclassified offline/manual, red for a **solver-performance** reason; see §9
>   item 10.
> - **The adversarial review over the finished diff caught the change's own overclaims — all mine.** A
>   6-dimension review (each finding independently verified) confirmed 8, every one a documentation or
>   spec accuracy defect, the runtime clean: a cost explanation that was mathematically void (a "large
>   constant" cancels from a ratio — corrected by measuring the achieved bond), three stale `L = 5`
>   claims left after the `L = 8` move, and a mask test that passed on the *un-clamped* raw mask. This is
>   the same pattern as change 4's own mistakes — **the remediation needs the same adversarial pass the
>   audit applied to the code**, and running it is now standard for these changes.
>
> **Phase 2, change `fix-ramc-vibrational-relaxation-pair` is implemented and archived**, closing item
> **7 / B-1**, the last certification blocker. It corrected `REDUCED_MASS_AMU` from the invalid `7.0`
> (the N–N pair of two nitrogen *atoms*, which have no vibrational mode) to the N₂–N₂ value `14.007`,
> derived from the constituent masses at a single definition, and added a checked constructor that
> rejects a monatomic relaxing species. What it found beyond this report:
>
> - **The correction removed the headline, exactly as predicted.** Re-measured, not carried over: the
>   Park-2T controller peak `n_e` moves `1.085e19 (+0.0 dec) → 5.310e17 (−1.27 dec)`, matching the
>   −1.27-dec figure the module auditor projected. Per D3 the band was not widened to re-admit the old
>   number; the gate now reports the offset as a tripwire on the corrected value.
> - **What survives is the network, not the controller.** The uncalibrated finite-rate network's renewal
>   arm lands `+0.35 dec`, inside the ±0.70 chemistry-spread band, so the order-of-magnitude flight-data
>   result holds while the closed-form controller's `+0.0` headline is retired as an artifact of the
>   invalid constant.
> - **The examples needed no band edit; retropulsion needed the right order.** Corridor and weather pass
>   unchanged (the evolved network is far less μ-sensitive than the closed-form controller). Retropulsion's
>   onset gate is a self-consistency check against `weather_table.csv`; regenerating it first
>   (weather → retropulsion) makes the flown onset match the interpolated one (12.60 s vs 12.61 s), no
>   bound moved.
> - **The adversarial pass over the finished diff caught the change's own overclaims, all mine — and
>   two passes still left a tail.** The first pass (each finding independently verified) found seven
>   stale-headline defects: the harness's own README still asserting the retired "within 3×" claim, a
>   method docstring citing the μ = 7 α, and a contradiction where the pair table called N₂–N₂ the
>   "longest" τ while listing the heavier N₂–O₂ above it (resolved by stating O₂ is dissociated out at
>   8044 K), among others. All fixed. The second pass, extended to the project website, found three more —
>   a **blocker** on the public roadmap still listing "+0.0 decades" as a works-today result, a stale
>   corridor branch-table, and a grammar slip — plus a source doc claim ("the marched closure lands
>   within the chemistry spread") that is false for the corrected controller. All fixed. **One class is
>   left open and recorded:** the website *tutorial* walkthroughs (`stage-1-corridor`, `stage-2-weather`,
>   `stage-3-retropulsion`, `handle-regime-change`) still quote the pre-baseline example outputs (onset
>   74.7 km, the coarse-miss table, the 1.78 m/s landing) — a separate, larger figure-sync than the
>   validation pages, deferred to a follow-up. So the honest tally is "seven found and fixed, three more
>   found and fixed, one class deferred", **not "clean"** — the same lesson as change 4 and
>   `close-qtt-solver-envelope`: even the adversarial pass leaves a residue, and claiming otherwise is
>   itself the overclaim the audit exists to catch.
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

> **Status 2026-07-24: all four resolved.** B-2 and B-3 closed in Phase 1, B-4 in Phase 2 change 4, and
> B-1 in Phase 2 change `fix-ramc-vibrational-relaxation-pair`. The entries below are kept at their
> as-found wording, with resolution noted inline, so they stay comparable to the module reports.

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

**Resolved 2026-07-24** (`fix-ramc-vibrational-relaxation-pair`). The pair is N₂–N₂; `μ` is derived from
its constituent masses (`m(N) = 14.007`, `m(N₂) = 28.014`, `μ = m(N₂)/2 = 14.007`) at a single definition in
`deep_causality_cfd`, re-exported to the harness and the example world, with a checked constructor that
rejects a monatomic relaxing species. The RAM-C chain and the three plasma-blackout examples were
re-measured and re-baselined, and the gates were re-derived from the corrected physics, not re-tuned. The
Park-2T controller now lands −1.27 decades below the anchor (5.310e17), measured, and that offset is reported
rather than re-admitted; the finite-rate network's renewal arm survives at +0.35 dec. See the remediation
block at the top of this report.

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

### B-4 — `BlendedMap` documents a fold check it does not perform — ✅ RESOLVED (2026-07-21)
`src/coordinate/blended.rs:17,171`

> **Resolved by change 4, and the finding understated the defect.** The doc did not merely fail to
> enforce one-signedness — it argued the property followed from the two charts being
> orientation-compatible, which is false for a linear blend: **275 accepted configurations fold**.
> `BlendedMap::new` now scans the closed domain and refuses a non-finite, sign-changing or
> near-singular determinant against a floor **relative** to `dr·span_y`. See the header block.

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

  ✅ **Resolved in configuration (`close-qtt-solver-envelope`):** `ETA` is now from a 2.5 % wall-error
  target and the grid is refined to `L = 8` so `√(ην) ≈ dx` is resolved, not the mask skirt. The gate
  cannot be *run* in CI at that resolution (~4-9 h) and is reclassified offline/manual; the remaining
  blocker is solver performance, not the parameter choice (§9 item 10).

**Two ESKF defects compound.** `Q` is added with no `dt` scaling (`eskf.rs:105`), so covariance growth
tracks step count rather than elapsed time; and `update_scalar` divides by an unguarded innovation
covariance with no validation of the measurement variance, reachable from the public API
(`P[i][i]=0, r_var=0` → `k = NaN` written into state and covariance). To the code's credit the same
function uses a correct Joseph-form update citing Groves (2013) §3.4.3.

**`wall_heat_flux` is not a flux** — it is a temperature-weighted volumetric rate with no gradient,
conductivity or surface normal, and the production path hardcodes `t_wall = 0` with no way to configure
it. For a re-entry TPS consumer this is the safety-critical quantity. — ✅ **RESOLVED (2026-07-21):**
renamed `penalization_heat_integral`, freeing `wall_heat_flux` for a genuine Fourier-law
implementation; `T_w` moved into `QttBody` beside `ubx`/`uby` and recorded in the run output.
**The reserved name was then filled (2026-07-22):** `wall_heat_flux` is a real surface integral
`q = −k ∮_S ∇T·n dA` on the DEC cut-cell path, so the safety-critical quantity now exists.

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

**No bound was widened.** Per owner decision the gate is kept, so a known-open finding does not block
merges. **Update (`close-qtt-solver-envelope`):** item 10 resolved the *configuration* (η from a
wall-error target, `L = 8` resolving the layer), but the acceptance harness then cost ~4-9 h to run, so
the harness was **reclassified from nightly `KNOWN-FAILING` to `OFFLINE_HARNESSES`** in
`.github/workflows/cfd_verification.yml` — still accounted for by the completeness check, run manually,
not in CI. The failure it recorded (`C_d` tracks blur width, no η→0 limit) was a *resolution* finding;
the current blocker is *solver performance* at the resolution the physics needs (§9 item 10).

This is the clearest vindication of the audit's premise: the harness passed for years while gating
three things — no-slip (provably invariant across the whole smoothing sweep), bond convergence (bound
eleven orders above the measurement), and `0 < C_d < 100` — none of which constrained the drag.

---

## 5c. What remediation taught about the audit's own method

*Added 2026-07-21 during Phase 2 change 4.*

Three lessons, each earned by a mistake made while fixing this report's findings rather than by
analysis. They are recorded because they change how the remaining Phase 2 items should be worked.

**1. A "by construction" claim may be false, not merely unchecked.** This report classified B-4 as a
documented guarantee that nothing enforced — the milder reading. The guarantee was also *wrong*: the
doc's argument (orientation-compatible charts keep `det J` one-signed) fails for a linear blend, and
275 accepted configurations fold. The other 86 doc-overclaims should be read the same way — **disprove
the claim before enforcing it**, because enforcing a false claim produces a check that rejects valid
input or, worse, one that cannot fire.

**2. Writing the falsifiability test is what finds the hole in the fix.** Change 4's first
determinant scan looked complete and passed everything. Constructing the near-singular case that was
*supposed* to fail revealed the scan skipped the domain's closed boundary, where the sampled minimum
decays only as `~1/nx` — undetectable below `lx ≈ 20`. Phase 1's rule was "show the gate bites"; the
sharper form is **write the input that must fail before believing the check is complete**.

**3. A gate can verify a copy of the code instead of the code.** Gate BM-A ran its own line-for-line
duplicate of the constructor's Jacobian algebra, so it was green regardless of what `BlendedMap`
actually did — and underneath sat a `span_y` definition that agreed with the crate's only by numerical
coincidence. This is a distinct failure mode from the 72 tautologies catalogued in §7: not a gate that
cannot fail, but a gate measuring **the wrong artifact**. The QTT convection gate flagged in §4b
("tests a re-implementation, not the shipped solver") is the same shape, which suggests auditing the
remaining harnesses for it specifically.

**4. A deferral is a claim, and this report accepted one unexamined.** Item 11's design deferred a
real wall heat flux to "the Gap-2 reacting energy equation". That was wrong: the cut-cell surface —
area, outward normal, centroid — was already shipped and already integrated by
`viscous_surface_force`. The only missing piece was a temperature field on the DEC path. The cost of
not checking was a safety-critical quantity left absent behind a plausible-sounding dependency. The
remaining Phase 2 and Phase 3 items carry several deferrals of the same shape; each should be
checked against what is actually in the tree before it is planned around.

**5. The deferral sweep (2026-07-22) — six tested, two false, one rationale wrong.** Acting on
lesson 4, every deferral making a factual claim about the tree was checked against it:

| Deferral | Verdict | Evidence |
|---|---|---|
| Nav attitude (a) is "feature-sized" | **FALSE** | `Quaternion` with `from_axis_angle`, `to_rotation_matrix`, `normalize`, `slerp` ships in `deep_causality_num_complex` — already a `deep_causality_cfd` dependency. (a) is a field plus two call sites. **(a) promoted into change 3; (b) dropped.** |
| `collect_constrained_edges`' consumer is `aperture-resolved-noslip` | **FALSE** | That capability is implemented, is the **default** (`NoSlipConstraint::new(.., true)`), and derives constraints via `cut_face_constraints` in `no_slip.rs` — not via the hook. The hook has no known consumer. Corrected at both docstrings. |
| RAMC: "the composition it needs is already computed" | **TRUE** | `air_n2_mole_fraction` / `air_o2_mole_fraction` are shipped kernels, already called at `qtt_ramc_stagline/main.rs:197-198`. The mixture follow-up is small. |
| Item 12 — negative pressure reaches the flux | **TRUE** | `marcher_2d.rs:135` floors `p` only for the wave speed. |
| Item 13 — QTT constructors validate nothing | **TRUE** | Zero `return Err` / `is_finite` checks in `QttImmersed2d::new`. |
| Item 14 — mask not clamped to `[0,1]` | **TRUE** | No clamp on the quantized mask. |

**Two of six were false, and both had been stated confidently.** The pattern in each is the same: a
negative asserted from an incomplete search. The attitude one was found only because the owner named
the crate I had not looked in — I had searched `algebra`, `num` and `cfd`, concluded "no concrete
Quaternion type exists", and was wrong. **A deferral resting on "X does not exist" is only as good as
the search behind it, and the search must be stated so it can be checked.**

**And one caution about the remediation itself.** Change 4's first anti-drift test scraped source text
with `include_str!` — a test that asserts on *code as text*, inside a change whose purpose was
replacing "documented" with "demonstrated". It would have passed on a hook mentioned only in a
comment. It was rejected by the owner, and a standing rule adopted: **no test may assert on source
text; behavioural coverage through the public API only.** It also broke `bazel test`, which sandboxes
each target — `cargo test` passed and concealed it, so **`bazel test //...` is the check that
matters** before claiming a change is verified.

---

## 6. Production readiness ranking

Ranking reflects **assurance**, not elegance. Critical counts are post-review.

| Rank | Module | Readiness | Crit | Basis |
|---|---|---|---|---|
| 1 | Pointwise theories | `needs-work` | 0 | Governing equations check out; issues are doc parity and constant traceability |
| 2 | DEC NS rate kernel | `needs-work` | 0 | RK4 tableau exact; Δ = dδ+δd correctly ordered; δ = M⁻¹BM is the true discrete adjoint, so −νΔ provably dissipates; Leray projection inside every RK4 stage. Docs describe a different convective operator than the code marches |
| 3 | DEC solver driver | `needs-work` | 0 | Projection consistent; issues in diagnostics and tolerance justification |
| 4 | QTT compressible marchers | `needs-work` | 0 | RH exact (hand-verified); Sod matches exact Riemann; ✅ non-positive pressure now rejected before the flux across all four marchers (item 12) |
| 5 | CfdFlow DSL | `needs-work` | 0 | CoW fork and determinism largely as advertised; doc overclaims |
| 6 | Docs-vs-code parity | `needs-work` | 0 | 87 doc-overclaim + 39 doc-gap findings crate-wide |
| 7 | DEC boundary zones | `needs-work` | 0 | ✅ hook wired (union composition) and all six fold sites now covered behaviourally; remaining items are doc parity |
| 8 | Examples | `needs-work` | 0 | Reproducible and well-structured; two gates cannot fail |
| 9 | Crate-wide constants sweep | `needs-work` | 0 | ✅ negative-pressure flux and the f32 pressure-floor collapse fixed (item 12/12b); Mach-1.05 shock floor remains |
| 10 | Studies | `needs-work` | 0 | Several headline findings not supported by what the code measures |
| 11 | Test suite & build health | `needs-work` | 0 | Change-detector tests; verification suite absent from CI (rolled into B-2) |
| 12 | Coordinate + tensor bridge | `needs-work` | 0 | ✅ B-4 resolved — invertibility enforced over the closed domain, gate BM-A now measures the shipped constructor |
| 13 | Verification harnesses | `needs-work` | 1 | B-3; the layer that must be strongest is among the weakest |
| 14 | Plasma / blackout physics | **`needs-work`** | 0 | B-1 resolved; headline retired and re-derived (Park-2T −1.27 dec reported, network +0.35 dec survives). Open levers remain (T_e=T_ve lumping, single-pair τ, mixture-weighting follow-up) |
| 15 | Navigation / ESKF | **`not-ready`** | 0 | Two compounding filter defects (§4b); unguarded public API |
| 16 | QTT incompressible / immersed | **`not-ready`** | 0 | ✅ constructor envelope validated (item 13), mask `[0,1]` enforced (item 14), Brinkman envelope resolved in config (item 10); cylinder drag gate now offline (solver cost), so headline drag still unverified |

Modules 15 and 16 carry **zero criticals** yet remain `not-ready`. That is deliberate: their defects are
individually major but jointly remove the basis for trusting the module's headline output.

---

## 7. Systemic themes

| Axis | Count | Pattern |
|---|---|---|
| doc-overclaim | 87 | Prose describes intended design; code implements a subset. Recurring shape: a doc asserts a property holds "by construction" where no check exists. B-4 was the sharpest case and is now closed — instructively, its "by construction" argument was not merely unchecked but **false**, so the pattern is worth treating as a claim to disprove rather than a claim to enforce. |
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
- **The DEC family validates its envelope; the QTT family did not — ✅ now closed.** As found,
  `dec_ns_solver/step.rs` rejected CFL and diffusive-limit violations while `QttImmersed2d::new` and
  `QttIncompressible2d::new` validated nothing. `close-qtt-solver-envelope` gave the QTT constructors the
  same contract (η, `dt` vs the penalization and diffusive limits, ν, spacings), with matching
  diagnostics. The sibling no longer returns numbers for an out-of-envelope configuration.

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

**Phase 2 — Close the physics defects — ⬅ IN PROGRESS**

Specified as four openspec changes. **Three are implemented and archived:** change 4
(`resolve-cfd-contract-gaps`, items 8 / 11 / 15), `close-qtt-solver-envelope` (items 12 / 13 / 14, and
item 10 resolved in configuration), and `fix-ramc-vibrational-relaxation-pair` (item 7 / B-1, the last
certification blocker). The remaining one is **`fix-navigation-filter-correctness` (item 9)**, specified
and not started.

The `close-qtt-solver-envelope` group turned up the one place where "detectable" was not enough: item
10's acceptance test — the η ladder converging — needs `L = 8`, at which the harness costs hours, so it
cannot run in CI. The envelope is resolved in configuration and the gate is offline/manual; closing it
for real is a solver-acceleration follow-up, not a Phase-2 parameter fix (item 10).

7. ✅ **DONE** — Resolve `REDUCED_MASS_AMU` at both sites and re-baseline the RAM-C chain and the three
   examples. — **B-1** Corrected to the N₂–N₂ value `14.007`, derived from the constituent masses at a
   single definition, with a monatomic-species rejection guard. The chain and examples were re-measured
   and re-baselined; gates re-derived, not re-tuned (Park-2T −1.27 dec reported; network +0.35 dec
   survives). Two adversarial review passes; the website was updated as a downstream consumer.
8. ✅ **DONE** — Implement the fold/singularity check `BlendedMap` documents; guard ÷det J. — **B-4**
   Scan covers the **closed** domain against a floor relative to `dr·span_y`; a fold is reachable
   (275 configurations) so the sign check is falsifiable; gate BM-A rebuilt on the shipped constructor
   and its duplicate `jacobian_scan` deleted, retiring a latent `span_y` mismatch.
9. Scale ESKF `Q` by `dt`; guard the innovation covariance and validate measurement variance; add a
   horizon-invariance test; fix `correct_position` attitude handling. — §4b
10. Resolve the Brinkman envelope: choose η from a wall-error target, document the `η ≥ dx²/ν` resolution
    constraint, and state that it is currently violated 48×. — §4b, §5b.
    ⚙️ **PARTIALLY DONE (`close-qtt-solver-envelope`, implemented + archived): resolved in configuration,
    blocked on solver cost for verification.** η is now chosen from a 2.5 % wall-error target and the grid refined
    to L=8 (256²) so `√(ην) ≈ dx` is resolved — the config is physically correct and passes the new
    envelope checks. **But the acceptance test cannot be run at feasible cost:** measured per-step
    wall-clock rises 0.05 s → 16.3 s (L=5 → L=8, 326×). The field is low-rank at every L (achieved bond
    saturates at the cap 24 across L=5..8), so this is **not** the tensor-train `O(χ²·L)` cost — that
    predicts ~1.6× at fixed bond — but a superlinear bottleneck **elsewhere** in the per-step solve (the
    projection CG and/or a dense step) that QTT compression does not accelerate. A single march is
    ~17 min and the full η-ladder harness ~4-9 hours; L=9 would be days. The cylinder gate is therefore
    **reclassified as offline/manual, not retired** — red for a **solver-performance** reason, not a
    parameter choice, and the low-rank thesis is defeated by a non-QTT bottleneck. Closing it is a
    solver-acceleration follow-up (`cfd-industry-scaling`), and it directly substantiates the
    minutes-not-hours north-star as an open risk. The envelope-correctness fixes (items 12, 13, 14)
    landed fully.
11. ✅ **DONE** — Renamed `penalization_heat_integral`; `T_w` configurable via `QttBody` and recorded
    in the run output. — §4b
    **And the reserved name is now filled** (`2026-07-22-add-dec-scalar-transport-wall-heat-flux`):
    `wall_heat_flux` is a genuine Fourier surface integral `q = −k ∮_S ∇T·n dA` over cut-cell
    fragments, exact on a linear profile at every spacing. Required adding passive scalar transport
    to the DEC path, which the crate did not have; the cut-cell surface it differentiates against was
    already shipped.
12. ✅ **DONE** (`close-qtt-solver-envelope`) — Reject non-positive pressure in all four marchers, not
    just `marcher_2d`, and not by flooring: a shared guard refuses `p ≤ 0` before the flux, removing the
    `1e-300` floor (and its `f32`-degeneracy) by construction. Precision parity proven at f32 and f64.
13. ✅ **DONE** (`close-qtt-solver-envelope`) — Numerical-envelope validation added to
    `QttIncompressible2d::new` and `QttImmersed2d::new` (η, `dt` vs the penalization and diffusive
    limits, ν, spacings), with DEC-`cfl_check` diagnostic quality. No shipped config is refused.
14. ✅ **DONE** (`close-qtt-solver-envelope`) — Body mask clamped after quantization at the
    `mask_from_fn` chokepoint, with a gross-excursion rejection. Exact `[0,1]` on a lossy train proved
    infeasible; the enforceable contract is "in range to truncation tolerance", stated in the spec.
15. ✅ **DONE** — Wired into the constrained projection (union composition, applied after the
    free-slip un-pin); the intended consumer `aperture-resolved-noslip` is recorded at the hook. All
    six fold sites are covered behaviourally and demonstrated falsifiable by fault injection.

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

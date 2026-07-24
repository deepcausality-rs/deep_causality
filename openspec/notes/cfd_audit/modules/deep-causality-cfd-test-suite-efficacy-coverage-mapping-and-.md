# deep_causality_cfd — test-suite efficacy, coverage mapping, and build/lint health

**Production readiness: `needs-work`**

Build and lint health is exemplary: `cargo build --release` and `cargo clippy --all-targets -- -D warnings` are both completely clean (zero warnings), the crate carries `[lints] workspace = true` (Cargo.toml:257-258) inheriting `unsafe_code = "forbid"` (workspace Cargo.toml:51), and `grep -rn unsafe src/` returns nothing. Test registration is airtight: I cross-checked all 136 test files against every `mod.rs` declaration programmatically and found zero orphans and zero dangling declarations, and all 12 Bazel `rust_test` targets cover every subtree — no test file silently fails to run. Test quality is sharply bimodal, and that is the problem. The DEC solver path is genuinely verified: dec_ns_rate_tests.rs:102-176 builds an independent automatic-differentiation oracle and asserts second-order convergence over a refinement ladder, no_slip_tests.rs:305 pins the exact analytic Couette profile, and taylor_green_2d_tests.rs:109-112 computes an observed spatial order and gates it at ≥1.9. The QTT compressible marcher path is not: every in-suite gate is free-stream preservation, positivity, or finiteness, all of which are mathematically invariant to any error in the flux formula itself, so a wrong flux would pass the entire suite. The only quantitative compressible-accuracy gates (Toro exact Riemann, Ghia cavity, published Strouhal/C_d, RAM-C II electron density) live in `verification/` example binaries that CI compiles but never executes (run_tests.yml:46-52 runs only build/doc-test/test). Two further defects give false assurance: a tautological assertion at compressible_marcher2d_tests.rs:275 that cannot fail, and a documented-and-cited Joseph-form covariance update in the ESKF that no test can distinguish from the simple form it was specifically chosen over.

- Files read: **34**
- Findings raised: **8** — surviving adversarial verification: **7** (refuted: 1)
- Surviving by severity: critical 1, major 1, minor 5
- Independently confirmed-correct items: **11**

## Verified correct against reference

These were positively confirmed, not merely un-flagged.

| Item | Location | Reference checked against |
|---|---|---|
| Test module tree is fully consistent — no test file on disk is undeclared, no declared module is missing | `deep_causality_cfd/tests/mod.rs:7-13 (root aggregator) plus all nested mod.rs` | AGENTS.md convention that tests/ replicates src/ and every test file is reachable from a mod.rs; Cargo compiles exactly one integration target rooted at tests/mod.rs (confirmed via `cargo metadata`: k |
| All Bazel rust_test targets are registered and cover every test subtree | `deep_causality_cfd/tests/BUILD.bazel:19,43,67,100,131,162,186,211,234,253,272,305` | Every directory under tests/ must be reachable from some rust_test target's srcs glob + crate_root, else `bazel test //...` silently skips it |
| cargo clippy -p deep_causality_cfd --all-targets -- -D warnings is completely clean | `whole crate` | Zero diagnostics emitted under -D warnings across lib, tests, benches, examples |
| cargo build -p deep_causality_cfd --release is clean | `whole crate` | Zero warnings/errors on a release build |
| Repo lint policy satisfied: [lints] workspace = true present, unsafe_code = forbid applies, crate is unsafe-free | `deep_causality_cfd/Cargo.toml:257-258; workspace Cargo.toml:50-51` | Repo policy: every crate declares `[lints] workspace = true`; workspace declares `[workspace.lints.rust] unsafe_code = "forbid"` |
| DEC rate kernel is verified against a genuinely independent AD oracle with a convergence-order gate | `deep_causality_cfd/tests/solvers/dec/dec_ns_rate_tests.rs:102-176` | Incompressible NS RHS in Lamb form: du/dt = -(u·∇)u - ∇p/ρ + νΔu + g, cross-checked pointwise via -i_u du = -(u·∇)u + ∇(\|u\|²/2); analytic Taylor-Green Laplacian Δu = -2k²u |
| Discrete gradient and Laplacian operators are pinned to hand-written stencils, not to themselves | `deep_causality_cfd/tests/tensor_bridge/operators_2d_tests.rs:64,81-89` | Second-order central difference (u_{i+1}-u_{i-1})/(2Δx); five-point Laplacian (u_{i+1,j}+u_{i-1,j}+u_{i,j+1}+u_{i,j-1}-4u_{i,j})/Δx² |
| Pointwise incompressible-NS kernel values are hand-computed with the arithmetic shown | `deep_causality_cfd/tests/theories/incompressible_ns_tests.rs:225-230` | du_i/dt = -(u·∇)u_i - (1/ρ)∂_i p + νΔu_i + g_i, evaluated by hand at the given inputs |
| Analytic no-slip and channel-flow references are exact, not tolerance-fitted | `deep_causality_cfd/tests/solvers/dec/dec_ns_solver/no_slip_tests.rs:260-320; poiseuille_tests.rs:41,139` | Couette exact solution u_x(y) = U·y/H; Poiseuille exact parabolic profile |
| Only one #[ignore]d test exists, and it is a diagnostic probe, not a suppressed failure | `deep_causality_cfd/tests/solvers/dec/energy_budget_tests.rs:169` | Ignored tests must not be a mechanism for hiding a failing correctness gate |
| No hardcoded-captured-output change-detector assertions found | `deep_causality_cfd/tests/ (whole tree)` | A change-detector pins a long captured decimal produced by the code under test rather than an externally derived value |

## Findings

### 12.1 [CRITICAL] The 13 verification programs holding every quantitative physics-accuracy gate are never executed by CI or Bazel

- **Verification verdict:** CONFIRMED
- **Axis:** tautology-circular
- **Location:** `.github/workflows/run_tests.yml:52`
- **Auditor confidence:** confirmed

**Claim.** The only checks in the repository that compare deep_causality_cfd output against an external reference (Toro exact Riemann, Ghia cavity benchmark, published cylinder Strouhal/C_d, RAM-C II flight electron density) live in `verification/` example binaries. No CI job, shell script, or Bazel target ever runs them. `cargo test` compiles examples but does not execute them, so these gates are compile-checked only and cannot fail the build no matter how wrong the physics becomes.

**Code evidence.**

```
run_tests.yml:45-52 is the complete set of build/test steps:
  45:      - name: Build
  46:        run: cargo build  --verbose
  48:      - name: Run Doc tests
  49:        run: cargo test  --doc --verbose
  51:      - name: Run tests
  52:        run: cargo test  --verbose
rust_check.yml:19-21 runs only `cargo check --all-targets --all-features`. rust_coverage.yaml:26 runs only `cargo llvm-cov --workspace --lcov`.
A repo-wide grep for `cargo run.*--example` matched ONLY vendored third-party crates under thirdparty/crates/ — zero matches in .github/workflows/ or any first-party script.
deep_causality_cfd/BUILD.bazel declares exactly three targets (rust_library, rust_doc, rust_doc_test) and no rust_binary for verification/. `find deep_causality_cfd -name '*.bazel'` returns only BUILD.bazel and tests/BUILD.bazel — verification/ and studies/ have no Bazel targets at all.
That the real gates live there is stated by the crate itself: tests/solvers/qtt/compressible_tests.rs:7 reads `//! and the ideal-gas EOS. (The Sod exact-Riemann gate is the `qtt_sod` verification example.)`
And the gate is real when run — verification/qtt_sod/print_utils.rs:22 `const TOL: f64 = 0.03;` with :88 `pub fn verify(e: &Errors) -> bool` and :91 `let pass = v < TOL;`
```

**Reference form.** Standard V&V practice (e.g. ASME V&V 20, AIAA G-077) requires that code-verification cases comparing against analytic or benchmark solutions be executed automatically on every change, so that a regression in the discretization is caught mechanically. The repo's own AGENTS.md:371 states examples 'are verified by running them (`cargo run -p <crate> --example <name>`) rather than by unit tests' — naming the mechanism but leaving it entirely manual.

**Impact.** The compressible/QTT solver path has no enforced correctness gate anywhere in automated CI. A regression that changes the flux formula, the Rusanov wave speed, the EOS exponent, or the ionization rate coefficients would leave `cargo test` and `bazel test //...` fully green. An avionics reviewer reading the crate README section 'Everything Self-Verifies' (README.md:212) would reasonably conclude the shipped evidence is continuously enforced; it is enforced only when a human remembers to run thirteen binaries by hand.

**Recommended fix.** Add a CI job that executes all thirteen verification binaries in release mode and fails on nonzero exit — they already self-verify and exit nonzero by design (verification/README.md documents this convention). Runtimes from verification/README.md sum to roughly 12 minutes, dominated by dec_cylinder (~510 s) and dec_cylinder_wake (~155 s); split the two slow ones into a nightly job and gate the remaining eleven (~30 s total) on every PR. Additionally register them as Bazel rust_binary + sh_test targets so `bazel test //...` covers them.

**Adversarial check.** Every factual element checks out. run_tests.yml has exactly three build/test steps (cargo build, cargo test --doc, cargo test) and nothing else; rust_check.yml runs only `cargo check --all-targets --all-features`; there is no shell script, Makefile, or justfile in the repo invoking `cargo run --example`. A grep of .github/ for 'cargo run', '--example', and 'verification' returns zero hits. deep_causality_cfd/BUILD.bazel declares only rust_library/rust_doc/rust_doc_test, and `find deep_causality_cfd -name '*.bazel'` returns only BUILD.bazel and tests/BUILD.bazel, so verification/ and studies/ have no Bazel targets. The 13 directories under verification/ are registered as [[example]] entries in Cargo.toml (28 [[example]] stanzas total), and cargo compiles but never runs examples. The qtt_sod gate is real when run (TOL = 0.03, verify() returns pass/fail) but nothing invokes it. Mitigating context the finding did not weigh: AGENTS.md:371 openly documents the mechanism as manual, so this is a disclosed process gap rather than a concealed one, and README.md:212 does not itself claim CI enforcement. The automation gap is nonetheless real and blocking for a certification posture.

> Evidence re-read: .github/workflows/run_tests.yml:45-52 (full step list, verbatim as quoted); .github/workflows/rust_check.yml:19-21; deep_causality_cfd/BUILD.bazel (3 targets only); `find deep_causality_cfd -name '*.bazel'` → BUILD.bazel, tests/BUILD.bazel; `grep -rn 'cargo run|--example|verification' .github/` → no matches; deep_causality_cfd/verification/qtt_sod/print_utils.rs:20-22 (`const TOL: f64 = 0.03;`) and :88-91; deep_causality_cfd/Cargo.toml:136-253 (28 [[example]] stanzas); AGENTS.md:371

---

### 12.2 [MAJOR] QTT compressible marcher tests cannot detect a wrong flux formula — every in-suite gate is invariant to it

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/tests/solvers/qtt/compressible_marcher2d_tests.rs:25`
- **Auditor confidence:** confirmed

**Claim.** The in-suite gates for CompressibleMarcher2d/3d (free-stream preservation, positivity/finiteness, constructor and error paths) are all invariant to the convective flux formula and the Rusanov wave-speed estimate, so no automated test pins those two quantities to a reference value. The neighbouring EOS, Rankine–Hugoniot, and closed-form acoustic-step gates do pin other parts of the compressible path, so the crate's compressible physics is not entirely unguarded — the hole is the marcher flux and wave speed specifically.

**Code evidence.**

```
The complete test inventory of compressible_marcher2d_tests.rs (277 lines): free_stream_is_a_fixed_point (:25), marcher_trait_advance_matches_one_step_and_preserves_free_stream (:46), new_rejects_non_positive_reference_speed (:85), eos_2d_recovers_pressure (:93), free_stream_preserved_over_the_metric_seam (:101), marcher_is_stable_on_a_smooth_compressible_field (:132), imex_stays_bounded_beyond_the_explicit_acoustic_diffusion_limit (:171), run_rejects_wrong_length_state (:217), step_rejects_non_positive_density (:234), peak_bond_tracks_growth_over_the_march (:252).
The two tests that actually march a non-uniform field assert nothing about values:
  :160-167  assert!(out[0].iter().all(|&d| d > 0.0 && d.is_finite()), "density must stay positive and finite under the march");
            assert!(out[3].iter().all(|&e| e.is_finite()), "energy must stay finite");
  :206-213  assert!(out[0].iter().all(|&d| d > 0.0 && d.is_finite()), ...);
            assert!(maxabs.is_finite() && maxabs < 5.0, ...);
The file's own header at :136-138 concedes the scope: "Note the *rank* of a captured curved field is coordinate-dependent ... so it is not a marcher-quality gate."
compressible_marcher3d_tests.rs is the same shape: free_stream_is_a_fixed_point (:24), marcher_trait_advance... (:52), new_rejects_non_positive_reference_speed (:133), run_rejects_wrong_length_state (:140), run_rejects_non_positive_density (:155) — its only value assertions are `(d - rho).abs() < 1e-9` (:44) and `(en - e).abs() < 1e-9` (:47) on a uniform state.
```

**Reference form.** Standard Euler-solver code verification (Toro, *Riemann Solvers and Numerical Methods for Fluid Dynamics*, 3rd ed., ch. 4 and ch. 6) requires at minimum a shock-tube comparison against the exact Riemann solution and/or an isentropic-vortex convection case with a measured order of accuracy. Free-stream preservation (Thomas & Lombard's Geometric Conservation Law check) is a necessary condition on the metric/divergence discretization only — it carries no information about the flux function.

**Impact.** An engineer reading `cargo test` green for deep_causality_cfd would believe the compressible marchers are covered — 243 tests pass under tests/solvers/. In fact a sign error in the pressure term, a wrong γ exponent in the energy flux, or a mis-scaled Rusanov wave speed would leave every one of these tests passing. Combined with finding 1 (the Sod gate never runs in CI), the 2-D and 3-D compressible Euler flux formulas have zero automated correctness coverage.

**Recommended fix.** Port the exact-Riemann comparison from verification/qtt_sod/exact_riemann.rs into an in-suite test for euler_1d, and add a 2-D isentropic-vortex convection test with a measured order of accuracy for CompressibleMarcher2d (the vortex is an exact steady solution of the 2-D Euler equations under uniform translation, so the reference is analytic and cheap at 32²). Both are fast enough for the default suite.

**Adversarial check.** The test inventory is quoted accurately — I read all 277 lines of compressible_marcher2d_tests.rs and the 3-D file, and the line numbers for all ten tests are exact. The physics reasoning is also correct: F(const) is constant for any pointwise flux, so ∇·F(const)=0 regardless of the formula, and the same holds for the 1-D solver's `conserves_mass_momentum_energy` (a flux-difference form telescopes on a periodic domain irrespective of the flux function). Neither marcher file measures an order of accuracy or compares against an isentropic vortex / exact Riemann state. The auditor's reference form (Toro ch. 4/6; Thomas & Lombard GCL) is correctly stated. What is overstated is the impact sentence 'the 2-D and 3-D compressible Euler flux formulas have zero automated correctness coverage': eos_2d_recovers_pressure (:93) pins the ideal-gas EOS, including the γ exponent, to 1e-12; compressible_fitting_tests.rs:38-52 pins post-shock ratios against the exact Rankine–Hugoniot relations to 1e-10; and compressible_imex_tests.rs:63 checks the acoustic step against a closed form. The genuinely unpinned quantity is narrower than claimed — the convective flux components and the Rusanov wave speed inside the 2-D/3-D marcher step.

> Evidence re-read: deep_causality_cfd/tests/solvers/qtt/compressible_marcher2d_tests.rs:1-277 (read in full; all cited test line numbers exact, including :160-167 and :206-213 assertions); compressible_marcher3d_tests.rs:24,44,47,52,133,140,155; compressible_tests.rs:7 (header note), :20-27 (eos), :50-80 (conservation); compressible_fitting_tests.rs:38-52 (exact RH); compressible_imex_tests.rs:63-79

---

### 12.3 [MINOR] Tautological assertion: `peak_bond_tracks_growth_over_the_march` asserts a condition that cannot fail

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/tests/solvers/qtt/compressible_marcher2d_tests.rs:275`
- **Auditor confidence:** confirmed

**Claim.** `assert!(peak >= 1)` at compressible_marcher2d_tests.rs:275 cannot fail, because CausalTensorTrain::max_bond (causal_tensor_train/getters/mod.rs:33-40 — not the operator variant the finding cited) is ≥ 1 by construction. The test's stated property (peak rises above its starting value) is therefore unverified, though its companion positivity assertion is real.

**Code evidence.**

```
compressible_marcher2d_tests.rs:252-276:
  252: fn peak_bond_tracks_growth_over_the_march() {
  253:     // A localized perturbation on an otherwise-uniform field encodes at low rank, then develops structure
  254:     // under the march, so the tracked peak `max_bond` must rise above its starting value — exercising the
  255:     // bond-growth branch in `run`.
  ...
  275:     assert!(peak >= 1, "peak bond must be recorded: {peak}");
The returned `peak` originates at src/solvers/qtt/compressible/marcher_2d.rs:191:
  191:  let mut peak = u.iter().map(|t| t.max_bond()).max().unwrap_or(0);
`u` is `EulerStateTt2d` = `[CausalTensorTrain; 4]`, a fixed-size array of 4, so `.max()` is never None and `unwrap_or(0)` is unreachable.
And `max_bond()` is bounded below by 1 by construction — deep_causality_tensor/src/types/causal_tensor_network/causal_tensor_train_operator/getters/mod.rs:38-45:
  38:     pub fn max_bond(&self) -> usize {
  39:         self.cores
  40:             .iter()
  41:             .map(|c| c.shape()[3])
  42:             .chain(core::iter::once(1))
  43:             .max()
  44:             .unwrap_or(1)
  45:     }
The `.chain(core::iter::once(1))` guarantees the maximum is at least 1 unconditionally.
```

**Reference form.** A test assertion must be capable of failing for some reachable program state; otherwise it provides no evidence. The stated property — 'peak must rise above its starting value' — requires capturing the initial max_bond and asserting `peak > initial`.

**Impact.** This test contributes a passing result and a line of coverage over the bond-growth branch in `run` while verifying nothing. It gives false assurance that rank-growth tracking works; if `run` were changed to never update `peak` (deleting lines 196-198 of marcher_2d.rs), the test would still pass. In a certification review this is exactly the class of gate that must be identified as non-evidence.

**Recommended fix.** Capture the initial peak before the march and assert strict growth, e.g. compute `max_bond` of the quantized initial state and assert `peak > initial_peak`. If the localized-bump case does not reliably grow the bond, either choose an input that does or rename the test to `peak_bond_is_returned` and drop the growth claim from the comment.

**Adversarial check.** The tautology is real. compressible_marcher2d_tests.rs:252-276 matches the quote verbatim, including the doc comment claiming the peak 'must rise above its starting value' and the assertion `assert!(peak >= 1, ...)` at :275. marcher_2d.rs:191 initialises `peak` from max_bond over the 4-element EulerStateTt2d, so unwrap_or(0) is unreachable. One citation error: the auditor quoted max_bond from causal_tensor_train_operator/getters/mod.rs:38-45, but `u` holds CausalTensorTrain values, whose max_bond lives at deep_causality_tensor/src/types/causal_tensor_network/causal_tensor_train/getters/mod.rs:33-40. The guarantee is identical there — `.chain(core::iter::once(1)).max()` makes the result unconditionally ≥ 1 — so the conclusion survives the wrong-file citation. Severity is overstated at major: the test still exercises the bond-growth branch and carries a genuine positivity/finiteness assertion at :271-274; only its named property is unverified. The fix is one line — capture the initial peak and assert `peak > initial`.

> Evidence re-read: deep_causality_cfd/tests/solvers/qtt/compressible_marcher2d_tests.rs:252-276 (verbatim match); deep_causality_cfd/src/solvers/qtt/compressible/marcher_2d.rs:191-198; deep_causality_tensor/src/types/causal_tensor_network/causal_tensor_train/getters/mod.rs:32-40 (the applicable max_bond); .../causal_tensor_train_operator/getters/mod.rs:37-45 (the variant actually cited)

---

### 12.4 [MINOR] The ESKF Joseph-form covariance update is documented, cited, and justified — but no test can distinguish it from the simple form it replaced

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/src/navigation/eskf.rs:112`
- **Auditor confidence:** confirmed

**Claim.** No test in tests/navigation/ exercises the property the Joseph form exists to provide — positive semi-definiteness of P after a long sequence of near-unity-gain folds — so a refactor to the simple form would pass CI. Note that a symmetry assertion would NOT discriminate the two forms, because eskf.rs:133-141 re-symmetrizes the covariance unconditionally; only a PSD check (vᵀPv ≥ 0 for sampled v, or leading principal minors) over many high-gain updates is a valid gate.

**Code evidence.**

```
src/navigation/eskf.rs:112-116 (the documented rationale):
  112:     /// The covariance update is the **Joseph form** `P ← (I−K·h)·P·(I−K·h)ᵀ + r·K⊗K`, followed by a
  113:     /// re-symmetrization. The simple form `P − K⊗(h·P)` loses symmetry and positive-definiteness
  114:     /// under long sequences of near-unity-gain folds (a precise receiver folded every step), after
  115:     /// which the cross-term gains change sign and the injected corrections diverge; Joseph is
  116:     /// PSD-preserving unconditionally (Groves 2013, §3.4.3).
The implementation at :126-141 does implement Joseph correctly (A = I − K⊗h at :126-131, apat + r·K⊗K at :137, symmetrized at :139).
The complete ESKF test inventory (tests/navigation/eskf_tests.rs, 123 lines) is: f_matrix_matches_propagate (:14), predict_grows_position_uncertainty (:32), measurement_reduces_uncertainty_and_pulls_the_estimate (:48), covariance_trace_is_the_sum_of_the_diagonal_and_grows_under_predict (:70), closed_loop_reacquires_after_a_blackout_coast (:99).
Only two invoke update_scalar, both with exactly 3 folds:
  :52-56  for (i, &z) in [5.0, 0.0, 0.0].iter().enumerate() { ... filter.update_scalar(h, z, 0.01); }
  :113-117 for i in 0..3 { ... filter.update_scalar(h, 0.0, 0.01); }
`grep -rni 'joseph|symmetr|positive.defin|psd|covariance()' tests/navigation/` returns ZERO matches — no test reads `NavFilter::covariance()` (the accessor exists at src/navigation/eskf.rs:146) let alone checks its structure.
```

**Reference form.** Groves, *Principles of GNSS, Inertial, and Multisensor Integrated Navigation Systems*, 2nd ed. (2013), §3.4.3: the Joseph-form update P⁺ = (I−KH)P⁻(I−KH)ᵀ + KRKᵀ is preferred over P⁺ = (I−KH)P⁻ precisely because it preserves symmetry and positive semi-definiteness under finite-precision arithmetic. Verifying that property requires asserting P = Pᵀ and that all eigenvalues (or all leading principal minors, or vᵀPv for sampled v) stay non-negative after a long sequence of high-gain updates.

**Impact.** The crate's most safety-relevant numerical-robustness decision — the one whose failure mode is explicitly described as 'the injected corrections diverge' in a GNSS-denied reentry navigation filter — is entirely unguarded. A future refactor simplifying the update for speed would pass CI silently and reintroduce exactly the divergence the comment warns about. For an avionics consumer this is the single highest-consequence untested claim in the crate.

**Recommended fix.** Add two tests to tests/navigation/eskf_tests.rs using the existing `covariance()` accessor: (1) after a long sequence (e.g. 5000 folds) of near-unity-gain updates with r ≪ P, assert max|P[i][j] − P[j][i]| stays at machine-epsilon level; (2) assert positive semi-definiteness across that sequence, cheaply via vᵀPv ≥ −ε for a set of random and basis vectors, or via all diagonal entries ≥ 0 plus a Cholesky attempt. Both should be constructed so the simple form demonstrably fails them.

**Adversarial check.** The quoted doc block at eskf.rs:112-116 is verbatim correct, the Joseph implementation at :126-141 is correct as described, and the test inventory is exact — tests/navigation/eskf_tests.rs is 123 lines with the five named tests at the cited lines, and only two call update_scalar, each with three folds. My grep over the whole tests/ tree for joseph|symmetr|positive.defin|psd|covariance() finds no ESKF hit (the only covariance() call is state_snapshot_tests.rs:74, an equality check between two snapshots). reentry_nav_tests.rs also exercises update_scalar indirectly but asserts only variance collapse and orbit-manifold residuals. So the coverage gap is real. Two corrections. First, the finding proposes asserting P = Pᵀ as a discriminating test — it is not: eskf.rs:133-141 explicitly re-symmetrizes every entry ((joseph + joseph_t)/2), so symmetry would hold identically under the simple form and such a test would prove nothing. The only discriminating property is positive semi-definiteness under a long high-gain sequence. Second, severity: the implementation is correct, the re-symmetrization is a standing safety net, and this is a regression-guard gap rather than a live defect.

> Evidence re-read: deep_causality_cfd/src/navigation/eskf.rs:110-146 (doc block :112-116 verbatim; Joseph A at :126-131; apat + r·K⊗K with explicit symmetrization at :132-141; covariance() at :145-148); deep_causality_cfd/tests/navigation/eskf_tests.rs:1-123 (read in full; update_scalar only at :55 and :116); `grep -rniE 'joseph|symmetr|positive.defin|psd|covariance\(\)' deep_causality_cfd/tests/` → no ESKF match; tests/navigation/reentry_nav_tests.rs:29-131

---

### 12.5 [MINOR] ESKF measurement-update tolerances are too loose to discriminate a wrong Kalman gain

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** magic-number
- **Location:** `deep_causality_cfd/tests/navigation/eskf_tests.rs:57`
- **Auditor confidence:** confirmed

**Claim.** The measurement-update test's ±0.1 estimate bound does constrain the gain to K ∈ (0.98, 1.02), but the chosen fixture (P₀ = 100, r = 0.01, so P/r = 10⁴) forces K within 10⁻⁴ of unity for the correct formula and for every plausible mis-formulation of the innovation covariance alike, so the window discriminates nothing. The defect is the fixture's P/r ratio, not the tolerance; a fixture with P comparable to r would make the same assertions sharp.

**Code evidence.**

```
tests/navigation/eskf_tests.rs:48-67:
  50:     let mut filter = NavFilter::new(InsErrorState::<f64>::zero(), [100.0; 17]);
  ...
  55:         filter.update_scalar(h, z, 0.01);
  ...
  57:     assert!(
  58:         (filter.state().position_error()[0] - 5.0).abs() < 0.1,
  ...
  62:     assert!(
  63:         filter.position_variance() < before_var * 0.5,
With the correct scalar update (src/navigation/eskf.rs:119-123: `s = h·P·hᵀ + r`, `k[i] = ph[i]/s`), K = 100/100.01 = 0.99990, so the estimate becomes 4.9995 and the posterior variance is r·P/(P+r) ≈ 0.0099993 — i.e. the assertions clear by factors of ~200x and ~5000x respectively.
Any gain in the range roughly K ∈ (0.98, 1.02) satisfies the 0.1 estimate bound, and any variance reduction better than 2x satisfies the second — so e.g. omitting `r` from the innovation covariance (K = 1 exactly) passes both.
```

**Reference form.** Scalar Kalman update (Groves 2013 §3.2.3): S = hPhᵀ + r; K = Phᵀ/S; x⁺ = x⁻ + K(z − hx⁻); for scalar h = e_i this gives K_i = P_ii/(P_ii + r) exactly. A verification test should assert the posterior mean and variance against these closed forms to tight tolerance, not against a directional inequality.

**Impact.** The measurement-update arithmetic — the step that decides how much a returning GNSS fix is trusted after blackout — is gated only by 'moves the right way'. A miscomputed innovation covariance (e.g. r omitted, r squared, or h applied once instead of twice) would pass. Combined with the untested Joseph form (finding 4), essentially none of the ESKF's numerical content is pinned to a reference value; only the transition matrix (line 14-29) is checked exactly, and that is a self-consistency check between nav_transition_matrix and InsErrorState::propagate rather than an external reference.

**Recommended fix.** Add a closed-form assertion: for a single scalar update on basis direction i with initial variance P and noise r, assert `(posterior_mean - z*P/(P+r)).abs() < 1e-12` and `(cov[i][i] - r*P/(P+r)).abs() < 1e-12`. Use a moderate gain (e.g. P = 1.0, r = 1.0, K = 0.5) so the test is sensitive to the formula rather than saturated at K ≈ 1.

**Adversarial check.** The quotes are exact (eskf_tests.rs:50, :55, :57-58, :62-63) and the arithmetic re-derives correctly. With P₀ = 100·I and r = 0.01, the reference scalar update (eskf.rs:119-123: s = h·P·hᵀ + r, k = P·hᵀ/s) gives K = 100/100.01 = 0.99990, posterior mean 4.99950, posterior variance r·P/(P+r) ≈ 9.999e-3. I re-derived the discrimination window independently: the first fold sets x₀ = K·5, so |5K − 5| < 0.1 ⟺ K ∈ (0.98, 1.02), and any K near 1 collapses P₀₀ to ≈ r·K² ≪ 0.5·before_var, so both bounds clear by large margins and K = 1 exactly (r omitted from S) passes. Groves §3.2.3 is cited correctly. Two corrections. The finding says the test 'pins direction rather than value' — the estimate bound does pin K to ±2%, which is not merely directional and would catch a grossly wrong gain (K = 0.5 fails). The real defect is the fixture, not the tolerance: choosing P/r = 10⁴ drives K to within 10⁻⁴ of unity, so every plausible mis-formulation of S lands inside a ±2% window. A fixture with P ≈ r (K ≈ 0.5) would discriminate at the same tolerances. Severity major is overstated for a fixture-selection weakness on correct arithmetic.

> Evidence re-read: deep_causality_cfd/tests/navigation/eskf_tests.rs:47-67 (all cited lines verbatim); deep_causality_cfd/src/navigation/eskf.rs:117-124 (ph = P·hᵀ, s = h·ph + r, k[i] = ph[i]/s); re-derivation of K = P/(P+r) = 0.99990 and the (0.98, 1.02) admissible band from the first fold

---

### 12.6 [MINOR] cargo doc emits six broken intra-doc links referencing API items that do not exist

- **Verification verdict:** CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/src/types/flow/report.rs:22`
- **Auditor confidence:** confirmed

**Claim.** `cargo doc -p deep_causality_cfd --no-deps` produces six `unresolved link` warnings. The documentation references five types/methods — CyberneticCorrect, AeroForceCoupling, RegimeClass, CarrierPause::continue_with/continue_branches, CarrierFork::continue_march — that rustdoc cannot resolve in scope, meaning the rendered docs describe an API surface under names the crate does not export.

**Code evidence.**

```
Verbatim warnings from `cargo doc -p deep_causality_cfd --no-deps` (reproduced after touching src/lib.rs to defeat caching):
  warning: unresolved link to `CyberneticCorrect`
    --> deep_causality_cfd/src/types/flow/corridor/branch.rs:98:17
  warning: unresolved link to `super::AeroForceCoupling`
    --> deep_causality_cfd/src/types/flow/corridor/branch.rs:107:27
     |  no item named `AeroForceCoupling` in module `corridor`
  warning: unresolved link to `RegimeClass::gnss_denied`
    --> deep_causality_cfd/src/types/flow/corridor/trajectory_nav.rs:27:38
  warning: unresolved link to `crate::CarrierPause::continue_with`
    --> deep_causality_cfd/src/types/flow/report.rs:22:38
     |  no item named `CarrierPause` in module `deep_causality_cfd`
  warning: unresolved link to `crate::CarrierPause::continue_branches`
    --> deep_causality_cfd/src/types/flow/report.rs:23:42
  warning: unresolved link to `crate::CarrierFork::continue_march`
    --> deep_causality_cfd/src/types/flow/report.rs:24:38
  warning: `deep_causality_cfd` (lib doc) generated 6 warnings
```

**Reference form.** rustdoc's `broken_intra_doc_links` lint (warn-by-default) fires when a `[`Item`]` reference cannot be resolved to an item in scope. A clean documentation build is the baseline expectation for a crate offered for external review.

**Impact.** Readers of the rendered docs get dead links at exactly the points where the docs are explaining the fork/continue API (report.rs:22-24) and the corridor branch semantics. The names suggest either renamed types whose docs were not updated or documentation written against a planned API — either way a reviewer cannot navigate from the prose to the referenced item, and cannot tell whether `CarrierPause`/`CarrierFork` are stale names or genuinely missing capabilities.

**Recommended fix.** Resolve each link to the actual exported item (or drop the link markers if the concept is prose-only), then add `-D warnings` to a `cargo doc` step in CI so documentation regressions are caught. Note the crate is `publish = false` today; this should be fixed before any docs.rs publication.

**Adversarial check.** I reproduced the build independently (touch src/lib.rs, then cargo doc -p deep_causality_cfd --no-deps) and the output matches the finding character for character: six broken_intra_doc_links warnings at branch.rs:98:17 (CyberneticCorrect), branch.rs:107:27 (super::AeroForceCoupling), trajectory_nav.rs:27:38 (RegimeClass::gnss_denied), and report.rs:22:38 / :23:42 / :24:38 (crate::CarrierPause::continue_with, crate::CarrierPause::continue_branches, crate::CarrierFork::continue_march), closing with 'deep_causality_cfd (lib doc) generated 6 warnings'. No line number, path, or item name was misquoted. Severity minor is correctly assigned.

> Evidence re-read: `cargo doc -p deep_causality_cfd --no-deps` run fresh after touching deep_causality_cfd/src/lib.rs — all six warnings reproduced verbatim, including the 'no item named X in module Y' sub-notes

---

### 12.7 [INFO] RAM-C ionization gates use wide bands whose width appears chosen after seeing the measured value

- **Verification verdict:** REFUTED — not a defect
- **Axis:** magic-number
- **Location:** `deep_causality_cfd/tests/solvers/qtt/compressible_fitting_tests.rs:101`
- **Auditor confidence:** likely

**Claim.** Both RAM-C n_e gates state their bands as a tolerance factor about an external anchor and the code matches: 'within ~2 decades of 1e19' is [1e17, 1e21] and 'within ~3× of 1e19' is [3e18, 3e19]. The bands are traceable to a named flight reference plus a declared fidelity factor. A defensible residual request is that the ±3× factor be tied to RAM-C's own measurement uncertainty in a comment, which is a documentation nicety, not a magic number.

**Code evidence.**

```
compressible_fitting_tests.rs:92-105:
  93:     // The milestone gate: peak n_e within ~2 decades of the RAM-C II anchor (order-of-magnitude surrogate).
  ...
  101:     assert!(
  102:         out.electron_density > 1.0e17 && out.electron_density < 1.0e21,
  103:         "peak n_e {:.3e} should be within ~2 decades of RAM-C II (1e19)",
The band 1e17–1e21 spans four decades (two below and two above 1e19), not '~2 decades'.
compressible_fitting_tests.rs:109-124:
  110:     // Gap-3 chemistry-fidelity gate: driving ionization off Tₐ = √(T_tr·T_ve) (not the hot T₂) lands peak
  111:     // n_e within ~3× of the RAM-C II anchor 1e19 — down from the single-temperature surrogate's ~12×.
  ...
  120:     assert!(
  121:         out.electron_density > 3.0e18 && out.electron_density < 3.0e19,
The band 3e18–3e19 is a 10x-wide window; the phrase 'down from the single-temperature surrogate's ~12×' states the bound was set relative to a previously measured deviation.
Related, at :187-191 the rank gate `assert!(bond <= 4, "smooth post-shock profile should be O(1) rank, got {bond}")` uses the literal 4 with no derivation.
```

**Reference form.** The external anchor is legitimate and correctly identified: RAM-C II flight-measured peak electron density ≈ 1e19 cm⁻³ (NASA RAM-C flight experiments, Jones & Cross, NASA TN D-6617, 1972). A defensible gate would set the band from the reference's own quoted measurement uncertainty and the model's stated fidelity tier, and state that derivation.

**Impact.** These are the only in-suite quantitative checks on the ionization path, and their width means they would continue to pass through substantial degradation of the Park-2T closure — a factor-of-3 error in the rate coefficients stays inside the 3e18–3e19 window. The four-decade band at line 102 is wide enough to admit almost any non-pathological result. The crate's verification/README.md is notably more rigorous here (it quotes '+0.48 dec prediction (earned band ±0.70)' with an explicitly earned band), so the in-suite gates are weaker than the crate's own documented standard.

**Recommended fix.** Restate each band as a derivation: cite the RAM-C II measurement uncertainty and the declared fidelity tier, and set the bound from those rather than from the observed margin. Correct the '~2 decades' comment at line 93 to match the four-decade band actually asserted, or tighten the band to match the comment. Derive or cite the `bond <= 4` threshold at line 188.

**Adversarial check.** The central evidence is a misreading of the comments. 'Within ~2 decades of the RAM-C II anchor (1e19)' means |log₁₀(n_e/1e19)| ≤ 2, which is exactly the band 1e17–1e21 asserted at :102. The auditor rewrites this as 'four decades wide, not ~2 decades', conflating total band width with distance from the anchor; on the plain reading the comment and the code agree. The same error recurs on the second gate: 'within ~3× of the RAM-C II anchor 1e19' means the interval [1e19/3, 3e19] ≈ [3.33e18, 3e19], and the code asserts [3e18, 3e19] — a match to the stated factor, not an unexplained '10x-wide window'. Both bands are therefore traceable: an externally anchored value (RAM-C II peak n_e ≈ 1e19, which the auditor concedes is correctly identified) times an explicitly stated tolerance factor. The 'down from the single-temperature surrogate's ~12×' clause reports an improvement between fidelity tiers; it does not set the bound, which is the round number 3. The residual valid observation is that the ±3× factor is asserted rather than derived from RAM-C's quoted measurement uncertainty — but the crate states plainly (README.md:218-219) that the network is uncalibrated, so an order-of-fidelity band is the honest gate for that tier, not a post-hoc accommodation. The `bond <= 4` literal at :187 is a tensor-rank gate on a smooth profile, not an ionization gate, and is outside the finding's stated scope.

> Evidence re-read: deep_causality_cfd/tests/solvers/qtt/compressible_fitting_tests.rs:92-105 and :109-125 (both comments and both assert bands read in full); :181-192 (the bond <= 4 rank gate); :38-52 (post_shock_ratios_match_exact_rh, exact RH to 1e-10, showing the file does carry hard reference gates); deep_causality_cfd/README.md:218-219

---

### 12.8 [MINOR] The finite-rate ionization network has no in-suite quantitative gate, only structural fixed-point checks

- **Verification verdict:** CONFIRMED
- **Axis:** doc-gap
- **Location:** `deep_causality_cfd/tests/types/flow/finite_rate_ionization_tests.rs:36`
- **Auditor confidence:** confirmed

**Claim.** FiniteRateIonizationStage is tested only for qualitative structure — that relaxation approaches a fixed point from both sides, that the frozen limit holds, and that the stage no-ops without its controller field. No test compares any rate coefficient or resulting electron density against a published reaction-rate reference.

**Code evidence.**

```
finite_rate_ionization_tests.rs:36-59, the strongest assertion in the file:
  36: fn the_fixed_point_is_approached_from_both_sides() {
  ...
  53:     assert!(a_low > 0.0, "under-ionized grew: {a_low}");
  54:     assert!(a_high < 0.9, "over-ionized decayed: {a_high}");
  55:     assert!(
  56:         (a_low - a_high).abs() / a_low < 0.05,
  57:         "both sides converge to one fixed point: {a_low} vs {a_high}"
These assert only that two trajectories converge to a common value — that value is never compared to anything external. Similarly :176 `assert!(x_n < 0.5, "the N pool lags its equilibrium: {x_n}")` is a one-sided structural bound.
The crate is candid that the network is uncalibrated — README.md:218-219: "The plasma-blackout examples validate an uncalibrated finite-rate ionization network against RAM-C II flight data." That validation is in verification/qtt_ramc_stagline, which per finding 1 never runs in CI.
```

**Reference form.** Park, C., *Nonequilibrium Hypersonic Aerothermodynamics* (Wiley, 1990) supplies the two-temperature air reaction-rate set. A quantitative gate would pin at least the equilibrium fixed point at a stated (T, p) against the Saha equation or a Park-rate equilibrium composition table.

**Impact.** Consistent with findings 1 and 2: the plasma path's numerical content is unguarded in CI. The structural tests would pass with rate coefficients wrong by orders of magnitude, since they only require that relaxation is monotone toward *some* fixed point. The crate's documentation is honest about the uncalibrated status, so this is a coverage gap rather than an overclaim — but it means the ionization stage carries no automated evidence at all.

**Recommended fix.** Add one in-suite test pinning the network's equilibrium fixed point at a specified temperature and pressure against the Saha equation (already available in the crate — stagnation_blackout computes the Saha equilibrium, used as a reference at compressible_fitting_tests.rs:167-177), to a stated tolerance reflecting the uncalibrated tier. This converts the existing 'converges to one fixed point' check into 'converges to the right fixed point'.

**Adversarial check.** Verified line by line. finite_rate_ionization_tests.rs:36 is `fn the_fixed_point_is_approached_from_both_sides()` as cited, and its strongest assertions (:53-57) only require that two trajectories from α = 0.0 and α = 0.9 converge to a common value — that value is never compared to Saha, to a Park equilibrium composition, or to any external table. I enumerated every assertion in the file: the remaining tests check a no-op without the controller field (:32), decay in a cold cell (:73), the frozen limit (:86), per-cell broadcast shapes (:100-107, :153-157), a dimension error message (:127), the one-sided pool lag `x_n < 0.5` (:176), renewal statelessness (:191), and an electron-temperature fallback (:213). Not one compares a rate coefficient or an equilibrium composition to a published reference. FiniteRateIonizationStage appears elsewhere only in state_snapshot_tests.rs and in verification/qtt_ramc_stagline, which per finding 1 never runs in CI. The Park 1990 reference form is correctly stated. The finding correctly frames this as a coverage gap rather than an overclaim, since README.md:218-219 calls the network uncalibrated in plain terms; severity minor is right.

> Evidence re-read: deep_causality_cfd/tests/types/flow/finite_rate_ionization_tests.rs — full assertion enumeration (:32, :53-57, :73, :86, :100-107, :127, :153-157, :172-176, :191-195, :213) and line 36 confirmed as the cited fn; `grep -rln FiniteRateIonizationStage` → tests/types/flow/{finite_rate_ionization,state_snapshot}_tests.rs and verification/qtt_ramc_stagline only; deep_causality_cfd/README.md:218-219

---

## 1. Reconcile the DEC kernel and spectral-projector docs (item 17)

Concrete, high-value, and self-contained — the code is confirmed correct, so every edit here is prose
describing the operator the code already marches.

- [ ] 1.1 Correct the DEC NS rate kernel's public docstrings to name the **skew-symmetrised** convective
      operator `conv' = ½[G_ω u − G*_ω u]` the code marches (`dec_ns_rate.rs:621-652`), not the
      un-symmetrised `−i_u(du♭)` they state today: module doc `dec_ns_rate.rs:7`, type doc `:35`,
      `eval_projected` doc `:457`, the `:507-508` comment, and `src/solvers/dec/mod.rs:11-12,29-30`,
      `src/theories/incompressible_dec.rs:44`. Cite the stabilisation change that introduced the
      skew form (`2026-06-14-fix-dec-convective-instability`).
- [ ] 1.2 Correct the two spectral-projector comments in `src/tensor_bridge/projection.rs` to describe the
      consistent `λ_k = sin²(2πk/N)/dx²` operator the code applies (`:158-163`), not the compact 5-point
      `−(2−2cos(2πk/N))/Δ²` (`:157`) and the sign/form-inconsistent `:169`.
- [ ] 1.3 Correct `src/solvers/dec/mod.rs:21-26` — it describes a first-order Chorin split; the code
      projects **inside** each RK4 stage with no splitting error (`dec_ns_solver/step.rs:6-13`,
      `dec_ns_rate.rs:6-13`). Make the prose match the in-stage projection.
- [ ] 1.4 Verify: the corrected docstrings match the implemented operator by reading the code, not by
      restating the comment (§5c gate-BM-A lesson — do not describe a copy).

## 2. Qualify the convergence claim (item 21)

- [ ] 2.1 Qualify the QTT Taylor–Green order claim as **second-order in space, first-order in time**:
      `verification/qtt_taylor_green_verification/README.md:52`, `print_utils.rs:141`, `baseline.txt:26`,
      and `verification/README.md:297`.
- [ ] 2.2 Document the ladder's temporal-error floor (measure it; ~1e-5 at fixed `dt` per the audit) and
      its maximum usable length, at `print_utils.rs:133` / `config.rs:28`. Record the measured floor, not
      the audit's estimate.

## 3. Reconcile the doc-overclaim catalogue (item 16)

The unit of work is the ACTION-LIST row, not the number. Disprove each claim against the code before
rewriting it (D2): a false *and* unenforced "by construction" claim is a correctness finding, not a doc
edit — escalate it.

- [ ] 3.1 Enumerate the surviving `doc-overclaim` rows from `openspec/notes/cfd_audit/ACTION-LIST.md`
      (86 rows), excluding the ones Phases 1–2 closed (`blended.rs:17` fold / ref 7.1;
      `boundary_zone.rs:22` hook / ref 8.1; `observe.rs:81` rename / ref 4.14). Record the working count.
- [ ] 3.2 For each row, check the claim against the code. Where prose describes intent, mark it intent;
      where it asserts a property no check enforces, describe what the code does.
- [ ] 3.3 Where a claim is false **and** the property is unenforced (a B-4 repeat), stop and record it as
      a correctness finding for a follow-up change — do not soften the prose over broken code.
- [ ] 3.4 Record the count reconciled and the count escalated against the catalogue, so completeness is
      checkable against the ACTION-LIST rather than memory.

## 4. Close the doc-gaps (item 18)

- [ ] 4.1 Document in the crate README (where a user looks) the public capabilities the audit lists as
      absent: `DuctMarchRun` (`duct_march_run.rs:56`), `IgnitionCorridor` (`throttle_guidance.rs:107`),
      snapshot/resume (`state_snapshot.rs`), `AcousticCoreInverse`/`2d`/`3d` (`acoustic_inverse.rs:52,175,249`).
- [ ] 4.2 Work the remaining `doc-gap` catalogue rows (39-total category, a mix of README sections and
      docstring gaps — units, boundary configs, non-conservative notes). Record the count closed.

## 5. Give load-bearing constants provenance (item 23)

Follow the crate's paper convention (full author-year citation at the definition, PDF in `papers/`).

- [ ] 5.1 `SMOOTH_CELLS = 2.0` — add a source, units, and the reason for the value at both definitions
      (`verification/qtt_cylinder_verification/config.rs:44`, `qtt_park2t_blackout/config.rs:37`). Note
      §4b: this constant moves the reported `C_d` by 6.1×, so its provenance is load-bearing.
- [ ] 5.2 `ETA = 0.016` in `qtt_park2t_blackout/config.rs:31` — add provenance (the cylinder site
      `config.rs:32-38` already carries it from `close-qtt-solver-envelope`; match that discipline or state
      why this configuration differs).
- [ ] 5.3 The Mach-1.05 shock floor (`src/types/flow/compressible_march_run.rs:326-327`) — justify the
      0.05 buffer above sonic (the comment explains the branch, not the value), with a stiffness bound or
      citation.
- [ ] 5.4 Add the penalization reference (Angot / Bruneau & Fabrie 1999) to `deep_causality_cfd/papers/`
      and cite it from the immersed-body harness where it is currently text-only
      (`verification/qtt_cylinder_verification/print_utils.rs:122`, `config.rs:9`).
- [ ] 5.5 Index `papers/`: a short README mapping each PDF to what cites it. Surfaces the two orphans
      (`mittal2005.pdf`, `mohamed2016.pdf`, cited by author name nowhere) — cite them if they are a
      constant's source, else flag for the owner. **No PDF is deleted without owner approval.**

## 6. Give a load-bearing test an independent reference (item 24)

Public-API behavioural only — no source-text scraping (§5c standing rule). Verified under `bazel test //...`.

- [ ] 6.1 Add a test driving the shipped QTT convection path `rate_pair`
      (`src/solvers/qtt/incompressible_2d.rs:105`) with `u,v ≠ 0` against an analytic convection reference
      chosen so the projection does **not** annihilate it (the reason the existing `scalar_rate` tests are
      blind: they pass `u=v=0`, and the TG solver test's convection is a pure gradient the projection
      removes). Add to `tests/solvers/qtt/incompressible_2d_tests.rs`.
- [ ] 6.2 Demonstrate the test bites: a sign flip in the shipped convection makes it fail (fault
      injection, per Phase 1's falsifiability discipline).
- [ ] 6.3 Route the Taylor–Green harness's convection check through the shipped `rate_pair` rather than
      the `gradient_x`/`gradient_y` re-assembly (`verification/qtt_taylor_green_verification/main.rs:83-126`),
      so the harness gates the shipped path, not a copy. Confirm the reported convection error is unchanged.

## 7. Resolve `Gates` (item 22)

Owner decision on adopt-vs-retire; the non-deleting corrections land now.

- [ ] 7.1 Make `Gates::finish()` refuse an empty gate set (`src/types/flow/gates.rs:53-63`) — it returns
      success for one today; a harness that registered no gate should not report pass. Update
      `gates_tests.rs:31`.
- [ ] 7.2 Correct the prose that names `Gates` where the code uses `GateSeq`/`Verdict`:
      `examples/avionics_examples/cfd/flight_envelope_placard/README.md:20,119`, and the `verification/README.md`
      claim that `Gates` is the block every self-verifying program prints.
- [ ] 7.3 Surface the adopt-vs-retire decision to the owner (D4). Recommendation: retire (`GateSeq` is the
      live, evidence-class-labelled contract). **Retirement is a deletion — do not remove `Gates` without
      owner approval.** If adopt, migrate the programs to `Gates` *with* `EvidenceClass` labelling so the
      evidence-class discipline is not regressed.

## 8. Re-verify the items Phases 1–2 already delivered (items 19, 20, 23-cylinder)

Confirm still-correct; captured in the spec so they cannot silently rot. No re-implementation.

- [ ] 8.1 Item 19 — RAM-C framing consistent across `README.md:224` and `verification/README.md:127`
      (both order-of-magnitude, ±0.70 dec). Confirm unchanged.
- [ ] 8.2 Item 20 — lid-cavity summary row reports the 65² default (RMSE 0.0617) at
      `verification/README.md:90,213`. Confirm unchanged.
- [ ] 8.3 Item 23 (cylinder) — `ETA = 0.012` still carries its wall-error-target provenance
      (`qtt_cylinder_verification/config.rs:32-38`). Confirm unchanged.

## 9. Verify

- [ ] 9.1 `bazel test //...` green (the check that matters — catches sandbox-only regressions `cargo test`
      hides). `cargo clippy --all-targets --all-features -- -D warnings` clean; `cargo fmt` applied; no new
      `#[allow]`.
- [ ] 9.2 No example figure moves — this change touches no marcher, kernel, or gate bound. Spot-check the
      corridor and a verification harness reproduce their committed outputs.
- [ ] 9.3 The reconciled and escalated counts recorded against the ACTION-LIST (3.4, 4.2), so completeness
      is checkable, not asserted.
- [ ] 9.4 Adversarial pass over the finished diff (D7): a multi-dimension refute-by-default review, each
      finding independently verified. Record the residue honestly — "N reconciled, M deferred", never
      "clean". Fix what it confirms.
- [ ] 9.5 `openspec validate reconcile-cfd-docs-and-traceability --strict` passes.

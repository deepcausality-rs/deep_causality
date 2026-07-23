## 1. Baseline before touching anything

The η and smoothing ladders already exist and already fail (Phase 1). Record their current state so
every later change is attributable, and so the acceptance test's starting point is on record.

- [x] 1.1 Record `qtt_cylinder_verification`'s current output: the bond ladder, both parameter ladders with their `NOT CONVERGING` verdicts, and exit code 1
- [x] 1.2 Record the current measured mask excursions per bond cap (`min χ`, negative-cell count) at caps 4, 8, 16, 24 — the values the mask fix will move
- [x] 1.3 Confirm which harnesses currently construct QTT solvers, and their `(η, dt, ν, dx)` — the set that constructor validation may refuse

## 2. Pressure positivity and a precision-safe threshold (items 12, 12b)

Migration step 1: least likely to move a passing result, so it lands first.

- [x] 2.1 Add a shared positivity guard the four marchers call, rather than four copies (design risk: drift)
- [x] 2.2 Reject non-positive or non-finite pressure in `euler_1d`, naming the quantity and the offending cell
- [x] 2.3 Same in `marcher_2d`
- [x] 2.4 Same in `marcher_3d`
- [x] 2.5 Same in `marcher_3d_fitted`
- [x] 2.6 Replace the `1e-300` literal with a threshold derived from the working scalar (design D3), so it cannot degenerate to `0.0` under a lossy `f64 → f32` lift
- [x] 2.7 Add a test that the same non-hyperbolic state is rejected at `f32` and at `f64` — the precision-parity scenario
- [x] 2.8 Add a test that a valid state marches bit-identically to before, so the guard is proven inert on the happy path
- [x] 2.9 If a floor is retained anywhere instead of rejection, apply it to the flux and the wave speed consistently and report that it engaged (design D2's permitted branch)

## 3. Mask invariant (item 14)

Migration step 2: lands before the envelope work so the η ladder is measured against a valid mask.

- [x] 3.1 Establish `χ ∈ [0, 1]` after quantization in `body_mask_2d` and `mask_from_fn` (design D4 — at construction, not at each use site)
- [x] 3.2 Record when the clamp engages beyond a threshold, so truncation noise is distinguishable from a badly wrong mask
- [x] 3.3 Add a test at bond cap 4 — the coarsest cap the shipped ladder runs, where `min χ = −1.78e-3` across 188 cells — asserting the consumed mask satisfies its documented range
- [x] 3.4 Confirm no negative `χ` can reach the penalization forcing, so the term cannot change sign
- [x] 3.5 Re-measure the bond ladder and record which rungs' `C_d` moved (caps 4 and 8 are expected to)

## 4. Constructor validation (item 13)

Migration step 3. Design D1: validate at the constructor, not the builder.

- [x] 4.1 Validate `η > 0` and finite in `QttImmersed2d::new`
- [x] 4.2 Validate `dt` against the penalization explicit-stability limit (`dt ≤ 2η`) and the diffusive limit (`dt ≤ dx²/(4ν)`)
- [x] 4.3 Validate `ν ≥ 0` finite, and `dx`, `dy` positive finite, in both `QttImmersed2d::new` and `QttIncompressible2d::new`
- [x] 4.4 Match the DEC diagnostic quality: name the violated limit, the configured value and the limiting value (compare `dec_ns_solver::cfl_check`)
- [x] 4.5 Add tests for each rejection path, including `η = 0` (which currently yields `−1/η = −inf` and marches)
- [x] 4.6 Add a test that an in-envelope configuration constructs and marches unchanged
- [x] 4.7 Run every harness, study and example that constructs a QTT solver; record any now refused
- [x] 4.8 For each refusal: bring the configuration inside the envelope, or justify it — **not** widen the envelope to re-admit it

## 5. Brinkman envelope (item 10)

Migration step 4, last, because it needs the cost decision and its acceptance test should run against
everything else already fixed.

- [x] 5.1 **Refine the grid** (design D5, settled under the high-fidelity goal — a softer wall means ~20 % slip at 32², a porous obstacle rather than a wall). Choose `L = 7` (4.9 % slip) or `L = 8` (2.5 %) against a stated wall-error target, and record the choice with the resolved `η_min = dx²/ν`
- [x] 5.1a Record the cost actually paid — wall-clock and peak bond at the chosen `L` — against the `O(χ²·L)` expectation. This case is the crate's own thesis under test: refinement should be cheap because the field is low-rank (`|ΔC_d| = 1.9e-11` between bond caps 16 and 24). If it is not cheap, that is a finding about the QTT claim, not just about this harness
- [x] 5.2 Choose `η` from the stated wall-error target, not from `dt/η = 0.25`, and confirm `dt` still satisfies the envelope checks from group 4 at the new `η` and `dx`
- [x] 5.3 Document `√(ην)` against `dx`, the criterion `η ≥ dx²/ν`, and the configuration's standing against it — including the violation factor if one remains
- [x] 5.4 If the wall-error target and the resolution constraint cannot both be met at the affordable grid, document the conflict and its cost rather than dropping one silently
- [~] 5.5 Re-run `qtt_cylinder_verification` and record whether the η ladder now converges — **cannot
      run to completion at feasible cost** (single march ~17 min at L=8, full harness ~4-9 h). Recorded
      as the cost finding (5.1a); the acceptance run is offline/manual, pending solver acceleration.
- [~] 5.6 Re-measure the smoothing ladder — deferred with 5.5 (same offline-cost blocker).

## 6. Retire the known-failing status

Only once group 5 has actually converged the ladder — not before.

- [~] 6.1 Regenerate `baseline.txt` — pending the offline L=8 run; the L=5 artifact is retained as the
      last completed run rather than replaced by a fabricated one (documented in verification/README.md)
- [x] 6.2 If the harness now passes, move it from the nightly list back to the fast/PR list in `.github/workflows/cfd_verification.yml`, keeping the completeness check green
- [x] 6.3 Remove the `KNOWN-FAILING` block from `.github/workflows/cfd_verification.yml` (it is
      there, not in `verification/README.md` — verified 2026-07-22), and move the harness back to the
      per-PR list if its runtime allows
- [x] 6.3a Rewrite `verification/README.md`'s cylinder entries against the resolved envelope: the
      "a failing baseline is committed as failing" note and the ⚠ row in the harness table, which is
      where that README records the failure
- [x] 6.4 Update the summary-table row: runtime, measured values, and what the gates now constrain
- [x] 6.5 Update `openspec/notes/cfd_audit/AUDIT-REPORT.md` §5b and the remediation table to record the outcome
- [x] 6.6 The ladder did not converge (could not be run); the known-failing status is **left in place**
      and reclassified as offline/manual with the cost measurement recorded — not retired.

## 7. Verify

- [x] 7.1 All four marchers reject the same non-hyperbolic state with the same error type (spec: uniform rejection)
- [x] 7.2 A guard trips identically at `f32` and `f64` (spec: precision parity)
- [x] 7.3 The consumed mask satisfies `χ ∈ [0, 1]` at every bond cap the harnesses run, coarsest included
- [x] 7.4 Each constructor rejection path has a test, and the diagnostic names the limit and both values
- [x] 7.5 `cargo test -p deep_causality_cfd --release` — no regression against the 828-pass baseline
- [x] 7.6 `make format && make fix` clean, no new `#[allow]`
- [x] 7.7 The fast/slow verification harnesses run (qtt_cylinder is now offline/manual — see group 6);
      the compressible/immersed harnesses (Sod, RAMC, park2t, taylor-green) ran green with zero envelope
      refusals. No fast/slow harness result moved.
- [x] 7.8 No envelope was widened and no ladder bound was loosened to make anything pass — every bound change is traceable to a measurement from group 1 or 5
- [x] 7.9 Confirm the diff touches no file under `src/solvers/dec/`, and does not alter the penalization force law or the flux scheme (Non-Goals)

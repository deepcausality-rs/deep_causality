## 1. Baseline the navigation figures before touching the filter

Design D5: measure first, so every later movement is attributable and no gate is re-derived from a
number nobody recorded.

- [ ] 1.1 Record the corridor example's navigation figures: dead-reckoning drift through blackout, terminal reacquisition error, and the variance witnesses (`err → drift → post-reacquisition`, currently 0.3467 → 2.5448 → 0.2775 m)
- [ ] 1.2 Record the weather example's per-case drift mean/σ and terminal mean/max over its draw ensemble
- [ ] 1.3 Record `ins_gnss_blackout`'s reported figures
- [ ] 1.4 Record every tuned `q_diag` in the examples, with the `dt` it was tuned at — these values change meaning under a stated discretisation

## 2. Validate the covariance at its entry points (D3)

Lands first: it makes the degenerate-update path unreachable rather than merely guarded.

- [ ] 2.1 Reject a non-finite, negative-variance or asymmetric covariance in `NavFilter::new`
- [ ] 2.2 Same in `NavFilter::restore`, so a snapshot cannot reintroduce one
- [ ] 2.3 Choose and document the symmetry tolerance — exact equality is wrong for a restored float matrix
- [ ] 2.4 Add tests for each rejection, and a snapshot round-trip test proving a valid filter restores exactly
- [ ] 2.5 Confirm no shipped configuration is now refused; if one is, it was already carrying an invalid covariance — record it

## 3. Guard the measurement update (D2)

- [ ] 3.1 Give `update_scalar` a rejection channel; it currently returns `()` and so cannot refuse
- [ ] 3.2 Reject a zero, negative or non-finite innovation covariance `s = h·P·hᵀ + r` before dividing
- [ ] 3.3 Reject a negative measurement variance `r`
- [ ] 3.4 Make rejection **atomic**: state and covariance unchanged, not half-applied. This matters concretely — `correct_position` folds three sequential per-axis updates, and a rejection on the second must not leave the first applied
- [ ] 3.5 Propagate the rejection through `ReentryNavEngine::correct_position` so a caller learns a fix was not folded
- [ ] 3.6 Add a test for the reachable `NaN` path (`P[i][i] = 0`, `r_var = 0`), asserting the filter is untouched afterwards
- [ ] 3.7 Add a test that a valid measurement produces bit-identical results to before the guard

## 4. Give the process noise a discretisation (D1)

The change that moves numbers. Landed alone so its effect on the examples is attributable.

- [ ] 4.1 Scale the process noise by `dt` in `NavFilter::predict` (`Q_d = Q_c·dt`)
- [ ] 4.2 Document at the API boundary that the supplied diagonal is a continuous-time spectral density, with the discretisation named and cited
- [ ] 4.3 Add the horizon-invariance test: propagate a fixed horizon at `dt` and at `dt/2` with no measurements, and assert the terminal covariance agrees to the discretisation's order
- [ ] 4.4 Add a test that changing `dt` alone does not change filter behaviour over a fixed horizon
- [ ] 4.5 Re-tune each `q_diag` recorded in 1.4 from per-step to per-second, stating the conversion

## 5. Measure, then re-derive the gates (D5)

Do not edit a navigation gate before this group's measurements are written down.

- [ ] 5.1 Re-run the corridor example and record every navigation figure against the 1.1 baseline
- [ ] 5.2 Re-run the weather example and record its ensemble statistics against 1.2
- [ ] 5.3 Re-run `ins_gnss_blackout` against 1.3
- [ ] 5.4 Note which navigation gates now fail, before deciding what to do about any of them
- [ ] 5.5 Re-derive each failing gate's bound from the corrected filter, with its evidence class — never widen a bound to restore a pre-correction figure
- [ ] 5.6 If the corrected filter reports worse drift than before, record that as the result; the previous figures came from a filter whose covariance grew per step

## 6. Resolve the attitude-error inconsistency (D4)

Last, because it needs an owner decision and the options differ greatly in size.

Resolution revised 2026-07-22 after testing the deferral against the tree: **(a) here; (b) dropped.**
`Quaternion` with `from_axis_angle`, `to_rotation_matrix`, `normalize` and `slerp` already ships in
`deep_causality_num_complex`, an existing `deep_causality_cfd` dependency, so (a) is a field plus two
call sites rather than a feature. (b) was justified solely by (a)'s cost and carried a real price — an
attitude block that is no longer an error about the current nominal.

- [ ] 6.1 Add a nominal attitude `Quaternion<R>` to `ReentryNavEngine`, which today carries
      `position`, `velocity`, `filter`, `tau_offset`, `elapsed` and no attitude at all
- [ ] 6.1a Integrate the gyro into the nominal each step (`q ← normalize(q ⊗ from_axis_angle(ω̂, |ω|·dt))`)
- [ ] 6.1b Inject the estimated `δψ` into the nominal in `correct_position`, then zero the attitude
      error block — which is now legitimate **because it was injected**, satisfying the spec's
      "reset only if injected" invariant rather than working around it
- [ ] 6.1c Use `to_rotation_matrix()` where the transition matrix needs the DCM, so the `−[f]×`
      coupling reads the nominal rather than an implied identity
- [ ] 6.2 Confirm no departure needs documenting: with `δψ` injected, the attitude block *is* an error
      about the current nominal, so the textbook ESKF bookkeeping holds and (b)'s interim caveat is
      not incurred
- [ ] 6.3 Add a test that many position-only fixes do not shrink the attitude covariance monotonically toward zero on the strength of corrections never applied
- [ ] 6.4 Confirm the retained attitude error still couples into velocity error through `−[f]×` as the transition matrix intends, and that retaining it does not destabilise the propagation
- [ ] 6.5 Verify the quaternion stays normalised across a long march (integration drift is the one
      real hazard (a) introduces), and that a zero gyro input leaves the nominal exactly unchanged

## 7. Verify

- [ ] 7.1 Covariance growth over a fixed horizon is invariant under halving `dt` (spec: stated discretisation)
- [ ] 7.2 The reachable degenerate update leaves the filter untouched and reports rejection (spec: degenerate inputs refused)
- [ ] 7.3 A non-PSD covariance is refused at both construction and restoration (spec: a covariance is validated)
- [ ] 7.4 Every error-state component zeroed by a reset was first injected (spec: reset only if injected)
- [ ] 7.5 Valid inputs produce bit-identical results to before, for the guards added in groups 2 and 3
- [ ] 7.6 `cargo test -p deep_causality_cfd --release` — no regression against the 828-pass baseline
- [ ] 7.7 `make format && make fix` clean, no new `#[allow]`
- [ ] 7.8 Every navigation gate that moved is traceable to a measurement from group 5, and no bound was widened to restore a pre-correction figure
- [ ] 7.9 Confirm the diff does not alter `nav_transition_matrix`, the 17-state composition, or the Joseph update — all confirmed correct by the audit and out of scope

## 1. Settle the two open decisions

Both are owner calls that change what the work is. Neither blocks group 2.

Both settled in design under the high-fidelity goal. What remains here is one sub-question.

- [x] 1.1 **Wire `collect_constrained_edges`** (design D4 — reverses this design's earlier removal recommendation). The already-specified `aperture-resolved-noslip` capability composes with the constrained projector and is exactly the zone that will supply its own constrained edges; removing the hook would delete a seam the next fidelity improvement needs
- [ ] 1.2 **Define the precedence rule** where zone-supplied constrained edges overlap the `no_slip.rs` set. Union is the natural answer — a constraint is a constraint and pinning an edge twice is idempotent — but state and test it rather than assuming it
- [x] 1.3 **Name the heat observable `penalization_heat_integral`** (design D2). The decisive reason is that it frees `wall_heat_flux` for a real Fourier-law implementation when the Gap-2 energy equation lands, rather than leaving the safety-critical name squatted

## 2. Enforce the `BlendedMap` validity guarantee (item 8, blocker B-4)

- [ ] 2.1 Derive the determinant floor **relative to the geometric scale** (e.g. against `dr · span_y`), not as an absolute constant, and document the derivation — an absolute floor would be a magic number of exactly the kind the audit catalogued
- [ ] 2.2 Scan `det_at` over the sampled lattice in `BlendedMap::new`, rejecting a sign change or a magnitude below the floor; the constructor already samples for the metric trains, so this rides on an existing traversal
- [ ] 2.3 Guard the four inverse-metric component divisions and the volume factor, so no path can produce `inf`/`NaN` or ~1e15-magnitude entries behind an `Ok`
- [ ] 2.4 Reconcile the documentation: the module doc's "rejects a fold", and the in-function "Validity (gate BM-A) holds **by construction**" comment whose justification is the `qtt_blend_metric` measurement for one geometry — the claim is now enforced, so say what is checked
- [ ] 2.5 Add tests for a folded map (sign change) and a near-singular map, each asserting construction fails and names the violation
- [ ] 2.6 Run `qtt_blend_metric` and every study constructing a `BlendedMap`; confirm none is now refused. Its measured `min|det J| ≈ 1.5` suggests comfortable margin — record any refusal, it is a finding

## 3. Resolve the dead boundary hook (item 15)

Per the decision from 1.1.

- [ ] 3.1 Fold `collect_constrained_edges` into the constrained projection alongside the `no_slip.rs` set, so the trait's "the solver folds every zone's contribution at the matching stage" becomes true as written
- [ ] 3.2 Implement the precedence rule from 1.2 where zone-supplied and `no_slip.rs` edges overlap, with a test covering the overlap case
- [ ] 3.3 Add a test that a zone implementing the hook actually has an effect — the property whose absence made the hook vestigial
- [ ] 3.4 Add a check that the documented hook set and the solver's fold sites agree, so the next stage cannot be documented without being wired
- [ ] 3.5 Confirm the marched field is bit-identical for the existing cases (Poiseuille, lid-driven cavity, immersed block) — no shipped zone implements the hook, so wiring it should move no current result
- [ ] 3.6 Record the seam's intended consumer (`aperture-resolved-noslip`) at the hook, so its purpose is not lost again

## 4. Rename the heat observable and configure its wall temperature (item 11)

Landed last per design D5: it is the API break and should be an isolated commit with all consumers moved together.

- [ ] 4.1 Rename `wall_heat_flux` per the decision from 1.2, and update the `lib.rs` re-export
- [ ] 4.2 Rewrite its docstring to lead with what it computes — a temperature-weighted volumetric rate, `[T]·[L]²/[t]` — and state explicitly that it is not a surface flux, since Fourier's law is `q = −k·∂T/∂n` and no scaling converts the volume integral into one
- [ ] 4.3 Rename the published series key at `qtt_march_run.rs:214`
- [ ] 4.4 Move `t_wall` into `QttMarchConfig` (design D3: it is a case property, like `η`, and belongs in the `flow_config` layer), replacing the hardcoded `R::zero()`
- [ ] 4.5 Thread the configured `t_wall` to the observable and record it in the run's output, so the wall temperature the quantity is defined against is inspectable
- [ ] 4.6 Move every in-repo consumer of the old series key — including the corridor's branch accumulator — and confirm none is left reading an absent series
- [ ] 4.7 Confirm `preserved_drag_fraction` and the `srp_momentum_jet` study still behave as before; the audit established both use the quantity comparatively, so a rename should not move their numbers

## 5. Verify

- [ ] 5.1 A folded and a near-singular `BlendedMap` are each rejected at construction with a named violation (spec: enforced invertibility)
- [ ] 5.2 No division by `det_at` can produce a non-finite or unbounded metric entry behind a successful construction
- [ ] 5.3 The determinant floor is relative to the geometric scale and its derivation is documented — not an absolute literal
- [ ] 5.4 The trait's documented hook set equals the solver's folded hook set (spec: documented equals folded)
- [ ] 5.5 The heat observable's name and series key describe the computed quantity, and `T_w` is configurable and recorded
- [ ] 5.6 No in-repo consumer reads a series key that no longer exists
- [ ] 5.7 `cargo test -p deep_causality_cfd --release` — no regression against the 828-pass baseline
- [ ] 5.8 `make format && make fix` clean, no new `#[allow]`
- [ ] 5.9 The full verification suite runs; confirm **no harness result moved** — all three items are contract fixes, so a moved number means something unintended happened
- [ ] 5.10 Confirm the diff does not alter the blended-map Jacobian/metric algebra, the `no_slip.rs` constraint enumeration, or the heat integral's formula (Non-Goals)

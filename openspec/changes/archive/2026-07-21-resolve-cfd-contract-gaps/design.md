## Context

Three gaps between a documented contract and the shipped behaviour. They are grouped not because they
share a subsystem — they touch the coordinate layer, the QTT observables and the DEC boundary
abstraction — but because they share a *shape*: in each, the code is defensible and the promise around
it is not, and in each the fix is a decision about which side to move.

| | Site | Documented | Actual |
|---|---|---|---|
| 8 | `coordinate/blended.rs` | constructor "rejects a fold"; validity "by construction" | no check; four `÷ det_at` sites and the volume factor unguarded |
| 11 | `solvers/qtt/observe.rs` | name and series key say `wall_heat_flux` | a volumetric rate `[T]·[L]²/[t]`; no gradient, conductivity or normal. `t_wall` hardcoded to 0, absent from config |
| 15 | `solvers/dec/boundary/boundary_zone.rs` | "the solver folds every zone's contribution at the matching stage", five hooks listed | 4 of 5 wired; `collect_constrained_edges` has zero call sites and zero implementors |

Each has a mitigating detail that matters for scoping:

- **8**: the Jacobian and metric algebra were confirmed correct by the audit. The map works; the
  guarantee is unenforced. And the "by construction" comment is not empty — it appeals to the
  `qtt_blend_metric` study, which measured `min|det J| ≈ 1.5` across the whole λ sweep. That is real
  evidence *for one geometry*, not a property of the accepted input range.
- **11**: the docstring is honest — it states the formula and marks the function *"Neutral — the seam
  the Gap-2 reacting energy equation replaces"*. What misleads is the name and the series key. The
  audit also confirmed no shipped result consumes the absolute value: `preserved_drag_fraction` is a
  same-configuration ratio, and the one study call site subtracts an ambient gauge term. So this is a
  latent trap rather than a live wrong number.
- **15**: the abstraction is sound and used — four hooks fold correctly, and the constraint set the
  fifth would feed is already built and working in `no_slip.rs`. This is one vestigial stage, not a
  broken design.

None of the three is urgent in the sense of producing a wrong number today. All three are the kind of
gap that produces one later, when someone trusts the name, the guarantee, or the hook.

## Goals / Non-Goals

**Goals:**

- `BlendedMap`'s documented validity property is enforced, or not claimed. The `det_at` divisions are
  guarded either way.
- The heat observable's name and series key describe what is computed; `T_w` is configurable.
- Every documented `collect_*` hook is one the solver folds.

**Non-Goals:**

- **The blended-map transformation itself.** Jacobian, metric and chain-rule confirmed correct.
- **The no-slip constraint enumeration** in `no_slip.rs` — working, used, and the reason hook 15 is
  vestigial rather than missing.
- **The Gap-2 reacting energy equation** that will eventually replace the heat-flux seam. This change
  renames a seam; it does not implement its successor.
- **Introducing a real wall heat flux.** Computing `q = −k·∂T/∂n` on an immersed body is a feature.
  Here the aim is only that the current quantity stops claiming to be one.

## Decisions

### D1 — For `BlendedMap`, enforce rather than withdraw

The constructor performs the check its documentation claims, rather than the documentation being
softened to match the absent check.

*Why:* the property is genuinely required — the marcher consumes the inverse metric, which is
`cofactor / det`, so a sign change or a near-zero determinant produces garbage that no downstream
consumer can detect. Withdrawing the claim would leave a public constructor that accepts inputs
producing `inf`-magnitude metric entries behind an `Ok`. The check is also cheap: the constructor
already samples `det_at` over the lattice to build the metric trains, so the sign and magnitude scan
rides along on a traversal that happens anyway.

*The floor is the subtle part.* `|det J| > 0` is not enough — the metric entries scale as `1/det`, so
the floor must be relative to the geometric scale (something like `dr · span_y`), not absolute. An
absolute floor would be a magic number of exactly the kind the audit catalogued.

*Alternative considered.* Guarding only the division, without the constructor check, was rejected: it
turns a silent garbage result into a silent `Err` at march time, far from the configuration that caused
it, and leaves the documented guarantee still false.

### D2 — Rename the observable; do not re-derive it

Item 11 resolves by renaming, not by implementing a real surface flux.

*Why:* the quantity is useful as-is — it is the penalization heat integral, the thermal analogue of the
penalization force integral, and `preserved_drag_fraction`-style ratios built on it are meaningful.
The defect is that its name promises Fourier's law. Renaming is a small, complete fix; implementing a
genuine wall heat flux is a feature that belongs with the Gap-2 energy equation.

**Recommended name: `penalization_heat_integral`.** It states the contraction rather than hedging.
Avoid `wall_heat_flux_neutral` or any qualified form — a qualifier on a misleading noun is exactly how
the current docstring disclaimer already fails to protect a reader who skims.

*The second-order benefit is the one that matters under a fidelity goal:* renaming **frees the name
`wall_heat_flux` for an actual Fourier-law implementation**, `q = −k·∂T/∂n`, when the Gap-2 reacting
energy equation lands. For a re-entry thermal-protection consumer that is the safety-critical
quantity, and a future correct implementation should not have to fight a squatted name or ship as
`wall_heat_flux_2` because the good name was taken by something that is not one.

### D3 — `t_wall` becomes configuration, not a constructor argument

The wall temperature joins `QttMarchConfig` rather than remaining a parameter threaded to a hardcoded
zero at the call site.

*Why:* it is a property of the case, like `η` or the free-stream speed, and the crate's own
configuration/execution split (the `flow_config` layer holds owned descriptions, `flow` materialises
runs from them) puts case properties in the config. Leaving it as a call-site constant is what made it
invisible.

### D4 — Wire the hook into the constrained projection

`collect_constrained_edges` is folded into the constrained projection alongside the `no_slip.rs` set.

**This reverses an earlier draft of this design, which recommended removal.** That draft reasoned that
`no_slip.rs` already enumerates the constraint set (axis-aligned walls, the periodic case, and the
cut-cell solid-incident set), so wiring would create a second path able to disagree with the first,
and that nothing in the crate wanted zone-supplied constrained edges since no zone implements the hook.

The second half of that reasoning was wrong, and checking the roadmap found the counter-evidence:
`openspec/specs/aperture-resolved-noslip/` is an **already-specified capability** whose requirement
*"Composition with the constrained projector and cut Hodge star"* is explicitly about the
constrained-edge set. Fragment-accurate cut-cell no-slip is precisely a zone that supplies its own
constraints — it is the higher-fidelity wall treatment, and it is already on the books.

*Why wiring is right under the high-fidelity goal:* removing the hook would delete the seam that
capability needs, and re-adding it later is strictly more work than leaving it and connecting it now.
The abstraction is sound — four of five hooks fold correctly — and the fifth is the one the next
fidelity improvement will use.

*The precedence question the removal argument raised is real and must be answered, not dodged:* where
a zone supplies constrained edges that overlap the `no_slip.rs` set, the composition rule must be
defined. The natural rule is union — a constraint is a constraint, and pinning an edge twice is
idempotent — but it should be stated and tested rather than assumed.

### D5 — Land the three independently

No shared ordering constraint; they touch different subsystems and none depends on another.

*Why it matters anyway:* item 11's rename is the only one with an API break, so it should be a clean
isolated commit. Item 8's check may reject study geometries, so it wants its own verification pass.
Bundling them into one commit would make a rejected geometry and a renamed series key indistinguishable
in a bisect.

## Risks / Trade-offs

- **The `BlendedMap` check may reject geometries the studies construct.** → Then those geometries were
  producing unguarded metric entries, and the rejection is the finding. `qtt_blend_metric` measured
  `min|det J| ≈ 1.5`, so the shipped blend sweep should pass comfortably; anything that does not is
  worth knowing about.
- **Choosing the determinant floor badly reintroduces a magic number.** → D1 requires it be relative to
  the geometric scale and documented with its derivation, per the evidence-class discipline Phase 1
  established.
- **Renaming a published series key breaks every consumer.** → In-repo only (`publish = false`), and
  the corridor's branch accumulator is the main one. The spec requires no consumer be left silently
  reading an absent series.
- **Removing the hook may foreclose a planned zone type.** → The reason D4 is flagged for the owner
  rather than decided. Wiring instead of removing is a superset of the work, so deferring the decision
  costs little.
- **Three unrelated fixes in one change can read as a grab-bag.** → Accepted: each is too small to
  justify its own proposal, they share a shape, and D5 keeps them separable in the history.

## Migration Plan

No runtime migration. One API break (D2's rename), in-repo only.

1. **`BlendedMap` check and guarded divisions** — self-contained; verify against the study geometries.
2. **`collect_constrained_edges`** — resolve per D4 once the owner has ruled; either direction is a
   small diff.
3. **The observable rename and `t_wall` configuration** — last, because it is the API break and should
   be an isolated commit with all its consumers moved in the same change.

Each step is independently revertible.

## Open Questions

- **Wire or remove `collect_constrained_edges`?** ✅ **Resolved: wire it**, per D4 under the
  high-fidelity goal — reversing this design's earlier removal recommendation. The already-specified
  `aperture-resolved-noslip` capability composes with the constrained projector and is exactly the
  zone that will supply its own constrained edges. The open sub-question is the **precedence rule**
  where zone-supplied edges overlap the `no_slip.rs` set; union is the natural answer and must be
  stated and tested rather than assumed.
- **What should the heat observable be called?** ✅ **Resolved: `penalization_heat_integral`**, per D2.
  The decisive reason is second-order: it frees `wall_heat_flux` for a real Fourier-law implementation
  when the Gap-2 energy equation lands, rather than leaving the safety-critical name squatted by a
  quantity that is not a flux.
- **What is the right determinant floor?** ✅ **Resolved: `1e-6 × (dr · span_y)`.** `det` is an area
  ratio — `−dr·span_y` in the Cartesian limit, `−r·dr·dθ` in the fitted limit — so the floor is a
  fraction of that scale and is dimensionally meaningful at any geometry. The shipped sweep measures
  `min|det J| = 1.506` against a scale of `2.121`, i.e. **5.9 orders of margin**, so no shipped
  geometry is near it.

  Two things were learned while implementing it that the design had not anticipated:

  1. **A fold is reachable, so the sign check is falsifiable.** The module doc argued that
     orientation-compatible charts keep `det J_λ` one-signed; that is false for a linear blend, and a
     parameter sweep found 275 accepted configurations that fold. The doc's reasoning was replaced
     with a statement of what is checked.
  2. **The scan had to cover the *closed* domain.** `sample_grid` forms `ξ = i/nx` for `i` in
     `0..nx`, so it never evaluates `ξ = 1` or `η = 1`. Scanning only the sampled points admitted a
     map degenerating exactly on the fan's outer boundary — the sampled minimum falls off only as
     `~1/nx`, so it would not have been caught until `lx ≈ 20`. This was found by trying to write the
     falsifiability test for the near-singular case, which is the argument for writing it.

- **Does the `qtt_blend_metric` gate agree with the constructor?** ✅ **Resolved: the duplication is
  gone.** BM-A now constructs a real `BlendedMap` per sweep point and reads `det_margin()`, the
  shipped scan's own measured `min|det J|` and floor. `jacobian_scan` — a line-for-line copy of the
  constructor's Jacobian algebra — was deleted.

  Two things were wrong here, one of them mine:

  1. **My claim that collapsing would move BM-B was false.** BM-A went through `jacobian_scan`; BM-B
     goes through a separate `position()`. They were never coupled. I asserted the coupling without
     checking it, which is the failure mode this whole audit is about.
  2. **The study's `span_y` was a different formula from `BlendedMap`'s.** The study used
     `2·RSHOCK·sin(½dθ)` (fan width at the *standoff* radius); the crate uses `2·(r0+½dr)·sin(½dθ)`
     (the chord at *mid* radius). They agree at 2.121320 **only because `r0+½dr = 1.5` coincides with
     `RSHOCK = 1.5`**. The shock standoff is physically independent of the fan geometry, so moving it
     to 1.6 would have silently pointed both gates at a chart the crate never builds, 6.7% wide of
     it. This was a live trap, not a stylistic duplication.

  Neither gate's number moved: `min|detJ| = 1.506` and bonds `114 → 107 → 92 → 54 → 5`, identical to
  before. That is the evidence the copy *did* agree at the shipped geometry — and it can no longer
  drift from it. BM-A is now falsifiable against the shipped code: inflating `DET_FLOOR_FRACTION`
  to `1e6` makes it exit 1 naming the constructor's refusal, which was impossible while it measured
  its own copy.

## Context

Two solver families live in this crate. The DEC family states an envelope and enforces it:
`dec_ns_solver/step.rs::cfl_check` rejects an advective or diffusive CFL violation with
`PhysicalInvariantBroken`, quoting the limit, the configured value and the terms that formed the limit.
The QTT family states an envelope in prose and enforces nothing.

The gap is not one oversight but four, and they compound:

| Item | Site | Current behaviour |
|---|---|---|
| 13 | `QttImmersed2d::new`, `QttIncompressible2d::new` | destructure straight into `Ok(Self{..})`; no check on `η`, `dt`, `ν`, mask. `QttMarchConfigBuilder::build` above them checks only `2^L` grid and seed shapes |
| 12 | all four compressible marchers | density positivity enforced; **unfloored** `p` pushed into `f[1]`, `f[3]`; only `c` uses `p_floor` |
| 12b | same four sites | `1e-300` floor is exactly `0.0` at `f32` — `num-traits` `f64→f32` is an infallible cast, so `unwrap_or_else` never fires |
| 14 | `tensor_bridge/mask.rs` | `χ ∈ [0,1]` documented twice, never enforced; truncation gives `min χ = −1.78e-3` (188 cells) at bond 4 |
| 10 | `qtt_cylinder_verification/config.rs` | `η` pinned by `dt/η = 0.25`; `√(ην) = 0.144·dx`, criterion `η ≥ dx²/ν = 0.771` violated 48× |

Item 10 is the one with consequences already visible. Phase 1 added η and mask-smoothing ladders to
the immersed-cylinder harness; both report `NOT CONVERGING`, establishing that `C_d ≈ 23.8` tracks the
mask blur width rather than the body. **The acceptance test for this change is already written and
already red** — which is unusual and worth exploiting: success is defined before implementation starts.

The four items also explain each other. The mask can go negative (14) because nothing validates it
(13); the negative `χ` inverts the penalization forcing, which is the term `η` scales (10); and the
same "floor it rather than reject it" instinct that hides a negative pressure (12) is what let a
negative `χ` pass unremarked. They are one habit with four expressions.

## Goals / Non-Goals

**Goals:**

- The QTT family refuses configurations it cannot integrate, with DEC-quality diagnostics.
- Non-positive pressure is rejected uniformly across all four compressible marchers.
- Guards behave identically at `f32`, `f64` and `Float106`.
- `χ ∈ [0, 1]` holds for the mask a solver consumes, not just the one before truncation.
- `η` chosen from a wall-error target; the resolution constraint stated and the configuration's
  standing against it reported.
- The immersed-cylinder η ladder converges — the pre-existing acceptance test goes green.

**Non-Goals:**

- **The penalization force law.** `F = (1/η)∫χ(u−u_b)dV` with `C_d = F_x/(½ρU²D)` was confirmed
  correct against Angot et al. during the audit, by probe. The defect is parameter choice, not the
  force law, and re-deriving it would be scope creep dressed as rigour.
- **The flux scheme.** Rusanov, the wave-speed estimate, TT-cross EOS — untouched beyond the
  positivity guard.
- **The DEC family**, the tensor-train codec, and the compressible marchers' conservation properties.
- **The other Phase-2 items.** Chemistry (change 1), navigation (change 3), contract gaps (change 4).

## Decisions

### D1 — Validate at the constructor, not the builder

The envelope is enforced in `QttImmersed2d::new` / `QttIncompressible2d::new`, and the builder is left
to its structural checks.

*Why:* the constructor is the only chokepoint every path passes through. The builder is one caller
among several — the solvers are public API and are constructed directly by tests, studies and the
immersed-body harness. Validating at the builder would leave the direct-construction paths open, which
is exactly the shape of the current gap.

*Alternatives considered.* Validating in both was rejected as duplicated logic that will drift.
Validating only in the builder was rejected for the reason above.

### D2 — Reject non-positive pressure; do not floor it

Item 12 resolves toward rejection rather than a consistent floor.

*Why:* the ideal-gas EOS is not hyperbolic at `p ≤ 0`. The scheme's premise fails there, so a flux
computed from the floored value is not an approximation to the solution — it is a different problem
solved quietly. Density is already treated this way three lines above; treating pressure differently is
the inconsistency, not the fix.

The spec permits a floor as an alternative, but only if applied **consistently** to both the flux and
the wave speed **and reported**. That branch exists because a robustness floor is a legitimate
engineering choice for a production solver; what is not legitimate is the current arrangement, where
the wave speed is repaired and the flux is not, and neither is reported.

### D3 — Derive the guard threshold from the working scalar, not an `f64` literal

The `1e-300` literal is replaced by a threshold established in `R` — e.g. from `R::epsilon()` or the
scalar's own minimum positive value — so it cannot degenerate to zero under a lossy lift.

*Why:* the bug is not the constant's value but its provenance. Any `f64` literal below `f32::MIN_POSITIVE`
has this failure mode, and `num-traits`' infallible float-to-float cast means the usual
`unwrap_or_else` defence does not fire. Deriving from the scalar makes the guard's existence a property
of the type rather than of a literal someone chose.

### D4 — Enforce the mask invariant at construction, not at every use

`body_mask_2d` (and the general `mask_from_fn`) establishes `χ ∈ [0, 1]` after quantization; consumers
then rely on it.

*Why:* the invariant is documented on the mask, so it should be the mask's responsibility. Checking at
each use site would spread the same clamp across the penalization term, the drag contraction, the
interior-speed diagnostic and the heat-flux observable — four places to forget.

*Clamp or reject:* clamping is the pragmatic choice, since a `−1.78e-3` excursion at bond 4 is
truncation noise rather than a modelling error, and rejecting it would fail a bond ladder whose whole
purpose is to run at coarse caps. But the clamp must be **recorded** when it engages beyond a
threshold, so a mask that is badly wrong is distinguishable from one that is slightly rounded.

### D5 — Resolve `η` by stating the conflict, not by assuming it away

`η` is chosen from a wall-error target, and the resolution constraint `η ≥ dx²/ν` is stated together
with the configuration's standing against it.

**Recommendation under the high-fidelity goal: refine the grid.** At the marginal-resolution point
`η = dx²/ν` the slip is exactly `dx` in these units, so refinement buys wall fidelity linearly:

| L | N | `η_min = dx²/ν` | slip `√(ην)` |
|---|---|---|---|
| 5 | 32 | 0.771 | **0.196 — 19.6 % of U∞** |
| 6 | 64 | 0.193 | 9.8 % |
| 7 | 128 | 0.048 | **4.9 %** |
| 8 | 256 | 0.012 | 2.5 % |
| 9 | 512 | 0.003 | 1.2 % |

**A softer wall is not admissible under this goal.** Satisfying the resolution constraint at `32²`
requires `η ≥ 0.771`, giving ~20 % slip at the body — that is a porous obstacle, not a wall, and no
fidelity claim survives it. The measured interior slip at `η = 0.128` was already `5.6e-1`, more than
half the free stream.

**The QTT architecture is the reason refining is affordable here, and this is the case that tests it.**
A `2^L` grid costs `O(χ²·L)`, so `L = 5 → 8` is roughly 1.6× the representation cost where a dense
solver would pay 64×. The field is measurably low-rank: `|ΔC_d| = 1.9e-11` between bond caps 16 and 24,
i.e. fully bond-saturated. Declining to refine on cost grounds would abandon the crate's own thesis in
precisely the case that would demonstrate it.

**Target `L = 7` or `L = 8`** — 4.9 % or 2.5 % slip with the penalization layer resolved. That is the
step which converts the reported `C_d` from a blur-width artifact into a drag.

*Consequence to accept:* the immersed-cylinder case gets more expensive in wall-clock, and its `C_d`
moves. Both are the price of the number meaning something. The current state — a wall whose layer is
7× thinner than a cell, reported as a cylinder drag — is not a cheaper alternative, it is a different
quantity wearing the same name.

### D6 — Let the pre-existing ladders judge the outcome

Success is `qtt_cylinder_verification`'s η ladder converging. No new acceptance criterion is invented
for this change.

*Why:* the ladder was added in Phase 1, observed failing, and its bound (`LADDER_TOL_REL = 0.05`) was
set before any fix was contemplated. Using it as the acceptance test means the target cannot be
adjusted to fit whatever the fix produces — the same discipline as Phase 1's "measure before you gate".

## Risks / Trade-offs

- **The envelope may refuse configurations the shipped harnesses use.** → Likely, and informative. Any
  harness refused is a harness that was outside its solver's stability envelope while reporting
  results. Each such case is brought inside the envelope or its configuration justified; neither is
  "widen the envelope until it passes".
- **Resolving `η` costs grid or wall fidelity (D5).** → Accepted and surfaced as an explicit choice.
  The alternative is retaining a number with no demonstrated limit.
- **Rejecting non-positive pressure may abort runs that previously completed.** → That is the point,
  but it changes failure modes from silent to loud. Any harness that now aborts was producing a flux
  from a non-hyperbolic state.
- **Clamping the mask changes results at coarse bond caps.** → Expected: bond 4 and 8 currently consume
  a mask with up to 188 negative cells, so their published `C_d` moves. The bond ladder's coarse rungs
  are diagnostics, not headline results, and the audit already noted the geometry differs between rungs.
- **Four near-identical edits across the marcher family invite drift.** → Prefer a shared guard the
  four call over four copies of the same check, so the next marcher inherits it.

## Migration Plan

No runtime migration; no public API signature changes (the constructors already return `Result`).

1. **The pressure guard and the precision-safe threshold** (items 12, 12b) — self-contained, testable
   in isolation, and the least likely to move any passing result.
2. **The mask invariant** (item 14) — changes coarse-cap results; land it before the envelope work so
   the η ladder is measured against a valid mask.
3. **Constructor validation** (item 13) — may refuse existing configurations; landing it third means
   any refusal is diagnosed against an already-corrected mask and pressure path.
4. **The Brinkman envelope** (item 10) — last, because it is the one requiring a cost decision, and
   because its acceptance test should run against everything else already fixed.

Each step is independently revertible. Steps 1–3 should not change the η ladder's verdict; if one
does, that is itself a finding worth recording.

## Open Questions

- **Finer grid or softer wall?** ✅ **Resolved: refine the grid**, per D5 under the high-fidelity goal.
  A softer wall means ~20 % slip at `32²`, which is a porous obstacle rather than a wall. Target
  `L = 7`–`8`. The remaining sub-question is which of the two, and that is a wall-error-target choice
  (4.9 % vs 2.5 %) to settle against the case's purpose when task 5.1 runs.
- **Does any shipped harness fall outside the new envelope?** Unknown until item 13 lands. The
  immersed-cylinder case is the obvious candidate, since `dt/η = 0.25` was chosen to sit exactly at an
  explicit-stability ratio.
- **Should the coarse bond rungs remain in the published ladder** once the mask is clamped and their
  `C_d` moves? They demonstrate the rank/accuracy trade-off, but they no longer encode the same
  geometry as the fine rungs.

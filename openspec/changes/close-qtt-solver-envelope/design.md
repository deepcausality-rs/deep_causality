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

*Implementation finding (2026-07-22): clamp-at-construction cannot make the stored train exactly
`[0, 1]`.* A fixed-rank tensor train cannot represent an arbitrary clamped field, so dequantize → clamp
→ re-quantize only *reduces* the excursion (measured `−1.78e-3 → −1.21e-3` at bond cap 4), it does not
remove it. D4's literal form — "establish `χ ∈ [0, 1]` after quantization by clamping" — is therefore
not achievable on the stored train at a coarse cap. The enforceable contract, and what the spec now
states, is: clamp to remove the bulk, **reject** a gross excursion (a wrong mask, not rounding noise),
and accept a residual bounded by the truncation tolerance. The residual reaches the forcing but is
truncation noise: on the shipped `L = 8` ladder the η sweep runs at `sweep_cap = 48`, where the mask
measures `min χ ≈ −7e-7` — non-negative only *to truncation tolerance*, not exactly (the `sweep_cap`
figures below are the `L = 5` values from when this note was first written; the table applies to the
32² probe). The gross-excursion threshold `0.05` of the range is what keeps that residual small.

*Clamp or reject:* clamping is the pragmatic choice, since a `−1.78e-3` excursion at bond 4 is
truncation noise rather than a modelling error, and rejecting it would fail a bond ladder whose whole
purpose is to run at coarse caps. But the clamp must be **recorded** when it engages beyond a
threshold, so a mask that is badly wrong is distinguishable from one that is slightly rounded.

*The violation is one-sided, which the implementation should not assume away.* A probe over the
shipped cylinder mask at every bond cap the ladder runs:

| bond cap | min χ | max χ | cells < 0 | cells > 1 |
|---|---|---|---|---|
| 4 | −1.7800e-3 | 0.981896 | 188 | **0** |
| 8 | −6.5151e-5 | 0.991258 | 84 | **0** |
| 16 | +1.8099e-8 | 0.991837 | 0 | 0 |
| 24 | +1.8099e-8 | 0.991837 | 0 | 0 |

Only the lower bound is breached; `max χ` never reaches 1 at any cap, because the smoothed indicator
peaks below 1 and truncation lowers it further. So the upper clamp is unexercised by the shipped
configuration. Implement both bounds regardless — the invariant is `[0, 1]` and a future geometry or
smoothing width can breach the top — but do not expect the upper half to fire, and do not treat a
test that only exercises the lower bound as covering the clamp.

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

## Premise verification (2026-07-22, before implementation)

Every falsifiable premise in this change was tested against the tree. **Fifteen of sixteen hold; one
was wrong.** No premise that decides the approach failed, so the change is implementable as written.

| Premise | Verdict | How checked |
|---|---|---|
| DEC `cfl_check` rejects with `PhysicalInvariantBroken` | ✅ | `step.rs:138-142` |
| QTT constructors validate nothing | ✅ | both destructure straight into `Ok(Self{..})` |
| Builder checks only structure | ✅ | only `DimensionMismatch` on grid/seed |
| Unfloored `p` in `f[1]`/`f[3]`, floored only for `c` | ✅ | `marcher_2d.rs:135-140` — line-precise, all four sites |
| `1e-300` floor is exactly `0.0` at `f32` | ✅ | **executed**: `f32::from_f64(1e-300) == Some(0.0)` |
| `f32` is a reachable precision | ✅ | blanket `impl CfdScalar`, documented at `cfd_scalar.rs:11` |
| Mask negatives: −1.78e-3/188 at bond 4, −6.5e-5/84 at bond 8 | ✅ | **probe reproduced exactly** |
| `dt/η = 0.25`, `√(ην) = 0.144·dx`, `η ≥ dx²/ν` violated 48× | ✅ | recomputed: 0.2500, 0.1441·dx, 48.2× |
| D5's whole L-ladder (η_min and slip, L = 5…9) | ✅ | every row reproduces to three digits |
| η and smoothing ladders currently red | ✅ | committed baseline shows both `NOT CONVERGING` |
| `LADDER_TOL_REL = 0.05` | ✅ | `print_utils.rs:41` — note: **not** `config.rs` |
| `ΔC_d` bond-saturated | ✅ | design's 1.9e-11 absolute ≡ baseline's 7.936e-13 relative |
| Constructors already return `Result` (no API break) | ✅ | both `-> Result<Self, PhysicsError>` |
| `KNOWN-FAILING` is in `verification/README.md` | ❌ **FALSE** | it is in `.github/workflows/cfd_verification.yml`; corrected in the proposal and task 6.3 |

Two notes for the implementer, both from this sweep: `LADDER_TOL_REL` lives in `print_utils.rs`, not
the `config.rs` the Impact section lists; and the mask's `[0,1]` breach is one-sided (see D4).

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

## Group 5 implementation findings (2026-07-22)

Refining the grid (D5) is not a one-line `L` change: it interlocks with the guards groups 2–4 added,
and with the η ladder's own resolution. Three findings, all forced by physics or by this change's own
new checks — none by fitting to pass.

**1. The η ladder resolves fully only at `L = 9`.** The ladder sweeps `η ∈ [0.128 … 0.008]`, but each
point is physically meaningful only where the penalization layer is resolved, `η ≥ dx²/ν`:

| L | grid | `dx²/ν` | resolved ladder η |
|---|---|---|---|
| 7 | 128² | 0.048 | 0.128, 0.064 (2/5) |
| 8 | 256² | 0.012 | 0.128 … 0.016 (4/5) |
| 9 | 512² | 0.003 | all five |

D5 recommended `L = 7`/`L = 8`; D6 forbids editing the ladder. Those are in tension — below `L = 9` the
ladder's smallest η values are under-resolved. **Resolution: refine to `L = 8`** (the largest grid that
runs in reasonable wall-clock; 4/5 points resolved) and **keep the ladder unchanged**, reporting the
`η = 0.008` tail as the documented under-resolved point rather than deleting it. Verifying the last
point needs a separate `L = 9` run (its cost is the open question below).

**2. `dt` is now forced by the group-4 envelope, not chosen.** At `L = 8`, `dx = 0.02454`, so the
diffusive explicit-stability limit `dt ≤ dx²/(4ν) = 3.01e-3` binds — the old `dt = 0.004` is **refused
by `QttImmersed2d::new`** (the check this change added). `dt` drops to `0.0025`, and `STEPS` rises
`40 → 64` to hold the physical horizon `steps·dt = 0.16`. The envelope check governing the harness
config is the intended coupling, not a coincidence.

**3. The bond ladder's coarse caps are forced up by the group-3 mask guard.** A fixed bond cap
represents a coarser mask on a finer grid: at `L = 8`, bond 4 gives `min χ = −0.15` (measured), which
the mask `[0, 1]` guard rejects as a wrong mask (>5% of range). The bond ladder rises `[4, 8, 16, 24] →
[24, 48]` — the rungs where the mask is valid (`min χ = −1.4e-3` at 24, `−7e-7` at 48) — still
demonstrating rank convergence. This is the group-3 guard doing its job: bond 4 on 256² *is* a garbage
mask.

Every one of these is compelled: by the ladder's physics (1), by the envelope check this change added
(2), or by the mask guard this change added (3).

**4. The refined harness is not affordable — task 5.1a's finding, and it is a finding about the QTT
thesis.** Measured per-step wall-clock for the immersed-cylinder march at a fixed bond cap 24:

| grid | per-step | vs L=5 |
|---|---|---|
| L=5 (32²) | 0.05 s | 1× |
| L=6 (64²) | 0.90 s | 18× |
| L=7 (128²) | 6.51 s | 130× |
| L=8 (256²) | 16.3 s | 326× |

The field stays low-rank at every resolution: the marched velocity's achieved bond **saturates at the
cap 24 at all of L=5, 6, 7, 8** (measured). So this is not rank growth — and, importantly, it is **not
the tensor-train `O(χ²·L)` scaling either.** At a fixed bond, that model predicts only ~1.6× from L=5
to L=8 (the core count rises 10 → 16; the χ factor cancels from the ratio, so a "large constant" cannot
explain 326× — a constant cancels too). The measured 326× is therefore a **superlinear bottleneck
outside the tensor-train arithmetic** — the divergence-free projection's CG (whose iteration count
grows with resolution) and/or a dense `O(N²)` step in the per-march path — which QTT compression does
not accelerate. Pinning the exact term is itself part of the follow-up; what is established here is that
the low-rank *representation* is intact and the cost is elsewhere. A single 64-step march at L=8 is
~17 min, and the acceptance harness (13 marches over the bond, η and smoothing ladders) is **~4–9 hours**;
L=9 (which the η ladder needs and which forces `dt` down another ~4×) would be **days**.

**Consequence:** the Brinkman envelope can be *resolved* (the L=8 config is physically correct and
passes the envelope checks), but its acceptance test cannot be *run* at feasible cost with the current
solver. This is exactly the risk `project_cfd_minutes_northstar` names — cylinder reference accuracy
must run in minutes, and at the resolution that makes the drag physical it runs in hours. So group 5
does not retire the known-failing status; it reclassifies it: the cylinder gate is red not because the
physics is wrong but because a non-tensor-train part of the solver is too slow at the resolution the
physics needs. Closing it is a **solver-performance** follow-up (the acceleration ladder in
`cfd-industry-scaling`), whose first step is to profile which term (projection CG vs a dense step)
carries the superlinear cost — not a
parameter fix.
 The `η`/`dt`/`STEPS`/`L`/bond-cap changes are all
traceable to a stated constraint, per task 7.8.

## Adversarial review (2026-07-23)

A 6-dimension adversarial review (each finding independently verified) ran over the finished diff.
It confirmed 8 findings, all in **documentation/spec accuracy** — the runtime was clean — and all
traceable to one root: moving the harness `L = 5 → L = 8` in group 5 without updating claims written
against the `L = 5` assumptions. Corrected:

- **The cost explanation was wrong.** The design attributed the 326× slowdown to the tensor-train
  `O(χ²·L)` cost "with a large constant". That is void — at a fixed bond a constant cancels from the
  ratio, and `O(χ²·L)` predicts only ~1.6×. Measuring the achieved bond showed it saturates at the cap
  24 at every L, so the superlinearity is a bottleneck *outside* the tensor-train arithmetic. The
  finding is now stated correctly (see above).
- **Two mask claims were stale.** `MASK_GROSS_EXCURSION`'s rationale cited `L = 5` "diagnostic rungs"
  (bond 4/8) that the `L = 8` ladder no longer runs, and the clamp docstring plus a spec scenario
  claimed the acceptance-cap mask is "non-negative" — measured `min χ ≈ −7e-7` at the shipped `L = 8`
  cap 48, i.e. non-negative only to truncation tolerance. Both corrected; the spec scenario now states
  the honest "in range to truncation tolerance" guarantee.
- **A test did not guard what it named.** `the_clamp_removes_the_gross_mask_excursion` asserted only
  `min > −0.05`, which the *un-clamped* raw mask already satisfies — so it passed with the clamp
  removed. Replaced with `the_clamp_strictly_reduces_the_coarse_cap_excursion`, which compares the
  clamped mask against the raw `quantize_2d` output and asserts the clamp strictly reduces the
  excursion; verified to fail when the clamp is removed.

The review earned its keep: every confirmed finding was a real overclaim, and the cost-explanation
error was the kind of plausible-but-wrong reasoning this whole audit exists to catch — here in my own
change.

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

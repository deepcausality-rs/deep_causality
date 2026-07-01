<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Gap 2, Resolution 4 — body-fit as a free parameter, not a hard-wired specialization

**What this is.** A TRIZ/ARIZ resolution of the **structural assumption** the Tier-B plan
([`tier-b-compressible-marcher.md`](tier-b-compressible-marcher.md), design
[D1](../../changes/add-cfd-compressible-qtt-marcher/design.md)) treats as a fixed trade-off: *"structured /
body-fitted only — buys the rank win at the cost of generality."* The body-fitted coordinate is the **measured**
rank lever (`χ ~ O(10)` fitted vs `√side` captured), but as built it is a hard-wired analytic sphere-cone map
chosen once at assembly — so the solver is bound to one geometry. This note shows the trade-off is not
load-bearing: **body-fittedness can be a free, continuous parameter** while the solver stays a general
structured-CFD marcher.

It is the first of three coupled Tier-B resolutions that share a single mechanism — the **Feature-Adaptive
Coordinate** (the spatial dual of Tier-A's LER), named in
[Resolution 7](gap-two-resolution-7-feature-adaptive-coordinate.md). See also
[Resolution 5](gap-two-resolution-5-dynamic-rank-lever.md) (the dynamic, marching version) and
[Resolution 6](gap-two-resolution-6-implicit-acoustics.md) (the conditioned implicit step).

Honesty convention: **[holds]**, **[holds under precondition]**, **[open]**.

---

## 0. Frame

- **Key problem (no solution words):** keep the alignment-driven rank win **without binding the solver to one
  geometry**.
- **System / main function:** the coordinate map `T: (ξ,η,ζ) → (x,y,z)` and the metric MPOs derived from it
  (`coordinate/BodyFittedCoordinate`); *to align grid lines with the dominant solution features so the field is
  low-rank in computational space.*
- **The constraint treated as fixed — the lever:** that `T` is a **hard-wired analytic map chosen at assembly**.
  Drop it: **the map is *data*, not *structure*.**

---

## A. Reformulate (the ARIZ spine)

**A2 — Technical contradictions, both ways:**
- **TC-1:** specialize the map to the body → rank collapses to `O(10)` (good), but the solver works for **one
  geometry** (bad).
- **TC-2:** use the identity / Cartesian map → **any** geometry (good), but rank → `√side` at the shock (bad).

**A3 — Intensify.** Push TC-1 to perfect fit → `O(1)` rank, exactly one body. Push TC-2 to the identity → fully
general, `√side`. The extremes expose the truth: *fittedness is a **continuum**, not a binary.* "Structured xor
general" is a false dichotomy — the real axis is the **degree of alignment**, a knob.

**A5 — Resources already present (no new substance):**
- The **metric MPOs** (`∂ξ/∂x …`) are **already** TT operands multiplied into the operators via
  `hadamard_rounded` — the map already flows through the solver as *data*.
- The map is already **parameterized by a few scalars** (`r0, Δr, θ0, Δθ`).
- A coordinate map **is itself a low-rank TT field**; a convex blend of two low-rank fields is low-rank.
- The **`PhysicsStage`/`Coupling` cons-tuple static-dispatch seam already exists** as the house template for
  "swap the implementation, keep the solver."

**A7 — Smart Little People.** Put an agent on every grid node. Each holds *two* candidate positions — where it
would sit on a Cartesian grid and where it would sit on the fitted grid — and stands at a weighted blend `λ`
between them. Near the wall and shock the agents lean fully fitted (`λ→1`); out in the freestream they relax to
Cartesian (`λ→0`). One blend knob per node, and nowhere does the *solver* know which body it is serving.

**Physical contradiction:** the map must be **specific** (aligned to this geometry, for rank) **and generic**
(bound to no geometry, for reuse). **Resolve by separation across scale** (generic at the whole-solver level,
specific at the map-component level) **and across space** (fitted near the body, Cartesian in the far field).

→ Reformulation cracks it. The matrix lookup is confirmatory.

---

## B. Solve — the `MetricProvider` seam + a body-fit blend field

Two moves, both idiomatic in this codebase:

**1. Promote the map to a `MetricProvider<D, R>` trait seam (static dispatch).** The solver consumes only
*"there exist metric MPOs `G_ij` and a Jacobian"* and **never branches on geometry**:

```text
trait MetricProvider<const D: usize, R: CfdScalar> {
    fn metric(&self) -> &[CausalTensorTrainOperator<R>];   // ∂ξ_i/∂x_j as low-rank MPOs
    fn jacobian(&self) -> &CausalTensorTrain<R>;            // det, low-rank
}
```

`CartesianIdentity` is one impl, `BodyFittedCoordinate` another, `BlendedMap<A, B>` a cons-style composition —
**exactly the `PhysicsStage` / `Coupling` cons-tuple pattern already in the crate.** Generality is recovered at
the *type* level, at zero asymptotic rank cost.

**2. Make fittedness a continuous blend.**

```text
T_λ = (1 − λ)·T_identity + λ·T_fitted          λ ∈ [0, 1]
```

Both endpoints are low-rank TT fields, so the blend is low-rank (ranks add, then round). `λ = 0` → fully general
capture (`√side`, *any* body); `λ = 1` → fully fitted (`O(10)`, this body). **Body-fit is now a free scalar
parameter.** The strong form is **`λ(ξ)` as a spatial field** — `λ→1` near wall/shock, `λ→0` in the smooth far
field — one map that is fitted where it matters and generic where it does not; the blend field is smooth ⇒ low
rank, and this matches the physics (no body-fitting is needed in the freestream).

> **TRIZ principles used:** **separation by scale** (generic solver / specific map) and **by space** (fitted
> here / Cartesian far); **#6 Universality** (one seam serves every geometry); **#1 Segmentation** (geometry is
> a swappable component); **#15 Dynamics** (rigid map → adjustable); **#24 Intermediary** (the computational
> frame is a removable go-between). **Effects database:** convex homotopy between two coordinate charts is a
> standard construction.

---

## C. Verify & harvest

- **Physical contradiction removed, not compromised?** Yes. Generic solver and specific map coexist by scale
  separation; fitted and Cartesian coexist by space separation (the `λ(ξ)` field). No trade — the rank win and
  the generality are held simultaneously.
- **Only A5 resources?** Yes. The seam reuses the existing cons-tuple pattern; the blend reuses the metric-MPO
  machinery already multiplied into the operators. No new substance, no new house.
- **Satisfies the IFR?** Yes — the map keeps the field low-rank for **any** supplied body, throughout the march,
  by being *data through a seam* rather than a hard-coded code path.

**New harm (the next problem).** A convex blend of two diffeomorphisms is **not guaranteed non-singular** for
every `λ` if the two charts are "twisted" relative to each other (a folded cell ⇒ negative Jacobian ⇒ garbage
metrics). **Mitigation:** blend grid-point displacements with a **bounded `λ`-gradient** and a **positive-
Jacobian homotopy guard** (reject / re-scale any step that would fold a cell). **[open: blend-validity guard]**

**Generalized method.** *Promote the coordinate to a data-supplied, blendable `MetricProvider` seam; then
"body-fittedness" is a tunable parameter (scalar or spatial field), recovering general-structured-CFD generality
at zero asymptotic rank cost, and specializing the instant a body is supplied.* The structured-vs-general
dichotomy was an artifact of hard-wiring the map.

**Inverse / scaling.** `λ→0` recovers Cartesian capture (the D1 fallback) for arbitrary or unknown geometry;
`λ→1` is the fitted limit; intermediate `λ` is partial alignment (useful for multi-body or imperfectly-known
shapes). As the geometry library grows, each new body is **one more `MetricProvider` impl**, not a solver
change.

---

## Measured (study `qtt_blend_metric`, 2026-06-30)

Probed before the build. The position-blend `T_λ = (1−λ)·Cartesian + λ·fitted`, over compatibly-oriented charts
in front of the nose, stays a **valid map**: `det J` holds one sign with `min‖det J‖ ≈ 1.5` across the whole
`λ ∈ {0, 0.25, 0.5, 0.75, 1}` sweep, so no cell folds (the blend-validity residual holds for compatible charts).
A fixed physical shock sampled on the blended lattice runs **monotonically from bond 114 at `λ=0` (Cartesian
capture) to 5 at `λ=1` (body-fitted)**. So `λ` is a clean, continuous rank dial. **[holds: blend valid +
dialable for compatibly-oriented charts]** The open part stays the *incompatibly-oriented* (twisted) chart pair,
where a fold is possible and the positive-Jacobian guard does real work.

## Verification gates (what a spec/PR must prove)

1. **`λ`-sweep on one solver:** with the *same* marcher, `λ = 0` reproduces the Cartesian control's `√side`
   growth and `λ = 1` reproduces the fitted `O(10)` bound — the lever is demonstrably a *parameter*, on a single
   binary. **[holds: blend of low-rank maps is low-rank]**
2. **Jacobian positivity across the sweep:** `det J > 0` at every node for every `λ` tested (no folded cells) —
   the blend-validity guard works.
3. **Seam purity:** the marcher is generic `over M: MetricProvider<D, R>`, static dispatch, **no geometry
   branch**, no `dyn` (honors the kernel/solver convention).
4. **Blend field stays cheap:** the spatial `λ(ξ)` field is itself low-rank (bounded bond), so it does not
   inflate the operator rank it is meant to control.

---

## Related

- [`tier-b-compressible-marcher.md`](tier-b-compressible-marcher.md) — the measured `√side`-vs-`O(10)` lever
  this note turns into a parameter.
- [`gap-two-resolution-5-dynamic-rank-lever.md`](gap-two-resolution-5-dynamic-rank-lever.md) — the **dynamic**
  version: the same map, **feedback-updated** to track the feature as it moves.
- [`gap-two-resolution-7-feature-adaptive-coordinate.md`](gap-two-resolution-7-feature-adaptive-coordinate.md) —
  the unifying mechanism this is the generality-facet of.
- `add-cfd-compressible-qtt-marcher/design.md` — D1 (the trade-off this resolves), D8 (the seam), Stage 0.5.
- `deep_causality_cfd/src/coordinate/mod.rs` — `BodyFittedCoordinate`, to become one `MetricProvider` impl.

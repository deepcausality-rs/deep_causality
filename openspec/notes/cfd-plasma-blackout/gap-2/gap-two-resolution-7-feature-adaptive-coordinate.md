<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Gap 2, Resolution 7 — the Feature-Adaptive Coordinate: one mechanism for the Tier-B challenges

**What this is.** The **recurring theme** behind Resolutions
[4](gap-two-resolution-4-body-fit-parameter.md),
[5](gap-two-resolution-5-dynamic-rank-lever.md), and
[6](gap-two-resolution-6-implicit-acoustics.md), and the resolution of the **remaining** Tier-B challenges
(shock-fitting composition, the bounded-χ gates, HLLC-vs-Rusanov, coordinate generation). Working those three
challenges through ARIZ independently, the *same* reformulation discharged each — which is the tell of a single
underlying mechanism, exactly as the Tier-A resolutions all collapsed onto **LER**. This note names that
mechanism, the **Feature-Adaptive Coordinate (FAC)** — the **spatial dual of LER** — and shows it is one
architectural commitment, not four.

Honesty convention: **[holds]**, **[holds under precondition]**, **[open]**.

---

## 0. Frame

- **Key problem (no solution words):** make the live Mach-25 reacting flowfield **low tensor-train rank by
  construction**, **generically**, **throughout the march** — and condition the implicit solve while doing it.
- **The recurring contradiction (one sentence):** *physical* space is hostile (sharp / stiff / high-rank /
  geometry-specific); *computational* space must be benign (smooth / well-conditioned / low-rank / generic).
  Every Tier-B difficulty is an instance of this one contradiction.

---

## The mechanism, named once

> **Move the difficulty out of the field and into a cheap, low-rank, data-supplied, feedback-updated coordinate
> map.** Whatever is sharp / stiff / high-rank / geometry-specific in *physical* space is made smooth /
> well-conditioned / low-rank / generic in *computational* space by a map that is

1. **data through a `MetricProvider` seam** — generality (Resolution 4);
2. **feedback-updated to track the dominant feature each step** — bounded rank by construction = shock-fitting
   (Resolution 5);
3. **jump-confining**, so the interior operator is near-constant-coefficient and the implicit acoustic solve is
   a small perturbation of a closed-form inverse (Resolution 6).

**FAC is the spatial dual of LER.** LER (Resolutions 1–3) confines a **stiff source in time** — relax toward a
state-derived equilibrium target in closed form, so stiffness never reaches the marcher. FAC confines a **sharp
feature in space** — pin the singularity to a moving coordinate surface in closed-form metric form, so
high rank / bad conditioning never reaches the field. **Both refuse to fight the hard thing in its native
representation; both change the representation so the hard thing becomes easy.** One idea, two axes.

---

## One commitment discharges many challenges

| Challenge (design / Tier-B note) | How FAC discharges it |
|---|---|
| **Structured/body-fitted only** (D1 trade-off) | `MetricProvider` seam + blend field `λ`; generic solver, specific map, blends of low-rank maps stay low-rank → **body-fit is a free parameter** (Res 4) |
| **Make-or-break: static lever survives marching** | feature pinned to a coordinate line ⇒ coordinate-stationary ⇒ `O(1)` rank **by construction**, independent of time (Res 5) |
| **Make-or-break: AMEn convergence** (D3) | jump confined to the interface ⇒ smooth interior ⇒ constant-coefficient closed-form inverse is an exact preconditioner ⇒ `I + small` (Res 6) |
| **Stage 4: shock-fitting "unprecedented" coupling** | fitting **is** the dynamic `MetricProvider` — not bolted onto the QTT bulk, it **is** the coordinate the bulk already runs in; the jump is an interface BC, the interior stays smooth and low-rank |
| **Stage 5/6: bounded χ gates** | the gate becomes a **test of the by-construction invariant** (the pinned feature's cross-axis bond stays `O(1)`) vs the Cartesian control (`λ = 0`) reproducing `√side` |
| **Open Q: HLLC vs Rusanov** | fitting supplies the sharp jump by **exact RH**, so the interior flux only transports **smooth** relaxation ⇒ Rusanov's diffusivity is acceptable; **the theme decides the open question** and demotes the flux-scheme choice |
| **Open Q: coordinate generation** | `MetricProvider` makes analytic-vs-computed a **swappable impl**; start analytic (sphere-cone, closed-form), allow a computed grid later, no solver change |

> **Recurring TRIZ principles:** **#15 Dynamics**, **#23 Feedback**, **#6 Universality**, **#24 Intermediary**,
> **#1 Segmentation / #35 Parameter change**, **#22 Blessing in disguise**; **separation by space** (sharp here /
> smooth far) and **by scale** (generic solver / specific map) throughout. The seam is the *same* static-dispatch
> + cons-tuple pattern already carrying `PhysicsStage` / `Coupling` — **no new house.**

---

## C. Verify & harvest

- **Physical contradiction removed, not compromised?** Yes — the hostile-physical / benign-computational tension
  is dissolved by changing frames, the same move LER makes in time. No averaging, no accuracy-for-rank trade.
- **Only existing resources?** Yes — the metric MPOs, the cons-tuple seam, the live front location, the Rusanov
  floor, and a standard constant-coefficient inverse. No new substance, no new architecture.
- **One mechanism, not four?** Yes — generality, dynamic rank, conditioning, and fitting are **facets of the
  single FAC commitment**; that they fell out of three independent ARIZ passes is the evidence.

**The honest boundary (derived, not asserted).** FAC pins **one** dominant feature per structured map.
Single-feature regions — the **forebody sheath, the bow shock, the stagnation line** — are low-rank by
construction and **gateable**. **Multi-feature unsteady regions — the wake** — cannot be pinned by one structured
map (shock + contact + shear + separation at once) and are **genuinely open** (multi-patch / overset fitting, or
accept that turbulence is a non-goal). This is precisely the design's Stage-6 scope boundary and the
`qtt_rank_3d` residual — now *explained* by the mechanism rather than merely stipulated. **[open: multi-feature /
wake pinning]**

**Generalized method.** *For any marcher whose difficulty is a localized singularity (a shock, a sheath, a
boundary layer, a flame front), make the coordinate a first-class, dynamic, blendable, data-supplied operand and
move the singularity into it; the field stays low-rank and the operators stay well-conditioned by construction.*
Its temporal twin is LER. Together they are the two halves of one design law for this codebase: **confine
stiffness in time, confine singularities in space — never in the field.**

**Inverse / scaling.** As the number of dominant features → 1, FAC is exact and the QTT thesis holds with room
to spare; as features proliferate and go unsteady, FAC degrades to capture (`√side`) and the honest move is to
declare the region out of scope (the wake) rather than to assert a bound that was never measured.

---

## Verification gates (the meta-gate)

The four resolution gate-sets **compose**; the change is *done* when the by-construction invariant holds
end-to-end:

1. **Res 4** — `λ`-sweep: one solver spans `√side` (`λ=0`) to `O(10)` (`λ=1`); Jacobian positive throughout.
2. **Res 5** — rank-vs-time: a *marched* fitted single-feature shock holds `max_bond` bounded while the
   Cartesian control grows `√side`.
3. **Res 6** — stability beyond the acoustic CFL with the closed-form-inverse-preconditioned step; perturbation
   spectral radius `< 1` on the fitted interior.
4. **End-to-end (Stage 6)** — the 3-D **forebody** sheath marches at bounded χ; the **wake** is reported, never
   gated. **[open: wake]**

---

## Related

- [`gap-two-resolution-1-stiff-source.md`](gap-two-resolution-1-stiff-source.md),
  [`-2-temperature-provenance.md`](gap-two-resolution-2-temperature-provenance.md),
  [`-3-ionization-lag.md`](gap-two-resolution-3-ionization-lag.md) — the **temporal** dual (LER) this mirrors.
- [`-4-body-fit-parameter.md`](gap-two-resolution-4-body-fit-parameter.md),
  [`-5-dynamic-rank-lever.md`](gap-two-resolution-5-dynamic-rank-lever.md),
  [`-6-implicit-acoustics.md`](gap-two-resolution-6-implicit-acoustics.md) — the three facets unified here.
- [`tier-b-compressible-marcher.md`](tier-b-compressible-marcher.md) — the measured rank evidence FAC is built
  on.
- `add-cfd-compressible-qtt-marcher/design.md` — D1, D8–D10, Stage 0.5, the revised risks and open questions.

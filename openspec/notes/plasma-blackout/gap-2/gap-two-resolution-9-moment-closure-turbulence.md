<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Gap 2, Resolution 9 — turbulence as a modeled region: moment closure (the fourth confinement axis)

**What this is.** A TRIZ/ARIZ resolution of the residual left after
[Resolution 8](gap-two-resolution-8-spectral-pinning.md): the **fully-turbulent far-wake**, where the adaptive
Schmidt rank `K(t)` grows without bound because turbulence is, by definition, a high-information / high-rank
field. The honest headline is **not** "turbulence is solved" — it cannot be represented at low rank, that is
information-theoretic. The lever is that **the flagship never needed the turbulent *field*; it needs the mean
electron density**, and the *mean* is low-rank. The residual was an artifact of conflating *model turbulence*
with *resolve turbulence*. A Reynolds/moment closure carries turbulence's **statistical effect** on the mean,
and that closure **is the LER pattern lifted one level** — so it rides the machinery already built.

This completes the confinement family: LER (time, [Res 1](gap-two-resolution-1-stiff-source.md)), FAC (space,
[Res 5](gap-two-resolution-5-dynamic-rank-lever.md)), DLRA (Schmidt basis,
[Res 8](gap-two-resolution-8-spectral-pinning.md)), and **moment closure (statistical moments, here)**.

Honesty convention: **[holds]**, **[holds under precondition]**, **[open]**.

---

## 0. Frame

- **Key problem (no solution words):** obtain the blackout-relevant **mean** fields in a turbulent region
  without carrying the fluctuations that make the field high-rank.
- **System / main function:** the marched field over the turbulent near-wake / sheath; *to deliver the mean
  electron density (and the mean transport that drives it).*
- **The constraint treated as fixed — the lever:** that "handle turbulence" means **resolve the instantaneous
  turbulent field** (DNS/LES). Blackout `n_e` is a **statistic**, not a realization — that equivalence is the
  psychological inertia, and the lever.

---

## A. Reformulate (the ARIZ spine)

**A2 — Technical contradictions, both ways:**
- **TC-1:** resolve turbulence (DNS/LES) → physically faithful, but **unbounded rank** (the residual).
- **TC-2:** ignore turbulence → low-rank, but **wrong mean mixing / transport → wrong `n_e`**.

**A3 — Intensify.** Push toward fully-developed turbulence: instantaneous information content → maximal →
tensor-train rank → unbounded **by definition**. Resolving it at low rank is **information-theoretically
impossible** — so stop trying. Push the *other* way: what does the comms link actually integrate? A
**time/ensemble-averaged** plasma frequency over the sheath. Averaging annihilates the fluctuation. **The
required output lives entirely in the low-order moments.** That is the crack.

**A4 — Conflicting pair.** *Product:* the field in the turbulent region. *Tool:* the representation. Conflicting
states: the representation must **contain the fluctuation** (faithfulness) and **omit it** (low rank).

**A5 — Resources already present (no new substance):**
- **Reynolds / moment decomposition** `φ = φ̄ + φ′` — standard physics (RANS, k–ω SST, Reynolds-stress
  transport, γ–Reθ transition); the engineering standard for hypersonic aerothermodynamics.
- **RANS mean fields are smooth → low tensor-train rank** (no instantaneous eddies in them).
- The **LER stage machinery** ([Res 1](gap-two-resolution-1-stiff-source.md)): a stiff source integrated by a
  closed-form relaxation toward a state-derived equilibrium target — *exactly* the structure of turbulent
  kinetic-energy production/destruction (`k`, `ω`).
- The **coefficient-field-as-low-rank-MPO** machinery (FAC metrics, `hadamard_rounded`): an **eddy-viscosity**
  field is just another spatially-varying coefficient on the diffusion operator.
- **`advance_scalar`** (Gap 1): carries extra transported scalars — `k`, `ω` are scalars exactly like the
  Tier-A species/temperature.
- **DLRA / `tdvp`** ([Res 8](gap-two-resolution-8-spectral-pinning.md)): for any residual *resolved*
  unsteadiness (URANS / hybrid RANS–LES).
- The **Tier-A reacting-LER stages** — structurally identical to a turbulence closure (see B), so they are the
  literal template.

**A6 — Operating zone / time.** The conflict lives only where turbulence is energetic *and* affects the mean
`n_e` — the near-wake shear and the transitional sheath. The far-wake fine structure is downstream of the
antenna and does not feed the forebody sheath; it never needed resolving.

**A7 — IFR + physical contradiction.**
- **IFR:** the turbulent region, using only the LER-stage / coefficient-MPO / FAC / DLRA machinery already
  present, delivers the correct mean `n_e` — by carrying turbulence's **statistical effect** (a stress / eddy
  viscosity), never its field.
- **Physical contradiction:** the field must be **high-rank** (contain fluctuations) **and low-rank** (QTT).
  **Resolve by separation in statistical scale — the Reynolds decomposition:** keep the **mean** as a resolved
  low-rank field; represent the **fluctuation only through its low-order moments** (Reynolds stress / eddy
  viscosity), themselves low-rank smooth fields. The high-rank object is **never instantiated**.

→ Reformulation cracks it: the Reynolds decomposition **is** a separation-by-scale, and the moment closure **is**
the LER pattern lifted one level.

---

## B. Solve — moment closure as "LER for turbulence"

The Tier-A reacting closure and a turbulence closure are the **same pattern**: *an unresolved fast process
carried as extra scalar moments relaxing toward a state-derived equilibrium.* So the shipped reacting-LER stages
are the template. Each closure piece maps onto built machinery:

| Turbulence-closure piece | Rides on (already built) |
|---|---|
| Mean fields `(ρ̄, ū, ρĒ, n̄ₑ)` | the low-rank QTT marcher — RANS means are smooth |
| Turbulent kinetic energy `k`, rate `ω` (or `ε`) | **LER stages (Res 1)** — stiff production/destruction by closed-form relaxation toward local-equilibrium turbulence; transported by `advance_scalar` |
| Eddy viscosity `μ_t(k, ω)` | a **low-rank coefficient MPO** on the diffusion operator (FAC's `hadamard_rounded` coefficient path) |
| Mean shock / shear discontinuities | **FAC geometric pinning (Res 5)** — the mean still has a sharp bow shock |
| Transition onset (laminar → turbulent) | a γ–Reθ **transported scalar = another LER stage** |
| Residual resolved unsteadiness (URANS / hybrid) | **DLRA / `tdvp` (Res 8)** — the `K(t)` budget decides RANS vs hybrid |

**Smart Little People.** Agents in a cell do not track eddies. Each holds a local `k` (how energetic the
unresolved swirl is) and `ω` (how fast it turns over), relaxes them toward the production/dissipation balance,
and reports an **enhanced diffusivity** to the mean-field transport. No fluctuation is ever stored.

> **TRIZ principles used:** **separation by scale** (the Reynolds decomposition resolves the physical
> contradiction); **#26 Copying** (carry a cheap statistical surrogate, not the real field); **#6 Universality**
> (the reacting-LER stage also serves turbulence); **#23 Feedback** + **#35 Parameter change** (`k`/`ω` as
> relaxing derived states); **#16 Partial action** (model, do not resolve). **Effects database:** Reynolds-
> averaged moment closure is the established hypersonic-aerothermo standard — physics, not invention.

---

## C. Verify & harvest

- **Physical contradiction removed, not compromised?** Yes. High-rank-vs-low-rank is dissolved by the Reynolds
  split: the mean is resolved (low-rank), the fluctuation is carried only as moments (low-rank); the high-rank
  field is never built.
- **Only A5 resources?** Yes — LER stages, coefficient-MPOs, FAC, DLRA, `advance_scalar`, the Tier-A
  reacting-stage template. Nothing new.
- **Satisfies the function?** Yes for the **mean** `n_e` — exactly the blackout observable.

**New harm — the honest line (this is a *modeling* lever, not a solution to turbulence).**
- RANS is a **model**: it inherits eddy-viscosity-closure limits — strong separation, streamline curvature,
  non-equilibrium turbulence, and especially **transition prediction** are the shaky parts. For the reentry
  **forebody / near-wake** this is the accepted engineering standard, but it is a *commitment*, not exact.
  **[holds under precondition: RANS adequacy for this regime]**
- Closure coefficients need **compressible / hypersonic corrections** (dilatation-dissipation, compressibility)
  — calibrated, cite-able, not first-principles. **[holds under precondition: hypersonic closure corrections]**
- It yields **no instantaneous turbulent structure** — that stays genuinely high-rank and is now *correctly*
  out of scope, because the flagship never required it. **[open: instantaneous fine structure — never needed]**

**The residual collapses but does not vanish.** What remains is two much smaller, named, defensible things:
(i) instantaneous turbulent fine structure (out of scope by construction — not needed for `n_e`), and (ii)
RANS-closure modeling fidelity (the caveat all of hypersonic CFD lives with). That is categorically smaller than
"turbulence is unsolved."

**Generalized method.** *Carry only what the question integrates. When the required output is a statistic,
represent the field by its low-order moments (low-rank) and close the moment transport with the LER relaxation
pattern; never instantiate the high-rank realization.* This is the **fourth confinement axis**:

- **LER** — confine stiffness in **time** (relax to an equilibrium target).
- **FAC** — confine a singularity in **space** (move to a coordinate frame).
- **DLRA** — confine residual complexity in the **Schmidt basis** (rank-adaptive `K(t)`).
- **Moment closure** — confine turbulence in the **statistical moments** (carry mean + stress, not the field).

All four are one meta-move: **represent only the information the question needs, and relax/evolve that surrogate
with the cheap machinery.** Turbulence was never the residual for the *blackout* question — it was the residual
for a *DNS* question nobody asked.

**Inverse / scaling.** As turbulence intensity → 0 the closure degenerates to laminar FAC (the forebody, the
Tier-A regime); as the fidelity demand rises toward instantaneous structure it escalates RANS → URANS → hybrid
(DLRA `K(t)` budget) → LES → DNS, walking **off** the low-rank guarantee in a controlled, *observable* way. The
`K(t)` tripwire (Res 8) is exactly the dial that announces when RANS stops sufficing and hybrid is needed.

---

## What this changes for Gap 2 closure

1. **Turbulence stops being a hard hole in the domain** — the turbulent near-wake becomes a **modeled** region
   delivering a valid mean `n_e`, on the machinery already built; the marcher is *closed* over the whole
   blackout-relevant domain.
2. **The non-goal sharpens from "turbulence" to two named caveats** — instantaneous fine structure (not needed)
   and RANS fidelity (standard engineering caveat) — each defensible, neither blocking.
3. **Buildable on the committed architecture** — a turbulence-LER stage (`k`/`ω`/γ–Reθ) is the **same shape** as
   the reacting-LER stage already shipped in Tier-A; the eddy viscosity is a coefficient MPO; no new house.

Net: across the four resolutions the irreducible residual has gone from *"the live reentry flowfield may not be
low-rank at all"* to *"instantaneous turbulent fine structure (never needed) + RANS-model fidelity (standard
caveat),"* with `K(t)` marking the observable boundary where modeling must escalate.

---

## Verification gates (what a future spec/PR would prove)

1. **Mean closure works:** a turbulent near-wake shear case reproduces a reference **mean** profile (e.g. a
   canonical compressible mixing layer / RANS benchmark) at **bounded rank**, where a resolved attempt grows
   rank without bound. **[open until built]**
2. **Closure is an LER stage:** `k`/`ω` are advanced by the closed-form relaxation kernel (the Res-1 template),
   not an explicit stiff step; grep shows the shared stage shape.
3. **Eddy viscosity is a low-rank coefficient field:** `μ_t` enters as a bounded-bond coefficient MPO; it does
   not inflate the operator rank.
4. **Escalation is observable:** when `K(t)` (Res 8) exceeds budget, the marcher flags "RANS insufficient →
   hybrid" rather than silently degrading — the modeling boundary is detected.
5. **Degenerate limit:** turbulence intensity → 0 reproduces laminar FAC (the forebody) to round-off.

---

## Related

- [`gap-two-resolution-8-spectral-pinning.md`](gap-two-resolution-8-spectral-pinning.md) — the DLRA `K(t)`
  tripwire that decides RANS vs hybrid; the residual this addresses.
- [`gap-two-resolution-1-stiff-source.md`](gap-two-resolution-1-stiff-source.md) — LER, the stage template the
  `k`/`ω`/transition closure reuses verbatim in shape.
- [`gap-two-resolution-5-dynamic-rank-lever.md`](gap-two-resolution-5-dynamic-rank-lever.md) — FAC, which still
  pins the **mean** bow shock.
- [`gap-two-resolution-7-feature-adaptive-coordinate.md`](gap-two-resolution-7-feature-adaptive-coordinate.md) —
  the confinement family this completes (fourth axis).
- [`tier-b-compressible-marcher.md`](tier-b-compressible-marcher.md) — the wake / turbulence out-of-scope
  statement this reframes as a modeled region.
- `add-cfd-compressible-qtt-marcher/design.md` — the wake / turbulence non-goal and open question this updates.
- `deep_causality_cfd` — the Tier-A reacting-LER `PhysicsStage`s (template), `advance_scalar`; `deep_causality_tensor`
  — `tdvp` (hybrid escalation).

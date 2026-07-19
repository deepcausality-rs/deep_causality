<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Gap 2, Resolution 8 — multi-feature regions: spectral pinning (FAC + DLRA hybrid)

**What this is.** A TRIZ/ARIZ resolution of the **standing residual** left open by
[Resolution 5](gap-two-resolution-5-dynamic-rank-lever.md) and named in
[Resolution 7](gap-two-resolution-7-feature-adaptive-coordinate.md): the Feature-Adaptive Coordinate (FAC) pins
**one** dominant feature per structured map, so a region with **several** independently-moving sharp features —
the separated/unsteady **wake** (shock + contact + shear + vortices at once) — cannot be kept low-rank by any
single geometric map, and is excluded as out-of-scope (`qtt_rank_3d`). This note finds the lever: **stop trying
to geometrically pin what only needs *spectral* pinning.** It does **not** make turbulence low-rank (nothing
can), but it (a) extends low-rank closure from the forebody to the **transitional near-wake**, and (b) converts
the wake exclusion from an a-priori assumption into a **measurable runtime tripwire** `K(t)`.

It composes with, and does not replace, FAC ([Res 4](gap-two-resolution-4-body-fit-parameter.md)–[7](gap-two-resolution-7-feature-adaptive-coordinate.md)).

Honesty convention: **[holds]**, **[holds under precondition]**, **[open]**.

---

## 0. Frame

- **Key problem (no solution words):** represent a region holding several independently-moving sharp features at
  low tensor-train rank, when a single coordinate map can align only one.
- **System / main function:** the coordinate map + the TT field over the post-shock / wake region; *to keep every
  dominant feature aligned so the field stays low-rank.*
- **The constraint treated as fixed — the lever:** that there is **one global structured map** *and* that
  "align" must mean **geometric** alignment (grid lines parallel to the feature). The **second half is the deeper
  lever** — geometric alignment is only mandatory for a feature that is *infinite-rank if unaligned*.

---

## A. Reformulate (the ARIZ spine)

**A2 — Technical contradictions, both ways:**
- **TC-1:** one global map aligned to the strongest feature → that feature is `O(1)` rank, but every other
  feature is **oblique** → high rank.
- **TC-2:** enrich one Cartesian-ish frame to resolve all features → captures everything, but `√side` rank
  (back to capture — no QTT benefit).

**A3 — Intensify.** Push feature count `N → large`. One map cannot align `N` independent surfaces
**geometrically**. The extreme exposes two escapes: **(a)** many frames (one per feature — overset), or **(b)**
**stop requiring geometric frames at all** and align in a *different space*. (b) is the non-obvious one.

**A4 — Conflicting pair.** *Product:* the field in the multi-feature region. *Tool:* the coordinate map.
Conflicting states: the map must be simultaneously aligned to feature A **and** feature B — incompatible
orientations at the same point.

**A5 — Resources already present (inventoried aggressively, no new substance):**
- Each feature is **individually low-rank in its own frame** — the difficulty is only their *superposition* in
  one frame.
- A tensor train is a **Schmidt / sum representation**: `K` coherent structures cost **Schmidt rank ~K**,
  *independent of their geometric orientation*. `add` / `round` / `fit` (ALS) exist.
- **`tdvp` is already in `deep_causality_tensor`.** TDVP (time-dependent variational principle / projector-
  splitting) **is dynamical low-rank approximation (DLRA) for tensor trains** — it **evolves a rank-adaptive
  basis in time**. This is the literature already surveyed for Gap 2 (Einkemmer; Koellermeier–Krah–Kusch;
  Peng–McClarren–Frank; robustness to small singular values: Kieri–Lubich–Walach).
- `eigen` / DMRG → a flux-Jacobian characteristic decomposition is reachable.
- The **`MetricProvider` / `Coupling` cons-tuple** already composes "several things, each handled its own way,"
  by static dispatch — the seam to hang a second integrator on.

**A6 — Operating zone / time.** The conflict is **local to interaction zones** (shock–shear crossing, vortex
merging), *not the whole wake*. Almost everywhere, each neighborhood has **one** locally-dominant feature → a
partial separation-by-space is already available. In time the features move and their relative geometry changes
— which is exactly what a *time-evolving* basis (DLRA), not a *fixed* map, is built for.

**A7 — IFR + physical contradiction.**
- **IFR:** the region, using only the existing TT-sum / `tdvp` / `MetricProvider` machinery, stays low rank
  while every feature stays resolved — by representing it as a **superposition of low-rank components that no
  single map must align**.
- **Physical contradiction:** the map at a point must be aligned to feature A **and** to feature B (incompatible
  orientations). **Resolve by separation in *representation space*:** geometric alignment is mandatory only for
  the feature that is *infinite-rank if unaligned* — a true **discontinuity** (the shock). Every smooth-but-sharp
  structure (contact, shear, vortex) is aligned **spectrally**, in the SVD/Schmidt basis, where `K` structures
  cost rank `K` **regardless of orientation**.

→ Reformulation cracks it: **pin geometrically only the discontinuity; pin everything else spectrally.**

---

## B. Solve — the levers, strongest first

**Lever 1 (primary) — FAC + DLRA hybrid.** Keep **FAC** for the one feature that *must* be geometrically pinned:
the **shock** (a captured discontinuity is unbounded rank; fitting is non-negotiable there). Carry the
**post-shock multi-feature field as a rank-adaptive low-rank evolution via `tdvp`** (= DLRA). The impossible
question *"align `N` surfaces with one map"* becomes the measurable, **adaptive** one *"is the post-shock Schmidt
rank `K(t)` bounded?"* — the integrator grows/shrinks `K` itself. FAC removes the worst rank source
geometrically; DLRA absorbs the residual `K−1` coherent structures spectrally. Both already in the codebase; the
hybrid is a second bulk integrator behind the existing `Marcher` / `Coupling` seam.

```text
shock (discontinuity)   → FAC: geometric pin (Res 5), O(1) rank by construction
post-shock K structures → DLRA/tdvp: spectral pin, rank-adaptive K(t), measured each step
coupling                → existing StepContext / Coupling cons-tuple
```

**Lever 2 (engineering fallback) — overset / multi-patch fitted charts.** Tile the domain; each patch's
`MetricProvider` pins its locally-dominant feature; couple at overlaps (Chimera/overset). A known production-CFD
pattern (bounded risk), but **TT-overset interpolation between differently-coordinatized patches is unproven**
and adds boundary bookkeeping. Use if DLRA rank proves unbounded *but* features stay spatially separable.

**Lever 3 (complementary conditioner) — characteristic / wave-family separation.** The shock lives in the
acoustic characteristic field, the contact in the entropy field, vorticity in its own. A map pinning the shock
in the acoustic field does **not** fight the contact in the entropy field, so decompose by wave family (flux-
Jacobian `eigen` / DMRG) and pin each field's *own* dominant feature. *Honest limit:* globally-valid
characteristic variables do not exist for multi-D nonlinear systems — locally valid only, so this conditions
the hybrid, it does not replace it.

**Lever 4 (honest-scoping) — a deliberate low-rank wake closure.** The blackout driver is the forebody sheath;
the wake barely feeds back upstream. Replace the resolved multi-feature wake with a coarse low-rank
"sponge" / eddy-viscosity closure that **absorbs** it without resolving its features. Keeps the *global* domain
closed (no artificial truncation boundary) at the deliberate, already-accepted cost of wake fidelity.

> **TRIZ principles used:** **separation by scale / representation-space** (geometric for the discontinuity,
> spectral for the rest); **#1 Segmentation**; **#15 Dynamics** (evolving basis vs fixed map); **#23 Feedback**
> (rank adaptation); **#35 Parameter change** (carry the lag/structure as evolving rank); **#3 Local quality**
> (each feature handled where it lives). **Su-Field:** the incomplete element is the *basis* — supplied by
> TDVP. **Contradiction Matrix:** improving *adaptability/versatility* (35) vs worsening *device complexity*
> (36) → principles {1, 15, 35} — matching the found solution. **Effects database:** DLRA / projector-splitting
> is an established reduced-model integrator, not an invention.

---

## C. Verify & harvest

- **Physical contradiction removed, not compromised?** Yes. "One map can't align `N` features" is dissolved by
  geometrically aligning only the one feature that needs it (the discontinuity) and spectrally aligning the rest;
  Schmidt rank `K` is orientation-independent, so the incompatible-orientation conflict never arises for the
  smooth structures.
- **Only A5 resources?** Yes — `tdvp` (DLRA), `eigen`, `fit`, `round`, the FAC seam, the `Coupling` cons-tuple.
  Literature already cited for Gap 2. Nothing smuggled in.
- **Satisfies the IFR?** Yes for the transitional regime; the region is a superposition of low-rank components
  no single map must align.

**New harm — and the true residual (state it honestly).**
- DLRA has its own failure modes: small-singular-value stiffness (the projector-splitting integrator is
  *provably robust* to it — Kieri–Lubich–Walach — but the **rank-adaptation heuristic** still needs tuning).
  **[holds under precondition: robust integrator + tuned rank budget]**
- Coupling the fitted-discontinuity part to the DLRA bulk is a **new seam** to gate (same `StepContext` pattern,
  but unproven for this pairing). **[open: hybrid seam]**
- **The irreducible limit:** for a **fully turbulent far-wake**, `K` is **genuinely unbounded** — turbulence is
  high-rank almost by definition. DLRA does **not** make turbulence low-rank; it raises the ceiling from
  **1 feature → `K` coherent structures**, covering the **transitional near-wake** but **not** the turbulent
  far-wake. **[open: turbulent far-wake — the genuine non-goal]**

**Generalized method.** *Pin geometrically only what is infinite-rank-if-unaligned (discontinuities); carry
everything else with a rank-adaptive spectral basis (DLRA / TDVP), and **measure** `K(t)` rather than assume it.*
This is the third member of the family: LER confines stiffness in **time** (Res 1), FAC confines a singularity in
**space** (Res 5), spectral pinning confines the **remaining complexity in the Schmidt basis**. Each refuses to
fight the hard thing in its native representation.

**Inverse / scaling.** As `K → 1` the hybrid degenerates to pure FAC (the forebody); as `K → ∞` it degenerates to
DNS (the turbulence non-goal). Crucially it degrades **observably** — `K(t)` is monitored — so the model
announces where it stops being trustworthy instead of failing silently.

---

## What this changes for Gap 2 closure

1. **Larger validated domain** — closure extends from the forebody to the **transitional near-wake** (a handful
   of coherent post-shock structures), not just the single-feature sheath.
2. **The wake non-goal becomes a runtime tripwire** — instead of an a-priori exclusion, the marcher **monitors
   `K(t)`** and declares the region out-of-scope **when the adaptive rank grows past budget**. The
   `qtt_rank_3d` residual gains an *observable* boundary. **[upgrades the residual from assumed to measured]**
3. **Incremental on the committed architecture** — FAC is unchanged; DLRA is an **alternative bulk integrator**
   behind the same `Marcher` / `Coupling` seam, reusing `tdvp`. No new house.

Net: the genuinely-irreducible residual shrinks from *"any multi-feature region"* to *"the fully-turbulent
far-wake only,"* with a measurable `K(t)` tripwire marking where the model stops being trustworthy.

---

## Verification gates (what a future spec/PR would prove)

1. **Spectral pinning works on a known multi-feature case:** a post-shock shear/contact pair (no single aligning
   map) is carried by `tdvp` at bounded `K` where a fixed-map capture grows `√side`. **[open until built]**
2. **`K(t)` is a faithful tripwire:** under a transitional → turbulent ramp, `K(t)` stays bounded in the
   transitional regime and grows monotonically into the turbulent regime — the out-of-scope boundary is
   *detected*, not assumed.
3. **Hybrid seam conserves:** the FAC-fitted shock and the DLRA bulk couple without violating the conservation /
   positivity invariants (Res 6 / design D4) at the interface.
4. **Degenerate limits:** `K → 1` reproduces pure FAC (forebody) to round-off; turning DLRA off recovers the
   prior marcher exactly.

---

## Related

- [`gap-two-resolution-5-dynamic-rank-lever.md`](gap-two-resolution-5-dynamic-rank-lever.md) — FAC single-feature
  pinning; the residual this addresses.
- [`gap-two-resolution-7-feature-adaptive-coordinate.md`](gap-two-resolution-7-feature-adaptive-coordinate.md) —
  the FAC family this extends; spectral pinning is the third confinement axis (time / space / Schmidt basis).
- [`gap-two-resolution-1-stiff-source.md`](gap-two-resolution-1-stiff-source.md) — LER, the temporal confinement
  member of the same family.
- [`tier-b-compressible-marcher.md`](tier-b-compressible-marcher.md) — the rank evidence and the wake
  out-of-scope statement this gives a measurable boundary.
- `add-cfd-compressible-qtt-marcher/design.md` — the "multi-feature / wake" open question and the residuals
  list this updates.
- `deep_causality_tensor` — `tdvp` (DLRA / projector-splitting), `eigen` (characteristic decomposition), `fit`.

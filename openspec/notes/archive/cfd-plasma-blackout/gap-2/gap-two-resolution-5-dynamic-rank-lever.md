<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Gap 2, Resolution 5 — bounded rank by construction under marching (the dynamic lever)

**What this is.** A TRIZ/ARIZ resolution of the **make-or-break** Tier-B assumption: that the **measured**
low-rank lever survives **dynamic time-marching**. The rank studies
([`tier-b-compressible-marcher.md`](tier-b-compressible-marcher.md)) measured `χ ~ O(10)` for a body-fitted
shock — but **statically** (a re-coordinatized snapshot). The same studies measured that **nonlinear steepening
grows rank in time** (`qtt_rank_nonlinear`). So the open question is whether a fitted compressible **marcher** —
moving shock + per-step recompression — *keeps* the field low-rank as it evolves, or whether rank creeps up
step by step until QTT loses to a dense grid and the whole thesis collapses. This note makes bounded rank
**true by construction**, not hoped-for.

It is the dynamic facet of the **Feature-Adaptive Coordinate**
([Resolution 7](gap-two-resolution-7-feature-adaptive-coordinate.md)); it turns the static seam of
[Resolution 4](gap-two-resolution-4-body-fit-parameter.md) into a **feedback-updated** map, and it shares its
DNA with the temporal **LER** confinement of [Resolution 1](gap-two-resolution-1-stiff-source.md).

Honesty convention: **[holds]**, **[holds under precondition]**, **[open]**.

---

## 0. Frame

- **Key problem (no solution words):** prevent the tensor-train rank of the marched field from growing over
  time, even though nonlinear steepening and per-step operations inflate it.
- **System / main function:** the per-step pipeline `state → flux → apply → add → round → state`; *to keep the
  marched field's bond dimension bounded at `O(10)`, step over step, for all time.*
- **The constraint treated as fixed — the lever:** that rank is **whatever `round()` leaves** — an *output*,
  observed and hoped bounded. Make bounded rank an **enforced invariant**, an *input*.

---

## A. Reformulate (the ARIZ spine)

**A2 — Technical contradictions, both ways:**
- **TC-1:** round aggressively (loose tol) → bounded rank (good), but truncate real physics → **wrong shock
  speed** (bad).
- **TC-2:** round conservatively (tight tol) → accurate (good), but rank grows **unbounded** as the front
  steepens (bad).

**A3 — Intensify.** Let `t → ∞` with a captured steepening shock: rank grows without limit (the
`qtt_rank_nonlinear` study measured exactly this). Now pin the same shock to a coordinate line: in computational
space it is a **stationary one-axis step function** → rank `O(1)` *forever*. The extreme says it plainly — *move
the steepening into the metric, not the field.*

**A5 — Resources already present (no new substance):**
- The **shock location is observable every step** (the cell of maximum pressure gradient).
- The map is now a **`MetricProvider`** (Resolution 4) — it can be **re-evaluated each step** from live state.
- **Rusanov artificial viscosity is already present** — it sets a *thickness floor*, i.e. a rank cap, on any
  feature it touches (a resource, not only a cost).
- `max_bond` is observable — the invariant can be *checked* each step.

**A7 — Smart Little People.** Agents sit on the shock. If they hold still in *physical* space, the fixed grid
slides past them and the front steepens against the cells — rank grows. Instead each agent **grabs the nearest
coordinate line and drags it along** as it moves. In *computational* space the front never moves and never
steepens; the steepening is absorbed by the moving grid, not stored in the field.

**Physical contradiction:** in **physical** space the shock steepens and moves (high, growing rank); in
**computational** space it must stay resolved and put (constant, low rank). Same field, two frames. **Resolve by
separation in space** — the choice of frame.

→ Reformulation cracks it, and unifies with shock-fitting.

---

## B. Solve — the feature-pinned (feedback) coordinate

**The map must move with the feature.** Make the fitted interface a **feedback-updated `MetricProvider`**:
re-pin its `η = const` surface to the live shock location every step.

```text
each step:  locate front (max |∇p|)  →  update map params  →  rebuild metric MPOs (low-rank)  →  march
```

Then the discontinuity is **coordinate-stationary** → `O(1)` rank on its axis **for all time**. Rank is held
**by construction**, not by rounding. The recurring theme, stated loudly: **rank is controlled by the
coordinate, not by the rounder** — `round()` is only a backstop.

The Rusanov dissipation you already need for stability sets a **thickness floor** on the *un-pinned* features
(the contact, weak compressions) → it caps *their* rank too. One mechanism, two jobs.

> **TRIZ principles used:** **separation in space** (the frame); **#15 Dynamics** (rigid map → adjustable);
> **#23 Feedback** (re-pin from live state); **#22 Blessing in disguise** (the stabilizing viscosity doubles as
> the un-pinned-feature rank cap); **#6 Universality** (one moving map controls rank, alignment, and
> conditioning). **Effects database:** shock-fitting / front-tracking is classical; the novelty is only its use
> as a *rank* controller on a TT bulk.

---

## C. Verify & harvest

- **Physical contradiction removed, not compromised?** Yes. The two frames coexist by space separation; there
  is no accuracy-for-rank rounding trade — the front stays sharp *and* low-rank.
- **Only A5 resources?** Yes. Live front location, the re-evaluable `MetricProvider`, the existing Rusanov
  floor. No new substance.
- **Makes low-rank true by construction?** Yes — a coordinate-stationary feature cannot grow rank, *independent
  of marching time*. This is exactly what the make-or-break demanded.

**New harm — and the honest scope line.** A **single** dominant feature (the bow shock) can be pinned exactly.
**Multiple interacting unsteady features** (shock + contact + shear layer + separation) **cannot all be pinned
by one structured map.** The forebody sheath has *one* dominant discontinuity plus smooth relaxation behind it →
single-feature pinning suffices → **gateable**. The **wake** has many unsteady features → not pinnable by one
map → **genuinely open** (the `qtt_rank_3d` residual; needs multi-patch/overset or accepts the turbulence
non-goal). This mechanism *explains* why the flagship target is gateable and the wake is not. **[open:
multi-feature pinning]**

**Generalized method.** *Pin the dominant singularity to a computational coordinate via a feedback-updated map
so it is coordinate-stationary and therefore constant low rank, independent of marching time; the rounder is a
backstop, not the mechanism.* This is the spatial dual of LER: LER confines a stiff source in **time** (closed-
form relaxation toward a state-derived target); this confines a sharp feature in **space** (a moving coordinate
surface).

**Inverse / scaling.** As feature count → 1 the method is exact (the forebody); as feature count → ∞ and
unsteady (the wake) it degrades gracefully to capture (`√side`) — the honest scope boundary, not a silent
failure.

---

## Measured (study `qtt_rank_fitted_dynamic`, 2026-06-30)

The dynamic lever was probed before the build, the sequel to `qtt_rank_nonlinear`. Three marched cases at two
resolutions each:

- **Axis-aligned front:** bond holds `7` at both `64²` and `128²` (flat in resolution). When the feature stays
  on a grid axis throughout the march, the bond is bounded by construction. This is the lever, confirmed
  dynamically. **[holds]**
- **Misaligned curved shock:** bond grows `20 → 25` with resolution, reproducing the `√side` threat under a
  marcher. **[holds]**
- **Static body-fitted coordinate (set once):** bond grows `25 → 35`, no better than the capture. Under
  Cartesian fluxes the marched front drifts off a *fixed* curvilinear chart and the rank climbs. **A one-time
  fitted coordinate does not self-bound.** **[measured: static fit insufficient]**

So the study **sharpens the design**: alignment bounds the rank, but holding alignment as the front moves
requires the **feedback re-pinning of B/D9, not a static chart**. Re-pinning is mandatory, not a refinement.

A follow-up study (`qtt_repin_marcher`) prototyped the re-pinned marcher and sharpened it further, with one
honest negative result and one positive:

- **Re-pinning the coordinate *alone* does not curb the growth.** A re-pinned Cartesian-flux march still grows
  `25 → 35` (18 re-pins fired at `128²`), identical to the static chart. The rank driver is **not** the front's
  drift; it is the **angular structure a Cartesian-flux march injects by carrying fluxes *through* a curved
  front**. **[measured: re-pin necessary, not sufficient]**
- **Aligning the transport with the coordinate bounds it.** A radial transport on the *same* re-pinned tracked
  interface — the front carried as an aligned interface, no flux marched across it — holds the bond at `8`, flat
  in resolution. **[holds: coordinate-aligned tracked transport is `O(1)` and resolution-flat]**

So the Stage-4 mechanism is **re-pin *and* treat the front as an exact Rankine–Hugoniot interface** (smooth each
side), never marching Cartesian fluxes across it. The fitting that controls the rank and the RH jump that handles
the discontinuity are the same act; a plain marcher in a good-but-static (or even re-pinned) coordinate is not.
This is the C7 "dissolved by fitting" line ([tier-b note](tier-b-compressible-marcher.md) §4), now measured.

## Verification gates (what a spec/PR must prove)

1. **Rank-vs-time:** a **re-pinned** fitted marcher (the map updated to the live front each step) holds
   `max_bond` bounded over a long run, while both the Cartesian control *and a static fitted chart* grow with
   resolution — the **dynamic** version of the static study, the make-or-break gate. (`qtt_rank_fitted_dynamic`
   established the static-chart growth and the aligned-bounded baseline; the re-pinned case is the Stage-4
   gate.) **[holds under precondition: single dominant feature + active re-pinning]**
2. **Correct shock speed:** conservation-preserving rounding (design D4) keeps the pinned front propagating at
   the exact Rankine–Hugoniot speed (no rounding-induced drift).
3. **Pin tracks the front:** the `η = const` interface follows the live max-gradient cell within tolerance each
   step (the feedback loop is stable, not chattering).
4. **Un-pinned features capped:** the contact's bond stays bounded under the Rusanov thickness floor.

---

## Related

- [`tier-b-compressible-marcher.md`](tier-b-compressible-marcher.md) §4 — the static measurement this makes
  dynamic; the `qtt_rank_nonlinear` time-growth result this defeats.
- [`gap-two-resolution-4-body-fit-parameter.md`](gap-two-resolution-4-body-fit-parameter.md) — the static
  `MetricProvider` seam this feedback-updates.
- [`gap-two-resolution-6-implicit-acoustics.md`](gap-two-resolution-6-implicit-acoustics.md) — fitting here also
  removes the worst coefficient jump from the interior acoustic operator there.
- [`gap-two-resolution-1-stiff-source.md`](gap-two-resolution-1-stiff-source.md) — the **temporal** dual (LER
  confinement) of this spatial confinement.
- `add-cfd-compressible-qtt-marcher/design.md` — D1 (fitting), D9 (this), Stage 4.

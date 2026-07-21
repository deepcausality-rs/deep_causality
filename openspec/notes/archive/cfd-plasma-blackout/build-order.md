<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Plasma-Blackout Corridor — build order (contract-first, staged)

**What this is.** The single, dependency-ordered roadmap from the post-Gap-2 state to a buildable
[plasma-blackout-corridor flagship](plasma-blackout-corridor.md). It **supersedes the scattered ordering hints**
in [`gap-analysis.md`](gap-analysis.md) §5, [`gap-3/gap-three-resolution-1-...md`](gap-3/gap-three-resolution-1-perturbed-conformal-trajectory.md)
(the "revisit after Tier-B Stage 4+" scheduling), and [`gap-3/gap-three-resolution-3-...md`](gap-3/gap-three-resolution-3-trajectory-axis.md)
Part D (the "Phase 1 now / Phase 2 later" split). The specification that implements this order is the OpenSpec
change **`add-plasma-blackout-corridor`**.

---

## The principle: contract-first, not feature-first

The waste in the prior plans is not "placeholder now, real later." It is a placeholder the design is **coupled
to**:

- A mock **behind a stable interface** (a trait, a closure, a typed seam) is a **one-line swap** — the consumer
  never changes.
- A mock the design is **shaped around** (a hardcoded blackout schedule other stages branch on; a
  Cartesian-capture assumption baked into the solver; a recovery-temperature reconstruction downstream stages
  read directly) is what forces a rewrite.

So do two things **first**, before anything builds on them:

1. **Promote-first** — lift the proven feasibility-study / example primitives into libraries.
2. **Contract-first** — define the real seams so every remaining mock is a swappable stub.

The single linchpin is the **blackout coupling interface** (gap item ④): the per-step force / heat / `n_e` /
blackout-flag contract the marcher *produces* and the trajectory engine + regime classifier *consume*. Build it
first (with a stub behind it), and the old "mock aero → rewrite" and "Cartesian capture → body-fitted rewrite"
disappear.

## The six stages (dependency order)

```
Stage 0  Foundations & contracts
  ├─ blackout-coupling-interface (④)   EXTEND the existing .couple seam (CoupledField/PhysicsStage) + stub stage
  ├─ ks-conformal-propagator (B1)      promote FS-1 (3-D KS + Strang hook) → deep_causality_physics
  ├─ sp2r-constraint-projection (B2)   promote → deep_causality_physics
  └─ forward clock kernel (B3)         ALREADY SHIPPED — consumed (two-clock s ≠ τ)
        ▼
Stage 1  CFD real-fidelity (fills ④ with real data)
  ├─ 3-D body-fitted MetricProvider    curved-shock rank O(10) not √side  (Gap-2 remainder)
  ├─ dynamic marched-rank re-pin       Res 5 / D9
  └─ blackout-marcher-coupling         marcher emits ④ via CfdFlow → swaps the Stage-0 stub
        ▼   (Stage 2 builds against the ④ STUB in parallel with Stage 1)
Stage 2  Trajectory/nav engine (built once, against ④)
  └─ trajectory-nav-engine             KS + 17-state ESKF + projection + two-clock + Encke↔Cowell switch
        ▼
Stage 3  Composition (fills the Stage-0 seams)
  └─ blackout-composition              classifier + continue_with branches + cybernetic bounded-correction + provenance
        ▼
Stage 4  CFD Flow DSL (re)design
  └─ blackout-flow-dsl                 compose the per-step coupling stack (loop body); run_coupled iterates; ~10–30 LOC
        ▼
Stage 5  Flagship
  └─ plasma-blackout-flagship          corridor §4 chain [1]–[7] written in the DSL, coupled validation gate
```

## What each stage replaces (the rebuilds this avoids)

| Prior-plan placeholder | Reordered fix | Stage |
|---|---|---|
| Gap-3 Phase-1 mock aero → Phase-2 real (engine rewrite) | build the engine against ④; mock is a stub behind the contract | 0 + 2 |
| Gap-3 Phase-1 mock blackout schedule → real Park-2T trigger | ④ carries the blackout flag; marcher fills it in Stage 1 | 0 + 1 |
| 3-D marcher Cartesian-capture → body-fitted rewrite | build the body-fitted `MetricProvider` before any 3-D run | 1 |
| Tier-A recovery-temperature reconstruction read downstream | Stage-1 marcher transports real `T_tr`/`T_ve`; chemistry reads those | 1 |
| FS study code in `cfd/studies/` reused ad hoc | promote to `deep_causality_physics` kernels once | 0 |

## The Stage-3 corrective gate: cybernetic loop, not Effect Ethos

The committed corrective safety gate ([6]) is `deep_causality_haft::CyberneticLoop::control_step`, **not** the
Effect Ethos layer. It is a deterministic sense→believe→decide→act step (`observe_fn: S×&C→B`,
`decide_fn: B×&C→A`) that compiles to tight machine code with no monadic allocation on the hot path — suited to
the **latency-bound guidance inner loop**, the one place the Effect monad's overhead is unsuitable. The loop's
Context `C` *is* the **verified safety envelope** (thermal corridor, g-load, physiological / ROE limits), so the
returned Action `A` is bounded into the envelope **by construction** (clamp, or return the Entropy `E` on an
unrecoverable breach) — a type-level bounded-correction guarantee, not a post-hoc rule check. Effect Ethos stays
for non-real-time deontic checks (mission-rule verification, offline audit).

Component map: **S** sensed coupled state (heat/g-load/miss) · **B** trajectory/thermal-margin belief · **C**
safety envelope · **A** bounded bank-angle correction · **E** unrecoverable-breach signal.

## Parallelism and the one honest serialization

Contract-first parallelizes *construction*: once ④ exists (stub behind it), Stage 2's engine builds and
unit-validates against the stub **while** Stage 1 matures the marcher; wiring the real output in is a stub swap.
But it does **not** parallelize *coupled validation*: the coupled gate — real `n_e` → real blackout window →
real INS drift → reacquisition — is a Stage-5 milestone, because it needs Stage 1's marcher behind ④. That is a
real constraint, not a placeholder.

**Stage 4 is a design stage — grounded in the current `CfdFlow`.** The existing Flow is a config→run split whose
`run_coupled` *is* the control loop, extended by a static cons-tuple of `PhysicsStage`s on a shared
`CoupledField` (no `dyn`), with counterfactuals as `seed_with`/`march_with` overrides. So the redesign is **not**
a new linear phase pipeline: it composes the flagship's per-step **loop body** as a coupling stack
(`Coupling::between_steps().then(RegimeClassify).then(TrajectoryNav).then(CyberneticCorrect)`) that `run_coupled`
iterates. The cybernetic gate rides the existing `?` short-circuit (a `PhysicsStage` that clamps the action into
the envelope and returns `Err(Entropy)` on a breach — no new primitive). The central control loop is the stack
plus one `run_coupled` call: ~10–30 lines. A preliminary design ships now in the change's `design.md`; minor
revision at Stage 4 is expected.

## Related

- [`plasma-blackout-corridor.md`](plasma-blackout-corridor.md) — the flagship this order builds toward.
- `openspec/changes/add-plasma-blackout-corridor/` — the specification implementing this order (proposal /
  design / tasks / seven capability specs).
- [`gap-analysis.md`](gap-analysis.md) — the gap tracker (Gap 2 closed; Gaps 3/4 sequenced here).
- [`gap-3/`](gap-3/) — the trajectory/timing resolutions + the three passed feasibility studies (FS-1/2/3).
- [`tensor-network/ACCELERATION-SOTA-FIRST.md`](../../tensor-network/ACCELERATION-SOTA-FIRST.md) — GPU/parallel
  acceleration is gated behind that survey, out of this build order.

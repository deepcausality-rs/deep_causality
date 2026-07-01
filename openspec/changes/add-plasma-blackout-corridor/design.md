# Design — Plasma-Blackout Corridor, reordered contract-first build

## Context

This change is the reordered successor to the scattered post-Gap-2 plans (gap-analysis §5;
[resolution 1](../../notes/plasma-blackout/gap-3/gap-three-resolution-1-perturbed-conformal-trajectory.md);
[resolution 3](../../notes/plasma-blackout/gap-3/gap-three-resolution-3-trajectory-axis.md) Part D;
[chemistry-fidelity-gap.md](../../notes/plasma-blackout/gap-3/chemistry-fidelity-gap.md)). Those notes decided
the *physics*; this change decides the *build order* so each real component is built once. The roadmap view —
the principle, the DAG, and what it supersedes — lives in
[`build-order.md`](../../notes/plasma-blackout/build-order.md).

## The reordering principle: contract-first, not feature-first

The waste in the prior plans is not "placeholder now, real later." It is a placeholder the design is **coupled
to**:

- A mock **behind a stable interface** (a trait, a closure, a typed seam) is a **one-line swap** — the consumer
  is unchanged. Example: an aero-force closure whose body returns mock drag today and real marcher force
  tomorrow.
- A mock the design is **shaped around** (a hardcoded blackout schedule other stages branch on; a
  Cartesian-capture assumption baked into the solver; a recovery-temperature reconstruction downstream stages
  read directly) is what forces a rewrite.

So the reorder rule is two moves, done **first**:

1. **Promote-first** — lift the proven FS/example primitives into libraries before anything builds on them.
2. **Contract-first** — define the real seams before their consumers, so every remaining mock is a swappable
   stub.

Do both and the build proceeds bottom-up with no rebuilds.

## The dependency DAG (why the stages are ordered as they are)

```
Stage 0  Foundations & contracts
  ├─ blackout-coupling-interface (④)   ── EXTEND the existing .couple seam (CoupledField/PhysicsStage/Trigger)
  ├─ ks-conformal-propagator (B1)      ── promoted FS-1 (3-D KS + hook)
  ├─ sp2r-constraint-projection (B2)   ── promoted FS-2/B2
  └─ (forward clock kernel, B3)        ── ALREADY SHIPPED, consumed
        │
        ▼
Stage 1  CFD real-fidelity (fills ④ with real data)
  ├─ 3-D body-fitted MetricProvider    ── removes Cartesian-capture rank placeholder (χ√side → O(10))
  ├─ dynamic marched-rank re-pin       ── Res 5 / D9
  └─ blackout-marcher-coupling         ── marcher emits ④ via CfdFlow
        │
        ▼                                        (Stage 2 can build against the ④ STUB in parallel with Stage 1)
Stage 2  Trajectory/nav engine (built once, against ④)
  └─ trajectory-nav-engine             ── KS + 17-state ESKF + projection + two-clock + Encke↔Cowell switch
        │
        ▼
Stage 3  Composition (fills the Stage-0 seams)
  └─ blackout-composition              ── classifier + continue_with branches + cybernetic bounded-correction + provenance
        │
        ▼
Stage 4  CFD Flow DSL (re)design           ── compose the per-step coupling stack (the loop body)
  └─ blackout-flow-dsl                 ── evolve .couple/run_coupled; control loop in ~10–30 LOC (preliminary design now)
        │
        ▼
Stage 5  Flagship
  └─ plasma-blackout-flagship          ── corridor §4 chain [1]–[7] *in the DSL*, coupled validation gate
```

**Parallelism.** Contract-first buys parallelism for *construction*: once the Stage-0 ④ interface exists (with a
stub behind it), Stage 2 can be built and unit-validated against the stub **while** Stage 1 matures the marcher.
Wiring Stage 1's real output into Stage 2 is a stub swap, not an engine change.

## What each stage replaces (the rebuilds this avoids)

| Prior-plan placeholder | Reordered fix | Stage |
|---|---|---|
| Gap-3 Phase-1 mock aero kick → Phase-2 real (engine rewrite) | build the engine against ④; mock is a stub behind the contract | 0 + 2 |
| Gap-3 Phase-1 mock blackout schedule → real Park-2T trigger | ④ carries the blackout flag; marcher fills it in Stage 1 | 0 + 1 |
| 3-D marcher Cartesian-capture → body-fitted rewrite | build the body-fitted `MetricProvider` before any 3-D run | 1 |
| Tier-A recovery-temperature reconstruction read downstream | Stage-1 marcher transports real `T_tr`/`T_ve`; chemistry reads those | 1 |
| FS study code in `cfd/studies/` reused ad hoc | promote to `deep_causality_physics` kernels once | 0 |

## Stage 0 — Foundations & contracts

**`blackout-coupling-interface` (the linchpin — an extension of the existing seam, not a greenfield contract).**
The ④ seam already largely exists in `deep_causality_cfd` (`types/flow/coupling.rs`): `CoupledField<R>` (the
owned per-step auxiliary state — named scalar fields + the `Ambient` the marcher reads), the `PhysicsStage<D,R>`
trait (one between-step transform; impl'd for `()` and `(Head, Tail)` so stages compose as a static cons-tuple
with **no `dyn`**), `StepContext` (dt, step, the D8 DEC/QTT backing), and `BlackoutTrigger` (peak `n_e` → denial
decision). Stage 0 **extends** it: add an **aero-force (Cartesian vector) channel** the trajectory kick reads and
a **control/action channel** the correction writes, plus the classifier-input contract (Knudsen, ionization
fraction, GNSS state) and the provenance/`EffectLog` schema. A **stub** `PhysicsStage` (mock drag + a scheduled
blackout) satisfies the contract so Stage 2 builds immediately; Stage 1 replaces it with the real marcher stage.

**`ks-conformal-propagator` (B1).** Promote FS-1 from planar (already shipped as `TwoBodyPropagator`) to the 3-D
KS regularization (Stiefel & Scheifele 1971): 3-D Kepler lifts to a 4-D harmonic oscillator in the fictitious
time `s` (`dt = r·ds`), advanced by a **constant** generator `ψ(s)=e^{Ωs}ψ(0)`, singularity-free and
perturbation-ready. The between-step **perturbation hook** applies a caller-supplied Cartesian acceleration as a
2nd-order Strang kick (FS-2) — its closure reads the ④ force channel. The heavier Bars `(4,2)` packaging is
optional (FS-1) and not built.

**`sp2r-constraint-projection` (B2).** Promote the projection onto the KS/Sp(2,R) bilinear surface (the
Leray/Hodge analogue), with a documented fixed gauge for uniqueness.

**B3 clock — consumed, not built.** The shipped `relativistic_clock_drift_rate_kernel` is wired for the
two-clock carry: the KS fictitious time `s` is **not** proper time `τ` (the FS-3 correction). Metric from state;
only `G`, `c`, EGM/IERS literal.

## Stage 1 — CFD real-fidelity

Completes the compressible-marcher's named open remainders (gap-analysis Gap 2): the **3-D body-fitted
`MetricProvider`** (the 3-D marcher is Cartesian-capture today, so a curved shock costs χ~√side; body-fitted
holds it O(10) — mandatory, not optional) and the **dynamic marched-rank re-pin** (Res 5/D9). Then
**`blackout-marcher-coupling`**: the marcher emits the extended ④ channels (real aero force + heat + `n_e` +
blackout flag) into the `CoupledField` each step, replacing the Stage-0 stub stage. Chemistry lever 3
(finite-rate ionization network) is optional here — lever 1 already lands peak `n_e` at ~1.1× of RAM-C, so
lever 3 only firms the band and is **not** a flagship blocker.

## Stage 2 — Trajectory/nav engine (built once)

The engine: predict = KS propagate + the ④ force via the Strang hook; correct = 17-state tightly-coupled ESKF
update + the B2 projection; carry the two clocks. Sensors (⑥): strapdown IMU (primary, through blackout),
through-plasma optical (~50 m 1σ aid), GNSS (denied when the ④ blackout flag is set), the carried clock. The
**Encke↔Cowell `select_integrator` switch** (B4, the `grmhd/select_metric` pattern) reads ε = a_aero/a_grav from
the ④ force; it is part of this stage, not a later phase, because the force arrives through the contract from day
one (stub ε in Stage 0/2, real ε once Stage 1 lands). No mock/real split. The engine's per-step step is packaged
as a `PhysicsStage` (`TrajectoryNav`) so it composes into the Stage-4 coupling stack; its nav state threads
through the `CoupledField` across steps.

The ESKF stays example-level unless a second consumer appears (YAGNI); the reusable math (KS propagator,
projection) is the promoted Stage-0 library.

## Stage 3 — Composition

The regime classifier (Knudsen + `n_e` + GNSS state → governing-model selection, the corridor §4 [2]/[3]
coupling), the counterfactual bank-angle branches (`continue_with`, [5]), the **cybernetic bounded-correction
gate** ([6]), and the provenance log ([7]) — each realized as a `PhysicsStage` (or the existing counterfactual
override), filling the existing seam rather than introducing a new one.

**Deviation from corridor §4 [6]: cybernetic loop, not Effect Ethos.** The corrective safety gate is
`deep_causality_haft::CyberneticLoop::control_step`, not the Effect Ethos layer. Three reasons:

1. **Real-time / low overhead.** The Effect monad carries logging/short-circuit/counterfactual machinery that
   compiles to non-trivial overhead — fine for the macroscope (the `continue_with` exploration at [5]), but the
   *committed corrective inner loop* is latency-bound (corridor §6). The cybernetic loop is a plain
   sense→believe→decide→act step (`observe_fn: S×&C→B`, `decide_fn: B×&C→A`) that compiles to tight machine
   code with no monadic allocation on the hot path — the one place Effect Ethos is not suitable.
2. **Bounded correction is structural.** The loop's Context `C` *is* the verified safety envelope (thermal
   corridor, g-load, physiological / ROE limits); `decide_fn` clamps the Action `A` into `C` by construction, and
   returns the Entropy `E` when no safe action exists. So "the correction is always inside a verified safety
   envelope" is a type-level guarantee of the gate, not a post-hoc check that a rule *happened* to reject — the
   difference between the causal-correction `intervene` (snap back) and a *bounded* intervene.
3. **Determinism.** Identical (Sensor, Context) → identical Action, which is what a certifiable guidance gate and
   a "black-box"-auditable reentry decision need.

Effect Ethos remains the right tool for **non-real-time deontic checks** (mission-rule verification, offline
audit); it is simply not the flagship's latency-bound corrective gate. Mapping: **S** = sensed coupled state
(heat/g-load/miss from the rollouts) · **B** = estimated trajectory/thermal margin · **C** = the safety envelope
· **A** = the bounded bank-angle correction · **E** = unrecoverable-breach signal. The gate depends on
`deep_causality_haft` (Tier 0), already in scope.

## Stage 4 — CFD Flow DSL (re)design (with a preliminary design)

**Grounded in the current design (read first, not guessed).** The present `CfdFlow`
(`types/flow/cfd_flow.rs`, `qtt_march_run.rs`, `coupling.rs`) is **not** a linear phase pipeline. It is:

- a **config → run split**: `CfdFlow::qtt_march(&config)` borrows a `QttMarchConfig` and yields a runnable
  `QttMarchRun`; the terminal **`.run_coupled(coupling, initial, trigger, kappa)` is the control loop** —
  `for step { advance; publish projections into CoupledField; transport carried scalar; coupling.apply(ctx, &mut
  field); sample observables }`;
- extended through the **`.couple` seam**: `Coupling::between_steps().then(stageA).then(stageB).build()` — the
  static cons-tuple of `PhysicsStage`s described in Stage 0, **no `dyn`**, errors short-circuit via `?`;
- with **counterfactuals** as the `seed_with` / `march_with` / `observe_with` overrides that spawn alternate runs
  from the same borrowed config, and `BlackoutTrigger` mapping peak `n_e` → the denial decision.

So the redesign is **not** a new `.a().b().c()` phase chain — an earlier sketch that did so was wrong: it
flattened the *loop body* into one-shot top-level *phases*. The correct move is to **compose the per-step
coupling stack** (the loop body) and let `run_coupled` iterate it. Two facts make this small and elegant:

1. **The ④ interface already exists** (Stage 0 extends it), so there is no new seam to invent — the trajectory,
   classifier, and correction are all `PhysicsStage`s on `CoupledField`.
2. **The cybernetic gate rides the existing short-circuit.** `PhysicsStage::apply -> Result<(), _>`; an `Err`
   short-circuits the whole coupling — exactly "return Entropy `E`, emit no unsafe action." So the
   bounded-correction gate is a `PhysicsStage` that clamps the action into the envelope (mutating the field) and
   returns `Err(Entropy)` on an unrecoverable breach, wrapping a direct `control_step` (no Effect-monad
   allocation). No new DSL primitive.

**Design goals:** elegance, concise expressiveness, very low overhead over the underlying mechanism. The
combinators stay a *naming* layer over the existing static cons-tuple composition (no `dyn`, monomorphized), so
the DSL compiles to the same code as the hand-written `Coupling`/`run_coupled`. Target: the flagship's **central
control loop reads in ~10–30 lines** — attainable because the loop already lives in `run_coupled`, so the
flagship code is just *composing the stage stack + one run call*.

**Preliminary design (expect minor revision at Stage 4).** Compose the loop body as a `PhysicsStage` stack, then
run:

```rust
// The per-step coupling stack IS the loop body (static cons-tuple, no dyn).
let coupling = Coupling::between_steps()
    .then(RegimeClassify::new(knudsen_cfg))   // [2] Knudsen + n_e + GNSS -> select governing model
    .then(ReactingIonization::ler(park2t))    // [3] transported n_e into the field (existing LER stages)
    .then(TrajectoryNav::new(ks, eskf))       // [4] KS predict + ESKF correct; reads aero force, GNSS gated by flag
    .then(CyberneticCorrect::new(envelope))   // [6] bounded bank-angle action; Err(Entropy) on breach
    .build();

// run_coupled IS the control loop: advance -> couple(stack) -> sample, each step, to the terminal stop.
let report = CfdFlow::qtt_march(&cfg)
    .run_coupled(coupling, initial_field, blackout_trigger, scalar_kappa)?;   // [1][4][7]

// [5] counterfactual bank angles: alternate runs from the same borrowed config (existing override mechanism).
let branches = bank_angles.iter().map(|theta|
    CfdFlow::qtt_march(&cfg).seed_with(seed_for(theta))
        .run_coupled(coupling_for(theta), init.clone(), trigger, kappa));
```

What Stage 4 actually adds (small, on the existing seam):

| Addition | Where | Corridor step |
|---|---|---|
| aero-force + control/action channels on `CoupledField` (or `Ambient`) | extend `coupling.rs` types | [3] ④ extension |
| `RegimeClassify`, `TrajectoryNav`, `CyberneticCorrect` as `PhysicsStage` impls | new stages | [2][4][6] |
| `CyberneticCorrect` = clamp-into-envelope + `Err(Entropy)` short-circuit (wraps `control_step`) | new stage | [6] |
| provenance emitted from the loop into `EffectLog` | extend `run_coupled` reporting | [7] |
| optional thin convenience over `qtt_march`/`run_coupled` if it improves readability | `cfd_flow.rs` | — |

Reconciliation: this **evolves** the existing `fluiddynamics-dsl` / `qtt-flow` capabilities and the `.couple`
seam (it does not re-specify them here); it keeps the config→run split, the cons-tuple `.then()` composition,
and the counterfactual overrides. The redesign is *what the loop body composes*, not a new pipeline shape.

### Counterfactual alternation (value / context / state)

A CFD counterfactual is usually **not** a Pearl `do()` on one value — it is "run the same solver law in a
different **world**, or from a different **condition**." So the flagship's counterfactuals ride the full
`deep_causality_core` `Alternatable<V, C, S>` family, **not** just `intervene` (which is `alternate_value` under
a causal-inference name). The core vocabulary is kept **verbatim and loud** — `alternate_context`,
`alternate_state`, `alternate_value` appear at every call site — so it is explicit that a run simulates an
alternate reality.

CFD channel mapping:

| Channel | In `CfdFlow` terms | Alternation means |
|---|---|---|
| **context** `C` | the **world**: a whole `QttMarchConfig` + `Ambient` / BCs / atmosphere / bank schedule / envelope | "same solver, different reality" |
| **state** `S` | the **marching state**: fluid trains + carried scalars + nav state + clocks | "same world, different starting point / mid-flight perturbation" |
| **value** `V` | an injected primary-state snapshot | the `intervene` analog (rare in CFD) |

**Two attach points, same three verbs.**

- *Pre-run* (subsumes `seed_with` / `march_with`): alternate before marching.
  ```rust
  let factual = CfdFlow::qtt_march(&config::nominal_reentry())
      .run_coupled(coupling, init, trigger, kappa)?;
  let steep = CfdFlow::qtt_march(&config::nominal_reentry())
      .alternate_context(&config::steep_reentry())          // !!ContextAlternation!!: nominal -> steep
      .run_coupled(coupling, init.clone(), trigger, kappa)?;
  ```
- *Mid-march* (a resumable, forkable loop — corridor [5] bank angles forked from one shared branch state):
  ```rust
  let onset = CfdFlow::qtt_march(&cfg)
      .run_until(coupling, init, trigger, kappa, StepView::at_blackout_onset)?;   // -> MarchPause
  let branches = worlds.iter().map(|w|
      onset.fork()                     // O(1): clone the Arcs (state/field/context) + log
          .alternate_context(w)        // !!ContextAlternation!! at the branch step
          .continue_march(remaining)   // rebuild solver from w, resume from the shared branch state
  ).collect::<Result<Vec<_>, _>>()?;
  ```

**State is `Arc` + copy-on-write.** The threaded marching state is `Arc`-wrapped so `fork`/`alternate_state`
share by reference in O(1); a stage that *writes* the state triggers the clone via `Arc::make_mut` — cost paid
only when a fork actually diverges. A read-only fork (or a stage that only reads `n_e`) copies nothing. This is
the "read → share, write → CoW" rule applied to the tensor-train fields.

**Whole named configs, not deltas.** Context alternation swaps a **whole** `QttMarchConfig`; each alternate world
is a checked-in named constructor (`config::nominal_reentry()`, `config::steep_reentry()`,
`config::dense_atmosphere()`, …), so a call site names exactly which reality it simulates and the world is
diffable in the repo. No config-delta DSL — the config is small enough that a handful of full worlds is clearer.

**Contract inheritance (the reproducibility payoff).** Alternation inherits the `Alternatable` contract: the
**error channel is never alternated** (a diverged run cannot be repaired by swapping its world), and every swap
appends an explicit `EffectLog` entry. So each branch's log self-documents its world and starting condition —
a counterfactual *study*, not a pile of ad-hoc runs.

**The one real cost.** Pre-run alternation is thin combinators on `QttMarchRun`. Mid-march fork needs the
**resumable loop** — `run_until(predicate) -> MarchPause`, `fork()`, `continue_march(steps)` — a genuine change
to `run_coupled`'s control structure (it currently runs to completion), plus the `Arc`-wrapping of the threaded
state. Bounded, but real; both attach points are in scope for Stage 4.

## Stage 5 — Flagship

The corridor §4 chain [1]–[7] as one auditable `CausalFlow`, **written in the Stage-4 DSL** (the coupling stack
+ `run_coupled` above), driven over the RAM-C trajectory, with a self-verifying gate. This is where the
**coupled** validation lives — real `n_e` → real blackout timing → real INS drift → reacquisition — which
genuinely cannot run until Stage 1 lands (the one irreducible serialization; Stage 2's engine logic is validated
earlier against the stub).

## Honest residual serialization

Contract-first parallelizes *construction*, not *coupled validation*. Stage 2's engine logic (coast exactness,
clock split, closed-loop nav against the stub) validates early. But the **coupled** gate — real electron density
driving the real blackout window driving the real drift — is a Stage-5 milestone, because it needs Stage-1's
marcher behind ④. This is a real constraint, labeled, not a placeholder.

## Non-goals

Bars `(4,2)` packaging; geopotential harmonics > J2; IERS 2PN clock; full 6-DOF entry; GPU/parallel
acceleration (gated behind
[`../../notes/tensor-network/ACCELERATION-SOTA-FIRST.md`](../../notes/tensor-network/ACCELERATION-SOTA-FIRST.md)).

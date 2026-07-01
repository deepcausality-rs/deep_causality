## ADDED Requirements

### Requirement: Evolve the CfdFlow coupling seam to carry the full flagship loop body

Stage 4 SHALL evolve the existing `CfdFlow` design — the config→run split (`CfdFlow::qtt_march(&config)` →
`QttMarchRun`, whose `run_coupled` is the control loop) and the static `.couple` cons-tuple of `PhysicsStage`s
(`Coupling::between_steps().then(..).build()`, no `dyn`) — so the whole plasma-blackout-corridor **loop body**
composes as a per-step coupling stack. It SHALL **not** introduce a linear phase pipeline; the iteration stays
in `run_coupled`, and the corridor §4 steps [2]/[3]/[4]/[6] are realized as `PhysicsStage`s over the shared
`CoupledField`. It SHALL evolve the existing `fluiddynamics-dsl` / `qtt-flow` capabilities rather than duplicate
them. A **preliminary design** is recorded in this change's `design.md`; minor revision at Stage 4 is expected.

#### Scenario: The corridor is a composed coupling stack run by the loop
- **WHEN** the flagship composes `RegimeClassify`, the reacting-ionization stages, `TrajectoryNav`, and
  `CyberneticCorrect` via `Coupling::between_steps().then(..)` and runs `CfdFlow::qtt_march(&cfg).run_coupled(..)`
- **THEN** each step advances the flow and applies the stack in order over the `CoupledField`, iterating to the
  configured stop — the control loop, not a one-shot pipeline

#### Scenario: Counterfactual branches reuse the override mechanism
- **WHEN** N bank-angle profiles are explored
- **THEN** each is an alternate `run_coupled` spawned via `seed_with` / `march_with` from the same borrowed
  config (the existing counterfactual mechanism), not a new pipeline phase

### Requirement: Cybernetic correction as a short-circuiting PhysicsStage

The cybernetic bounded-correction gate SHALL be a `PhysicsStage` that wraps a direct
`CyberneticLoop::control_step`: it clamps the bank-angle Action into the safety-envelope Context by construction
(mutating the `CoupledField` control channel) and returns `Err(Entropy)` on an unrecoverable breach — reusing
the coupling's existing `?` short-circuit as the "emit no unsafe action" semantics. It SHALL add no Effect-monad
allocation on the hot path.

#### Scenario: Breach short-circuits the coupling with Entropy
- **WHEN** no bank-angle action keeps the trajectory inside the envelope during a step
- **THEN** the `CyberneticCorrect` stage returns `Err`, the coupling short-circuits, and no out-of-envelope
  action is written to the field

### Requirement: Central control loop in ~10–30 lines

The flagship's central control loop SHALL be expressible in approximately **10–30 lines of Rust** — a
`Coupling::between_steps().then(..)` stack plus one `run_coupled` call — the elegance / concise-expressiveness
acceptance target.

#### Scenario: The flagship control loop meets the line budget
- **WHEN** the flagship's top-level control loop is written
- **THEN** it reads in ~10–30 lines: a `Coupling::between_steps().then(..)` stack plus the `run_coupled` call,
  with each corridor step a single stage

### Requirement: Counterfactual alternation on value, context, and state

The evolved DSL SHALL express CFD counterfactuals as **alternation** on the three mutable channels of the
`deep_causality_core` `Alternatable<V, C, S>` family — using the core vocabulary **verbatim**:
`alternate_value`, `alternate_context`, `alternate_state`. The word *alternate* SHALL appear at every call site
(no domain-flavored aliases), making it explicit that a run simulates an **alternate reality**, not a Pearl
`do()` on a single value. The CFD channel mapping SHALL be: **context** = the whole world (a `QttMarchConfig`
plus `Ambient` / BCs / atmosphere / bank schedule / envelope), **state** = the marching state (the fluid trains,
carried scalars, nav state, clocks), **value** = an injected primary-state snapshot (the `intervene` analog).
Alternation SHALL inherit the core contract: the **error channel is never alternated** (a diverged run cannot be
"fixed" by swapping its world), and each alternation appends an explicit audit entry (`!!ContextAlternation!!` /
`!!StateAlternation!!` / `!!ValueAlternation!!`) to the `EffectLog`.

#### Scenario: Context alternation runs the same solver in a different world
- **WHEN** `CfdFlow::qtt_march(&nominal).alternate_context(&alt_world).run_coupled(..)` runs
- **THEN** the same coupling stack marches under `alt_world`, an `!!ContextAlternation!!` entry records the
  swap, and the factual and alternate reports are comparable for the estimand

#### Scenario: State alternation branches from a different condition
- **WHEN** a run alternates the marching state (e.g. entry ionization ×2) via `alternate_state`
- **THEN** the same world marches from the alternated state, with a `!!StateAlternation!!` audit entry

#### Scenario: Alternation cannot repair an errored run
- **WHEN** the upstream chain has already errored and an alternation is applied
- **THEN** the error is preserved and the alternation is not applied (only the audit entry is appended)

### Requirement: Alternation attaches both pre-run and mid-march

The DSL SHALL support alternation at **both** attach points with the same three verbs. **Pre-run:** alternate the
context/state before marching (subsuming the existing `seed_with` / `march_with` overrides). **Mid-march:** a
**resumable, forkable** loop — `run_until(predicate)` SHALL return a `MarchPause` snapshot, `fork()` SHALL
produce an independent branch, and `continue_march(steps)` SHALL resume the (possibly alternated) branch from the
branch state. The corridor §4 [5] bank-angle branches SHALL be expressible as context alternations forked from a
single shared factual branch state (e.g. at blackout onset).

#### Scenario: Pre-run alternation subsumes the overrides
- **WHEN** a counterfactual is set up before marching
- **THEN** it is expressed as `alternate_context` / `alternate_state` (not ad-hoc field setters), and the factual
  run is unaffected

#### Scenario: Mid-march fork branches alternate worlds from one state
- **WHEN** the factual runs to a predicate (blackout onset) via `run_until`, then `fork().alternate_context(w)`
  for each bank-angle world `w`
- **THEN** each branch resumes from the shared branch state under its own world via `continue_march`, and each
  branch's `EffectLog` records its alternation

### Requirement: Alternatable marching state is Arc-shared, copy-on-write

The threaded marching state SHALL be `Arc`-wrapped so `fork` and `alternate_state` share by reference in O(1)
(no tensor data copied on a read-only fork), and a stage that **writes** the state SHALL trigger the clone via
copy-on-write (`Arc::make_mut`) — the cost is paid only when a fork actually diverges. A stage that only reads
the shared state SHALL NOT trigger a clone.

#### Scenario: Read-only fork copies nothing
- **WHEN** a fork is created and only read (e.g. a diagnostic branch, or a stage that reads `n_e`)
- **THEN** no tensor-train data is cloned; the `Arc` stays shared

#### Scenario: First write triggers copy-on-write
- **WHEN** a forked branch's stage first writes the marching state
- **THEN** `Arc::make_mut` clones the shared state once for that branch, and subsequent writes in the branch are
  in-place

### Requirement: Alternate worlds are whole named configurations

Context alternation SHALL swap a **whole** `QttMarchConfig` (coarse-grained), not a config delta. Each alternate
world SHALL be a checked-in, named configuration constructor (e.g. `config::nominal_reentry()`,
`config::steep_reentry()`, `config::dense_atmosphere()`), so a call site names exactly which reality it
simulates and the world is diffable in the repository.

#### Scenario: A call site names its world
- **WHEN** an alternation selects a world
- **THEN** it reads `alternate_context(&config::steep_reentry())` — a named whole configuration — with no
  config-delta chain to mentally resolve

### Requirement: Near-zero overhead over the low-level mechanism

The evolved surface SHALL remain a *naming* layer over the existing static cons-tuple composition: **static
dispatch, no `dyn`**, monomorphized, compiling to the same machine code as the hand-written
`Coupling`/`run_coupled`. The QTT marcher SHALL continue to ride its `EndoArrow` step + round unchanged (the
arrow-algebra marcher is reused, not re-expressed).

#### Scenario: DSL and hand-written coupling are equivalent
- **WHEN** a flow composed via the DSL is compared to the equivalent hand-written `Coupling` + `run_coupled`
- **THEN** they produce identical results, with no dynamic dispatch and no extra hot-path allocation introduced
  by the DSL layer

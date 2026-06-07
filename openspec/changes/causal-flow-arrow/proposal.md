## Why

The causal monad and the value-level Arrow algebra both exist in the codebase, on one categorical
foundation, yet they do not touch. `deep_causality_core` depends on `deep_causality_haft` but uses none
of its Arrow combinators. `CausalFlow`, the fluent facade over `CausalEffectPropagationProcess`, composes
one way only: a linear chain of `bind`s, consumed as it is built. Three shapes are missing as a result.

- **Loops.** Every time-stepped example hand-rolls a `for` loop with a `let mut process` that it rebinds
  each tick (`corrective_lane_keeping`, `geometric_tcas`, `hypersonic_2t`, `magnav`). The loop sits
  outside the monad, so error short-circuit, log accumulation, and a bound on iteration are all manual.
- **Branches.** The four `corrective_*` intervention examples are a loop whose body is "advance one tick,
  then *if* the monitor trips, intervene." That conditional is plain Rust `if` around a rebind, not a step
  the flow understands.
- **Reusable, composable pipelines.** A `CausalFlow` is a value mid-computation, spent when chained. You
  cannot build a stage once, name it, and run it on many inputs, and you cannot wire one authored pipeline
  into the next with the same fluent API you used to build each.

The structure that supplies all three is the Causal Arrow: the Kleisli arrow of the causal monad. Its
sequential composition is the reusable pipeline; its endomorphism iteration is the loop; choice over a
sum is the branch. The pieces are built in `haft`, but only for *pure* arrows that compose by ordinary
function application. What is absent is their realization *over the causal monad*, where each step threads
the error, state, context, and log channels.

This change builds that realization and updates the flow DSL on top of it. It is deliberately confined to
the causal monad and the flow DSL built on it; it touches nothing else.

## What Changes

- Add a **Causal Arrow** to `deep_causality_core` as the underlying engine: a reusable
  `A → CausalFlow<B, S, C>` value with **sequential composition**, so a stage can be built once, stored,
  and run on many inputs. The CausalFlow DSL below is the surface users write; the arrow stays out of the
  examples and the headline API.
- **Compose whole pipelines through the DSL.** Author process one, process two, and process three each as
  their own `CausalFlow` pipeline (a function `A → CausalFlow<B>`), then wire them with the high-level DSL:
  `CausalFlow::value(input).next(p1).next(p2).next(p3)`. The reified `CausalArrow` is the engine underneath
  for the case a composite must be held as data (stored, returned, reused as a value); it stays out of the
  examples and the headline API.
- Add **bounded loops** to the flow DSL: `iterate_n`, `iterate_until`, and `iterate_to_fixpoint`, each iterating a flow
  step with an explicit step bound and an honest report on convergence. This is the causal form of
  `EndoArrow`: an endomorphism on the flow carrier.
- Add **branches** to the flow DSL: `branch` (route on a predicate over the current value), `branch_with`
  (the predicate also reads state and context), and `either` (route a flow whose value is `Either<L, R>`).
  A small `Either<L, R>` sum is added for the typed form.
- **Showcase the new syntax.** Rewrite the loop-using examples so the hand-rolled `for` / `let mut`
  patterns become flow loops and branches: the `corrective_*` intervention family (loop plus conditional
  intervene) and the `geometric_tcas` / `hypersonic_2t` / `magnav` avionics simulations. Output is
  preserved; the before/after is the living documentation of the DSL.
- The change is **additive and non-breaking**. The existing `CausalEffectPropagationProcess`,
  `PropagatingEffect`, `PropagatingProcess`, `CausalMonad`, and the current `CausalFlow` surface are
  unchanged. The new surface is opt-in and lowers to the existing monad operations.

## Scope boundary (explicit)

This change set affects only the causal monad and the flow DSL built on top of it. The following are out
of scope and are not touched here:

- **Causal discovery** (SURD, BRCD, PC/GES/BOSS). No discovery operator is cast onto the arrow.
- **Multi-input / the monoidal product** (`split` / `***`, `fanout` / `&&&`, `first`, `second`). Wiring
  two structured inputs into one stage is out; every arrow here is single-input.
- **The static structural parameter** (a frozen graph or lattice a stage reads). That exists only to serve
  discovery, which is out, so it is out too.
- The **governance** and **action** fragments (effect ethos, causal state machine).

## Capabilities

### New Capabilities

- `causal-arrow`: a reusable Kleisli arrow over the causal monad. A `A → CausalFlow<B, S, C>` value with
  sequential composition (`next`), composition of whole pipelines into one, and bounded iteration. It
  threads the error, state, context, and log channels the pure `haft` Arrow does not.

### Modified Capabilities

- `causal-flow`: add bounded loops (`iterate_n` / `iterate_until` / `iterate_to_fixpoint`), branches (`branch` /
  `branch_with` / `either`), and application of a reusable pipeline to a flow (`and_then` over a
  `CausalArrow`). The existing construction, step, terminal, interop, and intervention requirements are
  unchanged.

## Impact

- **Code**: a new `causal_arrow` module in `deep_causality_core` (the Kleisli lift, sequential
  composition, the builder) and new loop/branch/composition methods on `CausalFlow` in the existing
  `causal_flow` module; plus a small `Either<L, R>` sum added to `deep_causality_haft` (D7: placed there for
  reuse by a future `ArrowChoice` and the CDL). No edits to the monad types beyond what the lowering
  requires.
- **APIs**: purely additive. Existing `pure` / `bind` / `with_state` / `CausalFlow` call sites keep
  working.
- **Examples**: the four `corrective_*` intervention examples and the three named avionics examples are
  rewritten to the new loop/branch surface, output preserved, as the showcase.
- **Crates touched**: `deep_causality_core` (the arrow and the DSL) and `deep_causality_haft` (`Either`
  only). No new external dependency. Edition 2024; `unsafe_code = "forbid"`; no `dyn`, no macros in `/src`;
  the lint and 100% coverage policy applies to the new code.

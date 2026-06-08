## 1. Foundations (the `Either` sum, in `haft`)

- [x] 1.1 Add `Either<L, R>` to `deep_causality_haft` (D7: placed in `haft` for reuse by a future
  `ArrowChoice` and the CDL). One type, one module: `deep_causality_haft/src/either/mod.rs` with the enum,
  constructors, and predicates (`is_left` / `is_right`); derive `Debug`, `Clone`, `PartialEq`. Re-export
  from the `haft` crate root. `deep_causality_core` imports it from `haft` (already a dependency).
- [x] 1.2 Register `deep_causality_haft/tests/either/either_tests.rs` in its `mod.rs` chain and in
  `deep_causality_haft/tests/BUILD.bazel`.

## 2. The Causal Arrow (reusable pipeline value + composition)

- [x] 2.1 Add the `causal_arrow` module to `deep_causality_core` (D2). Define `CausalLift<A, B, S, C, F>`
  (`F: Fn(A) -> CausalFlow<B, S, C>`) implementing `deep_causality_haft::Arrow` with
  `Out = CausalFlow<B, S, C>` and `run(&self, …)`. No `dyn`, no macros. Re-export the constructors from
  the crate root.
- [x] 2.2 `KleisliCompose<P, Q>` (`next`): bind `P`'s result into `Q` (D1); thread error/state/context,
  accumulate logs, short-circuit on `P` error. (`CausalFlowOut` projects the output value/state/context.)
- [x] 2.3 `CausalArrowBuilder`: `causal_arrow(f)` start, `.next` step, `.build` / `.run` terminals (D3).
  The nested carrier types stay hidden. Sequential fragment only; no product.
- [x] 2.4 Engine `next`: the builder's `next` composes a whole pipeline with another into one reusable
  `CausalArrow` value (the held-as-data composite). The engine stays out of examples.

## 2a. Pipeline composition through the DSL (the surface, D4)

- [x] 2a.1 `CausalFlow::next(pipeline)` where `pipeline: Fn(Value) -> CausalFlow<U, S, C>`: apply a whole
  pipeline (the next sub-process) to the flow, lowering to `bind` (the same lowering as `and_then`);
  short-circuit on an errored flow. No `CausalArrow` type is named.
- [x] 2a.2 Applying a reified `CausalArrow` engine value reuses the existing `and_then`, as
  `flow.and_then(|v| arrow.run(v))` (no dedicated `pipe` method). Confirmed `and_then` supports this shape;
  covered by a test, no new method.

## 3. Flow DSL loops (the causal `EndoArrow`, D6)

- [x] 3.1 `CausalFlow::iterate_n(n, step)` where `step: Fn(CausalFlow<V, S, C>) -> CausalFlow<V, S, C>`:
  bind the flow-endomorphism `n` times; an error mid-way short-circuits and skips the rest.
- [x] 3.2 `CausalFlow::iterate_until(pred, max_steps, step)`: bind until `pred(&value)` holds or the bound
  is hit; on bound-hit, short-circuit with `MaxStepsExceeded`.
- [x] 3.3 `CausalFlow::iterate_to_fixpoint(max_steps, step)` (`Value: PartialEq + Clone`): bind until the
  value stops changing or the bound is hit; on bound-hit, short-circuit with `MaxStepsExceeded`.
- [x] 3.4 Loop methods placed in `causal_flow/iterate.rs`; registered.

## 4. Flow DSL branches (D5)

- [x] 4.1 `CausalFlow::branch(cond, on_true, on_false)` with flow-endomorphism arms: peek the value, route
  the flow to one arm; an errored or value-less flow passes through.
- [x] 4.2 `CausalFlow::branch_with(cond, on_true, on_false)`: the predicate reads value, state, and
  context; otherwise as `branch`.
- [x] 4.3 `CausalFlow::either(left, right)` for `CausalFlow<Either<L, R>, S, C>`: route `Left`/`Right` to
  its arm; errored flow short-circuits, value-less flow passes through.
- [x] 4.4 Branch methods placed in `causal_flow/branch.rs`; registered.

## 5. Tests (100% coverage on new code; tests mirror the src tree)

- [x] 5.1 `Either` tests: predicates, accessors, equality, clone/copy, debug/hash, both variants.
- [x] 5.2 Arrow tests: an arrow is reusable across two runs; application equals the monad pipeline; `next`
  binds and short-circuits; the builder `run` terminal; three-pipeline composition; the `CausalArrow`
  marker bound; `and_then(|v| arrow.run(v))` applies a reified arrow.
- [x] 5.3 Loop tests: `iterate_n` applies exactly `n` (and zero); `iterate_until` stops on the predicate,
  fails at the bound (`MaxStepsExceeded`), and handles an initially-true predicate and a value-less
  carrier; `iterate_to_fixpoint` stops at a fixpoint and fails at the bound; a failing step short-circuits
  each loop; loops thread state on a stateful flow.
- [x] 5.4 Branch tests: `branch` / `branch_with` take each arm, are a no-op on error, and pass a value-less
  flow through; `branch_with` reads state/context; state threads through an arm; `either` routes
  `Left`/`Right`, is a no-op on error, and passes a value-less flow through.
- [x] 5.5 `next` parity: composing pipelines via `next` equals the hand-written chain; `next` short-circuits.
- [x] 5.6 Error-path coverage: every short-circuit and non-convergence branch in the new modules is
  exercised.
- [x] 5.7 Registered every new test file in its `mod.rs` chain and in `tests/BUILD.bazel` (added
  `types_causal_arrow` and `types_causal_flow` suites).

## 6. Showcase — rewrite the loop-using examples (output preserved) — NEXT STAGE

> Deferred to a dedicated follow-up commit (output-preservation work, per the owner). The new in-tree
> doctests already demonstrate the syntax: a loop with a branch (`causal_flow`) and pipeline composition
> via `next` (`causal_arrow`).

- [ ] 6.1 `causal_intervention_examples/corrective_lane_keeping` (W1, headline): closed loop becomes
  `iterate_n` + `branch_with` + `intervene_if`; open loop is the same `iterate_n` without the branch.
- [ ] 6.2 `corrective_glucose_pump`, `corrective_network_failover`, `corrective_decompression_stops`:
  the same loop-plus-branch rewrite.
- [ ] 6.3 `avionics_examples/geometric_tcas` (W2): `iterate_until(resolved, 30, tick)` with the interlock
  as a `branch`/`intervene` arm and one Euler step inside the tick.
- [ ] 6.4 `avionics_examples/hypersonic_2t` and `avionics_examples/magnav` (W3): the tracker / navigation
  `for` loops become `iterate_n`.
- [ ] 6.5 A focused pipeline-composition example binary (W4). (The doc-example variant is already done via
  the `causal_arrow` doctest.)

## 7. Verification

- [x] 7.1a Library verification: `cargo test` / `cargo clippy --all-targets` (0 warnings, no `#[allow]`) /
  `cargo fmt` for `deep_causality_haft` (66 tests) and `deep_causality_core` (191 integration + 3 doctests).
- [ ] 7.1b Example verification: build and run every rewritten example and diff its output against the
  pre-change output. (Next stage, with §6.)
- [x] 7.2 No `dyn`, no trait objects, no macros in `/src`; `unsafe_code = "forbid"` holds; edition 2024.
- [x] 7.3 Doc comments on each new type and method, naming the monad operation it lowers to and (for the
  arrow) the `haft` Arrow combinator it is the Kleisli analogue of; module-level doctests showing a loop +
  branch and a pipeline composition.
- [x] 7.4 Commit messages prepared (one per affected crate: `deep_causality_haft`, `deep_causality_core`);
  owner commits. Additive, non-breaking.

## 8. Notes

- Scope is confined to the causal monad and the flow DSL. Out of scope and not touched: causal discovery,
  the monoidal product (multi-input), the static structural parameter, and the governance/action
  fragments.
- **Stage split (per owner):** this commit is the library work (sections 1–5, 7); the example rewrites
  (section 6) land in a separate, dedicated commit in the next stage.

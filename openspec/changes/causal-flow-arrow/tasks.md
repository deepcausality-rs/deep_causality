## 1. Foundations (the `Either` sum, in `haft`)

- [ ] 1.1 Add `Either<L, R>` to `deep_causality_haft` (D7: placed in `haft` for reuse by a future
  `ArrowChoice` and the CDL). One type, one module: `deep_causality_haft/src/either/mod.rs` with the enum,
  constructors, and predicates (`is_left` / `is_right`); derive `Debug`, `Clone`, `PartialEq`. Re-export
  from the `haft` crate root. `deep_causality_core` imports it from `haft` (already a dependency).
- [ ] 1.2 Register `deep_causality_haft/tests/either/either_tests.rs` in its `mod.rs` chain and in
  `deep_causality_haft/tests/BUILD.bazel`.

## 2. The Causal Arrow (reusable pipeline value + composition)

- [ ] 2.1 Add the `causal_arrow` module to `deep_causality_core` (D2). Define `CausalLift<A, B, S, C, F>`
  (`F: Fn(A) -> CausalFlow<B, S, C>`) implementing `deep_causality_haft::Arrow` with
  `Out = CausalFlow<B, S, C>` and `run(&self, …)`. No `dyn`, no macros. Re-export the constructors from
  the crate root.
- [ ] 2.2 `KleisliCompose<P, Q>` (`next`): bind `P`'s result into `Q` (D1); thread error/state/context,
  accumulate logs, short-circuit on `P` error.
- [ ] 2.3 `CausalArrowBuilder`: `causal_arrow(f)` start, `.next` / `.next` steps, `.build` / `.run`
  terminals (D3). The nested carrier types stay hidden. Sequential fragment only; no product.
- [ ] 2.4 Engine `next`: confirm the builder's `next` composes a whole pipeline with another into one
  reusable `CausalArrow` value (the held-as-data composite). The engine stays out of examples.

## 2a. Pipeline composition through the DSL (the surface, D4)

- [ ] 2a.1 `CausalFlow::next(pipeline)` where `pipeline: Fn(Value) -> CausalFlow<U, S, C>`: apply a whole
  pipeline (the next sub-process) to the flow, lowering to `bind` (the same lowering as `and_then`);
  short-circuit on an errored flow. This is the composition verb examples and docs use; no `CausalArrow`
  type is named.
- [ ] 2a.2 Applying a reified `CausalArrow` engine value reuses the existing `and_then`, as
  `flow.and_then(|v| arrow.run(v))` (no dedicated `pipe` method), for the case a composite was built and
  held as data. Confirm `and_then` already supports this shape; add a test, not a method.

## 3. Flow DSL loops (the causal `EndoArrow`, D6)

- [ ] 3.1 `CausalFlow::iterate_n(n, step)` where `step: Fn(CausalFlow<V, S, C>) -> CausalFlow<V, S, C>`: bind
  the flow-endomorphism `n` times; an error mid-way short-circuits and skips the rest.
- [ ] 3.2 `CausalFlow::iterate_until(pred, max_steps, step)`: bind until `pred(&value)` holds or the bound
  is hit; on bound-hit, short-circuit with a non-convergence `CausalityError`.
- [ ] 3.3 `CausalFlow::iterate_to_fixpoint(max_steps, step)` (`Value: PartialEq + Clone`): bind until the value stops
  changing or the bound is hit; on bound-hit, short-circuit with a non-convergence `CausalityError`.
- [ ] 3.4 Place the loop methods in the `causal_flow` module per the one-type-one-module convention (e.g.
  `causal_flow/iterate.rs`); register them.

## 4. Flow DSL branches (D5)

- [ ] 4.1 `CausalFlow::branch(cond, on_true, on_false)` with flow-endomorphism arms: peek the value, route
  the flow to one arm; an errored flow is a no-op preserving error and logs.
- [ ] 4.2 `CausalFlow::branch_with(cond, on_true, on_false)`: the predicate reads value, state, and
  context; otherwise as `branch`.
- [ ] 4.3 `CausalFlow::either(left, right)` for `CausalFlow<Either<L, R>, S, C>`: route `Left`/`Right` to
  its arm; errored flow is a no-op.
- [ ] 4.4 Place the branch methods in `causal_flow/branch.rs`; register them.

## 5. Tests (100% coverage on new code; tests mirror the src tree)

- [ ] 5.1 `Either` tests: constructors, predicates, equality, both variants.
- [ ] 5.2 Arrow tests: an arrow is reusable across two runs; `next` binds and short-circuits; the builder
  result equals the explicit carriers; pipeline composition equals applying the pipelines by hand;
  `and_then(|v| arrow.run(v))` applies a reified arrow and is a no-op on error.
- [ ] 5.3 Loop tests: `iterate_n` applies exactly `n`; `iterate_until` stops on the predicate and fails at the
  bound; `iterate_to_fixpoint` stops at a fixpoint and fails at the bound; a failing step short-circuits each loop;
  logs accumulate across iterations; loops thread state on a stateful flow.
- [ ] 5.4 Branch tests: `branch` and `branch_with` take each arm and are a no-op on error; `branch_with`
  reads state/context in the predicate; `either` routes `Left`/`Right` and is a no-op on error.
- [ ] 5.5 Parity tests: each loop and branch equals the equivalent hand-written `bind` pipeline (value,
  error, logs), including the error short-circuit path.
- [ ] 5.6 Error-path coverage: every short-circuit and non-convergence branch in the new modules is
  exercised.
- [ ] 5.7 Register every new test file in its `mod.rs` chain (`#[cfg(test)]`) and in `tests/BUILD.bazel`.

## 6. Showcase — rewrite the loop-using examples (output preserved)

- [ ] 6.1 `causal_intervention_examples/corrective_lane_keeping` (W1, headline): the closed loop becomes
  `CausalFlow::from(initial).iterate_n(N_TICKS, |t| t.bind(simulate_step).branch_with(tripped, intervene,
  passthrough))`; the open loop is the same `iterate_n` without the branch. Output unchanged.
- [ ] 6.2 `corrective_glucose_pump`, `corrective_network_failover`, `corrective_decompression_stops`: the
  same loop-plus-branch rewrite (they share the shape). Output unchanged.
- [ ] 6.3 `avionics_examples/geometric_tcas` (W2): the 30-step encounter becomes `iterate_until(resolved,
  30, tick)` with the auto-pilot interlock as a `branch`/`intervene` arm and the kinematics as one Euler
  step inside the tick. Output unchanged.
- [ ] 6.4 `avionics_examples/hypersonic_2t` and `avionics_examples/magnav` (W3): the hand-rolled tracker /
  navigation `for` loops become `iterate_n` (and `magnav`'s `Result`-returning filter update rides the
  error channel via `try_step`). Output unchanged.
- [ ] 6.5 Add a focused pipeline-composition example or doc example (W4): three pipelines authored
  separately as `Value -> CausalFlow<U>` functions and wired through the CausalFlow DSL
  (`CausalFlow::value(input).next(p1).next(p2).next(p3)`), run once and reused inside a loop. The reified
  `CausalArrow` engine does not appear at the call site.

## 7. Verification

- [ ] 7.1 `cargo build -p deep_causality_core` and `cargo test -p deep_causality_core`; build and run every
  rewritten example and diff its output against the pre-change output. `make format && make fix` (0 clippy
  warnings, no `#[allow(...)]`).
- [ ] 7.2 Confirm no `dyn`, no trait objects, no macros in `/src`; `unsafe_code = "forbid"` holds; edition
  2024.
- [ ] 7.3 Doc comments on each new type and method, naming the monad operation it lowers to and (for the
  arrow) the `haft` Arrow combinator it is the Kleisli analogue of. A module-level example showing a loop,
  a branch, and a pipeline composition.
- [ ] 7.4 Commit message prepared; owner commits. Additive, non-breaking.

## 8. Notes

- Scope is confined to the causal monad and the flow DSL. Out of scope and not touched: causal discovery,
  the monoidal product (multi-input), the static structural parameter, and the governance/action
  fragments.
- A natural split point, if the reviewer prefers two changes: sections 1–5 (the arrow, loops, branches,
  and their tests) first, then section 6 (the example rewrites) second.

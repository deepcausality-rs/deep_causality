## 1. Flow: one bind, `next` as value-only sugar

- [x] 1.1 `CausalFlow::and_then` → the single stateful Kleisli bind `Fn(Value, State, Option<Context>) -> CausalFlow<U, State, Context>`, threading the continuation's state/context forward; propagate `None`/`ContextualLink`, error on `RelayTo`/`Map` residue, left-zero on error
- [x] 1.2 Remove the former value-only `and_then` and the never-shipped `and_then_stateful`/`and_then_value` drafts — there is exactly one bind
- [x] 1.3 `CausalFlow::next(pipeline: Fn(Value) -> CausalFlow<U>)` = `and_then(|v, _, _| pipeline(v))` — the value-only drop-in; pre-existing `.next(...)` sites unchanged
- [x] 1.4 `CausalFlow::from_parts` (general constructor / arrow unit `η`); drop the interim `with_state` helper

## 2. Arrow engine: one stateful stage

- [x] 2.1 `CausalLift` → `In = (A, S, Option<C>)`, stage `Fn(A, S, Option<C>) -> CausalFlow<B,S,C>`; variance marker `PhantomData<fn(A,S,Option<C>) -> (B,S,C)>`
- [x] 2.2 `KleisliCompose::run` → thread `P`'s `(value, state, context)` into `Q` via `and_then`; `Q::In = (P::Value, P::State, Option<P::Context>)`; `Clone` bounds stay removed (D4)
- [x] 2.3 `CausalArrow<A,B,S,C>` marker → `Arrow<In=(A,S,Option<C>), Out=CausalFlow<B,S,C>>`
- [x] 2.4 Builder: `causal_arrow(f)` + `.next(g)` over the one stateful stage; add `run_value(a) = run((a, (), None))`. (No `_stateful` variants — one stage type.)
- [x] 2.5 `lib.rs` re-exports unchanged (`causal_arrow` still the entry; `run_value` is a method — no new symbol)

## 3. Migrate callers (value-only `and_then` → `next`)

- [x] 3.1 `deep_causality_core/tests/types/causal_flow/flow_tests.rs`: `.and_then(|x| ...)` → `.next(|x| ...)`; named stage fns stay `Fn(Value)`
- [x] 3.2 `examples/avionics_examples/cfd/turbulence_flow/main.rs`: two `.and_then(...)` → `.next(...)`
- [x] 3.3 `examples/physics_examples/grmhd/main.rs`: five `.and_then(|s| ...)` → `.next(|s| ...)`
- [x] 3.4 `cargo build --workspace --examples --tests` green

## 4. Formalization (Lean file authored here — ships the correction proven)

- [x] 4.1 `lean/DeepCausalityFormal/Core/CausalArrow.lean` (self-contained): stage `A → S → Option C → Process B S C E Λ`; `karrow_id = eta`; `akbind` = value-threading `and_then`; `kcomp f g = fun a s c => akbind (f a s c) g`; proved `kcomp_left_id`/`kcomp_right_id`/`kcomp_assoc`/`kcomp_left_zero` by reduction to the monad theorems. Bare-`lean` typechecks (exit 0)
- [x] 4.2 Witnesses tagged in the existing `deep_causality_core/tests/types/causal_arrow/causal_arrow_tests.rs` (`arrow_threads_accumulated_state` → `core.causal_arrow.category_laws`; `arrow_error_short_circuit_preserves_state` → `core.causal_arrow.left_zero`). The `tests/formalization_lean/` mirror is **deferred to `formalize-deep-causality-core`**, which owns that scaffolding — witnessing in the existing file avoids duplicating the mirror across two changes and satisfies the CI gate (scans all `deep_causality_core/**/*.rs`)
- [x] 4.3 `THEOREM_MAP.md` rows for `core.causal_arrow.{category_laws,left_zero}`; `CausalArrow.lean` added to `lean/DeepCausalityFormal.lean`; local consistency gate green (44/44 ids bridged)

## 5. Docs + verify

- [x] 5.1 `causal_arrow/mod.rs` + `compose.rs` docs updated — the state/context-threading claim is now true and precise; module doctest uses `|x, _, _|` + `run_value`
- [x] 5.2 Arrow tests rewritten: stateless via `|x, _, _|`/`run_value`; added `arrow_threads_accumulated_state` and `arrow_error_short_circuit_preserves_state`
- [x] 5.3 `cargo test -p deep_causality_core` green (220); clippy clean
- [x] 5.4 `bazel test //...` green (960 pass); `lake build` + bare-`lean` on `CausalArrow.lean` (exit 0); `cargo fmt` + clippy clean on all touched crates (core, avionics_examples, physics_examples)
- [x] 5.5 Deviation ledger D2 → **Fixed** in `core-formalization-plan.md` (one state-threading bind + one arrow stage; stateless is the specialization). Per-crate commit messages prepared for the user (change not committed by the agent, per the golden rule)

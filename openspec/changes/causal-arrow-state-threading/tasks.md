## 1. CausalFlow support

- [ ] 1.1 Add `CausalFlow::with_state(self, s: State, c: Option<Context>) -> Self` (build/rebind a flow with explicit state/context); confirm `from_parts` exists and is the `η` builder
- [ ] 1.2 Add the `η` helper `from_parts(Ok(EffectValue::Value(a)), s, c, EffectLog::new())`
- [ ] 1.3 Add `CausalFlow::and_then_stateful<U, F>(self, f)` where `F: FnOnce(Value, State, Option<Context>) -> CausalFlow<U, State, Context>` — lowers to the monad `bind`, propagates `None`/`ContextualLink`, errors on `RelayTo`/`Map` residue
- [ ] 1.4 Keep the value-only `and_then`/`next` unchanged (lawful state-preserving step); confirm ~125 callers unaffected

## 2. Arrow engine

- [ ] 2.1 `CausalLift` → state-receiving: `F: Fn(A, S, Option<C>) -> CausalFlow<B,S,C>`; `Arrow<In=(A,S,Option<C>), Out=CausalFlow<B,S,C>>`; `run((a,s,c)) = f(a,s,c)`; variance marker `PhantomData<fn(A,S,Option<C>) -> (B,S,C)>`
- [ ] 2.2 `KleisliCompose::run` → thread `(value, state, context)`: run `p`, extract `(b, s1, c1)` via `and_then_stateful`, feed `g`; keep the `Clone` bounds removed (D4)
- [ ] 2.3 `CausalArrow<A,B,S,C>` marker → `Arrow<In=(A,S,Option<C>), Out=CausalFlow<B,S,C>>`
- [ ] 2.4 Run entry: `arrow.run((a, s0, c0))`; add `run_value(a)` = `run((a, (), None))`

## 3. Builder

- [ ] 3.1 Add `causal_arrow_stateful(f)` (state-receiving) + `.next_stateful(g)`
- [ ] 3.2 Keep `causal_arrow(f)` value-only + `.next(g)` as the state-preserving special case (`|a,s,c| f(a).with_state(s,c)`)
- [ ] 3.3 `lib.rs` re-exports: add `causal_arrow_stateful`

## 4. Formalization

- [ ] 4.1 `lean/DeepCausalityFormal/Core/CausalArrow.lean` (self-contained): stage `A → S → Option C → Process B S C E Λ`; `karrow_id = eta`; `kcomp f g = fun a s c => bind' (f a s c) g`; prove `kcomp_left_id`/`kcomp_right_id`/`kcomp_assoc`/`kcomp_left_zero` by reduction to the monad theorems. Bare-`lean` typecheck
- [ ] 4.2 `deep_causality_core/tests/formalization_lean/causal_arrow_tests.rs`: witnesses for `core.causal_arrow.{category_laws,left_zero}` — stateful pipeline threading accumulated state; left/right id; assoc; error short-circuit preserves state
- [ ] 4.3 `THEOREM_MAP.md` rows for `core.causal_arrow.*`; add the file to `lean/DeepCausalityFormal.lean` (coordinate with `formalize-deep-causality-core` to avoid double-authoring)

## 5. Docs + verify

- [ ] 5.1 Update `causal_arrow/mod.rs` and `compose.rs` docs — the state/context-threading claim is now true and precise
- [ ] 5.2 `causal_arrow_tests.rs`: adjust the retained value-only tests to use `run_value`
- [ ] 5.3 `bazel test //...` and `cargo test -p deep_causality_core` green; bare-`lean` on `CausalArrow.lean`
- [ ] 5.4 `make format && make fix` clean (fix clippy lints, do not suppress)
- [ ] 5.5 Update the deviation ledger: D2 → "Fixed — full state-threading Kleisli arrow of the causal monad (Option B)". Prepare a commit message per changed crate; ask before committing

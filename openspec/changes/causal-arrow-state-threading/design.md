## Context

The reified causal arrow (`deep_causality_core/src/types/causal_arrow/`: `CausalLift`,
`KleisliCompose`, `CausalArrowBuilder`, `CausalArrow`) composes stages of type
`A -> CausalFlow<B, S, C>`. Its `KleisliCompose::run` runs `p`, extracts the value, and feeds `g` —
dropping the upstream state/context (`causal_flow/steps.rs`'s value-only `and_then` binds
`|v, _state, _context|`). So the arrow is the Kleisli category of the stateless sub-monad only. Yet
`Core/CausalMonad.lean` proves the *full* state-passing monad lawful: its `bind'` hands the
continuation `(value, state, context)` and its unit `eta v s c` re-emits all three. This change makes
the Rust arrow match that carrier. Full plan (verbatim source): `causal-arrow-state-threading-plan.md`.
The D3 (`None`-preserving `and_then`) and D4 (drop vestigial `Clone` bounds) fixes already landed.

## Goals / Non-Goals

**Goals:**
- Widen the arrow stage to `(A, S, Option<C>) -> CausalFlow<B, S, C>` so composition threads
  state/context exactly as the monad's `bind`.
- Prove the arrow-from-monad laws (left/right identity, associativity, error left-zero) over arbitrary
  `S`, `C`, reducing to the already-proved monad theorems.
- Retain value-only ergonomics with zero regression (the ~125 `and_then`/`next` callers).

**Non-Goals:**
- Not touching the value/control conflation (that is `separate-control-channel`); the interim
  docstring notes the `RelayTo`/`Map` error residue until that lands.
- Not adding new arrow combinators beyond the state-threading pair and `run_value`.
- Not changing the value-only DSL's behavior.

## Decisions

### D1. Widen the stage (Option B) rather than document the erasure
The alternative — accept the value-only arrow and state in Lean that the model erases `S, C` — was
rejected ("Option B, full stop, no shortcuts"). The arrow must be the Kleisli category of the monad the
crate actually threads, not of a stateless shadow. Widening the stage to receive `(A, S, Option<C>)` is
the faithful correction; the Lean stage then matches `CausalMonad.lean`'s continuation exactly, and the
category laws reduce to the monad laws already proved.

### D2. Value-only stages become the state-preserving sub-case, not a separate hierarchy
`causal_arrow(f)` (value-only) is kept and lifted as `|a, s, c| f(a).with_state(s, c)` — the Kleisli
arrow that does not touch state. This is a lawful morphism in the *same* category (Option B subsumes the
old arrow), so no parallel type hierarchy is introduced and existing callers are untouched. This is the
key blast-radius container: state-threading is additive.

### D3. `and_then_stateful` lowers to the monad `bind`; keep value-only `and_then`
The new `CausalFlow::and_then_stateful<U, F>(self, f)` where
`F: FnOnce(Value, State, Option<Context>) -> CausalFlow<U, State, Context>` lowers directly to the
monad `bind` (which passes `(value, state, context)`), propagating `None`/`ContextualLink` and erroring
on `RelayTo`/`Map` residue. The value-only `and_then` stays as-is (already the lawful state-preserving
step after D3). `KleisliCompose::run` uses `and_then_stateful`; the builder exposes both.

### D4. `run_value` convenience for `S = C = ()`
`run((a, (), None))` is the common stateless entry; `run_value(a)` wraps it so existing tests and any
stateless user keep a one-argument call. Keeps the widening from leaking into the ergonomic path.

## Risks / Trade-offs

- **[Ergonomic regression for value-only users]** → Mitigated by keeping `causal_arrow`/`.next` and
  `run_value`; the plan verified no production/example user of the reified engine, only tests.
- **[Interim right-identity is conditional on the control residue]** → Right identity is unconditional
  on the value fragment now (post D3); fully unconditional only after `separate-control-channel`. The
  interim docstring states this; the downstream formalization asserts the unconditional form once both
  prerequisites are in.
- **[Variance/PhantomData correctness on the widened `CausalLift`]** → Update the marker to
  `PhantomData<fn(A, S, Option<C>) -> (B, S, C)>`; covered by the arrow tests compiling and running.
- **[Double-counting state updates if `and_then_stateful` mis-threads]** → The Lean proof + witness
  (a reusable stateful pipeline that accumulates state) pin the threading; a mis-thread fails
  associativity or the accumulation assertion.

## Migration Plan

Per the plan §6, in order (each step compiles + `bazel test //...`):
1. `CausalFlow::with_state` / confirm `from_parts`; `η` helper.
2. `CausalFlow::and_then_stateful` (state-threading bind).
3. `CausalLift` → state-receiving (`In = (A, S, Option<C>)`; update variance marker).
4. `KleisliCompose::run` → thread `(value, state, context)`.
5. Builder: `causal_arrow_stateful` + `.next_stateful`; keep value-only path.
6. `CausalArrow` marker signature; `run_value`.
7. `lib.rs` re-exports (add `causal_arrow_stateful`).
8. `causal_arrow_tests.rs`: keep value-only tests (via `run_value`), add state-threading tests.
9. `Core/CausalArrow.lean` + witnesses + `THEOREM_MAP` rows; bare-`lean` typecheck.
10. Update `mod.rs`/`compose.rs` docs (the threading claim is now true).
11. `bazel test //...`; bare-`lean` on the new file; `make format && make fix`.

Rollback: revert; compile-time-only change, no state migration.

## Open Questions

- Should the Lean `CausalArrow.lean` land in this change or in `formalize-deep-causality-core`? The
  plan §6 step 9 puts it here (the correction and its proof together). Recommendation: state the arrow
  laws here so the correction ships proven; the downstream change only adds the `THEOREM_MAP` wiring and
  `LEAN_CORE.md` row if not already present. (Resolve during apply — avoid double-authoring the file.)

## Why

The causal arrow claims (in `KleisliCompose`'s docstring) to be the Kleisli category of the causal
monad, but its stage type is value-only — `A -> CausalFlow<B, S, C>` — so composition **discards** the
state/context that the monad's `bind` threads (`causal_flow/steps.rs`, deviation D2 in
`core-formalization-plan.md`). It is therefore a Kleisli arrow only of the *stateless* sub-monad
(`S = C = ()`), invisibly, because every current test uses `S = C = ()`. `Core/CausalMonad.lean`
already proves the *full* state-passing monad lawful (its `bind'` hands the continuation
`(value, state, context)`); the arrow does not match that carrier. The author's decision was **Option
B, no shortcuts**: widen the arrow stage to receive state/context so composition threads
`s0 → s1 → s2` exactly as the monad does — making the arrow the faithful Kleisli category of the monad
the Lean model already describes. Full plan: `openspec/notes/causal-algebra/causal-arrow-state-threading-plan.md`.

## What Changes

- **One Kleisli bind:** `CausalFlow::and_then` becomes the single stateful bind
  `Fn(Value, State, Option<Context>) -> CausalFlow<U, S, C>`, threading the continuation's
  state/context forward exactly as the monad's `bind` does. The stateless case is a *specialization*
  (`S = C = ()`), not a separate operation; the former state-discarding value-only `and_then` (the D2
  bug) is removed. No `and_then_stateful` is introduced.
- **`next` is the value-only sugar:** `next(pipeline: Fn(Value) -> CausalFlow<U>)` =
  `and_then(|v, _, _| pipeline(v))` — the everyday drop-in for stateless pipelines. Migration is
  therefore `.and_then(|x| …)` → `.next(|x| …)`, and every pre-existing `.next(…)` (closures and named
  `Fn(Value)` stage fns) compiles unchanged.
- **One arrow stage:** the reified stage becomes `(A, S, Option<C>) -> CausalFlow<B, S, C>` (one type;
  stateless via `|a, _, _|`). `CausalLift::In = (A, S, Option<C>)`; `KleisliCompose::run` threads
  `P`'s `(value, state, context)` into `Q` via `and_then`; the `CausalArrow` marker and builder
  (`causal_arrow`, `.next`) use this stage. Add `run_value(a) = run((a, (), None))` for `S = C = ()`.
- Add `CausalFlow::from_parts` (the general constructor / arrow unit `η`). Update `mod.rs`/`compose.rs`
  docs so the state/context-threading claim is now true.
- **Prove it here:** author `lean/DeepCausalityFormal/Core/CausalArrow.lean`, reducing the arrow laws
  to the monad theorems already proved.

## Capabilities

### New Capabilities
- `stateful-causal-arrow`: The causal arrow as the faithful Kleisli category of the full state-passing
  causal monad — the state-receiving stage, its unit and composition, the state-threading `and_then`,
  and the retention of value-only stages as the lawful state-preserving sub-case. Defines the arrow
  laws (left/right identity, associativity, error left-zero) over arbitrary `S`, `C`.

### Modified Capabilities
<!-- None. The reified arrow engine is not covered by any existing spec; the value-only DSL
     (and_then/next) is behavior-preserved. This adds a new capability rather than modifying one. -->

## Impact

- **Prerequisite of `formalize-deep-causality-core`** (supplies `Core/CausalArrow.lean`'s state-
  threading category laws). Independent of `separate-control-channel` (either order).
- **Blast radius: low, mostly additive** (verified in the plan §4). The reified arrow engine
  (`causal_arrow`/`CausalArrow`/`CausalLift`/`KleisliCompose`/`CausalArrowBuilder`) has **no
  production or example user** — only the module, the `lib.rs` re-exports, and `causal_arrow_tests.rs`.
  The value-only DSL (`and_then`/`next`, ~125 callers) is preserved; state-threading is purely
  additive.
- **Files** (`deep_causality_core/src/types/causal_arrow/` + `causal_flow/`): `lift.rs`, `compose.rs`,
  `builder.rs`, `mod.rs`; `causal_flow/steps.rs`, `causal_flow/construction.rs`; `lib.rs` re-exports;
  `tests/types/causal_arrow/*`.
- **Deviation ledger:** D2 upgrades from "Fixed (value-only, `S=C=()` scope)" to "Fixed — full state-
  threading Kleisli arrow of the causal monad (Option B)."

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

- **Widen the Kleisli stage** to `(A, S, Option<C>) -> CausalFlow<B, S, C>` (in
  `deep_causality_haft::Arrow` terms, `In = (A, S, Option<C>)`, `Out = CausalFlow<B, S, C>`) — the
  stage now *receives* the incoming state/context and produces the next.
- **Unit** `η(a, s, c) = CausalFlow::from_parts(Ok(EffectValue::Value(a)), s, c, EffectLog::new())` and
  **composition** `f >=> g = λ(a,s,c). bind_stateful(f(a,s,c), g)` threading `(b, s1, c1)` into `g`,
  short-circuiting on error and propagating `None`/`ContextualLink` lawfully.
- Add `CausalFlow::and_then_stateful` (the state-threading Kleisli bind lowering to the monad `bind`)
  and `CausalFlow::with_state` / confirm `from_parts` (build/rebind a flow with explicit state/context).
- Add the state-receiving builder `causal_arrow_stateful(f)` + `.next_stateful(g)`; **keep** the
  value-only `causal_arrow(f)` + `.next(g)` as the lawful **state-preserving** special case
  (`|a,s,c| f(a).with_state(s,c)`), so the ~125 value-only callers are unaffected.
- Add a `run_value(a)` convenience (`run((a, (), None))`) for the `S = C = ()` case.
- Update `CausalLift`, `KleisliCompose`, the `CausalArrow` marker, and the `lib.rs` re-exports for the
  widened stage; update `mod.rs`/`compose.rs` docs so the state/context-threading claim is now true.

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

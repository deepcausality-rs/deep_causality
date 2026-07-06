## Context

The reified causal arrow (`deep_causality_core/src/types/causal_arrow/`) composed stages of type
`A -> CausalFlow<B, S, C>`, and the flow DSL's `and_then` bound a continuation `Fn(Value)` while
**discarding** the continuation's state and preserving the upstream's. So the arrow was the Kleisli
category of the *stateless* sub-monad only — deviation D2, invisible because every caller used
`S = C = ()`. `Core/CausalMonad.lean` already proves the *full* state-passing monad lawful (its
`bind'` hands `(value, state, context)`; `eta v s c` re-emits all three). This change makes the Rust
arrow match that carrier.

## Goals / Non-Goals

**Goals:**
- One Kleisli bind that threads state, with the stateless case as its specialization.
- The reified arrow stage receives `(A, S, Option<C>)` so composition threads state.
- Prove the arrow-from-monad laws over arbitrary `S`, `C`.
- Zero regression for the ~33 stateless `.next(...)` call sites.

**Non-Goals:**
- The value/control conflation (that is `separate-control-channel`); the interim `RelayTo`/`Map`
  arm surfaces `ValueNotAvailable`.
- Any new arrow combinator beyond `run_value`.

## Decisions

### D1. One bind, stateless is a specialization (author decision)
The stateful bind with `S = C = ()` *is* the stateless bind — so there is exactly one lawful
`and_then`, not a value-only/stateful pair. `and_then`'s continuation is
`Fn(Value, State, Option<Context>) -> CausalFlow<U, S, C>` and its returned state/context are threaded
forward. The former value-only `and_then` (discard-continuation-state, preserve-upstream) was the
broken D2 behavior, not a legitimate second operation, and is removed. No `and_then_stateful` is
introduced — that would be the same duplication under a new name.

### D2. `next` is the value-only sugar, the everyday verb
Since `and_then` is now three-arg and the vast majority of pipelines are stateless, `next(pipeline)`
where `pipeline: Fn(Value) -> CausalFlow<U>` is kept/defined as exactly `and_then(|v, _, _| pipeline(v))`.
It is **not** a second bind — it pre-ignores the state/context inputs. This makes it the migration
drop-in: `.and_then(|x| …)` → `.next(|x| …)`, and every pre-existing `.next(…)` (closures *and* named
`Fn(Value)` stage fns) compiles unchanged. `and_then` becomes the rarely-called stateful form. An
earlier idea of a separate `and_then_value` alias was rejected as redundant with `next`.

### D3. One arrow stage type, stateless via `|a, _, _|`
The reified arrow gets a single stage `(A, S, Option<C>) -> CausalFlow<B, S, C>` (not a value-only +
stateful pair). `CausalLift::In = (A, S, Option<C>)`; `KleisliCompose::run` threads
`P`'s `(value, state, context)` into `Q` via `and_then`; the `CausalArrow` marker and builder
(`causal_arrow`, `.next`) use this stage. `run_value(a) = run((a, (), None))` keeps the stateless
entry a one-argument call. The reified engine has no production/example users (only the arrow tests),
so this churn is contained; the value-only "state-preserving lift" adapter and the `CausalFlow::with_state`
helper that an earlier draft introduced were dropped as unnecessary once the split was removed.

### D4. `CausalFlow::from_parts` as the general constructor / arrow unit `η`
`η(a, s, c) = from_parts(Ok(EffectValue::Value(a)), s, c, EffectLog::new())`. A single public
constructor beneath `value`/`process`, used by the state-threading witness tests to build arbitrary
`(value, state)` flows.

### D5. Lean file lands here, proven
`Core/CausalArrow.lean` is authored in **this** change (ships the correction proven), reducing the
arrow laws to the monad theorems already proved. The downstream `formalize-deep-causality-core` change
only wires the `THEOREM_MAP` rows / `LEAN_CORE.md` entry if not already present — it does not
re-author the file. (Resolves the prior open question.)

## Risks / Trade-offs

- **[Breaking the flow DSL `and_then` signature]** → Mitigated by `next` value-only sugar: real churn
  is only `.and_then(|x| …)` → `.next(|x| …)` (a handful of sites: core flow test, two examples).
  Verified: full `cargo build --workspace --examples --tests` green.
- **[Interim right-identity conditional on control residue]** → Right identity holds on the value
  fragment now; fully unconditional after `separate-control-channel`. Interim docstrings note it.
- **[Variance on the widened `CausalLift`]** → marker updated to
  `PhantomData<fn(A, S, Option<C>) -> (B, S, C)>`; covered by the arrow tests compiling.

## Migration Plan

1. Flow: `and_then` → single stateful bind; `next` → value-only sugar over it; `from_parts` added.
2. Arrow: `CausalLift`/`KleisliCompose`/`CausalArrow`/builder → one stateful stage; `run_value` added.
3. Migrate `.and_then(|x| …)` → `.next(|x| …)` (core flow test, turbulence_flow, grmhd examples).
4. Rewrite arrow tests: stateless via `|x, _, _|`/`run_value`, plus state-threading + error-preserves-state.
5. `Core/CausalArrow.lean` + witnesses + `THEOREM_MAP`.
6. Verify: `cargo build --workspace --examples --tests`, `cargo test -p deep_causality_core`,
   `bazel test //...`; bare-`lean` on the new file; `make format && make fix`.

Rollback: revert; compile-time-only change, no state migration.

## Open Questions

- None outstanding. (The `CausalArrow.lean` location question is resolved in D5: authored here.)

# Refactor: make `CausalMonad` a correct state-threading trait

Status: PROPOSED — awaiting review before implementation.

## Problem (root cause)

`CausalEffectPropagationProcess<V, S, C, E, L>` carries a **State** channel `S`, which makes it a
*State monad*. In this concrete-snapshot encoding, a State monad's `bind` must hand the continuation
the prior state and take back the new state:

```
f: (value, state, context) -> next_process   // next_process carries the updated state
```

The standard value-only bind `A -> M<B>` cannot express this: `M<B>` is a snapshot holding a fixed
state, not a state transition. Every value-only bind in the tree is therefore forced to freeze state.
There are three of them, and all three are wrong for stateful `S`:

- `MonadEffect5::bind` for `CausalMonad<S, C>` (the marker struct) — overwrites with input state.
- `Monad::bind` for `CausalEffectPropagationProcessWitness<…>` — `state: m_a.state` (frozen).
- `Monad::bind` for `PropagatingProcessWitness<S, C>` — `state: m_a.state` (frozen).

The only correct bind is the **inherent** `CausalEffectPropagationProcess::bind`, whose continuation is
`FnOnce(EffectValue<V>, S, Option<C>) -> Process<U, …>` and which keeps the continuation's returned
state. The intent was implemented inherently but never encoded in a trait. This refactor encodes it.

The error is **not** in `deep_causality_haft`. `MonadEffect3/4/5`'s value-only bind is correct for
genuine fixed-channel effect systems (Writer/Except/Counter/Trace — see the haft test fixtures).
The error is that `deep_causality_core` shoehorned a State channel into that fixed-channel
abstraction.

## Target design

One canonical, state-threading bind, encoded as a trait, implemented once.

- **Trait:** `CausalMonad` (reuse the name; the current zero-sized `CausalMonad<S, C>` marker struct
  is removed).
- **Location:** `deep_causality_core/src/traits/causal_monad.rs`, exported via `traits` + crate root.
- **Implemented for:** `CausalEffectPropagationProcess<V, S, C, E, L>` (covers both the stateful
  `PropagatingProcess` and the stateless `PropagatingEffect`; `S = ()` threads trivially).

```rust
pub trait CausalMonad: Sized {
    type Value;
    type State;
    type Context;
    type Error;
    type Log;

    /// Lift a value into the monad: value set, state defaulted, no error, empty log.
    fn pure(value: Self::Value) -> Self;

    /// State-threading bind.
    /// - short-circuits on error (value -> None, state/context/log preserved);
    /// - on a present value, calls `f(value, state, context)` and KEEPS the
    ///   returned process's state/context;
    /// - appends logs.
    fn bind<U, F>(
        self,
        f: F,
    ) -> CausalEffectPropagationProcess<U, Self::State, Self::Context, Self::Error, Self::Log>
    where
        F: FnOnce(
            EffectValue<Self::Value>,
            Self::State,
            Option<Self::Context>,
        ) -> CausalEffectPropagationProcess<U, Self::State, Self::Context, Self::Error, Self::Log>;
}
```

- `bind`'s body is the current inherent `bind` body, moved verbatim into the trait impl. No `U: Default`.
- `bind_or_error` becomes a provided default method on the trait (same behavior, returns `None` + error).
- `intervene` stays on the existing `Intervenable` trait (unchanged).

## Keep

- `Monad` for `PropagatingEffectWitness<CausalityError, EffectLog>` (stateless, `S = C = ()`,
  value-only bind correct, already returns `None` on error). This is what the uniform-math HKT
  composition uses — that story is untouched.
- All `HKT` / `HKT5` / `Pure` / `Functor` / `Applicative` impls on the witnesses (type-level
  machinery; `ProcessWitness::pure` keeps working).
- `deep_causality_haft` `Effect3/4/5` + `MonadEffect3/4/5` (correct for fixed-channel systems).

## Delete

- `struct CausalMonad<S, C>` and its `impl MonadEffect5<CausalSystem<S, C>>`
  (`types/causal_monad/`). State does not belong in an `Effect5` slot.
- `impl Monad for CausalEffectPropagationProcessWitness<…>` (value-only, freezes state).
- `impl Monad for PropagatingProcessWitness<S, C>` (value-only, freezes state).
- The standalone inherent `bind` / `bind_or_error` methods — relocated into the `CausalMonad` trait
  impl (this is the "inherent bind becomes redundant" the review flagged: one bind, trait-encoded).
- Whether `CausalSystem` / its `Effect5` impl can be removed depends on remaining HKT5 witness
  references; assess during implementation and remove if it becomes dead.

## Downstream migration (all internal)

- `monadic_collection.rs`, `causaloid/causable.rs`: already call `.bind(|v, s, c| …)`; keep working
  via the trait (add `use … CausalMonad;`). Replace `CausalMonad::pure(x)` with the process `pure`.
- Examples using the old marker (`geometric_tilt_estimator`: `CausalMonad::bind` / `CausalMonad::pure`)
  migrate to the trait `.bind()` and process `pure`.
- Math examples already on `.bind(|v, _, _| …)`: add the trait import; signatures are identical.
- `core_examples` (`PropagatingEffectWitness::bind/pure`, stateless): unchanged.
- Tests in `deep_causality_core/tests/types/causal_monad/…` and process tests: update to the trait.

## Tests (close the gap that hid this)

- New: state **threads and updates** through `CausalMonad::bind` on a non-unit `S` (e.g. a counter
  that increments each step), asserting the final state reflects every step — the case the old
  value-only binds silently froze.
- New: error short-circuit yields `None`, preserves state/context/log, does not call `f`.
- Keep the existing Divergence-1 tests (already added).

## Verification

- `cargo build --workspace` clean (no warnings).
- `cargo test --workspace` green.
- Run the migrated examples; numeric results unchanged.

## Blast radius & interdependencies (gitnexus + grep)

gitnexus impact reported **LOW / 0 impacted** for `CausalMonad`, its `bind`, and
`PropagatingProcessWitness`. That reading is **not reliable here**: gitnexus does not resolve Rust
associated-function / trait-method calls (`CausalMonad::pure(x)`, `.bind(...)`) into graph edges
(`CausalMonad` shows only an outgoing `implements MonadEffect5` and zero incoming), and the index is
34 commits stale. The only edge it caught was `causaloid::evaluate -> bind` at confidence 0.5. Treat
gitnexus as a floor; the authoritative map below is from grep.

**Authoritative cross-crate map (the real radius):**

- `CausalMonad` — wide but shallow, almost entirely `CausalMonad::pure(...)`:
  - `deep_causality_core` (definition + uses).
  - `deep_causality`: production `monadic_collection.rs` (5x `CausalMonad::pure`), `lib.rs`
    re-export, and 3 **doc-comment** mentions of `MonadicCausable<CausalMonad>`
    (`causaloid/mod.rs`, `graph_reasoning/mod.rs`) — prose only, **not** a live type bound; plus
    test utils (`test_utils*.rs`, ~16x `CausalMonad::pure`).
  - examples: `mathematics_examples/algebra/geometric_tilt_estimator`, `chronometric_examples/gm_recovery`
    (main + pipeline), `avionics_examples/flight_envelope_monitor/model`.
  - `deep_causality/benches/.../bench_monad.rs`.
- `PropagatingProcessWitness` — tiny: core + `effect_process_witness_duality` example.
- `CausalEffectPropagationProcessWitness` — small: core + `effect_helpers.rs` + the kalman/diffusion
  examples.

**Consequence for the trait rename (important):** today `CausalMonad::pure(x)` resolves because
`CausalMonad` is a *struct* with an associated `pure` and `S`,`C` are inferred. Making `CausalMonad`
a *trait* breaks every bare `CausalMonad::pure(x)` call (a trait associated fn needs a `Self` type).
So the dominant migration cost is **~25 `CausalMonad::pure(x)` sites → a concrete `pure`**
(`PropagatingEffect::pure(x)` / `PropagatingProcess::pure(x)`), spread across core, `deep_causality`
(prod + tests), three example crates, and one bench. The `bind` migration is comparatively small.

**Containment:** no references in `deep_causality_tensor`, `_multivector`, `_topology`, `_sparse`,
`_num`, `_haft`, or `deep_causality_physics`. The math/physics crates depend only on the **stateless**
`PropagatingEffect` `Monad` (which we keep), so the refactor does **not** fan out into the numeric
stack. Radius is wide-but-shallow and confined to core + `deep_causality` + examples/benches; every
edit is mechanical (`pure`/`bind` call-site form), none is a type-bound change.

**Decision (ratified): full elimination of the struct, no `pure` shim.**
`CausalMonad::pure(x)` is used in production (`monadic_collection.rs`, 5 sites, all
`PropagatingEffect`-typed) plus test utils and a bench. It is replaceable 1:1: the carrier's inherent
`pure` already exists and today merely delegates to the struct
(`pub fn pure(v) -> Self { CausalMonad::<S,C>::pure(v) }`), so removing the struct just inlines that
struct-literal construction — byte-identical result. Every `CausalMonad::pure(x)` becomes
`PropagatingEffect::pure(x)` (stateless) or `PropagatingProcess::<T,S,C>::pure(x)` (stateful; none in
production). The `flight_envelope_monitor` reference is a comment, not code.

We deliberately do NOT keep a `CausalMonad::pure` shim. The struct is the source of the
"monad object vs. carrier — which do I use?" ambiguity; a shim would preserve it. End state, one
mechanism with zero wiggle room:

> Work with the carrier — `PropagatingEffect` (stateless) or `PropagatingProcess` (stateful). Both
> implement the `CausalMonad` trait, which provides `bind` (and `pure` via the carrier). There is no
> second monad type to choose.

So `pure` is constructed on the carrier type, `bind` is the trait method, and the ~25 `pure`
call-site rewrites are the API becoming uniform, not churn.

## Risk / non-goals

- Removing `Monad` for the stateful witnesses is safe only if no production generic code requires
  `…ProcessWitness: Monad`. Confirmed: uniform-math uses the stateless witness; `MonadEffect5` has no
  production generic bound. Re-confirm during implementation; if a generic consumer surfaces, it is
  consuming an incorrect bind and must move to `CausalMonad`.
- Non-goal: redesigning haft's generic effect-system traits. They are correct for fixed-channel
  systems and stay.

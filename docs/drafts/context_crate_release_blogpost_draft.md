# Context moves out: `deep_causality_context` and the 0.18 release

*Draft. Publish on release day alongside `deep_causality` 0.18.0.*

---

## Your build will break. Here is the one-line fix.

If your model uses `Context`, add one dependency:

```toml
[dependencies]
deep_causality = "0.18"
deep_causality_context = "0.1"
```

and change where the context types come from:

```rust
// before
use deep_causality::{Context, Contextoid, ContextoidType, Datable, Root};

// after
use deep_causality_context::{Context, Contextoid, ContextoidType, Datable, Root};
```

That is the whole migration for most users. The rest of this post explains why we did not spare
you the edit, and what else changed underneath.

## Why there is no re-export

We could have left a `pub use deep_causality_context::*;` in `deep_causality` and shipped this as a
non-breaking release. Every existing import would still compile. Nobody would have had to touch a
`Cargo.toml`.

We chose the breakage, because it is the point of the split.

Most causal reasoning needs no context. A `Causaloid` that computes a verdict from its input
carries no environment. Ship context inside the main crate and every user pays for it in their
dependency graph, used or not, while `cargo tree` stays silent about which crates actually reason
about an environment. A re-export preserves that silence.

So reaching `Context` now requires saying so in your manifest. The decision becomes explicit,
it lives in a file you can grep, and a dependency search finally answers the question honestly.

## The thing this actually fixes

DeepCausality expresses a causal model two ways. The structural side builds from `Causaloid` and
`CausaloidGraph`. The causal monad, `PropagatingProcess`, threads a value, a state and a context
through a chain of binds.

The monad lives in `deep_causality_core`. The typed `Context` lived one crate above it, in
`deep_causality`. So a monad chain could not name the context type. It had a `Context` channel and
nothing to put in it.

You can see what that cost in our own examples. All five monad-side classical causality examples
declared a private struct and threaded that instead:

```rust
// examples/classical_causality_examples/classical_via_causal_monad/rcm/model.rs, before
#[derive(Clone, Debug, PartialEq)]
pub struct TreatmentContext {
    pub drug_administered: bool,
    pub drug_effect_if_administered: f64,
}
```

Five examples, five bespoke context types, none of them talking to the context machinery sitting
one directory over. Meanwhile the causaloid-side examples next to them used the real thing.

After the split, a crate that depends on `deep_causality_core` and `deep_causality_context`, and
not on `deep_causality` at all, can do this:

```rust
use deep_causality_context::BaseContext;
use deep_causality_core::{PropagatingEffect, PropagatingProcess};

let ctx = BaseContext::with_capacity(1, "treatment world", 2);
let process: PropagatingProcess<f64, (), BaseContext> =
    PropagatingProcess::with_state(PropagatingEffect::pure(145.0), (), Some(ctx));
```

All five examples now carry a real `Context`. None of them declares a context struct any more.

One correction to the issue that opened this work. We first described the monad as untyped where
the causaloid was typed. Checking the code settled it: `Causaloid<I, O, STATE, CTX>` bounds
`CTX: Clone` and nothing else, matching the monad's channel exactly. Both are open generics. What
separated them was reach. `Context` shipped above `deep_causality_core`, so a monad-only consumer
had no way to name it. The extraction restores that reach.

## The symbolic dimension is gone

`Context` used to carry seven type parameters. One of them, `SYM`, existed for symbol nodes:
`BaseSymbol`, `SymbolKind`, the `Symbolic` trait, and a `ContextoidType::Symboid` variant.

Nothing ever constructed one. Searching the workspace turned up `Symboid` only inside the enum's
own match arms and a handful of tests. The parameter survived so that type aliases could fill it.

```rust
// before
Context<D, S, T, ST, SYM, VS, VT>

// after
Context<D, S, T, ST, VS, VT>
```

If you instantiated `Context`, `Contextoid` or `ContextoidType` by hand, drop the fifth argument.
If you used `BaseContext` or `UniformContext`, nothing changes; the aliases absorb it.

`SymbolicTime`, `TimeScale::Symbolic` and the `symbol_spacetime` nodes are unaffected. Those are
temporal and spacetime types that happen to have "symbolic" in the name. They never touched the
`Symbolic` trait.

## `Data<T>` no longer requires `Copy`

This one fixes a defect and breaks an API in the same stroke.

`Data<T>` bounded its payload as `T: Default + Copy + Clone + PartialEq`. Being a context node
demands none of that `Copy`. `Datable` states no bound at all, and `Context` asks for
`D: Datable + Clone`. One place wanted it: the `Adjustable` impl takes an `ArrayGrid`, whose
fixed-size array backing genuinely needs `T: Copy`.

So the bound sat one level too high. It now sits on the impl that needs it, which restated it
anyway. The result:

```rust
// This did not compile before. It does now.
type Series = Data<Vec<f64>>;

let oil_prices = Series::new(1, vec![50.0, 52.0, 55.0, 58.0]);
assert_eq!(oil_prices.get_data().len(), 4);
```

A context can hold a time series. Our Granger causality example needed exactly that, and had
carried a private struct for want of it. It now carries a
`Context<Data<Vec<f64>>, EuclideanSpace, EuclideanTime, EuclideanSpacetime, f64, f64>`.

`Data<T>` itself is no longer `Copy`. If you relied on an implicit copy, you need a `.clone()`.
`BaseContextoid` is unaffected, because it was never `Copy`: `EuclideanSpace` derives only `Debug`,
`Clone` and `PartialEq`, so the derive never applied.

## Smaller things

**`Identifiable` moved to `deep_causality_core`.** `Contextoid` implements it on one side;
`Causaloid`, `Model`, `Inference`, `Assumption`, `Observation` and `ProposedAction` implement it on
the other. Core is the only crate both reach, and it already owned the `IdentificationValue` that
`id()` returns. Both crates re-export it, so `use deep_causality::Identifiable;` still works.

**Ten duplicated type aliases collapsed onto core.** `deep_causality` had its own declarations of
`FloatType`, `NumericalValue`, `IdentificationValue` and seven others, all byte-identical to the
ones in `deep_causality_core`. The duplicates are gone and the live ones are re-exported, so no call
site changed. `deep_causality::TeloidTag` and `deep_causality::TeloidID` were dropped rather than
re-exported; they had no users, and `deep_causality_ethos` declares its own pair.

**`deep_causality_ethos` is breaking for its users too.** Its public API names `Context`, so
anything calling into `EffectEthos` or `TeloidStore` needs the new crate as well. It goes to 0.4.0.

## Versions

| Crate | Version | Kind |
|---|---|---|
| `deep_causality_context` | 0.1.0 | new |
| `deep_causality` | 0.18.0 | breaking |
| `deep_causality_ethos` | 0.4.0 | breaking |
| `deep_causality_core` | 0.12.2 | additive |

<!-- DRAFT NOTE: release-plz computes these from the commit footers. Confirm against the release
     PR before publishing. -->

`deep_causality_context` sits at tier 6 and depends on `deep_causality_core`,
`deep_causality_data_structures`, `deep_causality_uncertain` and `ultragraph`. It does not depend on
`deep_causality`; that edge runs one way, which is how a monad-only consumer reaches it.
`deep_causality_core` keeps its `no-std` feature, because core does not depend on the context crate.

`bazel test //...` runs 1409 targets and every one passes. The context crate accounts for 62 of
them, carrying 431 unit tests and 22 doctests.

## What we did not do

We did not bound the monad's `Context` parameter on a contract trait. That would need the trait to
live in `deep_causality_core`, which cannot see a hypergraph that requires `ultragraph`,
`deep_causality_uncertain` and `std`. It would also need an impl for `()`, and would change every
`CausalFlow`, `PropagatingProcess` and `Causaloid` signature in the workspace. The question stands
on its own, and we will take it separately.

We also did not feature-gate `deep_causality`'s own dependency on the context crate. The same logic
applies to it that applies to you, but the `#[cfg]` surface would have to cover `Model`, the
generative interpreter and the `Base*` aliases, which is a lot of conditional compilation to save
two transitive dependencies.

---

*Issue [#801](https://github.com/deepcausality-rs/deep_causality/issues/801). Questions and
migration trouble: open an issue or find us on Discord.*

# Context moves out: `deep_causality_context` and the 0.18 release

*Draft. Publish on release day alongside `deep_causality` 0.18.0.*

---

## Why?

DeepCausality expresses a causal model in two ways. The structural side builds from `Causaloid` and
`CausaloidGraph`. The causal monad, `PropagatingProcess`, threads a value, a state and a context
through a chain of binds.

The monad lives in `deep_causality_core`. The typed `Context` lived one crate above it, in
`deep_causality`. So a monad chain could not reach the context type. 

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

After the split, a crate that depends on `deep_causality_core` and `deep_causality_context` can do
this:

```rust
use deep_causality_context::BaseContext;
use deep_causality_core::{PropagatingEffect, PropagatingProcess};

let ctx = BaseContext::with_capacity(1, "treatment world", 2);
let process: PropagatingProcess<f64, (), BaseContext> =
    PropagatingProcess::with_state(PropagatingEffect::pure(145.0), (), Some(ctx));
```

All five examples now carry a real `Context`. None of them declares a context struct any more.

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

`SymbolicTime` and `TimeScale::Symbolic` are unaffected. They are a time type and a time scale that
happen to have "symbolic" in the name, and they never touched the `Symbolic` trait. Neither did
`CausalSetSpacetime`, which the removal leaves in place.

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


---

*Issue [#801](https://github.com/deepcausality-rs/deep_causality/issues/801). Questions and
migration trouble: open an issue or find us on Discord.*

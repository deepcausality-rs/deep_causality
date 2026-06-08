## Context

Stage 3 of the Causal Arrow program (build order §6 item 3). The authoritative notes are `openspec/notes/arrow/causal-arrow-generalization.md` (§5 "Missing: strength / the monoidal product", §8 "the builder is the syntax of the Arrow", §3 "the separation must be earned" — strength is where the non-Kleisli multi-input lives).

Current state: `causal-arrow-foundations` (archived) shipped, in `deep_causality_haft`:
- `Morphism<P: HKT2Unbound>` — a **witness-level** typeclass with `identity` + `apply`, and the `FnMorphism` function-pointer carrier. Its design doc (D1) records that composition is omitted because no single concrete carrier exists for it under the no-`dyn` policy, and defers it here.
- `Endomorphism<P>` — iteration/fixpoint combinators over `Morphism`.

The crate also already has witness-level `Bifunctor` (`bimap`/`first`/`second` over `HKT2Unbound` — the `⊗`), `Profunctor` (`dimap` — pre/post-processing), and `Promonad`/`CyberneticLoop`. These are the categorical scaffolding the note says strength "leans on."

Constraints (`AGENTS.md`): `unsafe_code = "forbid"`; static dispatch only, no `dyn`/trait objects; no external crates; no macros in `src/`; one type per module; tests mirror `src/`; 100% coverage; the writing guides bind prose.

## Goals / Non-Goals

**Goals:**
- Make composition **total** and add the **monoidal product** (`first`/`second`/`***`/`&&&`) — the strong-category / Hughes-Arrow surface — under static dispatch, no `dyn`, no macros.
- Express a pipeline as a fluent left-to-right chain (the textual form of a wiring diagram), per §8, **behind a builder that hides the combinator types and the witness machinery** (D5) — the user writes a chain and never names `Compose`/`Split` or `Morphism`.
- Witness the **multi-input** case (two aligned cohorts combined by `***`) that the causal monad's `bind` cannot express, keeping static structure a *parameter* (the §10 invariant) so the result stays non-Kleisli.

**Non-Goals (deferred):**
- Recasting SURD/BRCD as concrete arrows (`causal-arrow-cdl-unification`).
- The **causal process builder** over `PropagatingEffect`/`PropagatingProcess` (the monad/witness-hiding builder that extends today's CDL builder) — captured in `openspec/notes/arrow/causal-process-builder.md`, deferred to `causal-arrow-cdl-unification` since it needs the §10 carrier. This change ships only the carrier-free *generic* arrow builder.
- `ArrowChoice` (`+++`/`|||`) and `ArrowLoop` (feedback) — the latter overlaps `CyberneticLoop`; revisit when a consumer needs them.
- Any change to `Morphism`/`Endomorphism`/`Dual`, or to SURD/BRCD numerics.

## Decisions

### D1 — Value-level `Arrow` (not a witness extension): the only static realization of total composition

The witness-level `Morphism<P>` cannot host composition: `compose: P::Type<A,B> → P::Type<B,C> → P::Type<A,C>` needs the composite to be *one* concrete `P::Type<A,C>`, but composing two closures yields a fresh unnameable closure type; `Box<dyn Fn>` is forbidden; and an enum carrier `Comp(Arrow<A,X>, Arrow<X,B>)` needs an existential `X` Rust cannot express in a flat type. The static way out — standard in Rust arrow encodings — is to make **each combinator return a new concrete type**:

```rust
pub trait Arrow {
    type In;
    type Out;
    fn run(&self, input: Self::In) -> Self::Out;

    fn compose<G>(self, g: G) -> Compose<Self, G>
        where Self: Sized, G: Arrow<In = Self::Out> { Compose::new(self, g) }
    fn first<C>(self)  -> First<Self, C>  where Self: Sized { First::new(self) }
    fn second<C>(self) -> Second<Self, C> where Self: Sized { Second::new(self) }
    fn split<G>(self, g: G) -> Split<Self, G>
        where Self: Sized, G: Arrow { Split::new(self, g) }
    fn fanout<G>(self, g: G) -> Fanout<Self, G>
        where Self: Sized, G: Arrow<In = Self::In>, Self::In: Clone { Fanout::new(self, g) }
}
```

Combinator structs (each a one-field/two-field generic struct implementing `Arrow`):

| struct | `In` | `Out` | `run` |
|---|---|---|---|
| `Id<A>` | `A` | `A` | `x` |
| `Lift<A, B, F>` (lift `Fn(A)->B`) | `A` | `B` | `(f)(x)` |
| `Compose<F, G>` (`f >>> g`) | `F::In` | `G::Out` | `g.run(f.run(x))` |
| `First<F, C>` | `(F::In, C)` | `(F::Out, C)` | `(f.run(a), c)` |
| `Second<F, C>` | `(C, F::In)` | `(C, F::Out)` | `(c, f.run(a))` |
| `Split<F, G>` (`***`) | `(F::In, G::In)` | `(F::Out, G::Out)` | `(f.run(a), g.run(c))` |
| `Fanout<F, G>` (`&&&`) | `F::In` (`Clone`, `= G::In`) | `(F::Out, G::Out)` | `(f.run(x.clone()), g.run(x))` |

This is **total** (`Compose<F,G>` always type-checks when `G::In = F::Out`), **zero-cost** (monomorphized, no allocation, no `dyn`), **macro-free**, and `run(&self, …)` so an arrow is reusable. It is the strong category: `Id`/`Compose` give the category, `First`/`Split` give strength, `Fanout` gives the diagonal.

**Verified encoding (important).** `Lift` must be **`Lift<A, B, F>`** carrying the in/out types via `PhantomData<fn(A) -> B>`, **not** the bare `Lift<F>`: with `type In = A`/`type Out = B` associated and `A`/`B` appearing only in the `F: Fn(A) -> B` bound, Rust rejects the impl with `E0207` ("unconstrained type parameter"), because the type system does not treat `Fn`'s argument as uniquely determined by `F`. Carrying `A, B` in the `Self` type fixes it. The whole design — all combinators plus the builder — was compiled and its laws run as a `rustc` probe with this encoding; it builds clean and the laws pass. The other combinators (`Compose<F,G>`, `First<F,C>`, `Second<F,C>`, `Split<F,G>`, `Fanout<F,G>`) take their `In`/`Out` from `F`/`G`'s associated types and need no extra `PhantomData` beyond `First`/`Second`'s pass-through `C`.

*Alternatives considered.* (a) Witness-based `compose` on `Morphism` — rejected: cannot be implemented (above). (b) A defunctionalized free-category enum — rejected: the existential intermediate type is inexpressible and would force boxing. (c) `Box<dyn Fn>` carriers — rejected: forbidden trait objects, and not zero-cost.

### D2 — Relationship to `Morphism` (witness) and to `Bifunctor`/`Profunctor`

`Morphism` stays the **witness-level interface** (the typeclass a discovery operator instances); `Arrow` is the **value-level algebra** for actually wiring arrows. `Lift::new(f)` lifts a plain `Fn` (subsuming `FnMorphism::apply`), and `Compose` supplies the composition `Morphism` lacked — so `arrow-strength` *completes* foundations additively without altering it. The existing witness `Bifunctor` (the `⊗`) and `Profunctor` (`dimap`) are the categorical justification for `Split`/`Lift`-pre/post-processing; the value-level structs are their concrete, composable counterparts. We do **not** retrofit the witness traits; we add the value-level layer where composition is achievable.

### D3 — Fluent chain = wiring diagram (§8), and where it bites

The provided methods make a pipeline read left-to-right: `a.compose(b).compose(c)`, `f.split(g)`, `f.fanout(g)`. Per §8 this is the textual form of a string diagram, sound by construction. The known tax (§8): a 1-D fluent chain expresses sequential `∘` cleanly but the *parallel* product `⊗` needs explicit combinators (`.split()`, `.fanout()`) — which is exactly what D1 provides. Mis-typed chains leak combinator-struct types into errors; `#[diagnostic::on_unimplemented]` on `Arrow` softens this.

### D4 — Multi-input witness, with the non-Kleisli invariant intact

The witnessing example: an arrow over **two aligned data cohorts** `(Normal, Anomalous) → Fit`, built as `normal_arrow.split(anomalous_arrow) >>> combine` (a `Split` feeding a combiner). This is the shape `bind` cannot express (a genuine product input, not a dynamically-unfolded effect). The **static structure stays a parameter** of the arrow (a graph/lattice captured in the combinator struct's fields or a closure), never a flowing `In` — preserving the §10 invariant that keeps discovery Arrow-but-not-Kleisli. The test models this with simple types (no SURD/BRCD dependency); the real recast is the unification stage.

### D5 — The builder hides the categorical machinery (the user surface)

Binding design direction (see `openspec/notes/arrow/causal-process-builder.md`, confirming §8): **the categorical algebra — witness `Morphism`, the value-level `Arrow` combinators, and later the causal monad — is the *desugared form*; a fluent builder is the user-facing syntax, and the user never names the combinator types or the witness pattern.** This is the ergonomic resolution of the D1/D2 "two arrow notions" concern: a user writes a left-to-right chain and sees neither `Compose<Split<…>, …>` nor `Morphism` — only the builder.

Concretely, this change ships a thin **generic arrow builder** over the value-level `Arrow`:

```rust
pub fn arrow<A, B, F: Fn(A) -> B>(f: F) -> ArrowBuilder<Lift<A, B, F>> { … } // entry point

impl<S: Arrow> ArrowBuilder<S> {
    pub fn then<G>(self, g: G) -> ArrowBuilder<Compose<S, G>> where G: Arrow<In = S::Out> { … }
    pub fn then_fn<C, G: Fn(S::Out) -> C>(self, g: G) -> ArrowBuilder<Compose<S, Lift<S::Out, C, G>>> { … }
    pub fn par<G: Arrow>(self, g: G) -> ArrowBuilder<Split<S, G>> { … }   // ***
    pub fn fanout<G>(self, g: G) -> ArrowBuilder<Fanout<S, G>>
        where G: Arrow<In = S::In>, S::In: Clone { … }
    pub fn build(self) -> S { self.0 }                 // yield the composed Arrow
    pub fn run(&self, input: S::In) -> S::Out { … }    // or apply directly
}
```

The builder threads the (growing) `Arrow` type through `Self` exactly like the CDL typestate builder threads its witness — the types are real but **camouflaged**. `#[diagnostic::on_unimplemented]` on `Arrow` and sealing keep a mis-typed chain legible. The friendly aliases (`.then`/`.par`/`.fanout`) live on the builder; the categorical names (`.compose`/`.split`/`.fanout`) stay on the `Arrow` trait — both are offered (Open Questions). `then` takes a pre-built `Arrow`; `then_fn` lifts a raw closure (so the user need not write `Lift::new`). This whole builder was part of the verified `rustc` probe — `arrow(|x: i32| x + 1).then_fn(|x| x * 2).build()` compiles and runs, naming no combinator struct.

**Why this is in-scope here, and what is deferred.** The *generic* arrow builder is carrier-free and lands in `haft` now — it is the §8 mechanization evidence that the algebra is usable. The **causal process builder** that hides the monad + witness over the `PropagatingEffect`/`PropagatingProcess` carrier (extending today's CDL builder, per the note §2) is **deferred to `causal-arrow-cdl-unification`**, because it requires the §10 carrier rework that does not exist yet. The builder here is written so that stage instantiates the same pattern on `PropagatingEffect`.

**The invariant the builder must not break (§10).** The builder hides *machinery*, not the *separation*: static structure (graph, lattice, metric) is captured as an arrow **parameter** (a `.with_*`/closure capture), never threaded as a flowing `In`/`Out`. The method vocabulary should make parameter-vs-payload syntactically obvious, so the result stays Arrow-but-not-Kleisli.

## Risks / Trade-offs

- **[Two arrow notions — witness `Morphism` and value `Arrow`.]** Risk of confusion/duplication. → Mitigated by D2: distinct, complementary layers (interface vs. composable algebra), documented; `Lift`/`Compose` bridge them. No duplication of behavior — `Morphism` has no composition to duplicate.
- **[Type-inference / error ergonomics.]** Deeply nested `Compose<Split<…>, …>` types surface in errors. → `#[diagnostic::on_unimplemented]`, sealed where helpful, and the fluent methods keep call sites readable; accepted as the standard cost of static arrows.
- **[`run(&self)` vs `run(self)`.]** `&self` makes arrows reusable and composation cheap but requires the lifted `Fn` (not `FnOnce`). → Correct default for pipelines; one-shot arrows are out of scope.
- **[Scope creep toward `ArrowChoice`/`ArrowLoop`.]** → Explicitly deferred (Non-Goals); `CyberneticLoop` already covers feedback for now.
- **[Law coverage, not just types.]** The combinators type-check but must satisfy the arrow laws. → Tests assert the category laws and the strength/exchange laws on concrete arrows, not only that they compile.
- **[Entry-point closure annotation.]** The `rustc` probe confirmed the chain compiles, but the *first* lifted closure needs its input type annotated (`arrow(|x: i32| …)`); later `.then_fn` steps infer from `S::Out`. This is the standard inference limit at a generic entry point, not a design defect. → Document it; the entry closure annotation is a one-token cost and downstream steps are annotation-free.
- **[`Lift<A, B, F>` verbosity / `PhantomData`.]** The carried in/out types make `Lift` a 3-parameter struct. → Hidden from users by the `arrow(f)`/`Lift::new(f)` constructors and the builder; only the type signature carries them. `clippy::new_without_default` on `Id::new`/`Lift::new` is handled by also deriving/implementing `Default` where it applies.

## Migration Plan

Additive only; no migration. New trait + structs behind their own module and re-exports. Rollback is deletion of the new files and registration lines. Bazel test targets are glob-based, so new test files need no `BUILD.bazel` edit.

## Open Questions

- **Module placement (resolved).** A dedicated `src/arrow/` folder, one combinator struct per file (`id`/`lift`/`compose`/`first`/`second`/`split`/`fanout`/`builder`). The `Arrow` trait itself lives in `arrow/mod.rs` (not `arrow/arrow.rs`, which would trip `clippy::module_inception`). All items re-exported from the crate root.
- **Naming (resolved).** The lift combinator is `Lift`, not `Pure`: `haft` already exports a `Pure` trait (the monadic `pure`/`unit`), so `Pure` would collide. `Lift::new(f)` / `arrow(f)` both lift a function.
- **`&&&` input bound.** `Fanout` needs `Self::In: Clone`. Kept a bound on the method/impl (only `fanout` pays it) rather than on the trait.
- **Should `Arrow` supertrait or reference `Morphism`?** No hard supertrait (different shapes: value vs. witness); related by providing `Lift` and documenting the correspondence. Revisit if a consumer needs to treat them uniformly.

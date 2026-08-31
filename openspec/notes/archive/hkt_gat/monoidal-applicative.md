<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Re-found `Applicative` on the monoid structure

**Proposal.** Give `deep_causality_haft` a functor-level monoid structure, derive `Applicative` from
it, mark which monoid a witness is claiming with two first-class markers, and leave `Pure` untouched.
This admits fixed-arity products into the applicative hierarchy without requiring `Clone` of anything
that does not already have it.

**Status.** **Implemented**, via `openspec/changes/archive/2026-08-31-add-lax-monoidal-applicative`. The traits, the
Lean formalization, and adoption on the Cayley-Dickson types and `Dual` are all in. Two things in
this note did *not* ship: the class B and C adoptions remain future work, and the `Traversable` for
`Vec` in §6 finding 5 was implemented, measured and withdrawn — the required bound change takes
admissible inner applicatives from 19 to 3, losing every effect monad. The survey and the
measurements below stand as written; every compiler claim was run, and §9 reproduces them.

**Origin.** `unified_math_gaps.md` §4.1 item E1 asks for `Applicative` on the Cayley-Dickson types
and item E2 for `Functor` and `CoMonad` on `Dual`. Neither can be written, and the reason turned out
to be structural rather than local.

---

## 1. What blocks E1

`Complex<T>` is `{ re: T, im: T }`. Constructing one needs two values of `T`. `Pure::pure` receives
one, by value, bounded only by `Satisfies`, which is `pub trait Satisfies<C: ?Sized> {}`: empty, no
supertrait, granting nothing.

```
error[E0382]: use of moved value: `value`
   { Pair { a: value, b: value } }
```

`Applicative<F>: Functor<F> + Pure<F>`, and `Monad` names `Pure` too, so every fixed-arity product of
arity ≥ 2 is shut out of both. Three escapes were measured and all are closed:

| Attempt | Result |
|---|---|
| `+ Clone` on the impl's method | **E0276**, impl stricter than trait |
| `+ Default` on the impl's method | **E0276** |
| a `Constraint` admitting only `Clone` types | **E0599**, the marker carries no capability |

A sibling capability shaped like `CloneFunctor` does not help either, because `Applicative` names
`Pure` specifically:

```
error[E0277]: the trait bound `W: Pure<W>` is not satisfied
   impl Applicative<W> for W {
```

## 2. Why widening `Pure` is the wrong fix

Adding `T: Clone` to `Pure::pure` was tried end to end. It cascades: `Pure` → `Category::id` (in a
Kleisli category the identity arrow **is** `pure`) → `MonadEffect3/4/5` →
`Adjunction::left_adjunct` → its alias → the effect monads of two downstream crates.

Measured at the far end:

```
73 errors in deep_causality_cfd
 7 errors in deep_causality_discovery
```

and the types being asked to become `Clone` are precisely the wrong ones:

```
12 × study::Swept<Row>       4 × EnsembleMarched<_, _>     3 × study::Cases<T>
 2 × study::Marched<_, _, _> 2 × study::Configured<_, C>   2 × study::Prepared<_, A>
```

These are typestate pipeline states. They carry simulation data and are meant to be **moved** through
the pipeline; making them `Clone` invites silent deep copies of large state and dissolves the
move-only discipline that makes the typestate work. The change was reverted.

## 3. The mathematics that resolves it

An applicative functor is a **monoid object**, in the same sense the algebra crate already uses for
`AddMonoid` and `MulMonoid`. Only the monoidal product differs:

| | monoid object in | structure maps |
|---|---|---|
| `Monad` | (End(𝒞), ∘, Id) | η : Id ⇒ F, μ : F∘F ⇒ F |
| `Applicative` | (End(𝒞), ⊛_Day, Id) | η, μ : F ⊛ F ⇒ F |

"A monad is a monoid in the category of endofunctors" is the familiar half. Applicatives are the same
statement under Day convolution.

**Two presentations, and they come apart in Rust.**

1. **Lax monoidal functor.** η : I → F I, and φ : F A ⊗ F B → F (A ⊗ B).
2. **Day monoid.** η : Id ⇒ F, whose component at `a` is `a -> F a`, which is exactly `pure`.

Going (2) → (1) is free. Going (1) → (2) needs the **diagonal** Δ : A → A ⊗ A, because
`pure(a) = fmap(η(), |()| a)` calls the constant function once per slot.

A category with a diagonal is **cartesian**; one with only ⊗ is merely monoidal. Haskell's category
is cartesian, Δ is always available, and the two presentations coincide, which is why the question
never arises there.

**Rust's move semantics are not cartesian, and `Clone` is the diagonal.**

The crate has already committed to that identification, in code and with a citation.
`haft/src/monoidal/mod.rs` spells Δ as `A: Clone`:

```rust
/// Copy `Δ`: the diagonal `A → A ⊗ A`.
pub fn copy<A: Clone>(a: A) -> (A, A) { (a.clone(), a) }
```

with the module doc citing Fox 1976 for the fact that in a cartesian category every object carries a
unique cocommutative comonoid. So this is not a new finding; it is the existing `SymMonoidal` reading
applied one level up, from values to endofunctors.

It explains every measurement in §1 and §2. `Clone` types inhabit the cartesian part, where `pure`
exists. Single-slot containers never invoke Δ, so `Option`, `Box` and `Result` have `pure` without
it. Fixed-arity products need Δ to fill n slots. CFD's `Swept<Row>` is an object that deliberately
has **no** diagonal, so demanding one broke 73 sites. The earlier failure was not bad luck; it was
asking a category to be cartesian when it correctly refuses.

**`pure` is therefore not part of the monoid structure at all.** In presentation (1) the unit is a
morphism into `F` of the *unit object*, not a map `a -> F a`. `pure` is a cartesian convenience.

## 4. The proposal

Four pieces: a split structure, the right primitive, two promises, and the gate.

### 4.1 Split φ from η

The survey in §6 found the same thing twice, from two directions: **every context-carrying witness in
the workspace has a lawful `zip` and no lawful `unit`.** `pure(a)` at least receives a value;
`unit()` receives nothing and must still name a complex, a grade, a lattice, an adjacency map. Bundle
η with φ and the entire topology crate is excluded. Split them and it is admitted.

```rust
/// φ alone: the semigroupal structure. What `apply` actually needs.
pub trait Semigroupal<F: HKT>: Functor<F> { /* zip_with, zip */ }

/// φ with η: the full lax monoidal structure. What the unit laws need to be stated.
pub trait LaxMonoidal<F: HKT>: Semigroupal<F> {
    /// η : I → F I
    fn unit() -> F::Type<()>;
}
```

The split costs `Option`, `Vec`, `Complex`, `Quaternion` and `Octonion` nothing; they get both.

**On the name.** `Monoidal` is not available and should not be forced. `haft/src/monoidal/` already
holds `SymMonoidal`, the *cartesian* PROP, and its `unit<M: Monoid>() -> M::empty()` is a different
η at a different level. Two `W::unit()` calls reachable from one crate root, meaning different
things, is a trap. `LaxMonoidal` says the level, and it matches the vocabulary
`monoidal_merge.rs` and `MonoidalMerge.lean` already use. Put it in a new `src/lax_monoidal/`
module with a doc-link in each direction rather than inside the 30 lines of cartesian-PROP prose that
open `src/monoidal/mod.rs`.

### 4.2 `zip_with` is the primitive, `zip` is derived

```rust
pub trait Semigroupal<F: HKT>: Functor<F> {
    /// The primitive: φ followed by the payload map, in one step.
    fn zip_with<A, B, C, Func>(fa: F::Type<A>, fb: F::Type<B>, f: Func) -> F::Type<C>
    where
        A: Satisfies<F::Constraint>,
        B: Satisfies<F::Constraint>,
        C: Satisfies<F::Constraint>,
        Func: FnMut(A, B) -> C;

    /// φ : F A ⊗ F B → F (A ⊗ B), derived.
    fn zip<A, B>(fa: F::Type<A>, fb: F::Type<B>) -> F::Type<(A, B)>
    where
        A: Satisfies<F::Constraint>,
        B: Satisfies<F::Constraint>,
        (A, B): Satisfies<F::Constraint>,
    {
        Self::zip_with(fa, fb, |a, b| (a, b))
    }
}
```

This ordering is measured, not stylistic. With `zip` as the primitive the derived `apply` builds
`F::Type<(Func, A)>` and hands it to `fmap`, so the *tuple* must satisfy the witness constraint:

```
error[E0277]: the trait bound `(Func, A): Satisfies<<F as HKT>::Constraint>` is not satisfied
  --> Self::fmap(Self::zip(ff, fa), |(mut f, a)| f(a))
note: required by a bound in `fmap`
```

Adding that bound to `apply` fixes the trait and then leaks: every function generic over the witness
has to restate it or fail at the call site with the same error. `zip_with` never constructs the
tuple, so the bound disappears from `apply` and from callers, and `zip` carries it only where tuples
are genuinely wanted. All 86 real witnesses in the workspace use `NoConstraint`, which has a blanket
`impl<T> Satisfies<NoConstraint> for T`, so the obligation is latent today; it still has to be
written into a generic signature, and it is better absent.

The shape is also already in the crate twice. `MonoidalMerge::merge` in `src/monad/monoidal_merge.rs`
has this signature at the `HKT3Unbound` level, and `LatticeGaugeFieldWitness::zip_with` in the
topology crate has it concretely, returning `Result`. This proposal is their `HKT` sibling.

Worth fixing while nearby: `MonoidalMerge`'s docstring calls itself "the structure map of a **lax
monoidal functor**", but the trait carries only φ. There is no η anywhere in it and none is
derivable. Under the vocabulary above it is `Semigroupal`, and the docstring should say so.

### 4.3 The two promises, as first-class markers

Which monoidal product on End(𝒞) a witness claims a monoid over is not recoverable from the method
bodies, so the type system should carry it. Modelled on `deep_causality_algebra::Associative<O:
Operator>`, whose docstring is explicit that a marker recording an unverifiable promise must never be
handed out by inference:

```rust
/// Marker. Promises μ associates under **composition**: a monoid object in (End(𝒞), ∘, Id).
/// The monad associativity law, `bind(bind(m, f), g) == bind(m, |x| bind(f(x), g))`.
pub trait Compositional<F: HKT>: Monad<F> {}

/// Marker. Promises φ associates under **Day convolution**: a monoid object in (End(𝒞), ⊛, Id).
/// The coherence law `zip(zip(a, b), c) ≅ zip(a, zip(b, c))`, up to reassociation.
pub trait Convolutional<F: HKT>: Semigroupal<F> {}
```

No blanket impls, one line per witness, each line a deliberate assertion about one endofunctor. The
supertraits mean the promise can only be made where the structure exists to promise about.

### 4.4 The gate is a new trait, and `Applicative` is left alone

`Applicative` does not change. The Δ-free derivation goes on a sibling:

```rust
/// The applicative structure that comes from the monoid, with `apply` derived and Δ-free.
pub trait MonoidalApplicative<F: HKT>: Functor<F> + Convolutional<F> {
    fn apply<A, B, Func>(ff: F::Type<Func>, fa: F::Type<A>) -> F::Type<B>
    where
        A: Satisfies<F::Constraint>,
        B: Satisfies<F::Constraint>,
        Func: Satisfies<F::Constraint> + FnMut(A) -> B,
    {
        Self::zip_with(ff, fa, |mut f, a| f(a))
    }
}
```

This is the decision §6 finding 3 forces, and taking it here is what makes the whole proposal
additive. `Applicative<F>: Functor<F> + Pure<F>` keeps its signature, its `A: Clone` bound, its four
McBride-Paterson laws and all 22 of its impls. Nothing in `Traversable`, `Arrow`, the effect system
or the downstream crates has to move, and there is no cutover. `Vec`, `LinkedList` and `VecDeque`
stay exactly as they are: they keep `Applicative`, they do not get `MonoidalApplicative`, and the
`&mut`-capturing closures their callers pass today keep compiling.

A witness may hold both. When it does, the two `apply`s must agree, which is the same coherence
obligation §4.5 describes and is checkable by a law test.

The marker is load-bearing rather than documentary. A witness carrying the full structure but
withholding the promise cannot reach the derived `apply`:

```
error[E0277]: the trait bound `Unpromised: Convolutional<Unpromised>` is not satisfied
help: the trait `Convolutional<Unpromised>` is not implemented for `Unpromised`
note: required by a bound in `Applicative`
```

`Pure` keeps its signature. `Monad` keeps it as a supertrait, which is right: a monad's unit really is
`a -> M a`.

### 4.5 Two routes to `Applicative`, and what the markers are for

The markers earn their place because there are genuinely two ways a witness becomes applicative, and
they differ exactly on whether they need the diagonal.

**The monoidal route is Δ-free.** `zip_with` pairs slot with slot. Nothing is consumed twice, so
`Complex`, `Quaternion`, `Octonion`, `Dual`, `Option`, `Result` and `Box` need no `Clone` at all.

**The monadic route needs Δ.** Every monad induces `ap(ff, fa) = bind(ff, |f| fmap(fa, f))`, which
re-runs the continuation once per function and therefore consumes `fa` many times.

`VecWitness` is on the second route, measurably. Its `apply` is the cartesian list applicative, not a
broadcast and not `ZipList`:

```
apply (as written)          len 6 : [11, 12, 13, 100, 200, 300]
ap induced by its own Monad len 6 : [11, 12, 13, 100, 200, 300]
coherent: true
ZipList                     len 2 : [11, 200]
```

So `Vec` cannot move to a Δ-free `zip_with` without silently going from six results to two and
breaking the applicative-monad coherence law its own `Monad` docstring states. Its `Clone` is
intrinsic to the semantics, not an artifact of the signature, and §6 finding 3 shows it cannot get
the Δ-bound version either. `Vec` keeps the `Applicative` it has and does not gain the derived one;
§4.4 is what makes that a coexistence rather than a loss.

That is what the markers are for. They say which monoid a witness claims, and for a witness claiming
both, the conjunction is a checkable law: the two induced applicatives must agree. The `coherent:
true` line above is that law, measured on `Vec`. `Monad.lean` already proves it as
`haft.monad.applicative_coherence`, and its docstring currently says the recommendation is to state
it in the docs for every witness implementing both traits. The markers turn that recommendation into
a type-level obligation and name exactly who owes it.

## 5. E2 and the `Dual` struct bound

E2 asks for `Functor` and `CoMonad` on `Dual`. It is blocked by one line:

```rust
pub struct Dual<T: Real> { pub re: T, pub du: T }
```

The bound kills the witness at the GAT, before any method exists to argue about. Writing
`impl HKT for DualWitness { type Type<T> = Dual<T>; }` produces:

```
error[E0277]: the trait bound `A: Float` is not satisfied
  --> src/main.rs:12:5          // <-- `type Type<T> = Dual<T>;`
   = note: required for `A` to implement `Real`
note: required by a bound in `Dual`
  --> deep_causality_num_dual/src/dual/dual_number/mod.rs:46:20
```

`Real` is a wide trait, so each component reports separately and the count is not small: **268
errors, every one E0277, every one `required by a bound in Dual`**, on every line of the probe that
names `Dual<T>` for a generic `T`. The same bound also makes `Dual<()>` and `Dual<(A, B)>` ill-formed,
so `unit` and `zip` have no well-formed return type; `Real`'s domain is not closed under × and lacks
the unit object.

**Dropping the bound costs nothing.** `cargo check --workspace --all-targets` reports **0 errors**.
The impl blocks keep their own `impl<T: Real>` bounds, so every arithmetic operation stays exactly as
constrained as it is today. This is the call already made for `Complex<T>` and `Octonion<F>`.

The objection worth taking seriously is that `Dual`'s design rests on nesting, `Dual<Dual<T>>` giving
second derivatives, and that removing the bound might break it. It does not. Nesting rests on
`impl<T: Real + Div<Output = T>> Real for Dual<T>`, which keeps its own bound. Measured with the
struct bound removed:

```
f(3)   = 33     f'(3)  = 29     f''(3) = 18        // x³+2x via Dual<Dual<f64>>
g(2)   = 16     g'(2)  = 32     g''(2) = 48        // x⁴  via Dual<Dual<Dual<f64>>>
Dual<f64>: Real -> Dual { re: 2.0, du: 4.0 }
Dual<Dual<f64>>: Real -> ok
```

With the bound gone, all of E2 compiles and runs, and more than E2:

```
fmap to String : re=<3> du=<1>          // Dual<f64> -> Dual<String>, unrelated payloads
fold           : 4
extract        : 3
zip non-Clone  : Dual { re: (Tag("a"), 1), du: (Tag("b"), 2) }
unit           : Dual { re: (), du: () }
Dual<Vec<u8>>  : Dual { re: [1, 2], du: [3] }
```

`Dual` is a two-field product with no shape, so it sits in exactly the class E1 is about. Its
`zip_with` is componentwise, total, and needs neither `Clone` nor arithmetic. Componentwise is also
the *only* lawful choice: the swap variant `Dual { re: (fa.re, fb.du), du: (fa.du, fb.re) }` fails
the left unit law. The gaps note's caveat still applies to the docstring: this `fmap` maps `re` and
`du` independently, which is the pair functor and carries no chain rule. It is structural traversal
and precision migration, not forward-mode AD.

**Work list for E2**, mirroring the `num_complex` change:

| Step | File | Change |
|---|---|---|
| 1 | `src/dual/dual_number/mod.rs:46` | `pub struct Dual<T: Real>` → `pub struct Dual<T>` |
| 2 | `Cargo.toml` | add `deep_causality_haft`; add it to the `std` / `no-std` feature lists |
| 3 | `BUILD.bazel` | add `//deep_causality_unified_math/deep_causality_haft` to `deps` |
| 4 | `src/extensions/{mod.rs, hkt_dual.rs}` | new: `DualWitness` with `Functor`, `Foldable`, `CoMonad` |
| 5 | `src/lib.rs` | declare and re-export the module |
| 6 | `tests/extensions/` + `tests/BUILD.bazel` | law tests; the Bazel `rust_test_suite` globs `*_tests.rs`, so only the haft dep needs adding |

CI needs no change. The `theorem-map` job reads its crate list from `build/scripts/crates.sh`, which
derives it from the root `Cargo.toml`; the hand-maintained allowlist that used to live in
`formalization.yml` is gone.

One decision belongs to the author. `CoMonad::extend` on a two-slot product has no canonical
comultiplication: `extract` is forced to `re`, but what the `du` slot should see is a modelling
choice. The probe used the shifted dual, so `extend` reads the ε channel; the alternative hands both
slots the same focus and makes `extend` degenerate. Choose deliberately and write it into the
docstring.

## 6. The refactor inventory

57 witnesses were surveyed across the workspace and each finding was then given to an independent
agent told to refute it. Grouped by what the refactor actually requires:

| Class | Witnesses | What `zip_with` needs |
|---|---|---|
| **A. Canonical, Δ-free** | `Complex`, `Quaternion`, `Octonion`, `Dual`†, `Option`, `Result<E>`, `Box`, `Ident`, the three `MyEffectHktWitness` fixtures | Componentwise or single-slot. Total, no `Clone`, no bounds. Gains `Applicative` where it has none |
| **B. Partial only** | all 12 topology witnesses, `CsrMatrix`, `DenseMatrix`, `DenseVector`, `BTreeMap`, `HashMap`, `Cofree` | φ is total only on a shared context or matching shape. Needs a `Result` channel or a panic |
| **C. Multiplicity, Δ-bound** | `Vec`, `LinkedList`, `VecDeque` | No lawful Δ-free instance exists. See below |
| **D. No lawful φ** | `CausalTensor`, `CausalTensorTrain`, `CausalMultiField`, `GaugeField`, `LatticeGaugeField`, `CurvatureTensor`, `DeRhamSharpIso`, `Free`, `Tuple2/3`, `ResultUnbound` | Cannot be written lawfully at all |

† after the struct bound comes off.

Class A is the win and it is clean. For the Cayley-Dickson family the canonicity is provable rather
than asserted: `F(A) = A^S` for a finite index set `S`, so `F(())` is a singleton and η is forced,
and by Yoneda every natural φ is `φ(fa, fb)_s = (fa_{u(s)}, fb_{v(s)})` for fixed endofunctions `u,
v` of `S`; the two unit laws force `u = v = id`. Componentwise pairing is the unique lawful φ, and
associativity follows. The "a choice per witness" cost does not bite here at all. The resulting
applicative is Reader on a finite index set, whose `pure` is the constant map, which is Δ. The thesis
of §3, confirmed on the types it was written for.

**Six findings from the survey that change the design.**

1. **The η/φ split, above.** Without it, class C is excluded entirely and the witnesses in it are
   pushed into writing an unlawful `unit`. That failure mode has already happened once in this
   repo: the deleted `GaugeField` `MonoidalMerge` impl, where five tests asserted the defect as the
   specification.

2. **Class C wants `Result`, and the crate already agrees.** `Topology::cup_product` returns
   `Err(TopologyError::GenericError("Complex Mismatch"))` and `LatticeGaugeFieldWitness::zip_with`
   returns `Err(TopologyError::LatticeGaugeError)`. A `zip_with` returning `F::Type<C>` has no error
   channel, so adopting it as specified forces the panic branch on witnesses whose existing binary
   operations deliberately chose `Result`. That is a regression in the crate's own terms, and it
   means `apply` cannot be a plain provided method for class C.

3. **Class C has no lawful Δ-free instance, and this is the hard case.** `Vec` looks like a free
   choice between the cartesian product and `ZipList`; it is neither. `ZipList` has no unit, because
   the unit of a positional zip is the infinite repeat and a finite `Vec` cannot represent it. The
   cartesian product is pinned as the only lawful applicative by `haft.monad.applicative_coherence`,
   and it needs `Func: Clone`, which the Δ-free signature forbids and which an impl may not re-add
   (E0276, the same wall as §1). So the menu is empty rather than long. Two measured consequences,
   both on `VecWitness`:

   ```
   non-Clone FnMut Func accepted today : [11, 21]        // closure captured &mut log
   closure state observed              : [10, 20]
   stateful FnMut, today               : [11, 22, 33]    // one f reused across args
   stateful FnMut, cloned per pair     : [11, 12, 13]    // f reset per pairing
   ```

   The first is a hard compile break: a closure capturing by `&mut` is `FnMut` and not `Clone`, and
   today's `apply` accepts it. The second is a silent behaviour change on code that keeps compiling.
   Neither is excused by the "laws are stated for pure functions" caveat, which excuses a law
   failure, not a change in the result of a legal call. This is the finding that decides §4.4: the
   derived `apply` goes on its own trait, and class C keeps the `Applicative` it already has.

4. **Broadcast and φ are different functions.** `apply` can broadcast one `Func` over n arguments
   without `Clone`, because one owned `FnMut` is called n times. `zip_with` cannot, because n
   pairings need n owned `Func`s, which is Δ. `ManifoldWitness`, `CausalTensorWitness` and
   `CsrMatrixWitness` all broadcast today. For them the derived `apply` is a *different* function,
   so a migration that leaves hand-written `apply` bodies in place leaves two definitions coexisting
   and disagreeing, with which one a caller gets depending on whether that witness overrode the
   default. That is worse than either alone.

5. **`zip_with` unblocks `Traversable` for `Vec`, which `apply` cannot express.** This is an
   independent win and it was measured both ways. The usual list `sequence` folds an accumulator
   through the inner applicative, which puts a *function* inside `M`:

   ```rust
   acc = M::apply(M::fmap(acc, |v| move |a| { v.push(a); v }), m_a)
   ```

   `Applicative::apply` then requires the anonymous closure type to satisfy `M::Constraint`:

   ```
   error[E0277]: the trait bound `{closure@hkt_vec_ext.rs:150:17}: Satisfies<<M as HKT>::Constraint>`
                 is not satisfied
   ```

   `sequence` cannot declare that, and an impl may not add the bound itself (E0276). It is the same
   shape as the tuple problem in §4.2: the derivation manufactures an intermediate payload type the
   constraint has no way to admit. The `zip_with` fold never puts a function inside `M`, and it
   compiles and passes:

   ```rust
   acc = M::zip_with(acc, m_a, |mut v, a| { v.push(a); v });
   ```
   ```
   test probe_seq::probe_tests::vec_sequence_via_zip_with ... ok
   ```

   So `Vec<Option<A>> -> Option<Vec<A>>`, the example the `Traversable` docstring claimed for years
   behind a `rust,ignore` fence, becomes writable. Note this does not need `Vec` to be
   `MonoidalApplicative`; it needs the *inner* `M` to be `Semigroupal`, and `Option` and `Result`
   both are. Finding 3 and this one are compatible.

6. **`CausalMultiFieldWitness` must not get a `zip`, on soundness grounds.** Its HKT layer is
   `transmute_copy` plus `ptr::read` under a documented contract that the payload type must match
   the concrete `T`. Instantiating the element type at a tuple violates that contract by
   construction. Today's `apply` panics unconditionally, which is loud and safe; a derived `apply`
   would trade the panic for undefined behaviour. This one wants a separate look regardless of the
   proposal.

**The `Clone` bound on `apply`.** No impl in the workspace calls `.clone()` on the `A` value inside
`apply` except `VecWitness` and `LinkedListWitness`, which need it for the cartesian product.
`ManifoldWitness` clones the complex and metric, not the payload. So dropping `A: Clone` from the
trait signature is free for every other impl, and it also lets `Traversable::sequence` drop the
`A: Clone` it inherits.

**The migration is additive, because §4.4 puts the derivation on a new trait.** Had
`Applicative`'s own supertraits changed from `Pure<F>` to `Convolutional<F>`, it would have touched
all 22 impls at once and silently dropped `Pure` from scope for `Traversable`, whose `sequence` calls
`M::pure(None)` and gets it today only through `Applicative: Pure`. On the `MonoidalApplicative`
route none of that happens: `Applicative` is untouched, `Traversable`'s bound stays as written, and a
witness opts in one at a time by writing `zip_with` and the marker.

## 7. What it costs

**Two applicative traits in the crate.** `Applicative` and `MonoidalApplicative` coexist, and a
witness holding both owes a law test that they agree. That is one more concept in the vocabulary,
and it is the price of not breaking class C. It buys an additive migration in exchange.

**A choice per witness, for class B.** `zip_with` forces the implementer to say which applicative
they mean, and for the shape-carrying witnesses also which precondition failure means. Today that
choice is buried in an `apply` body; here it is named where law tests can pin it. Class A has no
choice to make, by the Yoneda argument above.

**The statable laws change, and coherence alone says less than the four laws do.** The applicative
laws become associativity of φ and the unit laws through η. That is not simply a re-basing: the
monoid coherence conditions do **not** pin `Vec`'s applicative, because both the function-major and
the argument-major cartesian products satisfy all of them. What selects the one `Vec` actually
implements is `haft.monad.applicative_coherence`, a law that lives outside the monoidal structure
entirely. Rewriting law tests against coherence would therefore drop the only constraint that
currently identifies `Vec`'s applicative, so the monad coherence law has to be kept and stated
alongside, not replaced.

**One line per witness for each marker**, times 22 `Applicative` and 17 `Monad` impls.

**A name and a module.** `LaxMonoidal` in `src/lax_monoidal/`, not `Monoidal` in `src/monoidal/`, for
the reasons in §4.1.

## 8. The Lean formalization

`lean/DeepCausalityFormal/Haft/` carries 30 files and 52 `THEOREM_MAP` ids, and CI enforces that
every id appears in both a Lean proof and a Rust witness test. The proposal touches four.

**A new file, `LaxMonoidal.lean`.** The `haft.monoidal.*` prefix is taken, by `SymmetricMonoidal.lean`:
`haft.monoidal.comonoid_laws`, `haft.monoidal.merge_monoid_laws`, `haft.monoidal.symmetry`. This is
the same collision as the Rust side and wants the same resolution. Reusing the prefix would also make
the `theorem-map` job's `grep -Fl "$id" lean/THEOREM_MAP.md` match the wrong row and hide a missing
entry. Use `haft.lax_monoidal.*`, and state three theorems over the `Option` carrier the existing
files already use:

| id | statement |
|---|---|
| `haft.lax_monoidal.naturality` | `zip(fmap(fa, f), fmap(fb, g)) = fmap(zip(fa, fb), f × g)` |
| `haft.lax_monoidal.assoc` | `zip(zip(fa, fb), fc) ≅ zip(fa, zip(fb, fc))`, modulo the associator |
| `haft.lax_monoidal.unit_laws` | `zip(unit(), fa) ≅ fa` and `zip(fa, unit()) ≅ fa`, modulo the unitors |
| `haft.lax_monoidal.apply_agreement` | for a witness holding both traits, the derived `apply` equals the hand-written one |

State naturality first. It is what makes φ a natural transformation rather than an arbitrary binary
function, and it is the law an implementer is most likely to violate by reaching for a shape-dependent
shortcut. Keep `assoc` on `Semigroupal` and `unit_laws` on `LaxMonoidal`, matching the Rust split, so
a class C witness can discharge the first two without owing the third.

**`Applicative.lean` keeps every law it has and gains one agreement theorem.** Because §4.4 leaves
`Applicative` alone, none of the four McBride-Paterson laws move. What is new is the obligation on a
witness holding both traits, that the derived `apply` and the hand-written one coincide:

```lean
theorem opt_apply_from_zip_with (fab : Option (A → B)) (fa : Option A) :
    optApply fab fa = optZipWith fab fa (fun f a => f a) := by
  cases fab <;> cases fa <;> rfl
```

Give it its own id, `haft.lax_monoidal.apply_agreement`, since it is a statement about the pair
rather than about either trait. That file also reports a deviation worth clearing while nearby: the
Rust docstring lists three laws and omits Composition.

**`Monad.lean` needs no change and gains a purpose.** `haft.monad.applicative_coherence` already
proves `apply f_ab f_a = bind f_ab (fun f => fmap f fa)`, and its docstring already says the
recommendation is to state it for every witness implementing both traits. Under §4.5 that becomes the
law owed by any witness carrying both markers, and the Rust side can finally name them.

**`Traversable.lean` and `Pure.lean` are untouched.** `Pure` keeps its signature and its naturality
theorem; `Traversable` keeps its laws, and only its Rust bound moves.

Two mechanical constraints, both in the file headers already and both easy to trip. Keep the new file
self-contained with no imports, as every other Haft file is, so it typechecks standalone with bare
`lean`; if an import ever becomes necessary, mirror it into `cache_roots` in `MODULE.bazel` or the
tree-shaken Mathlib download will not contain it and the build fails on the missing module. Bazel
needs no edit: `lean/BUILD.bazel` globs `DeepCausalityFormal/{ns}/**/*.lean` per namespace, so a new
file under `Haft/` is picked up on its own. The Rust witness goes in
`deep_causality_haft/tests/formalization_lean/`, beside `applicative_tests.rs` and `monad_tests.rs`;
without it CI fails the id as `MISSING Rust witness`.

## 9. Reproducing

Four probe crates under `scratchpad/`, each standalone with path dependencies on the real crates.

```rust
// zipwith_probe -- the full stack against real haft witnesses
pub trait Semigroupal<F: HKT>: Functor<F> {
    fn zip_with<A, B, C, Func>(fa: F::Type<A>, fb: F::Type<B>, f: Func) -> F::Type<C>
    where Func: FnMut(A, B) -> C, /* … */;
}
pub trait Convolutional<F: HKT>: Semigroupal<F> {}
pub trait Applicative<F: HKT>: Functor<F> + Convolutional<F> {
    fn apply<A, B, Func>(ff: F::Type<Func>, fa: F::Type<A>) -> F::Type<B>
    where /* … */ { Self::zip_with(ff, fa, |mut f, a| f(a)) }
}
```

`cargo run` in each. `zipwith_probe` prints the Δ-free `apply` over `Option`, `Vec` and a
fixed-arity product, including a payload that is neither `Clone` nor `Copy`; commenting the
`Convolutional` impl back in on `Unpromised` reproduces the gate error in §4.4. `dual_probe`
reproduces §5 in both directions: with `Dual`'s struct bound removed it prints the witness working
and the nested second derivatives, and with the bound restored it fails with 268 errors. `vec_sem`
reproduces the cartesian-versus-`ZipList` measurement, the monad coherence check, and the two class C
breakages: a `&mut`-capturing closure accepted by today's `apply`, and the stateful-`FnMut` result
change.

The `Pure + Clone` cascade in §2 is reproduced by adding the bound to `Pure::pure` and following the
errors through `Category::id`, `MonadEffect3/4/5`, `Adjunction::left_adjunct` and its alias, then
building the workspace rather than a single crate. Stopping at the first failing crate reports four
errors and hides the eighty.

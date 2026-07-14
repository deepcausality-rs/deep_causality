## Context

`deep_causality_haft` carries the free monad `Free<F, A> = Pure a | Suspend (f (Free f a))` with
`bind`/`map`/`fold` as **inherent methods** and `FreeWitness<F> : HKT + Pure` (`free_monad.rs`) —
the witness deliberately does **not** implement the `Monad` trait, because `Monad::bind`'s `FnMut`
cannot express the `Fn + Clone` a multi-hole functor node needs (one continuation copy per hole).
The crate also carries the `CoMonad` trait (`comonad.rs`), the `Functor` machinery, and
`Comonad.lean` proving the three comonad laws (Uustalu–Vene 2008) on the Env comonad.

Two things are missing. First, the **dual of `Free`** — the cofree comonad `Cofree<F, A>` — has no
carrier, so the `CoMonad` trait and `Comonad.lean` have no free/cofree instance to pair with `Free`.
Second, `Free` has **no `Eq`/`Debug`**: its recursive child sits under a GAT projection
`F::Type<Box<Free<F, A>>>`, and the derive/projection-bound path overflows the trait solver.

Both are requested by a downstream consumer that keeps its own `FreeMnd`/`CofreeCmnd` pair; the
gap report is that the crate's `Free` cannot be adopted without an `Eq`/`Debug` and without
splitting the free/cofree pair.

## Goals / Non-Goals

**Goals:**
- Add `Cofree<F, A>` (the cofree comonad) with `extract`/`map`/`extend`/`unfold` and
  `CofreeWitness<F> : HKT`, mirroring `Free`'s inherent-surface design.
- Add `EqFunctor`/`DebugFunctor` capability traits and opt-in generic `PartialEq`/`Eq`/`Debug` for
  `Free<F, A>` and `Cofree<F, A>`.
- Prove the `Cofree` comonad laws in Lean (bare-`lean`, THEOREM_MAP-bound, Rust witness), reusing
  `Comonad.lean`'s law statements; test every claim in Rust (Bazel-registered).
- Stay strictly additive and opt-in: existing `Free` code and its `fold`-based comparison approach
  are unchanged; new instances appear only for witnesses that implement the capability.
- Preserve crate invariants: `unsafe_code = "forbid"`, no `dyn`, macro-free `/src`, no external
  deps, no-std with `alloc`.

**Non-Goals:**
- A `CoMonad` **trait** impl for `CofreeWitness` (needs a by-ref / `CloneFunctor` capability; D4).
- `CloneFunctor` / `HashFunctor` capabilities (same Route-B pattern, additive later if needed).
- Changing `Free`'s representation, its inherent surface, or its "compare by folding" story.
- A free⊣cofree **adjunction** formalization (heavier; not required by the pair the consumer needs).

## Decisions

**D1 — `Cofree<F, A>` as the dual product carrier, `alloc`-gated (H1).**
```rust
pub struct Cofree<F, A>
where F: HKT<Constraint = NoConstraint>,
{ head: A, tail: F::Type<Box<Cofree<F, A>>> }   // fields private (public-type convention)
```
`Cofree` is a struct (product) where `Free` is an enum (coproduct) — the exact categorical dual.
Fields are private per the public-type field-visibility convention; construction and inspection go
through `new(head, tail)`, `head(&self) -> &A`, `tail(&self) -> &F::Type<Box<Cofree<F, A>>>`, and
`into_parts(self)`. `Box` is `alloc::boxed::Box`, so the whole module is `#[cfg(feature = "alloc")]`
like `free_monad`. Rationale: heap indirection on the recursive field is required (`Cofree` is
self-referential), identical to `Free`.

**D2 — Comonad surface as inherent methods, dual to `Free` (H1).** Provide, as inherent methods on
`Cofree<F, A>` where `F: HKT<Constraint = NoConstraint> + Functor<F>`:
- `extract(&self) -> A where A: Clone` — the counit ε (dual of `pure`); reads `head`.
- `map<B, Fun: Fn(A) -> B + Clone>(self, f: Fun) -> Cofree<F, B>` — the functor action; `Fn + Clone`
  for the same one-copy-per-hole reason `Free::map` requires it.
- `extend<B, K: Fn(&Cofree<F, A>) -> B>(self, k: &K) -> Cofree<F, B>` — cobind, the dual of `bind`:
  `extend k w = k(w) :< fmap (extend k) (tail w)`. `k` is passed by reference and threaded through
  every hole, so — unlike `Free::bind` — no `Clone` on `k` is needed.
- `unfold<X, C: Fn(X) -> (A, F::Type<X>)>(seed: X, coalg: &C) -> Cofree<F, A>` — the anamorphism,
  dual of `Free::fold`: `unfold(x) = let (a, fx) = coalg(x) in a :< fmap unfold fx`. This is the
  generator of `Cofree` values.

`duplicate(self) -> Cofree<F, Cofree<F, A>>` is `extend(&|w| w.clone())` and needs the tree to be
`Clone`; it is provided only when `Cofree<F, A>: Clone` (i.e. under the `Eq`/`Debug`-style clone
capability) and is otherwise omitted — the four methods above are the required surface. Rationale:
this mirrors `Free` exactly — the real comonad surface is inherent because the `CoMonad` trait's
`FnMut`/borrow signature cannot express what the encoding needs.

**D3 — `CofreeWitness<F> : HKT` only (H1).** `CofreeWitness<F>(PhantomData<F>)` with
`type Type<T> = Cofree<F, T>` and `type Constraint = NoConstraint`, mirroring `FreeWitness` (which
implements `HKT` + `Pure` but not the `Functor`/`Monad` traits). No witness-level `Functor` impl:
the functor action needs `Fn + Clone`, which the `Functor` trait's `FnMut` cannot carry — the same
reason `FreeWitness` omits it. Rationale: consistency with the established `Free` design; the
inherent `map` is the functor action.

**D4 — No `CoMonad` trait impl for `CofreeWitness` (Non-Goal, recorded).** `CoMonad::extend` is
`fn extend<A, B, Func>(fa: &F::Type<A>, f: Func) -> F::Type<B> where A: Clone, Func: FnMut(&F::Type<A>) -> B`
— it takes the context by **reference** and must rebuild it. Rebuilding a `Cofree` from a borrow
requires cloning the `F`-structure of children, which needs a `CloneFunctor` capability
(`Self::Type<T>: Clone when T: Clone`) not in this change. So the borrow-based `CoMonad` trait is
not implemented for `CofreeWitness`; the comonad *laws* are proved for the inherent by-value ops
(D2). `extract` alone fits the trait signature and MAY be exposed later as a thin partial step when
the clone capability lands. Rationale: force-fitting the trait would either pull in `CloneFunctor`
(scope creep) or silently clone; neither is warranted for the pair the consumer needs.

**D5 — `EqFunctor` / `DebugFunctor` capability traits (Route B) (H2).**
```rust
pub trait EqFunctor: HKT<Constraint = NoConstraint> {
    fn eq_type<T: PartialEq>(a: &Self::Type<T>, b: &Self::Type<T>) -> bool;
}
pub trait DebugFunctor: HKT<Constraint = NoConstraint> {
    fn fmt_type<T: core::fmt::Debug>(fa: &Self::Type<T>, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result;
}
```
A witness that opts in supplies the comparison / formatting of its `Type<T>` given `T: PartialEq` /
`T: Debug` — the same "witness supplies the operation" shape as `Functor::fmap`. Scoped to
`Constraint = NoConstraint` because `Free`/`Cofree` require it. Rationale: this is what breaks the
solver cycle (D6); it makes the instances opt-in per witness, leaving existing `Free` code
untouched.

**D6 — Generic `PartialEq`/`Eq`/`Debug` for `Free` and `Cofree`, cycle-free (H2).**
```rust
impl<F: EqFunctor, A: PartialEq> PartialEq for Free<F, A> {
    fn eq(&self, o: &Self) -> bool { match (self, o) {
        (Free::Pure(a), Free::Pure(b)) => a == b,
        (Free::Suspend(x), Free::Suspend(y)) => F::eq_type(x, y),   // recurse via the method
        _ => false,
    }}
}
impl<F: EqFunctor, A: Eq> Eq for Free<F, A> {}
impl<F: DebugFunctor, A: Debug> Debug for Free<F, A> { /* Pure(..) / Suspend(..) via F::fmt_type */ }
```
and the structurally identical three impls for `Cofree` (`head == o.head && F::eq_type(tail, ..)`).
The recursion runs through `F::eq_type::<Box<Free<F, A>>>`, whose obligation `Box<Free<F, A>>:
PartialEq` discharges against **this** impl's stable bounds (`F: EqFunctor`, `A: PartialEq`) — never
through an `F::Type<..>: PartialEq` projection bound — so it terminates exactly as a plain recursive
`enum List { Nil, Cons(i32, Box<List>) }` does. Verified compiling and running on `rustc 1.97.0` for
`Free`/`Cofree` over `OptionWitness`/`VecWitness`; the naive projection-bound path was verified to
overflow (`E0275`), and the downstream-orphan path to fail (`E0117`).

**D7 — Built-in witness capability impls (H2).** Provide `EqFunctor`/`DebugFunctor` for the crate's
built-in single-hole `Functor` witnesses that can carry `Free`/`Cofree` — `OptionWitness`,
`VecWitness`, `BoxWitness`, and the other single-hole functors (`LinkedList`/`VecDeque` witnesses).
Bodies are one-liners (`a == b`, `write!(f, "{:?}", fa)`) since the underlying container is
`PartialEq`/`Debug` when `T` is. A downstream functor opts in with the same one impl each. Rationale:
covers the crate's own carriers and documents the opt-in pattern by example.

**D8 — `Eq`/`Debug` carry no new categorical law (H2).** The instances are lawful derived
instances: given the witness's `eq_type` is a faithful equivalence (true for built-in `==`), the
generated `PartialEq` on `Free`/`Cofree` is reflexive/symmetric/transitive by structural induction.
This obligation is discharged by Rust property tests, not a Lean theorem — there is no new
categorical law to prove (the categorical content of this change is the `Cofree` comonad laws, D9).
No `formalization.yml` allowlist change: `haft` already carries Lean witnesses. Rationale: match the
project's law/witness discipline where a law exists, without manufacturing a vacuous one.

**D9 — Lean formalization of the `Cofree` comonad laws (H1).** `DeepCausalityFormal/Haft/Cofree.lean`
proves the three comonad laws for `Cofree` and the `unfold` computation rule. Following
`FreeMonad.lean`: Lean's positivity checker rejects `head :< f (Cofree f a)` for a *variable*
functor `f`, so the laws are proved over a **representative** functor (`f a = E × a`, and/or
`List`), with the proof depending only on `f`'s functor laws — discharging the general result as
`FreeMonad.lean` does for `Free`. The three laws reuse `Comonad.lean`'s statements
(`extend extract = id`; `extract ∘ extend f = f`; associativity). THEOREM_MAP ids
`haft.cofree.comonad_laws` and `haft.cofree.unfold`, each with a Rust witness in
`tests/formalization_lean/`, registered in `DeepCausalityFormal.lean`; bare-`lean` typecheck; house
style (self-contained, textbook citation — Uustalu–Vene 2008; Ghani–Uustalu–Vene on `Cofree` /
tree comonads — plus deviation notes).

## Risks / Trade-offs

- **[Positivity — Lean]** the inductive checker rejects a variable-functor `Cofree`. → Prove over a
  representative functor and argue functor-law-only dependence, exactly as `FreeMonad.lean` does
  (D9). Coinductivity of the true `Cofree` is not needed: the representative carrier is a finite
  inductive tree.
- **[Coinductive `Cofree` vs strict Rust]** `Cofree f a` is infinite for a functor with no empty
  shape. → `Cofree` is finitely constructible over functors that bottom out (`Option`, `Vec`, list
  functors) — the annotated-tree use the consumer needs; `unfold` terminates there. Document the
  finiteness precondition on `unfold`.
- **[`CoMonad` trait not implemented for `CofreeWitness`]** the linkage to the existing `CoMonad`
  trait is by law, not by a trait impl. → Recorded as D4; the comonad laws are proved for the
  inherent ops, and a trait impl can follow additively once a `CloneFunctor` / by-ref-map capability
  exists. No consumer requirement depends on the trait impl.
- **[Capability surface growth]** `EqFunctor`/`DebugFunctor` are two more witness traits. → They are
  the minimal set the gap report names (`Eq` + `Debug`); `Clone`/`Hash` follow the identical pattern
  and are deliberately out of scope until a consumer needs them.
- **[Doc drift in `free_monad.rs` NOTE]** the NOTE says "no `PartialEq`/`Debug`". → Update it to
  keep the correct `#[derive]`-is-impossible statement and point at the opt-in capability route; a
  doc-only edit, no behaviour change.

## Migration Plan

Additive. No existing type, trait, or requirement changes. Downstream consumers gain `Cofree` and
the opt-in `Eq`/`Debug` instances; a consumer's own functor witness opts in with one `EqFunctor` and
one `DebugFunctor` impl (one-liners). No deprecations. `Free`'s existing `fold`-based comparison
remains valid and unaffected.

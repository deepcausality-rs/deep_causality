## Context

`deep_causality_haft` carries `Free<F, A>` (free monad) and `Cofree<F, A>` (cofree comonad) as
heap-recursive carriers whose recursive child sits under the GAT projection `F::Type<Box<..>>`. The
`2026-07-16-haft-cofree-and-eq-debug` change added `EqFunctor`/`DebugFunctor` — witness-capability
traits over `HKT<Constraint = NoConstraint>` — to give both carriers opt-in `PartialEq`/`Eq`/`Debug`
without the trait-solver overflow (`E0275`) that a `#[derive]` or a projection-bound field hits. That
change deferred three things to a "same Route-B pattern, additive later" follow-up: a `CloneFunctor`
capability, `Cofree::duplicate`, and a `CoMonad` **trait** instance for `CofreeWitness`. Issue #718
requests them; a downstream consumer (catgraph) needs direct `Clone` on the carriers.

## Goals / Non-Goals

**Goals:**
- Add `CloneFunctor` and opt-in generic `Clone` for `Free<F, A>` and `Cofree<F, A>`, cycle-free via
  `clone_type`, with capability impls for the built-in functor witnesses.
- Add the inherent `Cofree::duplicate` (D2), unblocked by `Cofree<F, A>: Clone`.
- Add the by-reference `CoMonad` trait instance for `CofreeWitness` (D4), and the `Functor` instance
  its supertrait bound entails.
- Stay code-additive and opt-in: existing `Free`/`Cofree` code, the inherent comonad surface, and the
  `fold`/`eq_type`-based stories are unchanged; new instances appear only for witnesses that opt in.
- Preserve crate invariants: `unsafe_code = "forbid"`, no `dyn`, macro-free `/src`, no external deps,
  no-std with `alloc`.

**Non-Goals:**
- A `HashFunctor` (or further capabilities) — same Route-B pattern, additive later if a consumer needs
  it.
- Changing `Free`/`Cofree`'s representation or their inherent surface, or the "compare by folding"
  story.
- A new Lean theorem for `Clone` — it is a derived structural instance with no categorical law (D6),
  exactly as `Eq`/`Debug` were.
- A free⊣cofree adjunction formalization.

## Decisions

**D1 — `CloneFunctor` capability trait (Route B).**
```rust
pub trait CloneFunctor: HKT<Constraint = NoConstraint> {
    fn clone_type<T: Clone>(fa: &Self::Type<T>) -> Self::Type<T>;
}
```
A witness that opts in supplies the structural clone of its `Type<T>` given `T: Clone` — the same
"witness supplies the operation" shape as `EqFunctor::eq_type`/`DebugFunctor::fmt_type`. Scoped to
`Constraint = NoConstraint` because `Free`/`Cofree` require it. Placed in
`src/functor/clone_functor.rs` beside its twins, exported from `lib.rs`, `core`/`alloc`-free (so it
compiles in every feature configuration, like the `OptionWitness` impl).

**D2 — Generic `Clone` for `Free` and `Cofree`, cycle-free (H1).**
```rust
impl<F: CloneFunctor, A: Clone> Clone for Free<F, A> {
    fn clone(&self) -> Self {
        match self {
            Free::Pure(a) => Free::Pure(a.clone()),
            Free::Suspend(x) => Free::Suspend(F::clone_type(x)),   // recurse via the method
        }
    }
}
impl<F: CloneFunctor, A: Clone> Clone for Cofree<F, A> {
    fn clone(&self) -> Self {
        Cofree::new(self.head().clone(), F::clone_type(self.tail()))
    }
}
```
The recursion runs through `F::clone_type::<Box<Free<F, A>>>`, whose obligation
`Box<Free<F, A>>: Clone` discharges against **this** impl's stable bounds (`F: CloneFunctor`,
`A: Clone`) — never through an `F::Type<..>: Clone` projection bound — so it terminates exactly as a
plain recursive `enum List { Nil, Cons(i32, Box<List>) }` does, identical to `PartialEq` (verified
compiling and running on the crate's toolchain).

**D3 — Built-in witness capability impls (H1).** `CloneFunctor` for `OptionWitness`, `VecWitness`,
`BoxWitness`, `LinkedListWitness`, `VecDequeWitness` — one-line delegations `fa.clone()` (the
container is `Clone` when `T` is). `BoxWitness` spells the parameter through the HKT projection
(`&<Self as HKT>::Type<T>`) to avoid `clippy::borrowed_box`, matching its `EqFunctor` impl. A
downstream functor opts in with the same one impl.

**D4 — `Cofree::duplicate` (D2 of the prior change) and the `CoMonad`/`Functor` trait instances for
`CofreeWitness` (H2). This reverses a prior decision.**

`Cofree::duplicate` is the inherent comonadic `duplicate = extend (|w| w.clone())`:
```rust
pub fn duplicate(self) -> Cofree<F, Cofree<F, A>>
where F: CloneFunctor, A: Clone
{ self.extend(&|w| w.clone()) }
```
It sits on the existing `F: Functor<F>` inherent block, gated additionally on `F: CloneFunctor,
A: Clone`; it did not exist before because `Cofree<F, A>: Clone` did not.

The by-reference `CoMonad` trait instance rebuilds the tree from a borrow:
```rust
impl<F: HKT<Constraint = NoConstraint> + Functor<F> + CloneFunctor> CoMonad<CofreeWitness<F>>
    for CofreeWitness<F>
{
    fn extract<A: Clone>(fa: &Cofree<F, A>) -> A { fa.head().clone() }
    fn extend<A: Clone, B, K: FnMut(&Cofree<F, A>) -> B>(fa: &Cofree<F, A>, k: K) -> Cofree<F, B> {
        // head = k(fa); clone the children's F-structure (the borrow can't be moved into F::fmap),
        // then fmap `extend k` into it; `k` is threaded by &mut through the depth-first traversal.
    }
}
```
`extend` takes the context by **reference** (the `CoMonad` trait's signature) and must rebuild it;
cloning a node's `F`-structure of children before `F::fmap` consumes it is exactly what
`F: CloneFunctor` provides — the blocker the prior change recorded for D4.

`CoMonad<G>: Functor<G>`, so this instance **requires** `Functor<CofreeWitness<F>>`, which the prior
change decided `CofreeWitness` would *not* implement ("its `FnMut` cannot carry the `Fn + Clone`
functor action"). That decision is reversed here. The reversal is sound: the inherent `Cofree::map`
needs `Fn + Clone` because it clones the closure once per hole while it *owns and moves through* the
tree; a `Functor::fmap` with the trait's `FnMut` is implementable by consuming the tree and threading
a single `&mut f` through a depth-first traversal — the holes are visited in sequence, so no per-hole
clone is needed. For a pure `f` it computes the same relabelling as the inherent `map` (a test pins
the agreement). The inherent `Fn + Clone` `map` stays as the primary by-value surface; the trait
`fmap` exists to satisfy the `CoMonad` supertrait and for generic `Functor`-bound code.

Reversal impact: this is a spec-level change to `haft-cofree-comonad` (a `MODIFIED` requirement), but
a **code-additive** one — it adds trait impls, changes no signature, and removes nothing. Nothing
depended on `CofreeWitness` not being a `Functor`: no blanket impl conflicts (each witness impls
`Functor` individually), the inherent `map`/`extract`/`extend` are unaffected (`fmap`/trait-`extend`
do not collide with them — the witness carries the trait items, the value carries the inherent ones),
and no existing test asserts the absence. The full existing suite stays green.

**D5 — Placement mirrors the prior change.** `CloneFunctor` in `src/functor/` beside
`eq_functor`/`debug_functor`; `Clone for Free` in `src/monad/free_instances.rs` beside the
`PartialEq`/`Debug` impls; `Clone for Cofree`, `duplicate`, and the `Functor`/`CoMonad` impls in
`src/monad/cofree_comonad.rs` (the module that owns `Cofree`/`CofreeWitness`). Witness impls beside
each witness's existing `EqFunctor`/`DebugFunctor` impl.

**D6 — `Clone` carries no new categorical law (H1).** `clone` produces a value structurally equal to
its input (`clone_type(fa) == fa` for the built-in witnesses, which are also `EqFunctor`); the induced
`Clone` on `Free`/`Cofree` is a structural copy by induction on the tree. This is discharged by Rust
tests, not a Lean theorem — matching the `Eq`/`Debug` precedent (D8 of the prior change). The
`CoMonad` trait instance's comonad laws are the same laws already proved in
`DeepCausalityFormal/Haft/Cofree.lean` for the inherent by-value ops (`extract`/`extend`); the
by-reference ops compute the same result, so no new Lean is added — the laws are re-exercised by Rust
law-tests on the trait instance.

## Risks / Trade-offs

- **[Reversed decision — `CofreeWitness: Functor`]** the prior change's spec says `CofreeWitness`
  shall not implement `Functor`. → Reversed with rationale (D4): the trait `fmap` is a threaded-`FnMut`
  depth-first relabelling, distinct from and agreeing with the inherent `Fn + Clone` `map`. Recorded
  as a `MODIFIED` requirement on `haft-cofree-comonad`; the change is code-additive and the existing
  suite stays green.
- **[Two `map`-like operations on `Cofree`]** the inherent `map` (`Fn + Clone`, by value) and the
  trait `fmap` (`FnMut`, by value) coexist. → They do not collide (method vs. associated-function
  resolution) and agree for pure functions; the inherent one stays the ergonomic default, the trait
  one is required by the `CoMonad` supertrait.
- **[Capability surface growth]** `CloneFunctor` is one more witness trait. → It is the last member of
  the `Eq`/`Debug`/`Clone` family the gap report names; `Hash` etc. stay out of scope until needed.
- **[Doc drift in `free_monad.rs` NOTE]** the NOTE says "`Clone` is still absent". → Updated to list
  `Clone` on the opt-in route; a doc-only edit, no behaviour change.

## Migration Plan

Additive at the code level. No existing type, trait signature, or behaviour changes. Downstream
consumers gain `Clone` on `Free`/`Cofree` (opt-in per witness — one `CloneFunctor` impl each),
`Cofree::duplicate`, and the `CoMonad`/`Functor` trait instances for `CofreeWitness`. No deprecations.
The prior change's `fold`/`eq_type`-based stories remain valid and unaffected. The single non-code
change is the reversed `haft-cofree-comonad` decision, captured as a `MODIFIED` requirement.

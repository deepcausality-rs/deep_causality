# Note — `Cofree` comonad twin + opt-in `Eq`/`Debug` for `Free`/`Cofree`

## What

Two additive, strictly opt-in pieces for `deep_causality_haft`, requested by a downstream
consumer (`catgraph-dl`) that carries its own `FreeMnd`/`CofreeCmnd` pair and cannot adopt the
crate's root `Free` because (a) `Free` has no `Eq`/`Debug`, and (b) there is no `Cofree` twin, so
adopting `Free` alone would split the free/cofree pair its construction needs.

1. **`Cofree<F, A>` — the cofree comonad on a functor `F`.** The categorical dual of the existing
   `Free<F, A>`: a product (`head : A`, `tail : F (Cofree F A)`) where `Free` is a coproduct
   (`Pure | Suspend`). Comonad surface (`extract`, `map`, `extend`, `unfold`) as inherent methods,
   `CofreeWitness<F> : HKT`, comonad laws proved in Lean.

2. **`EqFunctor` / `DebugFunctor` capability traits (Route B).** A witness that opts in supplies
   `eq_type<T: PartialEq>` / `fmt_type<T: Debug>`; the crate then provides generic
   `PartialEq`/`Eq`/`Debug` for `Free<F, A>` and `Cofree<F, A>`. Existing `Free` code is unchanged;
   the instances appear only for witnesses that implement the capability.

## Why capability traits (not `#[derive]`)

`Free`/`Cofree` store the recursive child under a GAT projection: `F::Type<Box<Free<F, A>>>`. The
naive path — `#[derive]`, or a hand impl gated on `F::Type<Box<Free<F, A>>>: PartialEq` — makes the
instance *conditional on a projection bound*, so proving `Free<W, A>: PartialEq` at any concrete
witness `W` re-enters the solver and overflows:

```
error[E0275]: overflow evaluating the requirement `Free<OptionWitness, i32>: PartialEq`
  = note: required for `Box<Free<OptionWitness, i32>>` to implement `PartialEq`
  = note: 2 redundant requirements hidden
```

(Reproduced on `rustc 1.97.0`. This is exactly the failure the current `free_monad.rs` NOTE
records — the NOTE is correct for the `#[derive]` path.)

A capability trait breaks the cycle: the recursion runs through `F::eq_type` / `F::fmt_type`, never
through an `F::Type<..>: Trait` bound, so proving `Free<W, A>: PartialEq` discharges against the
generic impl's own stable bounds (`F: EqFunctor`, `A: PartialEq`) and terminates — the same way a
plain recursive `enum List { Nil, Cons(i32, Box<List>) }` derives `PartialEq`. Verified compiling
and running for `Free` and `Cofree` over `OptionWitness`/`VecWitness`.

This is the crate's established shape: a witness supplies the operation (`Functor::fmap`,
`Foldable::fold`); `EqFunctor`/`DebugFunctor` supply `eq_type`/`fmt_type` the same way.

## Why it has to land in `haft`

Orphan rule: a downstream crate cannot implement `PartialEq`/`Debug` for `Free<TheirWitness, A>`.
`Free` is foreign and not `#[fundamental]`, so nesting a local witness inside does not make the
impl local — `E0117`, verified. The instances can only be defined where `Free`/`Cofree` are.

## Constraints / decisions

- **Strictly additive, strictly opt-in.** `Free`'s definition and its existing "compare by folding
  to a canonical value" approach are unchanged. No existing requirement is modified.
- **`Cofree` comonad surface is inherent** (dual to `Free`'s inherent `bind`/`map`/`fold`). The
  functor action `map` takes `Fn + Clone` for the same one-copy-per-hole reason `Free::map` does.
- **The borrow-based `CoMonad` trait impl for `CofreeWitness` is out of scope.** Its signature
  (`extend(&F::Type<A>, FnMut) -> F::Type<B>`, `A: Clone`) requires rebuilding a `Cofree` from a
  borrow, which needs cloning the `F`-structure — a `CloneFunctor` capability not in this change.
  This mirrors `Free`, which implements `HKT`+`Pure` but not the `Monad` trait (`bind` is inherent
  because `Monad::bind`'s `FnMut` cannot express `Fn + Clone`). The comonad *laws* are proved for
  the inherent ops.
- **Finiteness.** `Cofree f a` is coinductive; in strict Rust it is finitely constructible only
  over functors with an "empty" shape (`Option`, `Vec`, a list functor that bottoms out) — which is
  exactly the annotated-tree use the downstream needs. `unfold` (the anamorphism, dual of
  `Free::fold`) is the generator; it terminates over such functors.
- **`Eq`/`Debug` carry no new categorical law.** They are lawful derived instances; the
  equivalence-relation obligation (reflexive/symmetric/transitive when `eq_type` is) is a Rust
  property test. The categorical Lean content is the `Cofree` comonad laws. No new THEOREM_MAP
  crate allowlist entry — `haft` already carries witnesses.
- Crate invariants hold: `unsafe_code = "forbid"`, no `dyn`, macro-free `/src`, no external deps,
  no-std with `alloc` (`Cofree` gated on `alloc` like `Free`).

## Scope non-goals

- No `CloneFunctor`/`HashFunctor` (same pattern, additive later if a consumer needs them).
- No `CoMonad` trait impl for `CofreeWitness` (needs the by-ref/clone functor capability above).
- No change to `Free`'s representation or its `fold`-based comparison story.

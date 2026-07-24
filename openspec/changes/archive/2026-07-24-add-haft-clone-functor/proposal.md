## Why

`deep_causality_haft` 0.4.1 ships the free monad `Free<F, A>` and the cofree comonad `Cofree<F, A>`,
plus the `EqFunctor`/`DebugFunctor` capability traits that give both carriers opt-in `PartialEq`/`Eq`/
`Debug`. `Clone` is the one member of that capability family still absent: the `free_monad.rs` NOTE
records "`Clone` is still absent — add a `CloneFunctor` the same way if needed", and issue #718 asks
for it. A downstream consumer (catgraph adopting haft's `Free`/`Cofree` as free-monad/cofree-comonad
carriers) currently works around the gap with equality-test reconstruction and `Arc`; direct
`Clone` completes the trait family.

`Free`/`Cofree` cannot derive `Clone`: the recursive child sits under the GAT projection
`Suspend(F::Type<Box<Free<F, A>>>)`, so `#[derive(Clone)]` — or any impl gated on the projection
bound `F::Type<Box<Free<F, A>>>: Clone` — makes the instance conditional on that projection, and
discharging it at a concrete witness re-enters the trait solver and overflows (`error[E0275]`, the
same failure `EqFunctor`/`DebugFunctor` were introduced to route around). A downstream crate cannot
add the impl either: `Free`/`Cofree` are foreign and not `#[fundamental]`, so the impl is an orphan
violation (`E0117`).

`Clone` also unblocks two comonad features the cofree change deferred (its Non-Goals list them
explicitly): the inherent `Cofree::duplicate` (D2), and the by-reference `CoMonad` **trait** instance
for `CofreeWitness` (D4), which must rebuild a `Cofree` from a borrow and therefore needs to clone a
node's `F`-structure of children.

This change adds all three, **strictly additively and opt-in**: existing `Free`/`Cofree` code is
unchanged, no instance appears for a witness that does not opt in, and the whole crate's existing
test suite stays green.

## What Changes

- **`CloneFunctor` capability trait, and the `Free`/`Cofree` `Clone` instances (H1).** A
  witness-capability trait `CloneFunctor: HKT<Constraint = NoConstraint>` with
  `fn clone_type<T: Clone>(fa: &Self::Type<T>) -> Self::Type<T>` — the `Clone` analogue of
  `EqFunctor::eq_type`. Generic `impl<F: CloneFunctor, A: Clone> Clone` for `Free<F, A>` and
  `Cofree<F, A>` drive their recursion through `clone_type`, never through an `F::Type<..>: Clone`
  bound, so they terminate. Capability impls are provided for the crate's built-in single-hole
  functor witnesses (`OptionWitness`, `VecWitness`, `BoxWitness`, `LinkedListWitness`,
  `VecDequeWitness`) — one-line delegations to the container's own `Clone`.

- **`Cofree::duplicate` (D2).** The inherent comonadic `duplicate(self) -> Cofree<F, Cofree<F, A>>`
  = `extend(&|w| w.clone())`, provided under `F: CloneFunctor, A: Clone` — the surface the cofree
  change omitted "until `Cofree<F, A>: Clone` exists".

- **`CoMonad` and `Functor` **trait** instances for `CofreeWitness` (D4).** `CofreeWitness<F>` gains
  `CoMonad<CofreeWitness<F>>` (the by-reference `extract`/`extend` twin of the inherent by-value
  ops), which requires `F: CloneFunctor` to rebuild the tree from a borrow. Because
  `CoMonad<G>: Functor<G>`, this entails a `Functor<CofreeWitness<F>>` instance — reversing the
  cofree change's decision that `CofreeWitness` would not implement `Functor`. The reversal is sound
  (a by-value `fmap` threads an `FnMut` through a depth-first traversal, so the per-hole closure
  clone the inherent `Fn + Clone` `map` needs is unnecessary) and code-additive (nothing depended on
  the absence; no existing test asserts it).

- **Doc reconciliation.** The `free_monad.rs` NOTE drops the "`Clone` is still absent" parenthetical
  and lists `Clone` alongside `PartialEq`/`Eq`/`Debug` on the opt-in capability route.

- **Tests.** Rust tests (Bazel-registered) for `clone_type` agreement per witness, `Free`/`Cofree`
  `Clone` reproduction, `duplicate`, and the three comonad laws on the `CoMonad` trait instance.
  `Clone` carries no new categorical law (like `Eq`/`Debug` before it), so no new Lean theorem and no
  `formalization.yml` allowlist change.

## Capabilities

### New Capabilities
- `haft-clone-functor`: the `CloneFunctor` capability trait and the opt-in generic `Clone` instances
  for `Free<F, A>` and `Cofree<F, A>`, with capability impls for the built-in functor witnesses.

### Modified Capabilities
- `haft-cofree-comonad`: `CofreeWitness` now also implements the `Functor` and `CoMonad` traits (the
  by-reference comonad surface), and `Cofree` gains the inherent `duplicate`; the prior "shall not
  implement `Functor`" decision is reversed.

## Impact

- **New `haft` surface:** `src/functor/clone_functor.rs` (`CloneFunctor`), exported from `lib.rs`;
  `impl Clone for Free` in `src/monad/free_instances.rs`; `impl Clone for Cofree`, inherent
  `Cofree::duplicate`, and `Functor`/`CoMonad` for `CofreeWitness` in `src/monad/cofree_comonad.rs`;
  `CloneFunctor` impls in the five built-in witness modules. One doc edit in `free_monad.rs`.
- **New Rust tests:** `deep_causality_haft/tests/algebra/clone_functor_tests.rs`, alloc-gated and
  picked up by the existing `algebra` glob in `tests/BUILD.bazel` — no BUILD change.
- **No new Lean.** `Clone` is a derived structural instance with no categorical law; the `Cofree`
  comonad laws (already proved) cover the comonad semantics the `CoMonad` trait instance witnesses.
- **No external dependencies;** `unsafe_code = "forbid"`, no `dyn`, macro-free `/src`, no-std with
  `alloc`. The alloc gating of `Free`/`Cofree` and the alloc-only witnesses is preserved
  (`CloneFunctor` itself and the `OptionWitness` impl are `alloc`-free).
- **Strictly additive at the code level.** No existing type, trait signature, or behaviour changes;
  the inherent `Cofree::map`/`extract`/`extend` and the `fold`-based comparison story are unaffected;
  every new instance is opt-in per witness. The one non-code change is a reversed spec decision on
  `haft-cofree-comonad` (see design D4).

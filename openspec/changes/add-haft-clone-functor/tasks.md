## 0. Context

- [x] 0.1 Read `AGENTS.md`, `docs/writing_guides/AiStyleguide.md`, issue #718, the archived
  `2026-07-16-haft-cofree-and-eq-debug` change (proposal/design/tasks/specs), and
  `deep_causality_haft/src/{functor/eq_functor,functor/debug_functor,monad/free_monad,monad/free_instances,monad/cofree_comonad,monad/comonad,hkt/mod}.rs`.
  Confirm `Free`/`Cofree` are `alloc`-gated, that `CoMonad<F>: Functor<F>`, and that the prior change
  recorded `CloneFunctor`/`duplicate`/`CoMonad`-for-`CofreeWitness` as deferred (D2/D4 non-goals).

## 1. `CloneFunctor` capability trait + built-in impls (H1)

- [x] 1.1 Add `src/functor/clone_functor.rs` (`CloneFunctor: HKT<Constraint = NoConstraint>`,
  `clone_type<T: Clone>(fa: &Self::Type<T>) -> Self::Type<T>`); register in `src/functor/mod.rs`;
  export from `lib.rs`. `core`/`alloc`-free. Doc mirrors `eq_functor.rs`, with a `compile_fail`
  opt-in doctest (a witness without `CloneFunctor` yields no `Free::clone`).
- [x] 1.2 Implement `CloneFunctor` for the built-in single-hole witnesses `OptionWitness`,
  `VecWitness`, `BoxWitness`, `LinkedListWitness`, `VecDequeWitness`, beside their existing
  `EqFunctor`/`DebugFunctor` impls. Bodies are `fa.clone()`; `BoxWitness` spells the parameter through
  the HKT projection to avoid `clippy::borrowed_box`.
- [x] 1.3 Rust tests: `clone_type` agrees with the underlying container's `Clone` for each witness.

## 2. `Clone` for `Free` and `Cofree` (H1)

- [x] 2.1 Add `impl<F: CloneFunctor, A: Clone> Clone for Free<F, A>` in `src/monad/free_instances.rs`
  and `impl<F: CloneFunctor, A: Clone> Clone for Cofree<F, A>` in `src/monad/cofree_comonad.rs` —
  recursion through `F::clone_type`, no `F::Type<..>: Clone` bound.
- [x] 2.2 Update the `free_monad.rs` NOTE: list `Clone` alongside `PartialEq`/`Eq`/`Debug` on the
  opt-in capability route; drop the "`Clone` is still absent" parenthetical.
- [x] 2.3 Rust tests: `Free`/`Cofree` `clone()` reproduces the tree (`clone() == original`) over
  `OptionWitness` and `VecWitness`; the `compile_fail` opt-in doctest (1.1) confirms opt-in.

## 3. `Cofree::duplicate` (D2) + `CoMonad`/`Functor` trait instances for `CofreeWitness` (D4)

- [x] 3.1 Add the inherent `Cofree::duplicate(self) -> Cofree<F, Cofree<F, A>>` = `extend(&|w|
  w.clone())` on the `F: Functor<F>` block, gated additionally on `F: CloneFunctor, A: Clone`.
- [x] 3.2 Add `impl Functor<CofreeWitness<F>> for CofreeWitness<F>` (`F: Functor<F>`) — a
  threaded-`FnMut` depth-first relabelling, agreeing with the inherent `map` for pure functions.
  Required by the `CoMonad` supertrait; reverses the prior change's "shall not implement `Functor`"
  decision (see design D4). No `dyn`, no `unsafe`.
- [x] 3.3 Add `impl CoMonad<CofreeWitness<F>> for CofreeWitness<F>` (`F: Functor<F> + CloneFunctor`):
  `extract` reads the head; `extend` rebuilds the tree from a borrow (clone the children's
  `F`-structure, then `fmap` `extend k` into it; thread `k` by `&mut`). The default `duplicate`
  applies.
- [x] 3.4 Rust law-tests on the trait instance: `extract` reads the head; left identity
  (`extend(w, extract) == w`), right identity (`extract(extend(w, f)) == f(w)`), associativity; the
  trait `fmap` matches the inherent `map`; the trait `duplicate` matches the inherent one.

## 4. Verify & hand off

- [x] 4.1 `cargo build`/`test`/`clippy`/`fmt` green for `deep_causality_haft`; the pre-existing
  full suite stays green (additivity); feature matrix (`--all-features`, default) builds. Confirm
  `unsafe_code = "forbid"` + no-`dyn` + macro-free `/src` intact in the new modules.
- [ ] 4.2 `bazel test //deep_causality_haft/...` green (the `algebra` glob picks up
  `clone_functor_tests.rs`; no BUILD change). Run once the spec is approved.
- [ ] 4.3 Confirm additivity: no existing type/trait signature/behaviour changed; the inherent
  `Cofree` surface and the `fold`/`eq_type` stories are unaffected; new instances opt-in per witness.
  Prepare a commit message — do not commit (await user).

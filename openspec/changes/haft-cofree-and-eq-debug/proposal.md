## Why

`deep_causality_haft` ships `Free<F, A>` — the free monad on a functor, with `fold` (a
catamorphism), `FreeWitness : HKT + Pure`, and Lean-proved monad laws. Two pieces a downstream
consumer needs are absent, and both are structural gaps rather than preferences:

- **No `Cofree` twin.** `Free` is the free monad; its dual, the **cofree comonad** `Cofree<F, A>`,
  is not in the crate. The `CoMonad` trait, `Comonad.lean` (Uustalu–Vene comonad laws), and the
  `Functor` machinery are all present, but there is no carrier that instantiates them as the
  free/cofree pair. A consumer building a free/cofree construction (e.g. an annotated-syntax /
  labelled-tree comonad alongside the free-monad program tree) cannot adopt the crate's `Free`
  alone without splitting that pair.

- **No `Eq`/`Debug` for `Free` (or a `Cofree`).** `Free` stores its recursive child under a GAT
  projection, `Suspend(F::Type<Box<Free<F, A>>>)`. `#[derive(PartialEq, Debug)]` — and any hand
  impl gated on the projection bound `F::Type<Box<Free<F, A>>>: PartialEq` — makes the instance
  conditional on that projection, so discharging it at a concrete witness re-enters the trait
  solver and overflows (`E0275`, reproduced on `rustc 1.97.0`; this is the failure the current
  `free_monad.rs` NOTE records). Consumers whose tests key on structural equality and `Debug`
  output therefore cannot use `Free` directly.

Both must live in `haft`: a downstream crate cannot add `PartialEq`/`Debug` for
`Free<TheirWitness, A>` — `Free` is foreign and not `#[fundamental]`, so the impl is an orphan
violation (`E0117`, verified).

This change adds both, **strictly additively and opt-in**: existing `Free` code is unchanged, the
existing "compare by folding to a canonical value" approach stays valid, and the new instances
appear only for witnesses that opt into a capability trait.

## What Changes

- **`Cofree<F, A>` — the cofree comonad (H1).** The categorical dual of `Free`: a product
  (`head : A`, `tail : F::Type<Box<Cofree<F, A>>>`) where `Free` is a coproduct. Comonad surface as
  inherent methods — `extract` (counit ε), `map` (functor action, `Fn + Clone`), `extend` (cobind,
  dual of `bind`), `unfold` (anamorphism, dual of `fold`) — plus `CofreeWitness<F> : HKT` mirroring
  `FreeWitness`. `alloc`-gated exactly like `Free`. The comonad laws are proved in Lean and
  witnessed in Rust.

- **`EqFunctor` / `DebugFunctor` capability traits, and the `Free`/`Cofree` instances (H2).** Two
  witness-capability traits over `HKT<Constraint = NoConstraint>` — `EqFunctor::eq_type<T: PartialEq>`
  and `DebugFunctor::fmt_type<T: Debug>` — supplied by a witness the same way `Functor::fmap` and
  `Foldable::fold` are. Generic `PartialEq`/`Eq`/`Debug` impls for `Free<F, A>` and `Cofree<F, A>`
  drive their recursion through these methods, never through an `F::Type<..>: Trait` bound, so they
  terminate (verified). Capability impls are provided for the crate's built-in single-hole
  `Functor` witnesses (`OptionWitness`, `VecWitness`, `BoxWitness`, and the other single-hole
  functors that can carry `Free`/`Cofree`).

- **Doc reconciliation.** The `free_monad.rs` NOTE is updated to point at the opt-in capability
  route while keeping its correct statement that `#[derive]` is impossible on the generic type.

- **Tests + Lean.** The comonad laws (left identity, right identity, associativity) and the
  `unfold` computation rule are exercised by Rust law-tests (Bazel-registered) and **proved in
  Lean** under `DeepCausalityFormal/Haft/Cofree.lean`, THEOREM_MAP-bound with a Rust witness,
  bare-`lean` typecheck. The `Eq`/`Debug` instances are exercised by Rust property tests
  (reflexive/symmetric/transitive; `Debug` round-trip); they carry no new categorical law and add
  no new Lean theorem.

## Capabilities

### New Capabilities
- `haft-cofree-comonad`: `Cofree<F, A>`, the cofree comonad on a functor, with `extract`/`map`/
  `extend`/`unfold` and `CofreeWitness<F> : HKT`; the comonad laws proved in Lean.
- `haft-eq-debug-functor`: `EqFunctor`/`DebugFunctor` capability traits and the opt-in generic
  `PartialEq`/`Eq`/`Debug` instances for `Free<F, A>` and `Cofree<F, A>`.

### Modified Capabilities
<!-- Additive new traits/types alongside the existing haft surface; no existing requirement changes. -->

## Impact

- **New `haft` surface:** `src/monad/cofree_comonad.rs` (`Cofree`, `CofreeWitness`), `alloc`-gated
  and exported from `lib.rs`; `src/functor/eq_functor.rs` + `src/functor/debug_functor.rs`
  (`EqFunctor`, `DebugFunctor`), exported; the generic `PartialEq`/`Eq`/`Debug` impls for
  `Free`/`Cofree` (in the respective type modules); capability impls for the built-in witnesses (in
  their existing witness/extension modules). One doc edit in `free_monad.rs`.
- **New Lean:** `DeepCausalityFormal/Haft/Cofree.lean` (comonad laws for `Cofree`, dual of
  `FreeMonad.lean`, reusing `Comonad.lean`'s law statements), registered in
  `DeepCausalityFormal.lean`; new `THEOREM_MAP.md` rows under the Haft layer. `haft` already carries
  Lean witnesses, so no `formalization.yml` crate-allowlist change is required.
- **New Rust tests:** `deep_causality_haft/tests/**` — comonad law-tests + `unfold` round-trip
  (Cofree) and equivalence-relation / `Debug` property tests (Eq/Debug), registered in
  `tests/BUILD.bazel`.
- **No external dependencies;** `unsafe_code = "forbid"`, no `dyn`, macro-free `/src`, no-std with
  `alloc`. Consistent with the crate's constraints. `Cofree` is the dual reification of the same
  machinery `Free` already uses — no new unsafe, no new type-system capability.
- **Strictly additive.** No existing requirement changes; existing `Free` behaviour and its
  `fold`-based comparison story are preserved. The new instances are opt-in per witness.

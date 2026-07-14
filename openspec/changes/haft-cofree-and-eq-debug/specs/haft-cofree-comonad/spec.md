## ADDED Requirements

### Requirement: The cofree comonad `Cofree<F, A>`

`deep_causality_haft` SHALL provide `Cofree<F, A>`, the cofree comonad on a functor `F`, as the
categorical dual of `Free<F, A>`: a product carrier with a value `head : A` and an `F`-structure of
sub-trees `tail : F::Type<Box<Cofree<F, A>>>`, where `F: HKT<Constraint = NoConstraint> +
Functor<F>`. The type SHALL be gated on the `alloc` feature exactly as `Free` is (the recursive
field requires heap indirection). Fields SHALL be private; construction and inspection SHALL be
provided through `new`, `head`, `tail`, and `into_parts`.

#### Scenario: Cofree is the dual product of Free

- **WHEN** `Cofree<F, A>` is constructed via `new(head, tail)` with `head: A` and
  `tail: F::Type<Box<Cofree<F, A>>>`
- **THEN** `head()` returns the value, `tail()` returns the `F`-structure of children, and
  `into_parts()` returns both — the product dual of `Free`'s `Pure | Suspend` coproduct

#### Scenario: Cofree is alloc-gated like Free

- **WHEN** the crate is built with `--no-default-features` (no `alloc`)
- **THEN** neither `Cofree` nor `CofreeWitness` is compiled, matching `Free`/`FreeWitness`

### Requirement: The comonad surface as inherent methods

`Cofree<F, A>` SHALL provide the comonad operations as inherent methods (mirroring `Free`'s inherent
`bind`/`map`/`fold`), for `F: HKT<Constraint = NoConstraint> + Functor<F>`:

- `extract(&self) -> A` where `A: Clone` — the counit ε, dual of `Free::pure`.
- `map<B, Fun: Fn(A) -> B + Clone>(self, f: Fun) -> Cofree<F, B>` — the functor action; `Fn + Clone`
  for the same one-copy-per-hole reason `Free::map` requires it.
- `extend<B, K: Fn(&Cofree<F, A>) -> B>(self, k: &K) -> Cofree<F, B>` — cobind, the dual of `bind`,
  computing `k(w) :< fmap (extend k) (tail w)`.
- `unfold<X, C: Fn(X) -> (A, F::Type<X>)>(seed: X, coalg: &C) -> Cofree<F, A>` — the anamorphism,
  dual of `Free::fold`, generating a `Cofree` from a seed and a coalgebra.

`unfold` SHALL terminate over functors that admit an empty shape (e.g. `Option`, `Vec`, a list
functor that bottoms out); this finiteness precondition SHALL be documented. The witness-level
borrow-based `CoMonad` trait SHALL NOT be required for `CofreeWitness` in this change (its `&`/`Clone`
signature would require a clone/by-ref functor capability out of scope here).

#### Scenario: extract reads the head

- **WHEN** `extract(&w)` is called on `w = Cofree::new(a, tail)`
- **THEN** it returns `a` (a clone of the focused value)

#### Scenario: extend refocuses on the observed sub-context

- **WHEN** `extend(w, k)` is called with `k: &Fn(&Cofree<F, A>) -> B`
- **THEN** the result's `head` is `k(&w)` and each child is `extend(child, k)` — every position
  carries `k` applied to its whole sub-tree

#### Scenario: unfold generates a finite annotated tree

- **WHEN** `unfold(seed, coalg)` is called with a `coalg` whose `F`-structure is empty below some
  depth
- **THEN** it produces the finite `Cofree` whose `head` at each node is the coalgebra's value and
  whose children are the unfolds of the coalgebra's seeds

### Requirement: `CofreeWitness<F>` is an HKT witness

`deep_causality_haft` SHALL provide `CofreeWitness<F>` implementing `HKT` with
`type Constraint = NoConstraint` and `type Type<T> = Cofree<F, T>`, mirroring `FreeWitness`.
`CofreeWitness` SHALL NOT implement the `Functor` trait (its `FnMut` cannot carry the `Fn + Clone`
functor action), consistent with `FreeWitness` not implementing `Functor`/`Monad`.

#### Scenario: the witness projects to the concrete cofree type

- **WHEN** `<CofreeWitness<F> as HKT>::Type<T>` is named
- **THEN** it is `Cofree<F, T>`, and `Cofree`/`CofreeWitness` are exported from `lib.rs`

### Requirement: Comonad laws are tested and proved in Lean

The `Cofree` comonad laws SHALL be exercised by Rust law-tests (Bazel-registered) and proved in Lean
under `DeepCausalityFormal/Haft/Cofree.lean`. The laws are left identity (`extend extract = id`),
right identity (`extract ∘ extend f = f`), and associativity, plus the `unfold` computation rule.
The Lean file SHALL be self-contained (bare-`lean`), SHALL reuse `Comonad.lean`'s law statements, and
SHALL follow `FreeMonad.lean`'s representative-functor treatment (Lean's positivity checker rejects a
variable-functor `Cofree`). The laws SHALL be bound by `THEOREM_MAP.md` ids
(`haft.cofree.comonad_laws`, `haft.cofree.unfold`) with Rust witnesses, and `Cofree.lean` SHALL be
registered in `DeepCausalityFormal.lean`.

#### Scenario: The code type matches the proved laws

- **WHEN** `THEOREM_MAP.md` is checked
- **THEN** the `haft.cofree.*` ids have `proved` Lean locations and passing Rust witnesses, and the
  inherent `extract`/`extend`/`unfold` on `Cofree` carry the laws the Lean proofs establish

#### Scenario: Cofree pairs with Free under the existing CoMonad laws

- **WHEN** `Cofree` is viewed alongside `Free` and `Comonad.lean`
- **THEN** `Cofree` is the cofree-comonad instance of the comonad laws `Comonad.lean` already states,
  restoring the free/cofree pair

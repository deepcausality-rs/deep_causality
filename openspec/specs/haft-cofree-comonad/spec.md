# haft-cofree-comonad Specification

## Purpose
TBD - created by archiving change haft-cofree-and-eq-debug. Update Purpose after archive.
## Requirements
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
`CofreeWitness<F>` SHALL additionally implement the `Functor` and `CoMonad` **traits** (the
by-reference comonad surface, see the requirements below) when `F` supplies the needed capabilities;
the inherent by-value `Cofree::map`/`extract`/`extend` remain the primary surface and are unchanged.
This supersedes the prior decision that `CofreeWitness` would not implement `Functor`: the trait
`fmap` is a threaded-`FnMut` depth-first relabelling (the trait's `FnMut` is carried by owning the
tree and visiting holes in sequence, so the per-hole closure clone the inherent `Fn + Clone` `map`
needs is unnecessary), and it agrees with the inherent `map` for pure functions. The addition is
code-additive — it changes no existing signature and removes nothing.

#### Scenario: the witness projects to the concrete cofree type

- **WHEN** `<CofreeWitness<F> as HKT>::Type<T>` is named
- **THEN** it is `Cofree<F, T>`, and `Cofree`/`CofreeWitness` are exported from `lib.rs`

#### Scenario: the trait Functor agrees with the inherent map

- **WHEN** `<CofreeWitness<F> as Functor<CofreeWitness<F>>>::fmap(w, f)` and `w.map(f)` are compared
  for a pure `f` over `F: Functor<F>`
- **THEN** they produce the same relabelled `Cofree`

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

### Requirement: `Cofree::duplicate`

`Cofree<F, A>` SHALL provide the inherent comonadic `duplicate(self) -> Cofree<F, Cofree<F, A>>` =
`extend(&|w| w.clone())`, for `F: HKT<Constraint = NoConstraint> + Functor<F> + CloneFunctor` and
`A: Clone`. It replaces each node's label with the whole sub-tree focused at that node; it is provided
now that `Cofree<F, A>: Clone` exists (the prior change omitted it "until `Cofree<F, A>: Clone`").

#### Scenario: duplicate labels each node with its focused sub-tree

- **WHEN** `w.duplicate()` is called on a `Cofree<F, A>` with `F: CloneFunctor`, `A: Clone`
- **THEN** the result's root label is `w`, and `extract` at each position recovers the sub-tree it was
  focused on (`extract ∘ duplicate = id`)

### Requirement: `CoMonad` trait instance for `CofreeWitness`

`deep_causality_haft` SHALL implement `CoMonad<CofreeWitness<F>>` for `CofreeWitness<F>` when
`F: HKT<Constraint = NoConstraint> + Functor<F> + CloneFunctor`. `extract` reads the node's `head`;
`extend` rebuilds the tree from a **borrow**, computing `k(w) :< fmap (extend k) (tail w)` — it clones
the children's `F`-structure (via `F: CloneFunctor`) before `fmap` consumes it, and threads the
observation `k` by `&mut` through the traversal so `k` needs no `Clone`. The instance's comonad laws
are the same laws proved in `DeepCausalityFormal/Haft/Cofree.lean` for the inherent by-value ops (the
by-reference ops compute the same result), so no new Lean theorem and no `formalization.yml` allowlist
change are added; the laws SHALL be exercised by Rust law-tests (Bazel-registered).

#### Scenario: the comonad laws hold on the trait instance

- **WHEN** `extract`/`extend` from `CoMonad<CofreeWitness<F>>` are exercised over a witness
  `F: Functor<F> + CloneFunctor` (e.g. `VecWitness`)
- **THEN** left identity (`extend(w, extract) == w`), right identity (`extract(extend(w, f)) == f(w)`),
  and associativity hold, and `CloneFunctor` supplies the cloning `extend` needs to rebuild from the
  borrow

#### Scenario: the trait instance requires the clone capability

- **WHEN** `CoMonad<CofreeWitness<F>>` is resolved for a witness `F` that is not `CloneFunctor`
- **THEN** the instance does not exist (the bound `F: CloneFunctor` is unsatisfied), leaving the
  inherent by-value `extract`/`extend`/`unfold` as the available surface


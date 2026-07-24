## MODIFIED Requirements

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

## ADDED Requirements

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

# Foundation: the categorical vocabulary

Examples for `deep_causality_haft`: higher-kinded types in Rust, and the categorical
structures built on them.

A witness stands in for a type constructor, so `Functor`,
`Monad`, `CoMonad` and the rest are written once and used over `Vec`, `Option`, `Result`, a
tensor or a multivector. Each example takes one structure, puts it on a domain problem, and
shows what the structure buys.

These are the vocabulary. `2_composition/` uses it to cross crate boundaries, and
`3_applications/` puts whole use cases on top.

Every example follows the three house rules: precision is a parameter (`FloatType`), values
cross the precision boundary through `deep_causality_num::lift`, and printing lives in helper
functions below `main`.

Run from the repository root:

```bash
cargo run -p mathematics_examples --example <example_name>
```

## The traits, one at a time

| File | Description | Command |
|------|-------------|---------|
| [functor.rs](functor.rs) | `Functor` for data anonymization: one masking rule applied across `Vec`, `LinkedList`, `VecDeque`, `HashMap`, `BTreeMap`, `Option`, `Result` and `Box` | `cargo run -p mathematics_examples --example haft_functor_examples` |
| [applicative.rs](applicative.rs) | `Applicative` over e-commerce order processing: independent effects combined in one step | `cargo run -p mathematics_examples --example haft_applicative_examples` |
| [monad.rs](monad.rs) | `Monad` over a configuration system: dependent steps that may each fail | `cargo run -p mathematics_examples --example haft_monad_examples` |
| [comonad.rs](comonad.rs) | `CoMonad` for system evolution: `extend` computes each new state from a view focused on the old one | `cargo run -p mathematics_examples --example haft_comonad_examples` |
| [foldable.rs](foldable.rs) | `Foldable` over e-commerce order processing: one traversal, many summaries | `cargo run -p mathematics_examples --example haft_foldable_examples` |
| [traversable.rs](traversable.rs) | `Traversable::sequence` turning `Vec<Option<T>>` into `Option<Vec<T>>` and `Vec<Result<T, E>>` into `Result<Vec<T>, E>`: the atomic batch | `cargo run -p mathematics_examples --example haft_traversable_examples` |
| [collectable.rs](collectable.rs) | `Collectable::collect` building a structure from any sequence, including one generated lazily — the inverse of `Foldable` | `cargo run -p mathematics_examples --example haft_collectable_examples` |
| [bifunctor.rs](bifunctor.rs) | `Bifunctor` over API response handling: `ResultUnboundWitness` mapping the success and error channels independently, and `Tuple2Witness` doing the same for a payload and the timing beside it | `cargo run -p mathematics_examples --example haft_bifunctor_examples` |
| [profunctor.rs](profunctor.rs) | `Profunctor` over search filters: adapting a function's input and output ends separately | `cargo run -p mathematics_examples --example haft_profunctor_examples` |
| [category.rs](category.rs) | `Category` on `Fun` and on `Kleisli<Option>`: identity and composition, and the Kleisli category every monad comes with | `cargo run -p mathematics_examples --example haft_category_examples` |
| [natural_transformation.rs](natural_transformation.rs) | `NaturalTransformation` via `OptionToVec`: mapping the container while leaving the payload alone, and the naturality law that lets the conversion move along a pipeline | `cargo run -p mathematics_examples --example haft_natural_transformation_examples` |
| [constrained_functors.rs](constrained_functors.rs) | `CloneFunctor`, `DebugFunctor` and `EqFunctor`: the witness supplying `Clone`, `Debug` and `PartialEq` for its container, which is what gives the recursive carriers those instances | `cargo run -p mathematics_examples --example haft_constrained_functors_examples` |
| [unbound_haft.rs](unbound_haft.rs) | Arity-2 and arity-3 witnesses in cybernetic sensor fusion: `Bifunctor`, `Profunctor` and `MonoidalMerge` | `cargo run -p mathematics_examples --example haft_unbound_examples` |
| [effect_system.rs](effect_system.rs) | The effect system over audited financial transactions: five effect channels accumulating as the computation is sequenced | `cargo run -p mathematics_examples --example haft_effect_system_examples` |

## The same traits on the math witnesses

The vocabulary above is written against `Option`, `Vec` and `Result`. These three put it on
the witnesses the math crates own, which is where `2_composition/` picks it up.

| File | Description | Command |
|------|-------------|---------|
| [functor_causal_tensor.rs](functor_causal_tensor.rs) | One generic `Functor` function serving `Option`, `Result` and `CausalTensor` at the same call site | `cargo run -p mathematics_examples --example functor_causal_tensor_examples` |
| [applicative_causal_tensor.rs](applicative_causal_tensor.rs) | `Pure` and `Applicative` on `CausalTensorWitness`, including a curried function taking its arguments one `apply` at a time | `cargo run -p mathematics_examples --example applicative_causal_tensor_examples` |
| [hkt_multivector.rs](hkt_multivector.rs) | `Functor`, `Pure` and `Applicative` on `CausalMultiVectorWitness`, and the dimension-changing tensor product written directly | `cargo run -p mathematics_examples --example hkt_multivector_examples` |

## Patterns written out by hand

Two examples build the pattern directly. The shipped implementations live in other crates, and
each row says where.

| File | Description | Command |
|------|-------------|---------|
| [adjunction.rs](adjunction.rs) | The Reader adjunction as global configuration access: the two adjuncts move a value in and out of an ambient context. The shipped `Adjunction` is `StokesAdjunction` in `deep_causality_topology`, shown in `2_composition/duality/` | `cargo run -p mathematics_examples --example haft_adjunction_examples` |
| [free_and_cofree.rs](free_and_cofree.rs) | `FreeWitness` holding a computation as data for an interpreter to give meaning to, and `CofreeWitness` growing a labelled structure from a seed and relabelling every node from what hangs below it | `cargo run -p mathematics_examples --example haft_free_and_cofree_examples` |
| [parametric_monad.rs](parametric_monad.rs) | An indexed monad as a type-safe state machine: the state type changes at each bind, so the compiler enforces the legal order | `cargo run -p mathematics_examples --example haft_parametric_monad_examples` |

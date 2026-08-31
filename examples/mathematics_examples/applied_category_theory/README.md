# Applied Category Theory Examples

Examples for the `deep_causality_haft` crate: higher-kinded types in Rust, and the
categorical structures built on them.

A witness stands in for a type constructor that Rust cannot name directly, so `Functor`,
`Monad`, `CoMonad` and the rest can be written once and used over `Vec`, `Option`,
`Result`, a tensor or a manifold. Each example takes a domain problem and shows what the
structure buys, rather than restating its definition.

These examples lived in `deep_causality_haft/examples` until they moved here, so that
every runnable example in the workspace sits under `examples/`.

Run from the repository root:

```bash
cargo run -p mathematics_examples --example <example_name>
```

| File | Description | Command |
|------|-------------|---------|
| [adjunction.rs](adjunction.rs) | `Adjunction` as global configuration access: the two adjuncts move a value in and out of an ambient context | `cargo run -p mathematics_examples --example haft_adjunction_examples` |
| [applicative.rs](applicative.rs) | `Applicative` over e-commerce order processing: independent effects combined without sequencing them | `cargo run -p mathematics_examples --example haft_applicative_examples` |
| [bifunctor.rs](bifunctor.rs) | `Bifunctor` over API response handling: mapping success and error channels independently | `cargo run -p mathematics_examples --example haft_bifunctor_examples` |
| [comonad.rs](comonad.rs) | `CoMonad` for system evolution: `extend` computes each new cell from a view focused on the old one | `cargo run -p mathematics_examples --example haft_comonad_examples` |
| [effect_system.rs](effect_system.rs) | The effect system over audited financial transactions: effects accumulate as the computation is sequenced | `cargo run -p mathematics_examples --example haft_effect_system_examples` |
| [foldable.rs](foldable.rs) | `Foldable` over e-commerce order processing: one traversal, many summaries | `cargo run -p mathematics_examples --example haft_foldable_examples` |
| [functor.rs](functor.rs) | `Functor` for data anonymization: one masking rule applied across `Vec`, `LinkedList` and `VecDeque` | `cargo run -p mathematics_examples --example haft_functor_examples` |
| [monad.rs](monad.rs) | `Monad` over a configuration system: dependent steps that may each fail | `cargo run -p mathematics_examples --example haft_monad_examples` |
| [parametric_monad.rs](parametric_monad.rs) | `ParametricMonad` as a type-safe state machine: the state type changes at each bind, so illegal orderings do not compile | `cargo run -p mathematics_examples --example haft_parametric_monad_examples` |
| [profunctor.rs](profunctor.rs) | `Profunctor` over search filters: adapting a function's input and output ends separately | `cargo run -p mathematics_examples --example haft_profunctor_examples` |
| [traversable.rs](traversable.rs) | `Traversable` over `Option` and `Result`: turning a structure of effects into an effect of a structure | `cargo run -p mathematics_examples --example haft_traversable_examples` |
| [unbound_haft.rs](unbound_haft.rs) | Arity-2 and arity-3 witnesses in cybernetic sensor fusion: `Bifunctor`, `Profunctor` and `MonoidalMerge` | `cargo run -p mathematics_examples --example haft_unbound_examples` |

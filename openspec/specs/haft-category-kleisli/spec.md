# haft-category-kleisli Specification

## Purpose
TBD - created by archiving change haft-categorical-machinery. Update Purpose after archive.
## Requirements
### Requirement: A named Category trait and a Kleisli category

`deep_causality_haft` SHALL provide a `Category` trait packaging identity and associative composition (`id`, `compose`), and a `Kleisli<M: Monad>` newtype implementing `Category` with `compose = bind`. The value-level `Arrow` SHALL also satisfy `Category`. This gives the interpretation functor a typed codomain and names the category the causal monad already generates. The informal Kleisli language in `io/mod.rs` SHALL be retired in favour of the named type.

#### Scenario: Kleisli composition is monadic bind

- **WHEN** two Kleisli arrows `A → M B` and `B → M C` are composed via `Category::compose`
- **THEN** the composite is `A → M C` computed by `bind`, and the category identity is the monad `pure`

#### Scenario: Arrow is a category

- **WHEN** the `Arrow` combinators are viewed through `Category`
- **THEN** `id`/`compose` satisfy the category laws, sharing the abstraction with `Kleisli`

### Requirement: Category laws are tested and proved in Lean

The category laws (left identity, right identity, associativity) for `Kleisli<M>` (and `Arrow`) SHALL be exercised by Rust law-tests (Bazel-registered) and proved in Lean under `DeepCausalityFormal/Haft/` (bare-`lean`), reducing to the already-proved `core.causal_arrow.category_laws` / `haft.arrow.category_laws` shape, bound by `THEOREM_MAP.md` ids (`haft.category.laws`, `haft.kleisli.category_laws`) with Rust witnesses.

#### Scenario: The code type matches the proved laws

- **WHEN** `THEOREM_MAP.md` is checked
- **THEN** the `haft.category.*` / `haft.kleisli.*` ids have `proved` Lean locations and passing Rust witnesses, and the code `Kleisli`/`Category` types carry the same laws the Lean proofs establish


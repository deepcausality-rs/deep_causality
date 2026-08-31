## Why

`unified_math_gaps.md` §4.1 items E1 and E2 cannot be written. `Applicative<F>: Functor<F> + Pure<F>`, and `Pure::pure` receives one value by value with no `Clone` and no `Default` in its signature, so it can fill exactly one slot. Every fixed-arity product of arity ≥ 2 is therefore shut out of `Applicative` and `Monad`, which is `Complex`, `Quaternion`, `Octonion` and `Dual`. Three escapes were measured and all are closed: `+ Clone` and `+ Default` on the impl's method are E0276, and a `Constraint` admitting only `Clone` types is E0599 because `Satisfies` carries no capability.

Widening `Pure` was tried end to end and reverted: it cascades through `Category::id`, `MonadEffect3/4/5` and `Adjunction::left_adjunct` into 73 errors in `deep_causality_cfd` and 7 in `deep_causality_discovery`, demanding `Clone` on typestate pipeline states that are deliberately move-only. The cause is structural rather than local. `Clone` is the categorical diagonal Δ, which the crate already says in `SymMonoidal::copy<A: Clone>`; `pure` is a cartesian convenience, not part of the monoid structure. The fix is to give `deep_causality_haft` the lax monoidal structure that needs no diagonal.

## What Changes

- Add `Semigroupal<F: HKT>: Functor<F>` carrying `zip_with` as the primitive and `zip` as a provided method. `zip_with` is the primitive because deriving `apply` from `zip` requires `(Func, A): Satisfies<F::Constraint>`, a bound that leaks into every generic caller.
- Add `LaxMonoidal<F: HKT>: Semigroupal<F>` carrying `unit`. The split from `Semigroupal` is load-bearing: every context-carrying witness in the workspace has a lawful `zip` and no lawful `unit`.
- Add two marker traits recording which monoid a witness claims: `Compositional<F>: Monad<F>` for composition, `Convolutional<F>: Semigroupal<F>` for Day convolution. No blanket impls, one deliberate line per witness, modelled on `deep_causality_algebra::Associative<O: Operator>`.
- Add `MonoidalApplicative<F: HKT>: Functor<F> + Convolutional<F>` with `apply` as a Δ-free provided method, gated on the `Convolutional` promise.
- Adopt the new stack on the Cayley-Dickson witnesses, closing E1. `ComplexWitness`, `QuaternionWitness` and `OctonionWitness` gain `Semigroupal`, `LaxMonoidal`, `Convolutional` and `MonoidalApplicative`.
- **BREAKING (data-only widening)** Drop the struct-level bound on `Dual<T: Real>`, and add `DualWitness` with `Functor` and `Foldable`, closing the part of E2 that has a caller. The `CoMonad` half of E2 is deferred: two comultiplications are lawful, nothing selects between them, and no code wants either. Removing a struct bound is a relaxation; `cargo check --workspace --all-targets` reports 0 errors, and `Dual<Dual<f64>>` second derivatives still compute correctly.
- Implement `Traversable` for `VecWitness`, which `Applicative::apply` cannot express and `zip_with` can.
- Add `lean/DeepCausalityFormal/Haft/LaxMonoidal.lean` with four `THEOREM_MAP` ids under the `haft.lax_monoidal.*` prefix, plus Rust witnesses.

`Applicative` itself does not change. It keeps its signature, its `A: Clone` bound, its four McBride-Paterson laws and all 22 of its impls. `Pure`, `Monad`, `Traversable`, `Arrow` and the effect system are untouched, and `Vec`, `LinkedList` and `VecDeque` keep the `Applicative` they have.

## Capabilities

### New Capabilities

- `haft-lax-monoidal`: the `Semigroupal` and `LaxMonoidal` traits, the `zip_with` primitive with `zip` derived, and the module placement that keeps them distinct from the existing cartesian `SymMonoidal`.
- `haft-monoid-markers`: the `Compositional` and `Convolutional` marker traits, their no-inference discipline, and the coherence obligation a witness holding both incurs.
- `haft-monoidal-applicative`: the `MonoidalApplicative` trait with its Δ-free derived `apply`, its gating on `Convolutional`, and its coexistence rules with `Applicative`.
- `haft-lax-monoidal-formalization`: the Lean file, the four `haft.lax_monoidal.*` theorem ids, and their Rust witnesses.
- `cayley-dickson-monoidal-applicative`: E1 closure. `Complex`, `Quaternion` and `Octonion` witnesses gain the monoidal applicative stack with componentwise `zip_with`.
- `num-dual-hkt-witness`: E2 closure for the functor layer. The `Dual` struct bound comes off and `DualWitness` gains `Functor`, `Foldable` and the monoidal applicative stack; `CoMonad` is deferred with the reason recorded.
- `haft-vec-traversable`: `VecWitness` gains `Traversable`, whose `sequence` is written through the inner witness's `zip_with`.

### Modified Capabilities

- `haft-formalization-docs`: the website Haft formalization page pins an exact row count of 49. Four new `haft.lax_monoidal.*` ids raise the `### Haft layer` table to 55 ids and the page to 53 rows, so the count in the requirement and its scenario changes.

## Impact

**Crates.** `deep_causality_haft` gains a module and four traits. `deep_causality_num_complex` gains four impls per witness. `deep_causality_num_dual` gains a `deep_causality_haft` dependency, an `extensions` module, and loses one struct bound. No other crate changes.

**Public API.** Additive in `haft` and `num_complex`. In `num_dual`, `Dual<T>` widens to admit component types that are not `RealField`; all arithmetic keeps its own `impl<T: Real>` bounds, so nothing that compiles today stops compiling.

**Build and CI.** `deep_causality_num_dual/Cargo.toml` and `BUILD.bazel` gain the haft dependency and the `std` / `no-std` feature wiring. The Bazel `rust_test_suite` globs `*_tests.rs`, and `lean/BUILD.bazel` globs each namespace, so neither needs a target added. The `theorem-map` job reads its crate list from `build/scripts/crates.sh`, which derives it from the root `Cargo.toml`, so CI needs no allowlist edit.

**Explicitly untouched.** `openspec/specs/haft-symmetric-monoidal-prop` keeps every requirement it has. This change adds a sibling structure at the endofunctor level and deliberately does not reuse its module, its `unit` name, or its `haft.monoidal.*` theorem prefix.

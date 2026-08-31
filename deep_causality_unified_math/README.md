<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Unified Math

Seventeen crates. One stack, seven tiers tall, rooted in three zero-dependency and zero unsafe.


Unified math grew from a small idea. Tensors and multivectors needed to compose, so they were given a shared higher-kinded interface. That interface turned out to want an algebra tower under it, the tower wanted numeric traits under that, and the composition kept finding new things it could reach: geometric algebra, discrete exterior calculus, chain complexes, spectral methods, uncertainty. 

## The stack

Every crate depends only on crates below it. The graph is drawn as its transitive reduction, so
`tensor -> num` is omitted where `tensor -> linear -> haft -> algebra -> num` already implies it.


```
tier 6   topology
tier 5   multivector
tier 4   calculus   fft   homology   tensor
tier 3   linear   num_complex   num_dual   uncertain
tier 2   haft   num_rational   rand
tier 1   algebra
tier 0   num   metric   ast
```

![Dependency graph of the deep_causality mathematics crates: seven tiers, from the roots num and
metric at tier 0 up through algebra and haft, then linear algebra and the number types, then
tensors, to topology at tier 6. The longest chain is highlighted.](graph.png)

The longest chain is seven crates:

```
num -> algebra -> haft -> linear -> tensor -> multivector -> topology
```

Every one of the thirteen crates above tier 1 reaches `algebra`, 
so a change to the tower recompiles the whole stack. `num` and `algebra` are also the two
crates that can never adopt `haft`, because `haft` depends on both.

## The seventeen

| Crate | Tier | What it holds |
|---|---|---|
| `deep_causality_num` | 0 | Numeric traits: casts, identity, float and integer predicates. The bottom of the workspace |
| `deep_causality_metric` | 0 | Metric signatures `Cl(p, q, r)` and Lorentzian sign conventions, east coast and west coast |
| `deep_causality_ast` | 0 | The expression AST the tensor and uncertainty layers build on |
| `deep_causality_algebra` | 1 | Groups, rings, fields, algebras, and isomorphism markers |
| `deep_causality_haft` | 2 | Applied category theory: HKT, functor, applicative, monad, foldable, arrow, and a type-encoded effect system |
| `deep_causality_num_complex` | 2 | Complex, quaternion and octonion number types |
| `deep_causality_num_dual` | 2 | Dual numbers, forward-mode automatic differentiation |
| `deep_causality_num_rational` | 2 | Exact rationals over the integers |
| `deep_causality_rand` | 2 | Random number generators and statistical distributions |
| `deep_causality_calculus` | 3 | Arrow-native differentiation and integration operators |
| `deep_causality_fft` | 3 | Fast Fourier transform: FFT, rFFT, N-dimensional |
| `deep_causality_linear` | 3 | Sparse CSR, dense and bit-packed 𝔽₂ matrices and vectors; eliminations, decompositions, conjugate gradient, exact integer path |
| `deep_causality_uncertain` | 3 | A first-order type for uncertain programming |
| `deep_causality_homology` | 4 | Chain complexes, boundary operators and homology over a chosen coefficient field. No geometry |
| `deep_causality_tensor` | 4 | N-index tensors, broadcasting, Einstein summation, the tensor-train stack |
| `deep_causality_multivector` | 5 | Multivectors for geometric algebra. The only crate joining both geometric roots |
| `deep_causality_topology` | 6 | Cell complexes, manifolds, discrete exterior calculus, gauge fields, differential geometry |

Nothing here has a **required** external dependency. Four crates reach crates.io at all: `num` for
`libm`, `rand` for `getrandom`, and `fft` and `topology` for `rayon`. All four sit behind features
that are off by default.

## How they compose

Composition runs through `deep_causality_haft`. A crate that owns a container generic in its element
declares a *witness* type, binds `type Type<T>` to that container, and implements the categorical
traits against the witness. Two mechanisms then fall out, and both are load-bearing in
`examples/mathematics_examples/composable_multi_math/`.

**Nesting.** A witness accepts any element type, including one another crate owns. 
A tensor of multivectors is an ordinary `CausalTensor<CausalMultiVector<T>>`, and
one `fmap` rotates every cell of a vector field by a single Clifford rotor:

```rust
let rotated: CausalTensor<CausalMultiVector<FloatType>> =
    CausalTensorWitness::fmap(field, |v| rotor.geometric_product(&v).geometric_product(&rotor_rev));
```

**Closure reach.** The comonadic `extend` hands a cursor to a closure, and the closure may call into
any crate it likes. `triple_hkt_stress_field` runs a six-step linear-elastic pipeline over a
tetrahedral mesh inside a single `ManifoldWitness::extend`, with topology supplying the walk, tensor
holding the strain, and multivector applying the material rotor.

Run them:

```bash
cargo run -p mathematics_examples --example tensor_x_algebra_rotation_field_examples
cargo run -p mathematics_examples --example triple_hkt_stress_field_examples
cargo run -p mathematics_examples --example capstone_spinor_minkowski_examples
```

The capstone parallel-transports a unit timelike spinor along a discretized Minkowski worldline in
`Cl(3,1)`. Four crates participate and the final drift against the closed-form `(cosh θ, sinh θ)` is
about `1.7e-31` at `Float106`.

| Trait | Implementers outside `haft` |
|---|---|
| `HKT`, `Functor`, `Foldable` | `linear`, `tensor`, `multivector`, `topology`, `num_complex`, `num_dual` |
| `Applicative`, `Pure`, `CoMonad` | `linear`, `tensor`, `multivector`, `topology` |
| `Monad` | `linear`, `tensor`, `topology` |
| `Semigroupal`, `MonoidalApplicative` | `num_complex`, `num_dual`, `tensor` |
| `Adjunction` | `topology` |
| `Arrow` | `calculus`, `tensor` |
| `Traversable`, `NaturalTransformation`, `Category`, `Kleisli`, `Bifunctor`, `Profunctor` | none |

That table is the work list. `openspec/notes/archive/unified_math/unified_math_gaps.md` carries the full
analysis: which absences are real gaps and which are correct (a `Ratio<A> -> Ratio<B>` under an
arbitrary `f` breaks coprimality, so `num_rational` is right to have none), what each costs, and a
ranking from mechanical to design fork. Two findings there were measured by running code rather than
read off the source: `CausalTensorWitness` and `CausalMultiVectorWitness` both violated monad right
identity. Both are now resolved. `CausalMultiVectorWitness` gave up `Monad`, because no metric choice
satisfies both identity laws, and `CausalTensorWitness::bind` keeps the input's shape when the map is
shape preserving, so a `[2, 3]` no longer comes back `[6]`.


## Further reading

| Document | What it holds |
|---|---|
| `openspec/notes/archive/unified_math/unified_math_gaps.md` | Where the categorical layer stops, and the ranked work to close it |
| `openspec/notes/archive/unified_math/deep_causality_unified_math.md` | The assessment for this consolidation, and what it predicted against what happened |
| `openspec/notes/archive/unified_math/HKT-LAW-FINDINGS.md` | Why the shaped witnesses stop at `Applicative` |
| `openspec/changes/archive-notes.md` | Reading an archived change whose paths predate a move |
| `examples/mathematics_examples/composable_multi_math/README.md` | Seven worked cross-crate compositions |
| `lean/THEOREM_MAP.md` | Lean theorems and the Rust witnesses bound to them |

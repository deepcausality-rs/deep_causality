---
title: Uniform mathematical foundation
description: One Functor/Monad/CoMonad surface across tensors, multivectors, manifolds, sparse matrices, and effect propagation.
section: concepts
order: 11
---

DeepCausality treats algebra, geometry, topology, and effect propagation as a single mathematical surface. The same `Functor`, `Monad`, and `CoMonad` operations run over tensors, multivectors, manifolds, sparse matrices, and propagating effects. Two crates do the work: [`deep_causality_haft`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_haft) provides the Higher-Kinded Type machinery; [`deep_causality_num`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_num) provides the algebraic trait floor that the math containers stand on.

## HKT in Rust via the witness pattern

Rust has no native Higher-Kinded Types. HAFT adds the abstraction with a witness pattern: a zero-sized struct that implements the `HKT` trait and stands in for the type constructor. Code generic over the witness picks up any container that implements the same functional trait:

```rust
fn double_value<F>(m_a: F::Type<i32>) -> F::Type<i32>
where
    F: Functor<F> + HKT,
{
    F::fmap(m_a, |x| x * 2)
}
```

`double_value::<OptionWitness>(Some(5))`, `double_value::<VecWitness>(vec![1, 2, 3])`, and `double_value::<ResultWitness<i32>>(Ok(5))` all type-check, all run, and none allocate for the witness itself. Default witness implementations ship for `Option`, `Result`, `Box`, `Vec` (full `Functor`/`Applicative`/`Monad`/`Foldable`) and for `BTreeMap`, `HashMap`, `VecDeque` (`Functor` and `Foldable` only).

Higher-arity traits (`HKT2` through `HKT5`) handle type constructors with more than one parameter. Unbound variants exist for `Bifunctor`, `Profunctor`, `Adjunction`, `ParametricMonad`, `Promonad`, `RiemannMap`, and `CyberneticLoop`. Each one is the right tool for a specific shape of dynamics; the HAFT README documents the intended use case for each.

A type-encoded effect system sits on top of the multi-arity HKTs. `Effect3`/`Effect4`/`Effect5` plus `MonadEffect3`/`MonadEffect4`/`MonadEffect5` let an effect type carry a primary value plus several fixed channels (error, log, trace) through `pure` and `bind`. The compiler checks that effects are handled or propagated; nothing leaks implicitly.

## The witness table for the math crates

The same pattern lifts the math containers into the same surface:

| Domain | Type | Witness | Role |
|---|---|---|---|
| Mechanics | `CausalTensor<T>` | `CausalTensorWitness` | Data container |
| Algebra | `CausalMultiVector<T>` | `CausalMultiVectorWitness` | Transformations |
| Topology | `Manifold<T>` | `ManifoldWitness` | Space and context |
| Structure | `CsrMatrix<T>` | `CsrMatrixWitness` | Sparse relations |
| Causality | `PropagatingEffect<T>` | `Effect5Witness` | Time and flow |

Each one implements `Functor` and `Monad` through HAFT. `Manifold` additionally implements `CoMonad`, which is what enables `extend` (apply a function to every local neighborhood) and `extract` (read the value at the current point). Code written against `fmap`, `bind`, `extend`, and `extract` runs over any of these without rewriting the call site.

Adjunctions (`BoundedAdjunction`) formally link Geometry (Multivectors) and Topology (Manifolds), so a problem stated in one category can be translated to the other along a typed bridge rather than ad-hoc glue.

## The algebraic trait floor

The numeric layer underneath ships an explicit algebraic hierarchy in [`deep_causality_num`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_num). The trait names follow standard abstract algebra:

```
Magma → Semigroup → Monoid → Group → AbelianGroup
                                        ↓
                                       Ring → CommutativeRing → Field → RealField → ComplexField<R>
```

with `Module<R>`, `Algebra<R>`, `AssociativeAlgebra<R>`, `DivisionAlgebra<R>`, and `EuclideanDomain` for vector and ring-with-division structures. Marker traits (`Associative`, `Commutative`, `Distributive`) make algebraic laws compile-time promises rather than convention. Concrete classifications:

| Type | Primary traits |
|---|---|
| `f32`, `f64` | `RealField`, `Field`, `DivisionAlgebra<Self>` |
| `Complex<T>` | `Field`, `DivisionAlgebra<T>`, `Rotation<T>` |
| `Quaternion<T>` | `AssociativeRing`, `DivisionAlgebra<T>`, `Rotation<T>` |
| `Octonion<T>` | `DivisionAlgebra<T>` (non-associative) |
| `i8`…`i128`, `isize` | `Ring`, `Integer`, `SignedInt` |
| `u8`…`u128`, `usize` | `Ring`, `Integer`, `UnsignedInt` |

The library will not let an algorithm that requires associativity use a non-associative type. The type system rejects it before the program runs.

## What this buys you in practice

The [`examples/mathematics_examples`](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/mathematics_examples) tree is the proof. Every example exposes a single type alias near the top of `main.rs`:

```rust
pub type FloatType = Float106;  // or f64, or f32
```

That alias flows through every tensor contraction, multivector rotation, manifold extension, and monadic step. Edit the line; the program re-runs at the new precision.

The capstone (`capstone_spinor_minkowski`) parallel-transports a unit timelike spinor along a discretized Minkowski worldline through four boost steps, then compares the composed result against `cosh(θ), sinh(θ)` for the summed rapidity:

| Precision | Composition drift |
|---|---|
| `f64` | ~1.1 × 10⁻¹⁶ |
| `Float106` | ~1.7 × 10⁻³¹ |

Fifteen orders of magnitude, recovered by editing one line. The algorithm, topology, Clifford rotor, and monadic chain are unchanged.

## When you need it

The same example tree also makes the opposite point. Integer arithmetic, fixed stencils, and single-shot transcendentals see no observable benefit from extended precision; switching from `f64` to `Float106` costs roughly 3-5× runtime and buys nothing measurable. Chained transcendental composition (Lie-group accumulation, long Kalman cascades, repeated rotor application, parallel transport) is where the precision dial actually pays. The decision table in the examples README is the practical guide.

## See also

- Reference READMEs: [`deep_causality_haft`](https://github.com/deepcausality-rs/deep_causality/blob/main/deep_causality_haft/README.md), [`deep_causality_num`](https://github.com/deepcausality-rs/deep_causality/blob/main/deep_causality_num/README.md).
- Examples: [`examples/mathematics_examples`](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/mathematics_examples) covers HKT composition (`tensor_x_algebra_rotation_field`, `tensor_x_topology_laplacian`, `triple_hkt_stress_field`), causal-monad composition (`effect_kalman_predict_correct`, `effect_diffusion_on_manifold`, `effect_tensor_algebra_roundtrip`), and the `Cl(3,1)` spinor capstone.
- Concept: [Higher-Kinded Types](/docs/concepts/hkt/), [Causal Monad](/docs/concepts/causal-monad/), [Effect Propagation Process](/docs/concepts/effect-propagation-process/).

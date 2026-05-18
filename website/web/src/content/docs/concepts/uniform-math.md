---
title: Uniform mathematical foundation
description: One Functor/Monad/CoMonad surface across tensors, multivectors, manifolds, sparse matrices, and effect propagation.
section: concepts
order: 11
---

DeepCausality treats algebra, geometry, topology, and effect propagation as a single mathematical surface. The same `Functor`, `Monad`, and `CoMonad` operations run over tensors, multivectors, manifolds, sparse matrices, and propagating effects. Two crates do the work: [`deep_causality_haft`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_haft) provides the Higher-Kinded Type machinery; [`deep_causality_num`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_num) provides the algebraic trait floor that the math containers stand on.

## Why Uniform Mathematic matters

Scientific code typically pays a hidden tax: every time the math crosses a domain (a mesh walk to a tensor contraction, a tensor to a rotor, a rotor back to a scalar field), the developer writes bridge code. Indices get repacked. Loops get rewritten. The contraction lives in one library, the rotation in another, the per-vertex traversal in a third. Each crossing is a place where bugs hide and where one library's conventions clash with another's.

A unified mathematical surface removes the tax. When a tensor, a multivector, and a manifold all implement the same `Functor` and `Monad` operations, they compose in the same way they would on paper. A walk over a mesh, a contraction on a per-vertex tensor, a rotation by a Clifford rotor, and an audit-logged monadic step are five operations from the same algebraic vocabulary, not five separate libraries that need translating.

The practical consequences are real:

- **One composition law applies everywhere.** `fmap`, `bind`, `extend`, and `extract` mean the same thing on a tensor, a multivector, a manifold, or a propagating effect.
- **Cross-domain pipelines stay readable.** Mesh walk and tensor algebra and rotor application can share a single closure. The structure of the code matches the structure of the math.
- **Numerical precision becomes a parameter, not an assumption.** Because the algebraic floor is generic, every container honors the same `RealField` / `Field` / `Ring` traits, and the float type can be changed in one place.
- **Algebraic laws are compile-time guarantees.** A type that is not associative cannot be passed where associativity is required. The compiler enforces what mathematicians prove by hand.

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

## A concrete example: three crates, one closure

The [`triple_hkt_stress_field`](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/mathematics_examples/triple_hkt_stress_field) example is the clearest demonstration of why the unification pays off. It runs a six-step linear-elastic stress pipeline on a 3D simplicial mesh and produces per-vertex von Mises stress in Pascals. The interesting part is that the entire pipeline lives inside one `ManifoldWitness::extend` call:

```rust
let result = ManifoldWitness::extend(&manifold, |w| {
    let i = w.cursor();
    if i >= N_VERTICES { return 0.0; }

    let strain   = prescribed_strain(i);                                 // STEP 1
    let stress   = hooke_isotropic(&strain, lambda, mu);                 // STEP 2  (tensor)
    let normal   = vertex_normal(i);                                     // STEP 3
    let traction = cauchy_traction(&stress, &normal);                    // STEP 4  (tensor)
    let _t_local = rotate_into_frame(&traction, &rotor, &rotor_rev);     // STEP 5  (multivector)
    von_mises(&stress)                                                   // STEP 6
});
```

Three crates participate in that closure:

- **`deep_causality_topology`** supplies the manifold and the per-vertex walk. `extend` is the comonadic operation that focuses each vertex in turn and exposes its neighborhood.
- **`deep_causality_tensor`** runs the constitutive law (Hooke's law mapping strain to stress) and the Cauchy traction contraction `t_i = σ_ij n_j` via `EinSumOp`.
- **`deep_causality_multivector`** applies the material-frame rotor sandwich `R t R~` in `Cl(3,0)`. The rotor is a Clifford-algebra multivector; the rotation is a geometric product.


With the uniform surface, `extend` walks the mesh. The tensor and multivector operations run on the focused value. The same closure that computes `stress` consumes a tensor and returns a tensor; the same closure that rotates `traction` consumes a multivector and returns a multivector. The composition is the closure body.

The same pattern scales. Replace the placeholder strain with a real displacement field; replace isotropic Hooke with J2 plasticity; replace the fixed rotor with a per-vertex material frame from a polar decomposition. The `extend` skeleton stays the same. Each step is a standalone function that the closure stitches together through ordinary Rust composition. The cross-crate algebra is the language of the closure, not a feature of any one step.

The [`effect_kalman_predict_correct`](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/mathematics_examples/effect_kalman_predict_correct), [`effect_diffusion_on_manifold`](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/mathematics_examples/effect_diffusion_on_manifold), and [`capstone_spinor_minkowski`](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/mathematics_examples/capstone_spinor_minkowski) examples push the same composition further: a Kalman predict-correct chain mixes tensor and rotor steps under a monadic `bind`; a heat equation alternates `extend` for the spatial Laplacian and `bind` for the time step; the capstone parallel-transports a unit timelike spinor along a discretized Minkowski worldline by composing all three crates plus the causal monad. Same composition law in every case.

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

## Precision as a parameter

The algebraic trait floor and the witness-based composition together produce a property that is hard to get any other way: numerical precision becomes a single line of the program. Every example in [`examples/mathematics_examples`](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/mathematics_examples) exposes one type alias near the top of `main.rs`:

```rust
pub type FloatType = Float106;  // or f64, or f32
```

That alias flows through every tensor contraction, multivector rotation, manifold extension, and monadic step. Edit the line; the program re-runs at the new precision. There is no parallel implementation, no `#ifdef`, no second copy of the math at a different precision. The compiler instantiates the generic code at whichever scalar field the alias resolves to.

The capstone (`capstone_spinor_minkowski`) parallel-transports a unit timelike spinor along a discretized Minkowski worldline through four boost steps, then compares the composed result against `cosh(θ), sinh(θ)` for the summed rapidity:

| Precision | Composition drift |
|---|---|
| `f64` | ~1.1 × 10⁻¹⁶ |
| `Float106` | ~1.7 × 10⁻³¹ |

Fifteen orders of magnitude, recovered by editing one line. The algorithm, topology, Clifford rotor, and monadic chain are unchanged. Chained transcendental composition (Lie-group accumulation, long Kalman cascades, repeated rotor application, parallel transport) is where the precision pays off the most.

## What the unification actually enables

The HKT machinery, the witness table, and the algebraic trait floor are three layers of one architecture. Read in isolation, each one looks like an implementation detail. Together they give the library a property that conventional scientific stacks struggle to deliver:

A single closure can walk a mesh, contract a tensor, apply a Clifford rotor, accumulate a state, append to an audit log, and short-circuit on error, in any order, with no glue between the layers. The composition law that makes the closure work is the same law that composes a stateless `bind` chain, a contextual Causaloid graph, a manifold extension, and a `PropagatingEffect` returned from the [Effect Propagation Process](/docs/concepts/effect-propagation-process/). The math, the data structures, and the runtime all speak one language.

## See also

- Reference READMEs: [`deep_causality_haft`](https://github.com/deepcausality-rs/deep_causality/blob/main/deep_causality_haft/README.md), [`deep_causality_num`](https://github.com/deepcausality-rs/deep_causality/blob/main/deep_causality_num/README.md).
- Examples: [`examples/mathematics_examples`](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/mathematics_examples) covers HKT composition (`tensor_x_algebra_rotation_field`, `tensor_x_topology_laplacian`, `triple_hkt_stress_field`), causal-monad composition (`effect_kalman_predict_correct`, `effect_diffusion_on_manifold`, `effect_tensor_algebra_roundtrip`), and the `Cl(3,1)` spinor capstone.
- Concept: [Higher-Kinded Types](/docs/concepts/hkt/), [Causal Monad](/docs/concepts/causal-monad/), [Effect Propagation Process](/docs/concepts/effect-propagation-process/).

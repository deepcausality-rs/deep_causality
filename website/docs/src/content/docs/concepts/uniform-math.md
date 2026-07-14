---
title: Uniform Math
description: One Functor/Monad/CoMonad surface across tensors, multivectors, manifolds, sparse matrices, and effect propagation.
sidebar:
  order: 12
---

DeepCausality treats algebra, geometry, topology, and effect propagation as a single mathematical surface. The same `Functor`, `Monad`, and `CoMonad` operations run over tensors, multivectors, manifolds, sparse matrices, and propagating effects. Two crates do the work: [`deep_causality_haft`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_haft) provides the Higher-Kinded Type machinery; [`deep_causality_num`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_num) provides the algebraic trait floor that the math containers stand on.

## Why Uniform Mathematics matters

Scientific code typically pays a hidden tax: every time the math crosses a domain (a mesh walk to a tensor contraction, a tensor to a rotor, a rotor back to a scalar field), the developer writes bridge code. Indices get repacked. Loops get rewritten. The contraction lives in one library, the rotation in another, the per-vertex traversal in a third. Each crossing is a place where bugs hide and where one library's conventions clash with another's.

A unified mathematical surface removes the tax. When a tensor, a multivector, and a manifold all implement the same `Functor` and `Monad` operations, they compose in the same way they would on paper. A walk over a mesh, a contraction on a per-vertex tensor, a rotation by a Clifford rotor, and an audit-logged monadic step are five operations from the same algebraic vocabulary, not five separate libraries that need translating.

The practical consequences are real:

- **One composition law applies everywhere.** `fmap`, `bind`, `extend`, and `extract` mean the same thing on a tensor, a multivector, a manifold, or a propagating effect.
- **Cross-domain pipelines stay readable.** Mesh walk and tensor algebra and rotor application can share a single closure. The structure of the code matches the structure of the math.
- **Numerical precision becomes a parameter, not an assumption.** Because the algebraic floor is generic, every container honors the same `RealField` / `Field` / `Ring` traits, and the float type can be changed in one place.
- **Algebraic laws are compile-time guarantees.** A type that is not associative cannot be passed where associativity is required. The compiler enforces what mathematicians prove by hand.

## A concrete example: GRMHD

The [`grmhd`](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/physics_examples/grmhd) example (General Relativistic Magnetohydrodynamics) is the sharpest demonstration of what the unification enables. It couples a general-relativity solver to a plasma physics solver, picks a metric signature dynamically based on local spacetime curvature, computes the Lorentz force in the selected geometry, and feeds the electromagnetic stress-energy back into the spacetime metric. Every one of those steps lives in a different mathematical regime. They are composed by a `bind` chain over the Causal Monad:

```rust
let result: PropagatingEffect<GrmhdState> = PropagatingEffect::pure(GrmhdState::new(&config))
    .bind(|state, _, _| {
        // [Step 1] GR Solver — tensor algebra.
        // Builds the Schwarzschild metric g_uv and Ricci tensor,
        // contracts them into the Einstein tensor G_uv = R_uv - ½ R g_uv.
        model::calculate_curvature(state.into_value().unwrap_or_default())
    })
    .bind(|state, _, _| {
        // [Step 2] Causal coupling — metric-signature selection.
        // Branches on curvature intensity and picks Metric::Minkowski(4)
        // (relativistic regime) or Metric::Euclidean(3) (classical regime).
        model::select_metric(state.into_value().unwrap_or_default())
    })
    .bind(|state, _, _| {
        // [Step 3] MHD Solver — Clifford Algebra.
        // Wraps the current J and magnetic field B as CausalMultiVector<f64>
        // in the metric chosen above, then computes F = J ∧ B as a bivector
        // through the Clifford geometric product.
        model::calculate_lorentz_force(state.into_value().unwrap_or_default())
    })
    .bind(|state, _, _| {
        // [Step 4] GRMHD coupling — back to tensor algebra.
        // Builds the EM field strength tensor F^uv (rank-2 CausalTensor)
        // and contracts it with the spacetime metric g_uv from Step 1
        // to produce the EM stress-energy tensor T^uv.
        model::calculate_energy_momentum(state.into_value().unwrap_or_default())
    })
    .bind(|state, _, _| {
        // [Step 5] Stability analysis
        // Scalar branching on the bivector intensity.
        model::analyze_stability(state.into_value().unwrap_or_default())
    });
```

Look at what the chain crosses. Step 1 is pure tensor algebra in `deep_causality_tensor`. Step 2 makes a runtime decision in `deep_causality_metric` that changes the geometry of the next step. Step 3 leaves tensor algebra entirely and computes in Clifford / geometric algebra through `deep_causality_multivector`. Step 4 returns to tensor algebra, but now coupled to the same spacetime metric produced in Step 1, which is what closes the GRMHD feedback loop. Step 5 is ordinary Rust.

With the uniform surface, `CausalTensor` and `CausalMultiVector` and `Metric` are all Functor / Monad instances over the same `PropagatingEffect` carrier. The Causal Monad's `bind` is the only composition operator. Each stage consumes a `GrmhdState` and returns an updated `GrmhdState` wrapped in a `PropagatingEffect`, regardless of which mathematical regime it works in. The pipeline reads like the physics: curvature → metric selection → Lorentz force → stress-energy → stability.

This is what "uniform mathematical foundation" means in practice. It is the ability to write a five-stage GR-plus-plasma simulation as a five-line monadic chain, with the type system enforcing dimensional consistency and the algebraic floor guaranteeing that every step uses the same scalar field at the same precision.

The [`mathematics_examples`](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/mathematics_examples) tree extends the same composition further: a Kalman predict-correct chain mixing tensor and rotor steps, a heat equation alternating `extend` for the spatial Laplacian with `bind` for the time step, and the `capstone_spinor_minkowski` example parallel-transporting a unit timelike spinor through `Cl(3,1)` along a discretized worldline. Same composition law in every case.

The uniform mathematical foundation is enabled by two distinct capabilities on the Deep Causality Project: Higher kinder types and an algebraic trait hierarchy.

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


## The algebraic trait hierarchy

The numeric layer underneath ships an explicit algebraic hierarchy in [`deep_causality_num`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_num). The trait names follow standard abstract algebra:

```
Magma → Semigroup → Monoid → Group → AbelianGroup
                                        ↓
                                       Ring → CommutativeRing → Field → RealField → ComplexField<R>
```

with `Module<R>`, `Algebra<R>`, `AssociativeAlgebra<R>`, and `DivisionAlgebra<R>` for vector and ring-with-division structures. Marker traits (`Associative`, `Commutative`, `Distributive`) make algebraic laws compile-time promises rather than convention. Concrete classifications:

| Type | Primary traits |
|---|---|
| `f32`, `f64` | `RealField`, `Field`, `DivisionAlgebra<Self>` |
| `Complex<T>` | `Field`, `DivisionAlgebra<T>`, `Rotation<T>` |
| `Quaternion<T>` | `AssociativeRing`, `DivisionAlgebra<T>`, `Rotation<T>` |
| `Octonion<T>` | `DivisionAlgebra<T>` (non-associative) |
| `i8`…`i128`, `isize` | `Ring`, `Integer`, `SignedInt` |
| `u8`…`u128`, `usize` | `Ring`, `Integer`, `UnsignedInt` |

The library will not let an algorithm that requires associativity use a non-associative type. The type system rejects it before the program runs.

## The math layers

The witness table earlier in this page lists each math container in one row. This section gives each one a bit more shape: what is in the box, and when to reach for it.

### Tensors: `deep_causality_tensor`

N-dimensional arrays with stride-based memory layout, broadcasting for element-wise operations, and **Einstein summation** for matrix products and tensor contractions. The crate ships `Functor`, `Applicative`, `Monad`, and `CoMonad` instances, so a contraction or a per-cell map composes through the same `fmap`/`bind` surface the rest of the stack uses. Reach for tensors wherever the math is rectangular data: relativistic field tensors, Kalman state and covariance, linear algebra mid-pipeline.

### Multivectors and Geometric Algebra: `deep_causality_multivector`

Clifford algebras over the dynamic signature space, with pre-configured constructors for **Pauli (Cl(3,0))**, **Spacetime Algebra (STA)**, **Conformal Geometric Algebra (CGA)**, **Projective Geometric Algebra in 3D (PGA3D)**, the **Dixon algebra** used in Standard Model particle physics, and the **Grand Unified Algebra hosting the full Spin(10) gauge symmetry**. `Functor`, `Applicative`, and `Monad` instances are implemented; `bind` realizes the **tensor product of algebras**, so dimension-changing compositions are monadic rather than ad-hoc.

The payoff is concrete. The [Maxwell example](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/physics_examples/maxwell) expresses the electromagnetic field `F = ∇A` as a single multivector and recovers `E` and `B` as its bivector grades. Six scalar components collapse to four, a ~50% compute reduction, without giving up correctness. Where vector calculus needs separate cross products, exterior derivatives, and Hodge stars to express the same physics, the geometric product handles it as one operation on one object.

### Topology and Differential Geometry: `deep_causality_topology`

Graphs, hypergraphs, **simplicial complexes**, **manifolds**, and point clouds, with first-class **exterior calculus operators** (exterior derivative `d`, Hodge star `⋆`, codifferential `δ`, Hodge-Laplacian) and a **lattice gauge field framework** supporting **U(1), SU(2), SU(3), and Lorentz** gauge groups. The lattice gauge implementation is verified against **24 reference results from Creutz's *Quarks, Gluons and Lattices***, a stable bedrock for anyone composing physical simulations from this stack.

`Manifold` is the layer's `CoMonad`; that is what makes `extend` (apply a function to every local neighborhood) and `extract` (read the value at the current point) first-class on geometric data. Graph convolutions and cellular automata become one comonadic walk over a typed neighborhood.

### Sparse matrices: `deep_causality_sparse`

CSR (Compressed Sparse Row) sparse matrices with `Functor`, `Applicative`, and `Monad` instances, used wherever the data is large and mostly zero: causal-graph adjacency, big covariance structures, transition matrices. The same `fmap`/`bind` surface applies.

### Metric signatures: `deep_causality_metric`

A single horizontal crate that defines metric signatures **once** and shares them across every layer above: the **East Coast convention** used in general relativity, the **West Coast convention** used in particle physics, and the broader Clifford signature space **Cl(p, q, r)** that the multivector and topology crates build on. A GR calculation in the tensor layer and a particle-physics calculation in the multivector layer can share sign conventions automatically; 

### Differentiation, integration, and quadrature: `deep_causality_calculus`

The analytic operators of the Causal Arrow: forward-mode automatic differentiation, endomorphism iteration for ODE integration, and composite-Simpson quadrature. **Differentiation** is the tangent functor. You write a model once as a `DifferentiableArrow`, or a multi-input `DifferentiableField<N>`, whose `run` is generic over the scalar. The fluent methods (`derivative`, `value_and_derivative`, `second_derivative`, `gradient`, `directional_derivative`) then seed and read the dual channel for you, so `Dual`, `ε`, and seeding never surface. **Integration** is endomorphism iteration. `Euler` and `Rk4` build value-level endo-arrows and iterate them three ways: to a fixed horizon, to a fixpoint, or until an event predicate first holds. `Rk4` drops in for `Euler` at higher accuracy. **Quadrature** is the free `quadrature` function, composite Simpson's rule exact through cubics; run it over `Dual` and the result carries the Leibniz rule. The derivative view `Diff` is an ordinary `Arrow`. It composes with the same `compose`/`first`/`split`/`fanout` combinators as the rest of the Arrow algebra, so the tangent functor extends that vocabulary rather than replacing it. Every operator is generic over `Scalar`. Precision is a free parameter, and duals nest for higher derivatives.

### Fast Fourier transforms: `deep_causality_fft`

Plan-based forward and inverse transforms, generic over `RealField` and operating on `Complex` data. `FftPlan` handles 1-D complex transforms of any length, `RfftPlan` the real-to-complex case in half-spectrum layout, and `FftPlanNd` / `RfftPlanNd` the N-dimensional case by row-column decomposition. `DctPlan` adds discrete cosine transforms (types I to III), the building block for Neumann-Poisson solves on wall-bounded boxes. The planner picks its algorithm by length. Small lengths, 2 to 32, use hardcoded kernels. Larger powers of two use a mixed radix-4/radix-2 Stockham pipeline. Every other length falls back to Bluestein's chirp-z method, so the cost stays O(N log N) at every size. Plans are immutable and hold all precomputed state, and execution borrows a caller-provided scratch buffer that allocates nothing on the serial path. Twiddles are computed per index rather than by recurrence, so a transform is as accurate as the scalar's `sin`/`cos`. At `Float106` that yields real extended precision rather than `f64`-limited results. The crate exists for one job: to give the DEC-native Navier-Stokes solver a spectral Poisson solve on periodic lattices. There it replaces a conjugate-gradient Leray projection that dominated the 388 ms (32³) solver step, cutting that projection to about 1.9 ms. The opt-in `parallel` feature fans the independent 1-D batches inside the N-dimensional plans out over Rayon.

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

A single closure can walk a mesh, contract a tensor, apply a Clifford rotor, accumulate a state, append to an audit log, and short-circuit on error, in any order, with no glue between the layers. The composition law that makes the closure work is the same law that composes a stateless `bind` chain, a contextual Causaloid graph, a manifold extension, and a `PropagatingEffect` returned from the [Effect Propagation Process](/concepts/effect-propagation-process/). The math, the data structures, and the runtime all speak one language.

## See also

- Reference READMEs: [`deep_causality_haft`](https://github.com/deepcausality-rs/deep_causality/blob/main/deep_causality_haft/README.md), [`deep_causality_num`](https://github.com/deepcausality-rs/deep_causality/blob/main/deep_causality_num/README.md).
- Examples: [`examples/mathematics_examples`](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/mathematics_examples) covers HKT composition (`tensor_x_algebra_rotation_field`, `tensor_x_topology_laplacian`, `triple_hkt_stress_field`), causal-monad composition (`effect_kalman_predict_correct`, `effect_diffusion_on_manifold`, `effect_tensor_algebra_roundtrip`), and the `Cl(3,1)` spinor capstone.
- Concept: [Higher-Kinded Types](/concepts/hkt/), [Causal Monad](/concepts/causal-monad/), [Effect Propagation Process](/concepts/effect-propagation-process/).

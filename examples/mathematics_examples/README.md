# Mathematics Examples

This directory consolidates examples for all four major DeepCausality mathematics
crates (`deep_causality_multivector`, `deep_causality_sparse`, `deep_causality_tensor`,
`deep_causality_topology`), alongside the cross-crate composition examples that
show how they fit together through the HKT machinery and the causal effect monad.

## Quick Start

Run any example from the repository root:

```bash
cargo run -p mathematics_examples --example <example_name>
```

---

## Layout

```
mathematics_examples/
├── 1_foundation/    the vocabulary, one crate at a time
│   ├── algebra/       the scalar tower: what each bound promises
│   ├── haft/          the categorical tower: Functor, Monad, CoMonad, ...
│   ├── calculus/      differentiation, quadrature, time integrators
│   ├── fft/           plan-based FFT and the Hermitian half-spectrum
│   ├── linear/        sparse CSR matrices
│   ├── multivector/   geometric algebra: multivectors and fields
│   ├── stats/         moments, correlation, distributions
│   ├── tensor/        N-index tensors, broadcasting, Einstein summation
│   └── topology/      graphs, complexes, manifolds, boundary operators
├── 2_composition/   how a value crosses a crate boundary
│   ├── nesting/       a witness whose element is another crate's type
│   ├── extension/     CoMonad::extend; the kernel reaches into other crates
│   ├── chaining/      Monad::bind, Kleisli; the value crosses in sequence
│   ├── alignment/     zip_with, sequence_zip; two structures paired by index
│   ├── operators/     Arrow; the computation is the value
│   └── duality/       Adjunction, iso bridges, witness duality
└── 3_applications/  one use case per example
```

| Folder | What's inside | README |
|---|---|---|
| [1_foundation](1_foundation/) | The two towers the workspace rests on, plus the API surface of each math crate. Start here to learn what a bound promises, what a trait buys, or what one crate can do | [1_foundation/README.md](1_foundation/README.md) |
| [2_composition](2_composition/) | Every example spans more than one crate, and the folder names the mechanism it uses to cross. A reader who learns `extend` on a graph can run it on a manifold, a point cloud or a sparse matrix | [2_composition/README.md](2_composition/README.md) |
| [3_applications](3_applications/) | One use case per example, in the least code that shows it | — |

Each folder carries its own README with the per-example table.

---

## Crates Used

| Crate | Purpose |
|-------|---------|
| `deep_causality_multivector` | Geometric algebra (`CausalMultiVector`, `HilbertState`) |
| `deep_causality_metric` | Metric signatures (`Metric::Euclidean`, `Metric::Minkowski`) |
| `deep_causality_tensor` | Tensor operations (`CausalTensor`, `EinSumOp`) |
| `deep_causality_topology` | Discrete geometry (`Graph`, `SimplicialComplex`, `Manifold`, `LatticeGaugeField`) |
| `deep_causality_linear` | Sparse `CsrMatrix` and dense matrices, used for boundary operators |
| `deep_causality_calculus` | Arrow-native differentiation and integration |
| `deep_causality_rand` | Entropy: generators, the Boolean draw, the Sobol sequence |
| `deep_causality_stats` | Moments, correlation, and the shaped distributions |
| `deep_causality_fft` | Plan-based FFT, rFFT, and the Hermitian half-spectrum |
| `deep_causality_haft` | Higher-kinded type traits (`Functor`, `Monad`, `CoMonad`, `Pure`) |
| `deep_causality_algebra` | The algebra tower (`Field`, `RealField`) the others bound against |
| `deep_causality_num` | Numerical traits (`Float106`, casts and predicates) |
| `deep_causality_num_complex` | Complex, quaternion and octonion number types |
| `deep_causality_num_dual` | Dual numbers, forward-mode automatic differentiation |
| `deep_causality_core` | `CausalEffectPropagationProcess` and witnesses |

---

## Float Precision Abstraction

Every example exposes a single type alias at the top of `main.rs`:

```rust
pub type FloatType = Float106;   // or f64, or f32
```

That alias flows through every tensor, every multivector, every manifold, and every
monadic step. Change the line; the example re-runs at the new precision, in one edit.

`make check_precision` enforces that. It flips every declared `FloatType` through
`Float106`, `f32` and `f64` and rebuilds, so the claim stays true as the examples change.
Where an example is fixed to one precision, `scripts/check_precision.sh` lists it together
with the bound that fixes it.

### Why numerical precision is important

`relativistic_spinor_transport` parallel-transports a unit timelike spinor
along a discretized Minkowski worldline through four boost steps, then compares the
composed result against `(cosh θ, sinh θ)` for the summed rapidity.

| Precision  | Composition drift |
|------------|-------------------|
| `f64`      | ~1.1e-16          |
| `Float106` | ~1.7e-31          |

That is **fifteen orders of magnitude** of additional precision recovered by editing
one line. The numerical algorithm is identical; the topology, tensor contraction,
Clifford rotor, and monadic chain are all the same. Only the underlying float type
changed.

This is the practical payoff of the HKT-and-algebraic-traits architecture: precision
is a parameter of the program, set in one place.

### When the precision dial actually matters

Switching precision is cheap; deciding whether you need it is the real question. The
rule of thumb from these examples:

> **Drift widens with precision only when there is a multi-step, non-rational,
> transcendental computation.**

Use that as the decision tree:

| Workload shape | Recommended `FloatType` | Why |
|----------------|-------------------------|-----|
| Integer or simple-fraction arithmetic (counting, stencils on rational inputs, mass-conserving updates) | `f32` or `f64` | Both representations are exact for the relevant values, and Float106 costs ~3-5× runtime. |
| Single-shot transcendental step (one rotation, one FFT bin, one solve) | `f64` | One rounding event of ~10⁻¹⁶ sits well under the modelling error. |
| Time-stepping or iterative loops on smooth fields (heat, wave, advection) with bounded operator norm | `f64` | Error per step is small and the operator damps it. Reach for Float106 at thousands of steps near a stability boundary. |
| Chained transcendental composition (parallel transport, repeated rotor application, Lie-group accumulation, long Kalman cascades) | `Float106` | Each step contributes ~10⁻¹⁶ of f64 rounding; chains amplify visibly. Float106 turns "noticeable drift" into "below any physical signal." |
| Ill-conditioned linear algebra (near-singular matrices, narrow eigengaps, GMRES on poorly preconditioned systems) | `Float106` | The condition number multiplies rounding error. Extra mantissa bits buy back lost digits directly. |
| Verification, reference implementations, regression baselines | `Float106` | The point is to expose error in the f64 path. Float106 is the oracle to diff against. |

The capstone example sits in the chained-transcendental row and visibly benefits. The
Laplacian and diffusion examples sit in the rational-arithmetic row, where `f64` is exact
for the values involved. The roundtrip example sits in the single-shot row, where Float106
surfaces a residual that `f64` rounds away.

Default to `f64`. Reach for `Float106` where the structure of the computation amplifies
rounding error.

---

## House rules

Every example here follows four:

1. **Precision is a parameter.** One `FloatType` alias, directly above `main`, threaded
   through every numerical site.
2. **Values cross the precision boundary through `deep_causality_num::lift`.** `lift`,
   `lift_usize`, `lift_count` on the way in; `lower` on the way out.
3. **Printing lives in helper functions below `main`.** `main` reads as the narrative.
4. **Fallible calls propagate with `?`** out of a `main` that returns `Result`.

---

## Adding New Examples

1. Decide which folder fits: `1_foundation/<crate>/` for one crate's API surface,
   `2_composition/<mechanism>/` for an example that spans crates, or `3_applications/`
   for a use case.
2. Create the source file (single-file examples) or directory (`<your_example>/main.rs`
   + `README.md`).
3. Single-file examples: pick a descriptive snake_case name. Multi-file: same, but
   the directory name carries it.
4. Register in `mathematics_examples/Cargo.toml`:
   ```toml
   [[example]]
   name = "your_example_examples"
   path = "<subfolder>/your_example.rs"   # or <subfolder>/your_example/main.rs
   ```
5. Every example name ends with the `_examples` suffix. Two older targets,
   `multifield_data_pipeline` and `tensor_sparse_memory_budget`, keep their bare names.
6. Add a row to the relevant subfolder's `README.md`.
7. Top-of-file `main.rs` declares `pub type FloatType = f64;` (or `f32` / `Float106`)
   and threads it through every numerical site. Run `make check_precision` before
   opening the PR. Four things break the flip while still building at `f64`:
   - a `const` holding the working type. A `const` takes a primitive literal, so
     `const ALPHA: FloatType = 0.15;` fixes the type to a primitive. Use a function
     returning `lift(0.15)`.
   - an inherent `.sqrt()`, `.powi()`, `.abs()` or `.cos()`. Those come from
     `deep_causality_algebra::Real` (and `powi` from `deep_causality_num::Float`);
     import the trait and call `Real::sqrt(x)`.
   - an accumulator seeded with a bare literal, which infers `f64`. Seed it with
     `lift::<FloatType>(0.0)`.
   - a container that pins its own coefficients, as in
     `SimplicialManifold<f64, FloatType>`. Both parameters take the alias.

   A threshold belongs in the working type too:
   `lift::<FloatType>(8.0) * <FloatType as Real>::epsilon()` moves with the alias.

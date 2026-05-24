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
├── algebra/                 — multivector / Clifford-algebra examples + algebraic_scanner
├── composable_multi_math/   — cross-crate composition (HKT + causal monad)
├── isomorphism/             — cross-crate bridges (tensor<->sparse, multifield, witness duality)
├── sparse/                  — CSR sparse-matrix examples
├── tensor/                  — CausalTensor examples
└── topology/                — graphs, manifolds, lattice gauge fields
```

Each subfolder has its own README with the per-example table.

| Subfolder | What's inside | Per-folder README |
|-----------|---------------|-------------------|
| [algebra](algebra/README.md) | Clifford / geometric algebra (`CausalMultiVector`, PGA, Dixon, Hopf, etc.) plus the `algebraic_scanner` study of complex structure in `Cl(p, q, r)` | [algebra/README.md](algebra/README.md) |
| [sparse](sparse/README.md) | Sparse matrix ops (`CsrMatrix`) and HKT integration | [sparse/README.md](sparse/README.md) |
| [tensor](tensor/README.md) | `CausalTensor` construction, `EinSumOp`, Einstein-field index gymnastics, HKT (Functor / Applicative) | [tensor/README.md](tensor/README.md) |
| [topology](topology/README.md) | Graphs, simplicial / cubical complexes, manifolds, differential forms, lattice gauge fields | [topology/README.md](topology/README.md) |
| [composable_multi_math](composable_multi_math/README.md) | Cross-crate composition through `Functor` / `Monad` / `CoMonad` and the causal effect monad. The "look at how these crates compose" gallery | [composable_multi_math/README.md](composable_multi_math/README.md) |
| [isomorphism](isomorphism/README.md) | Cross-crate `iso` bridges from `deep_causality_num::iso` / `deep_causality_haft::iso`: tensor <-> sparse via the `tensor-iso` feature, `CausalMultiField` <-> tuple carrier, and the `PropagatingEffect` / `PropagatingProcess` dual-witness pattern | [isomorphism/README.md](isomorphism/README.md) |

---

## Crates Used

| Crate | Purpose |
|-------|---------|
| `deep_causality_multivector` | Geometric algebra (`CausalMultiVector`, `HilbertState`) |
| `deep_causality_metric` | Metric signatures (`Metric::Euclidean`, `Metric::Minkowski`) |
| `deep_causality_tensor` | Tensor operations (`CausalTensor`, `EinSumOp`) |
| `deep_causality_topology` | Discrete geometry (`Graph`, `SimplicialComplex`, `Manifold`, `LatticeGaugeField`) |
| `deep_causality_sparse` | Sparse matrices (`CsrMatrix`) for boundary operators |
| `deep_causality_rand` | Random sampling for lattice gauge thermalization |
| `deep_causality_haft` | Higher-kinded type traits (`Functor`, `Monad`, `CoMonad`, `Pure`) |
| `deep_causality_num` | Numerical traits (`RealField`, `Complex`, `Float106`) |
| `deep_causality_core` | `CausalEffectPropagationProcess` and witnesses |

---

## Float Precision Abstraction

Every example exposes a single type alias at the top of `main.rs`:

```rust
pub type FloatType = Float106;   // or f64, or f32
```

That alias flows through every tensor, every multivector, every manifold, and every
monadic step. Change the line; the example re-runs at the new precision. Just one
single edit needed.

### Why numerical precision is important

The capstone (`capstone_spinor_minkowski`) parallel-transports a unit timelike spinor
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
is a parameter of the program, not a hardcoded assumption baked into a thousand call
sites.

### When the precision dial actually matters

Switching precision is cheap; deciding whether you need it is the real question. The
rule of thumb from these examples:

> **Drift widens with precision only when there is a multi-step, non-rational,
> transcendental computation.**

Use that as the decision tree:

| Workload shape | Recommended `FloatType` | Why |
|----------------|-------------------------|-----|
| Integer or simple-fraction arithmetic (counting, stencils on rational inputs, mass-conserving updates) | `f32` or `f64` | Both representations are exact for the relevant values. Float106 buys nothing and costs ~3-5× runtime. |
| Single-shot transcendental step (one rotation, one FFT bin, one solve) | `f64` | One rounding event of ~10⁻¹⁶ is usually well under any modelling error. Float106 is overkill. |
| Time-stepping or iterative loops on smooth fields (heat, wave, advection) with bounded operator norm | `f64` | Error per step is small and the operator damps it. Reach for Float106 only when running thousands of steps near a stability boundary. |
| Chained transcendental composition (parallel transport, repeated rotor application, Lie-group accumulation, long Kalman cascades) | `Float106` | Each step contributes ~10⁻¹⁶ of f64 rounding; chains amplify visibly. Float106 turns "noticeable drift" into "below any physical signal." |
| Ill-conditioned linear algebra (near-singular matrices, narrow eigengaps, GMRES on poorly preconditioned systems) | `Float106` | The condition number multiplies rounding error. Extra mantissa bits buy back lost digits directly. |
| Verification, reference implementations, regression baselines | `Float106` | The point is to expose error in the f64 path. Float106 is the oracle to diff against. |

The capstone example sits in the chained-transcendental row and visibly benefits.
The Laplacian and diffusion examples sit in the rational-arithmetic row and gain
nothing observable from Float106. The roundtrip example sits in the single-shot row,
where Float106 just reveals a residual that f64 happens to round away.

Translation: do not default to Float106 because it sounds safer. Default to f64.
Reach for Float106 when the structure of your computation actually amplifies
rounding error.

---

## Adding New Examples

1. Decide which subfolder fits: `algebra/`, `sparse/`, `tensor/`, `topology/`,
   `composable_multi_math/` (cross-crate composition), or `isomorphism/`
   (cross-crate `iso` bridges).
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
5. Every example name **must** end with the `_examples` suffix, except in
   `isomorphism/` where examples are registered under their bare names.
6. Add a row to the relevant subfolder's `README.md`.
7. Top-of-file `main.rs` declares `pub type FloatType = f64;` (or `f32` / `Float106`)
   and threads it through every numerical site.

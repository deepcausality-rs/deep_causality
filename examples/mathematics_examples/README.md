# Mathematicss Examples

This directory contains examples demonstrating the various mathematicss crates.

## Quick Start

Run any example from the repository root:

```bash
cargo run -p mathematics_examples --example <example_name>
```

---

## Float Precision Abstraction

Every example exposes a single type alias at the top of `main.rs`:

```rust
pub type FloatType = Float106;   // or f64, or f32
```

That alias flows through every tensor, every multivector, every manifold, and every monadic step. Change the line; the example re-runs at the new precision. Just one single edit needed.

### Why numerical precision is important

The capstone (`capstone_spinor_minkowski`) parallel-transports a unit timelike spinor along a discretized Minkowski worldline through four boost steps, then compares the composed result against `cosh(theta), sinh(theta)` for the summed rapidity.

| Precision | Composition drift |
|-----------|-------------------|
| `f64`     | ~1.1e-16 |
| `Float106` | ~1.7e-31 |

That is **fifteen orders of magnitude** of additional precision recovered by editing one line. The numerical algorithm is identical; the topology, tensor contraction, Clifford rotor, and monadic chain are all the same. Only the underlying float type changed.

This is the practical payoff of the HKT-and-algebraic-traits architecture: precision is a parameter of the program, not a hardcoded assumption baked into a thousand call sites.

### When the precision dial actually matters

Switching precision is cheap; deciding whether you need it is the real question. The rule of thumb from these examples:

> **Drift widens with precision only when there is a multi-step, non-rational, transcendental computation.**

Use that as the decision tree:

| Workload shape | Recommended `FloatType` | Why |
|----------------|------------------------|-----|
| Integer or simple-fraction arithmetic (counting, stencils on rational inputs, mass-conserving updates) | `f32` or `f64` | Both representations are exact for the relevant values. Float106 buys nothing and costs ~3-5x runtime. |
| Single-shot transcendental step (one rotation, one FFT bin, one solve) | `f64` | One rounding event of ~10^-16 is usually well under any modelling error. Float106 is overkill. |
| Time-stepping or iterative loops on smooth fields (heat, wave, advection) with bounded operator norm | `f64` | Error per step is small and the operator damps it. Reach for Float106 only when running thousands of steps near a stability boundary. |
| Chained transcendental composition (parallel transport, repeated rotor application, Lie-group accumulation, long Kalman cascades) | `Float106` | Each step contributes ~10^-16 of f64 rounding; chains amplify visibly. Float106 turns "noticeable drift" into "below any physical signal." |
| Ill-conditioned linear algebra (near-singular matrices, narrow eigengaps, GMRES on poorly preconditioned systems) | `Float106` | The condition number multiplies rounding error. Extra mantissa bits buy back lost digits directly. |
| Verification, reference implementations, regression baselines | `Float106` | The point is to expose error in the f64 path. Float106 is the oracle to diff against. |

The capstone example sits in the chained-transcendental row and visibly benefits. The Laplacian and diffusion examples sit in the rational-arithmetic row and gain nothing observable from Float106. The roundtrip example sits in the single-shot row, where Float106 just reveals a residual that f64 happens to round away.

Translation: do not default to Float106 because it sounds safer. Default to f64. Reach for Float106 when the structure of your computation actually amplifies rounding error.

---

## Examples Overview

### Standalone

| Example | Domain | Description |
|---------|--------|-------------|
| [algebraic_scanner](algebraic_scanner/README.md) | Abstract Algebra | Scans Clifford algebras for complex structure (I^2 = -1) |

### Mathematical Composition (HKT)

These examples compose two or three crates through the `Functor`, `Monad`, and `CoMonad` witnesses. No effect machinery is used.

| Example | Composes | Description |
|---------|----------|-------------|
| [tensor_x_algebra_rotation_field](tensor_x_algebra_rotation_field/README.md) | tensor x multivector | A `CausalTensor` of `CausalMultiVector` cells rotated by a single `fmap` |
| [tensor_x_topology_laplacian](tensor_x_topology_laplacian/README.md) | tensor x topology | Discrete Laplacian via `ManifoldWitness::extend` |
| [triple_hkt_stress_field](triple_hkt_stress_field/README.md) | all three | Stress traction (contract, lift, rotate) inside one `extend` |

### Mathematical Composition (Causal Monad)

These examples wrap mathematical operations in `CausalEffectPropagationProcess`. Each step is a `bind` with logging and a short-circuit error path.

| Example | Composes | Description |
|---------|----------|-------------|
| [effect_kalman_predict_correct](effect_kalman_predict_correct/README.md) | tensor + multivector + core | Predict / correct / verify chain with mixed tensor and rotor steps |
| [effect_diffusion_on_manifold](effect_diffusion_on_manifold/README.md) | topology + tensor + core | Heat equation: `extend` for space, `bind` for time |
| [effect_tensor_algebra_roundtrip](effect_tensor_algebra_roundtrip/README.md) | tensor + multivector + core | Lift to `Cl(3,0)`, rotate, lower; norm verified by tensor dot product |

### Capstone

| Example | Composes | Description |
|---------|----------|-------------|
| [capstone_spinor_minkowski](capstone_spinor_minkowski/README.md) | all three + core | Spinor parallel transport along a discretized worldline in `Cl(3,1)` |

## Crates Used

| Crate | Purpose |
|-------|---------|
| `deep_causality_multivector` | Geometric algebra (`CausalMultiVector`, `HilbertState`) |
| `deep_causality_metric` | Metric signatures (`Metric::Euclidean`, `Metric::Minkowski`) |
| `deep_causality_tensor` | Tensor operations (`CausalTensor`, `EinSumOp`) |
| `deep_causality_topology` | Discrete geometry (`SimplicialComplex`, `Manifold`) |
| `deep_causality_sparse` | Sparse matrices for boundary operators |
| `deep_causality_haft` | Higher-kinded type traits (`Functor`, `Monad`, `CoMonad`, `Pure`) |
| `deep_causality_core` | `CausalEffectPropagationProcess` and witnesses |

---

## Adding New Examples

1. Create directory: `examples/<your_example>/`
2. Add `main.rs` with doc comments (`//!` module docs)
3. Add `README.md` with:
    - How to run
    - Engineering value
    - Key concepts
    - APIs demonstrated
    - Adaptation suggestions
4. Register in `Cargo.toml`:
   ```toml
   [[example]]
   name = "your_example"
   path = "examples/your_example/main.rs"
   ```

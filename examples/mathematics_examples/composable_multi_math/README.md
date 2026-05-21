# Composable Multi-Math Examples

Cross-crate composition through the HKT machinery (`Functor`, `Monad`, `CoMonad`) and
the `CausalEffectPropagationProcess` effect monad. Each example here shows how tensor,
geometric algebra (multivector), topology (manifolds), and the causal effect system
compose through one uniform API.

Run any example from the repository root:

```bash
cargo run -p mathematics_examples --example <example_name>
```

## HKT-Only Composition (`Functor`, `Monad`, `CoMonad`)

These examples compose two or three crates through witness traits, with no effect
machinery involved.

| Example | Composes | Description | Command |
|---------|----------|-------------|---------|
| [tensor_x_algebra_rotation_field](tensor_x_algebra_rotation_field/README.md) | tensor × multivector | Rotates a grid of vectors by a single Clifford rotor via `Functor::fmap` on a tensor of multivectors | `cargo run -p mathematics_examples --example tensor_x_algebra_rotation_field_examples` |
| [tensor_x_topology_laplacian](tensor_x_topology_laplacian/README.md) | tensor × topology | Discrete Laplacian on a 1D simplicial manifold via `ManifoldWitness::extend` (CoMonad) | `cargo run -p mathematics_examples --example tensor_x_topology_laplacian_examples` |
| [triple_hkt_stress_field](triple_hkt_stress_field/README.md) | tensor × multivector × topology | 3D linear-elastic stress analysis blueprint on a tetrahedral mesh; six-step pipeline (strain, Hooke, normal, Cauchy traction, material rotor, von Mises) in one `extend` call | `cargo run -p mathematics_examples --example triple_hkt_stress_field_examples` |

## Causal Monad Composition (`CausalEffectPropagationProcess`)

These examples wrap operations in the causal monad. Each step is a `bind` with logging
and short-circuit error propagation.

| Example | Composes | Description | Command |
|---------|----------|-------------|---------|
| [effect_kalman_predict_correct](effect_kalman_predict_correct/README.md) | tensor + multivector + core | Predict / correct / verify skeleton (the structural shape of a Kalman filter): tensor matrix-multiply for predict, Clifford rotor for correct, NaN gate for verify | `cargo run -p mathematics_examples --example effect_kalman_predict_correct_examples` |
| [effect_diffusion_on_manifold](effect_diffusion_on_manifold/README.md) | topology + tensor + core | Heat equation on a 1D manifold: spatial Laplacian via `extend` (CoMonad), time stepping via `bind` (Monad), with stability short-circuit on CFL violation | `cargo run -p mathematics_examples --example effect_diffusion_on_manifold_examples` |
| [effect_tensor_algebra_roundtrip](effect_tensor_algebra_roundtrip/README.md) | tensor + multivector + core | Lift a 3-vector into `Cl(3,0)`, rotate, lower back, verify norm preservation by tensor dot product. Carried value type changes between `bind` calls; the monad threads them | `cargo run -p mathematics_examples --example effect_tensor_algebra_roundtrip_examples` |

## Capstone

| Example | Composes | Description | Command |
|---------|----------|-------------|---------|
| [capstone_spinor_minkowski](capstone_spinor_minkowski/README.md) | tensor + multivector + topology + core | Parallel transport of a unit timelike spinor along a discretized Minkowski worldline in `Cl(3,1)`. Topology supplies the path, tensor stores per-edge rapidities, multivector builds the boost rotors, the causal monad orders the steps. Final drift versus the closed-form `(cosh θ, sinh θ)` is ~1.7e-31 at `Float106`. That is fifteen orders of magnitude tighter than `f64` | `cargo run -p mathematics_examples --example capstone_spinor_minkowski_examples` |

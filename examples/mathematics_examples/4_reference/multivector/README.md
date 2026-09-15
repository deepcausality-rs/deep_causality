# Algebra Examples

Examples for the `deep_causality_multivector` crate (Clifford / geometric algebra) plus
the standalone `algebraic_scanner` study of complex structure in Clifford algebras.

Each example highlights an engineering or scientific use case where geometric algebra
provides a more robust, intuitive, or performant solution than traditional linear
algebra.

> [!NOTE]
> **MLX Backend Support:** Examples that use non-primitive field types such as
> `Complex<T>`, `Octonion<T>`, or `Quaternion<T>` cannot run on the MLX backend.
> MLX only natively supports numeric primitives (`f32`, `f64`, `u32`, etc.). Running
> these examples with `--features mlx` will result in a panic during tensor creation.

Run from the repository root:

```bash
cargo run -p mathematics_examples --example <example_name>
```

| File | Description | Command |
|------|-------------|---------|
| [algebraic_scanner/](algebraic_scanner/README.md) | Scans Clifford algebras `Cl(p, q, r)` for complex structure (`I² = -1`); decision tool for choosing a Clifford signature before committing to a multi-physics simulation | `cargo run -p mathematics_examples --example algebraic_scanner_examples` |
| [basic_multivector.rs](basic_multivector.rs) | Fundamental geometric-algebra operations: geometric, outer, and inner products | `cargo run -p mathematics_examples --example basic_multivector_examples` |
| [clifford_mhd_multivector.rs](clifford_mhd_multivector.rs) | "Metric agnosticism": the same Lorentz-force code computed in classical (Euclidean) and relativistic (Minkowski) regimes; the pattern that plasma physics relies on | `cargo run -p mathematics_examples --example clifford_mhd_multivector_examples` |
| [dixon_multivector.rs](dixon_multivector.rs) | Dixon algebra `Cl_C(6)` for high-energy particle physics; naturally encodes Standard Model symmetries via octonions | `cargo run -p mathematics_examples --example dixon_multivector_examples` |
| [geometric_tilt_estimator.rs](geometric_tilt_estimator.rs) | Adaptive gravity observer that uses a causal monad to estimate a gravity vector; eliminates gimbal lock; hardware-independent kernel for robotics state estimation | `cargo run -p mathematics_examples --example geometric_tilt_estimator_examples` |
| [grmhd_integration.rs](grmhd_integration.rs) | A multi-physics monad coupling a General Relativity solver with a Magnetohydrodynamics solver; the simulation adapts its mathematical basis to physical conditions | `cargo run -p mathematics_examples --example grmhd_multivector_examples` |
| [hkt_multivector.rs](hkt_multivector.rs) | `CausalMultiVector` as an HKT (Functor, Applicative, Monad) for composable physics pipelines built from algebraic primitives | `cargo run -p mathematics_examples --example hkt_multivector_examples` |
| [maxwell_multivector.rs](maxwell_multivector.rs) | Unifies electric and magnetic fields into a single electromagnetic-field bivector derived from a 4-vector potential; high-fidelity antenna and radar design | `cargo run -p mathematics_examples --example maxwell_multivector_examples` |
| [pga3d_multivector.rs](pga3d_multivector.rs) | Projective Geometric Algebra (PGA) for points, lines, and planes uniformly represented; "motors" handle rigid-body motions in graphics and robotics | `cargo run -p mathematics_examples --example pga3d_multivector_examples` |

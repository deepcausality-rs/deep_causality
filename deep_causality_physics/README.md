# DeepCausality Physics

**A library of physics formulas and engineering primitives for DeepCausality.**

`deep_causality_physics` provides physics kernels, causal wrappers, and physical quantity types
designed for use within the DeepCausality hyper-graph simulation engine. It leverages Geometric
Algebra (via `deep_causality_multivector`), Causal Tensors, and a shared topological backend
(`deep_causality_topology`) to model complex physical interactions with high fidelity at any
precision the caller chooses.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
deep_causality_physics = { version = "0.5" }

# For QCD hadronization (Lund string fragmentation), enable the os-random feature:
# deep_causality_physics = { version = "0.5", features = ["os-random"] }
```

## Two Pillars

The crate is organized along two complementary axes:

1. **Kernels** — pure, stateless, domain-specific computations. Schwarzschild radius, Lorentz
   force, Cahn-Hilliard flux, Lund string fragmentation, etc. Use these when you need to solve a
   specific equation in isolation. Every kernel is generic over `R: RealField` so the caller picks
   the precision (`f32`, `f64`, `DoubleFloat`, …).

   **See [README_KERNELS.md](./README_KERNELS.md)** for the full list of kernel domains,
   architecture details, and worked examples (Relativistic Dynamics, Chronometric GM Recovery,
   Lund String Fragmentation).

2. **Theories** — full physical theories on a shared topological backend, unified through
   Gauge Fields and Geometric Algebra. General Relativity, Electromagnetism, Weak Force, and
   Electroweak Theory are all implemented as `GaugeField<G>` over a manifold, so they compose
   cleanly when modelling cross-theory interactions.

   **See [README_GAUGE_THEORIES.md](./README_GAUGE_THEORIES.md)** for the architecture of the
   theory layer, gauge-group taxonomy, and how to switch precision per theory.

## Precision

All kernels, quantity wrappers, and theories are generic over `R: RealField`. The same source code
runs at `f32` for real-time visualisation, `f64` for standard engineering simulations, or
`DoubleFloat` (~31 decimal digits) for cosmology and quantum field theory. See the precision
sections in each of the two READMEs above.

## Configuration

The crate supports `no_std` environments via feature flags.

* `default`: Enables `std`.
* `std`: Usage of standard library (includes `alloc`).
* `alloc`: Usage of allocation (Vec, String) without full `std`.
* `os-random`: Enables OS-based secure random number generator and Lund string fragmentation for
  QCD hadronization.

## License

Licensed under MIT. Copyright (c) 2025 DeepCausality Authors.

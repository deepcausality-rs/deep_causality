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

3. **DEC Navier–Stokes** — an incompressible fluid solver native to discrete exterior calculus:
   velocity is an edge 1-form, each `Rk4` stage marches the Leray-projected rate
   `P(−i_u ω − ν Δ_dR u♭ + g♭)` (the projector *is* the incompressibility equation — no
   splitting error), and the `SolenoidalField` type-state makes "you cannot time-step an
   unprojected field" a compile-time fact. The hot loop streams through compiled DEC stencil
   tables (equivalence-gated against the generic operators); the grade-0 pressure solves
   dispatch to direct spectral solves (rFFT on tori, DCT-I/DFT on uniform wall-bounded boxes)
   or Jacobi-preconditioned CG.

   The solver covers **periodic and wall-bounded domains**: no-slip walls constrain
   wall-tangential edges through the constrained Leray projector (divergence-free **and**
   no-slip, exactly, at every step boundary), and `with_moving_wall` prescribes a tangential
   lid velocity (Couette, lid-driven cavity). The validation ladder gates Taylor–Green
   convergence tables, inviscid invariants, the double shear layer, exact Couette/Poiseuille
   steady states, and the Re-1000 lid-driven cavity against the Ghia et al. (1982) tables;
   `examples/avionics_examples/{dec_taylor_green_re1600, dec_lid_cavity_re1000}` produce the
   recognizable artifacts.

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
* `parallel`: Enables Rayon-parallel execution of the DEC operator loops underneath the
  Navier–Stokes solver (wedge, interior product, de Rham, sharp, and the CG matvecs), by
  forwarding to `deep_causality_topology/parallel`. Disabled by default; the parallel paths are
  granularity-thresholded, so small lattices run serial loops with no fork-join overhead either
  way.

## Contribution

Contributions are welcomed especially related to documentation, example code, and fixes.
If unsure where to start, just open an issue and ask.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in deep_causality by you,
shall be licensed under the MIT licence, without any additional terms or conditions.

## Licence

This project is licensed under the [MIT license](LICENSE).

## Security

For details about security, please read
the [security policy](https://github.com/deepcausality-rs/deep_causality/blob/main/SECURITY.md).

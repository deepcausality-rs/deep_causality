# DeepCausality Physics

**A library of physics formulas and engineering primitives for DeepCausality.**

`deep_causality_physics` provides physics kernels, causal wrappers, and physical quantity types
for DeepCausality models. It builds on Geometric Algebra (`deep_causality_multivector`), causal
tensors, and a shared topological backend (`deep_causality_topology`), and computes at whatever
precision the caller chooses.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
deep_causality_physics = { version = "0.5" }

# For QCD hadronization (Lund string fragmentation), enable the os-random feature:
# deep_causality_physics = { version = "0.5", features = ["os-random"] }
```

## Three Parts

The crate has three parts:

1. **Kernels** — pure, stateless, domain-specific computations. Schwarzschild radius, Lorentz
   force, Cahn-Hilliard flux, Lund string fragmentation, etc. Use them to solve a single equation
   in isolation. Every kernel is generic over `R: RealField` so the caller picks
   the precision (`f32`, `f64`, `Float106`, …).

   **See [README_KERNELS.md](./README_KERNELS.md)** for the full list of kernel domains,
   architecture details, and worked examples (Relativistic Dynamics, Chronometric GM Recovery,
   Lund String Fragmentation).

2. **Theories** — full physical theories on a shared topological backend, unified through
   Gauge Fields and Geometric Algebra. General Relativity, Electromagnetism, Weak Force, and
   Electroweak Theory are each a `GaugeField<G>` over a manifold, so they compose in
   cross-theory models.

   **See [README_GAUGE_THEORIES.md](./README_GAUGE_THEORIES.md)** for the architecture of the
   theory layer, gauge-group taxonomy, and how to switch precision per theory.

3. **DEC Navier–Stokes forms** — the typed differential forms of the incompressible
   discrete-exterior-calculus fluid solver in `deep_causality_cfd`: velocity is an edge 1-form
   (`VelocityOneForm`), pressure a vertex 0-form (`PressureZeroForm`), and the `SolenoidalField`
   type-state, constructible only through a projection, makes "you cannot time-step an
   unprojected field" a compile-time fact.

   The solver itself lives in `deep_causality_cfd`. Each `Rk4` stage marches the Leray-projected
   rate `P(−i_u ω − ν Δ_dR u♭ + g♭)`; the projector *is* the incompressibility equation, so
   there is no splitting error. The hot loop streams through compiled DEC stencil tables
   (equivalence-gated against the generic operators); the grade-0 pressure solves dispatch to
   direct spectral solves (rFFT on tori, DCT-I/DFT on uniform wall-bounded boxes) or
   Jacobi-preconditioned CG. It covers **periodic and wall-bounded domains**: no-slip walls
   constrain wall-tangential edges through the constrained Leray projector (divergence-free
   **and** no-slip, exactly, at every step boundary), and `with_moving_wall` prescribes a
   tangential lid velocity (Couette, lid-driven cavity). The validation ladder gates
   Taylor–Green convergence tables, inviscid invariants, the double shear layer, exact
   Couette/Poiseuille steady states, and the Re-1000 lid-driven cavity against the Ghia et al.
   (1982) tables; the verification binaries
   `deep_causality_cfd/verification/{dec_taylor_green_re1600_verification, dec_lid_cavity_re1000_verification}`
   produce the reference artifacts.

## Precision

All kernels, quantity wrappers, and theories are generic over `R: RealField`. The same source code
runs at `f32` for real-time visualisation, `f64` for engineering simulations, or
`Float106` (~31 decimal digits) for cosmology and quantum field theory. The two READMEs above
each have a precision section.

## Configuration

The crate supports `no_std` environments via feature flags.

* `default`: Enables `std`.
* `std`: Uses the standard library (includes `alloc`).
* `alloc`: Uses allocation (Vec, String) without full `std`.
* `os-random`: Enables the OS-backed secure random number generator and Lund string fragmentation
  for QCD hadronization.
* `parallel`: Enables Rayon-parallel execution of the DEC operator loops underneath the
  Navier–Stokes solver (wedge, interior product, de Rham, sharp, and the CG matvecs), by
  forwarding to `deep_causality_topology/parallel` and `deep_causality_par/parallel`. Disabled by
  default. The parallel paths are granularity-thresholded, so small lattices run serial loops
  with no fork-join overhead.

## Contribution

Contributions are welcome, especially documentation, example code, and fixes.
If unsure where to start, open an issue and ask.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in deep_causality by you,
shall be licensed under the MIT licence, without any additional terms or conditions.

## Licence

This project is licensed under the [MIT license](LICENSE).

## Security

For details about security, please read
the [security policy](https://github.com/deepcausality-rs/deep_causality/blob/main/SECURITY.md).

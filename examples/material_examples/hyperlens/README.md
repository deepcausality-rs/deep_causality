# Hyperbolic Metamaterial Lens

An ordinary lens cannot resolve detail finer than the wavelength of its light. This example shows
why, and shows one way around it, describing the material by a metric signature alone.

It is a demonstration, not a solver. Once the problem is written as a causal process over the
library's types, precision as a parameter and categorical composition follow at no extra cost: the
same source runs at four scalars, and the sweep is a `fmap` and a `fold`. [What this example is, and where it stops](#what-this-example-is-and-where-it-stops) says
what the model holds fixed and how it grows toward a device-grade treatment.

```bash
cargo run -p material_examples --example hyperlens_example
```

## The physics

High spatial frequencies carry fine detail. A periodic object of period `d` carries
`k_x = 2π/d`, and a wave leaving it has an out-of-plane wavenumber set by the dispersion relation.
For a TM wave in a uniaxial medium:

```text
k_x²/ε_z + k_z²/ε_x = k₀²        so        k_z² = ε_x·(k₀² − k_x²/ε_z)
```

A positive `k_z²` propagates. A negative one is evanescent: it decays instead of carrying its
detail to the far field.

**In vacuum** every principal permittivity is positive, the relation is a sphere, and a large
`k_x` drives `k_z²` negative. The cutoff is `k_x = k₀`, which is `d = λ`.

**In a Type I hyperbolic metamaterial** `ε_z` is negative while `ε_x = ε_y` stay positive. The
second term changes sign, the surface opens from a sphere into a hyperboloid, and `k_z²` stays
positive at every `k_x`. Nothing bounds the detail that propagates.

## The metric is the material

A metric signature is a sign pattern over principal axes, so the two materials are two metrics:

| Material | Metric | Signature | ε_x | ε_y | ε_z |
|---|---|---|---|---|---|
| vacuum | `Metric::Euclidean(3)` | (+, +, +) | +1 | +1 | +1 |
| Type I metamaterial | `Metric::Generic { p: 2, q: 1, r: 0 }` | (+, +, −) | +1 | +1 | −1 |

`model::permittivity` reads each sign with `Metric::sign_of_sq`, so the optics never writes a sign
of its own. Swapping the metric swaps the physics; that is the example's whole claim.

## What the code demonstrates

Two categorical operations sweep a range of object periods:

| Operation | Reads | Produces |
|---|---|---|
| `fmap` | one period | its `k_z²`, from the dispersion relation |
| `fold` | the whole sweep | the finest period that still propagates |

`fmap` applies the relation pointwise; the relation itself never sees the sweep. `fold` pairs each
period with its own result and reduces to the resolution limit.

## Output

```text
  period     vacuum k_z^2   outcome       lens k_z^2     outcome
    1000       +1.184e-4   propagates      +1.974e-4   propagates
     500        +0.000e0   propagates      +3.158e-4   propagates
     400       -8.883e-5   evanescent      +4.047e-4   propagates
      50       -1.563e-2   evanescent      +1.595e-2   propagates

Resolution limit, the finest period that still propagates
  vacuum                500 nm, which is the wavelength itself
  Type I metamaterial   50 nm, the finest period probed
```

Vacuum cuts off at exactly `d = λ = 500 nm`, where `k_z²` reaches zero. The metamaterial propagates
every period in the sweep, so its limit is set by how fine an object is probed rather than by the
optics.

## Precision is a parameter

```rust
pub type FloatType = Float106;
```

Every constant is declared at that type through `const_scalar_from_int!` and
`const_scalar_from_float!`, so no conversion runs at any call site. The alias is `Float106` rather
than `f64` so that a hard-coded `f64`, unnoticed while the alias *is* `f64`, fails to compile. All
four scalars run.

## What the example covers

The example reformulates the core of a physical problem as a causal process, which brings
precision as a parameter and categorical composition along. The core of a hyperlens is one sign
flip in a signature, so the model keeps that and holds everything else fixed: both permittivity
magnitudes sit at 1, the medium is unbounded and lossless, and the illumination is a single
frequency. What remains is the mechanism.

A production-grade solver adds the parts a device needs: magnitudes that differ per axis and vary
with frequency, a complex permittivity carrying loss, a finite layer stack with its own reflections,
and the curved geometry that magnifies the near field out to where a detector sits.

## How to grow the example toward a complete solver

Each step keeps the structure already here.

- **Per-axis and frequency-dependent magnitudes.** `permittivity` already takes an axis, so only
  its magnitude source changes: a per-axis table, then a Drude or Lorentz model in the frequency.
  Sweeping period against frequency makes the tensor rank 2, and the `fmap` carries over unchanged.
- **Loss.** A lossy medium has a complex permittivity, and the dispersion relation is the same
  expression over `Complex<FloatType>`, with the imaginary part of `k_z` giving the decay length.
  The one change is writing the relation over a scalar bound instead of the alias, so one
  definition serves both scalars.
- **A finite layer stack.** A real hyperlens is alternating layers, and a stack is a product of
  transfer matrices: `DenseMatrix` for the matrices and a `fold` for the product.
- **Curved geometry.** A cylindrical hyperlens magnifies as it propagates, which the topology crate
  carries as a manifold with the sweep as its payload.

## Files

| File | Holds |
|---|---|
| `main.rs` | the alias and the two categorical operations |
| `model.rs` | the optics constants, the metric-to-permittivity reading, the dispersion relation |
| `utils_print.rs` | the presentation, and the only `lower` calls |

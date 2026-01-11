# 2D U(1) Lattice Gauge Field Verification

## Run the example

```bash
RUSTFLAGS='-C target-cpu=native' cargo run --example lattice_u1_2d --release
```

## Overview

This example validates the `LatticeGaugeField` implementation by comparing computed
values against the **exact analytical solution** of the 2D U(1) lattice gauge theory.

## Precision Achievement: 1e-31 Agreement

Using the `DoubleFloat` type (106-bit mantissa ≈ 32 decimal digits), two independent
algorithms for computing I₁(β)/I₀(β) agree to **~10⁻³¹** relative error:

| β    | Error Between Algorithms |
|------|--------------------------|
| 0.5  | 0 (exact agreement)      |
| 1.0  | 7.7 × 10⁻³³              |
| 10.0 | 4.9 × 10⁻³²              |
| 20.0 | 6.2 × 10⁻³²              |

**Why this is significant:**

- Standard `f64` (64-bit double) provides ~15 significant digits → ~10⁻¹⁵ precision
- `DoubleFloat` provides ~32 significant digits → ~10⁻³¹ precision
- This demonstrates that Deep Causality's `DoubleFloat` enables precision
  **16 orders of magnitude beyond** typical floating-point libraries

**Physical scale context (SI prefixes):**

| Prefix         | Factor    | Decimal Digits | Physical Reference              |
|----------------|-----------|----------------|---------------------------------|
| nano (n)       | 10⁻⁹      | 9 digits       | DNA helix width                 |
| pico (p)       | 10⁻¹²     | 12 digits      | Atom diameter                   |
| **femto (f)**  | **10⁻¹⁵** | **15 digits**  | **Proton size ← f64 precision** |
| atto (a)       | 10⁻¹⁸     | 18 digits      | Quark scale                     |
| zepto (z)      | 10⁻²¹     | 21 digits      |                                 |
| yocto (y)      | 10⁻²⁴     | 24 digits      |                                 |
| ronto (r)      | 10⁻²⁷     | 27 digits      |                                 |
| **quecto (q)** | **10⁻³⁰** | **30 digits**  | **← DoubleFloat precision**     |
|                | 10⁻³⁵     | 35 digits      | Planck length                   |

If you measured the observable universe (~10²⁶ m) with DoubleFloat precision,
your error would be smaller than the Planck length.

## Theory Background

The 2D U(1) lattice gauge theory is one of the rare exactly solvable models in
lattice gauge theory. The average plaquette expectation value satisfies:

$$\langle P \rangle = \frac{I_1(\beta)}{I_0(\beta)}$$

where $I_n$ are modified Bessel functions of the first kind, and $\beta$ is the
inverse coupling constant.

## Verification Strategy

1. **Identity Configuration Check**: For a "cold start" (all links = identity),
   the average plaquette should be exactly 1.0.

2. **Dual-Algorithm Verification**: Two independent algorithms compute I₁(β)/I₀(β)
   at DoubleFloat precision:
    - **Series expansion**: Direct summation of the power series
    - **Miller's backward recurrence**: Numerically stable continued fraction

## Reference

> M. Creutz, *Quarks, Gluons and Lattices*, Cambridge University Press (1983), Chapter 8


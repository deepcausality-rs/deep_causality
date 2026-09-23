# 2D U(1) Lattice Gauge Field Verification

## Run the example

```bash
RUSTFLAGS='-C target-cpu=native' cargo run -p physics_examples --example gauge_lattice_u1_2d --release
```

## Overview

This example tests the `LatticeGaugeField` implementation against the **exact analytical
solution** of 2D U(1) lattice gauge theory. It thermalizes a field started at the identity
with Metropolis sweeps at each coupling $\beta$, measures the average plaquette, and compares the measurement
with $I_1(\beta)/I_0(\beta)$.

## Theory Background

The 2D U(1) lattice gauge theory is one of the few exactly solvable lattice gauge theories. In
the infinite-volume limit the average plaquette is

$$\langle P \rangle = \frac{I_1(\beta)}{I_0(\beta)}$$

where $I_n$ are modified Bessel functions of the first kind and $\beta$ is the inverse coupling
constant.

## Verification Strategy

1. **Identity Configuration Check**: For a "cold start" (all links = identity), the average
   plaquette is exactly 1.0 at every $\beta$. This check exercises the plaquette machinery,
   not the thermodynamics.

2. **Measured against exact**: On an `8x8` periodic lattice, 400 thermalization sweeps precede
   400 measured sweeps at each $\beta$ from 0.5 (strong coupling) to 10 (weak coupling). The
   measured $\langle P \rangle$ must lie within a per-$\beta$ band of $I_1(\beta)/I_0(\beta)$,
   from 0.050 at strong coupling to 0.010 at weak coupling, derived from the spread over 60
   seeds (see `AGREEMENT_TOLERANCES` in `main.rs`). Each band is below half of
   $1 - I_1/I_0$, so a field that never moves is reported. The run uses a fixed seed, so it
   reproduces exactly. This is the only check that tests the lattice and the physics together.

   The field starts at the identity, in the topological sector $Q = 0$. Metropolis tunnels
   between sectors hundreds of times per run at strong coupling and rarely at
   $\beta \ge 6$ (never in 60 runs at $\beta = 10$). There a random start stays in whatever
   sector it lands in and biases $\langle P \rangle$ low by up to `4e-2`. From the identity,
   the $\beta = 10$ run sits about `9e-4` above $I_1/I_0$. The exact finite-volume correction on
   this lattice accounts for `3.6e-4` of that; the rest is consistent with the run staying at
   $Q = 0$ and missing the weight of the other sectors.

3. **Reference Cross-Check**: Two independent algorithms compute $I_1(\beta)/I_0(\beta)$ and
   must agree to `1e-12`, so the reference curve is itself sound:
    - **Series expansion**: direct summation of the power series
    - **Miller's backward recurrence**: a numerically stable continued fraction

## Precision

`FloatType` is `f64`. The measurement is a Monte Carlo average whose seed-to-seed spread,
`8e-4` to `1.6e-2`, sits more than ten orders of magnitude above `f64` rounding. `Float106` would only
sharpen the reference curve.

## Reference

> M. Creutz, *Quarks, Gluons and Lattices*, Cambridge University Press (1983), Chapter 8

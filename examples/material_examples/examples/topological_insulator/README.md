# Topological Insulator Analysis (Berry Curvature)

A demonstration of **Topological Data Analysis** in Quantum Materials using DeepCausality.

## Overview

In Solid State Physics, the topology of the electron wavefunction over the Brillouin Zone (momentum space) determines the material's phase.
*   **Trivial Insulator**: Equivalent to vacuum.
*   **Topological Insulator (TI)**: Has a non-zero **Chern Number**. This topology forces the existence of conducting edge states ("bulk-boundary correspondence").

This example calculates the Chern Number $C$ for the **Qi-Wu-Zhang (QWZ) Model**, a prototype Chern Insulator.

## The Physics (QWZ Model)

The Hamiltonian is defined on a 2D square lattice:
$$ H(k) = \sin(k_x)\sigma_x + \sin(k_y)\sigma_y + (u + \cos(k_x) + \cos(k_y))\sigma_z $$

*   **Eigenstates**: calculated analytically/numerically for the lower energy band.
*   **Berry Connection ($U$)**: The overlap between neighboring quantum states $|\psi(k)\rangle$ and $|\psi(k+\delta k)\rangle$.
*   **Berry Flux ($F_{xy}$)**: The phase accumulated around a closed loop (plaquette) in momentum space.
    $$ F = \text{Im} \ln ( U_1 U_2 U_3 U_4 ) $$
*   **Chern Number**: The total flux over the entire Torus (Brillouin Zone), divided by $2\pi$. It must be an integer.

## Key Concepts

*   **`Complex64`**: High-precision complex number arithmetic.
*   **Manifold Integration**: We treat the Brillouin Zone as a discretized manifold and sum the curvature form (Flux) over all faces.

## Run Command

```bash
cargo run -p material_examples --example topological_insulator_example
```

## Expected Output

```text
[...] Analyzing Material: u = 3.0 (Expected: TRIVIAL (u > 2))
      Total Berry Flux: 0.0000 rad
      Chern Number: 0.00
      Verdict: TRIVIAL Insulator (C = 0)

[...] Analyzing Material: u = 1.0 (Expected: TOPOLOGICAL (-2 < u < 0 or 0 < u < 2))
      Total Berry Flux: 6.2832 rad
      Chern Number: 1.00
      Verdict: TOPOLOGICAL Insulator (C = 1)
      -> Protected edge states exist!

[...] Analyzing Material: u = -1.0 (Expected: TOPOLOGICAL (-2 < u < 0 or 0 < u < 2))
      Total Berry Flux: -6.2832 rad
      Chern Number: -1.00
      Verdict: TOPOLOGICAL Insulator (C = -1)
      -> Protected edge states exist!
```

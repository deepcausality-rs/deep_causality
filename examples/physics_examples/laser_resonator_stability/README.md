# Laser Resonator Stability

This example analyzes the stability of an optical cavity by propagating a Gaussian beam through optical elements using ABCD matrices.

## How to Run

```bash
cargo run -p physics_examples --example laser_resonator_stability
```

---

## Physics Overview

A laser cavity traps light between mirrors. For a stable mode to exist, the diffraction of the beam (spreading) must be counteracted by the focusing of lenses or curved mirrors.

We model the beam using the **Complex Beam Parameter** $q(z)$:
$$ \frac{1}{q(z)} = \frac{1}{R(z)} - i \frac{\lambda}{\pi w(z)^2} $$
*   $R(z)$: Radius of curvature of the wavefront.
*   $w(z)$: Beam spot size (radius).

Propagation through an optical element (Matrix M) transforms $q$:
$$ q_{out} = \frac{A q_{in} + B}{C q_{in} + D} $$

## Causal Chain

1.  **Beam Initialization**: Start with a defined waist $w_0$.
2.  **Drift**: Propagate through free space.
3.  **Lens**: Focus through a thermal lens.
4.  **Stability Check**: The `gaussian_q_propagation` wrapper automatically verifies the physical invariant `Im(q) > 0`. If the cavity is unstable, the beam size diverges (imaginary part goes to zero/negative in math), triggering a Causal Error.

## Key APIs

*   `deep_causality_physics::photonics::gaussian_q_propagation`
*   `deep_causality_physics::photonics::{AbcdMatrix, ComplexBeamParameter}`
*   `deep_causality_physics::photonics::lens_maker`

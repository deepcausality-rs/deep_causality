# Laser Resonator Stability

This example tests the stability of an optical cavity: it propagates a Gaussian beam through one full round trip with ABCD matrices and checks that the beam reproduces itself.

## How to Run

```bash
cargo run -p physics_examples --example laser_resonator_stability
```

---

## Physics Overview

A laser cavity traps light between mirrors. A stable mode exists only if lenses or curved mirrors focus the beam as fast as diffraction spreads it.

The **Complex Beam Parameter** $q(z)$ models the beam:
$$ \frac{1}{q(z)} = \frac{1}{R(z)} - i \frac{\lambda}{\pi w(z)^2} $$
*   $R(z)$: Radius of curvature of the wavefront.
*   $w(z)$: Beam spot size (radius).

Propagation through an optical element (Matrix M) transforms $q$:
$$ q_{out} = \frac{A q_{in} + B}{C q_{in} + D} $$

## Causal Chain

1.  **Beam Initialization**: Start at a waist $w_0$ = 1 mm, where $q = i z_R$.
2.  **Round Trip**: Six optical elements, each one `.bind` in a `CausalFlow`: out through a thermal lens to the far flat mirror, and back. `gaussian_q_propagation` enforces the physical invariant `Im(q) > 0`; a beam that diverges breaks it and sends the flow into the error channel.
3.  **Stability Check**: The product of the element matrices is the round-trip matrix $[[A, B], [C, D]]$. The cavity is stable when $m = (A + D)/2$ satisfies $-1 \le m \le 1$, and a stable cavity returns the $q$ it started with.

This cavity's round trip is exactly $-I$, so $m = -1$: the boundary of the stability range. The beam reproduces itself, but any drift in the thermal lens pushes the cavity out.

## Key APIs

*   `deep_causality_physics::gaussian_q_propagation`
*   `deep_causality_physics::{AbcdMatrix, ComplexBeamParameter}`
*   `deep_causality_physics::lens_maker`
*   `deep_causality_physics::beam_spot_size`
*   `deep_causality_tensor::EinSumOp::mat_mul` for the round-trip matrix

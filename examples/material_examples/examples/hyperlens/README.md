# Hyperbolic Metamaterial Lens (Hyperlens)

A demonstration of **Super-Resolution Imaging** using DeepCausality's `multivector` and `physics` crates.

## Overview

Traditional lenses are limited by the **Diffraction Limit**: they cannot resolve features smaller than half the wavelength of light ($\lambda/2$). This is because the light waves carrying precise details (high spatial frequencies, $k > k_0$) decay exponentially in vacuum (Evanescent Waves).

A **Hyperlens** uses a metamaterial with a hyperbolic dispersion relation (achieved by anisotropic permittivity $\epsilon$). This changes the "topology" of momentum space from a closed sphere (limiting max $k$) to an open hyperbola (allowing infinite $k$).

## The Physics

### Vacuum (Euclidean Metric)
The dispersion relation is spherical:
$$ k_x^2 + k_y^2 + k_z^2 = (\frac{\omega}{c})^2 $$
If $k_x$ is large (fine detail), $k_z^2$ becomes negative, so $k_z$ is imaginary $\to$ Decay.

### Hyperlens (Indefinite Metric)
The metamaterial has separate $\epsilon$ components with opposite signs. The underlying metric signature mimics Minkowski space (+ - -) in the spatial domain:
$$ \frac{k_x^2}{\epsilon_z} + \frac{k_z^2}{\epsilon_x} = (\frac{\omega}{c})^2 $$
If $\epsilon_z < 0 < \epsilon_x$, the equation becomes hyperbolic. Large $k_x$ values are balanced by large $k_z$ values, keeping the equation valid with **Real** $k_z$. The wave propagates!

## Key Concepts

*   **`Metric::Generic`**: We define a custom metric signature `Generic { p: 1, q: 2, r: 0 }` to represent the material property.
*   **Sub-Diffraction Imaging**: The simulation shows that for a source separation of 100nm (with 500nm light), the signal decays in vacuum but propagates in the Hyperlens.

## Run Command

```bash
cargo run -p material_examples --example hyperlens_example
```

## Expected Output

```text
[1] Defined Vacuum Metric: Euclidean(3)
[1] Defined Hyperlens Metric: Generic(1, 2, 0)

[3] Simulating Sub-Wavelength Transmission...
    Light Wavelength: 500 nm
    Source Separation: 100 nm (Sub-diffraction limit)

    -> Vacuum Verdict: k_z^2 is Negative.
       Result: EVANESCENT DECAY.

    -> Hyperlens Verdict: k_z^2 is Positive.
       Result: PROPAGATION.
       [SUCCESS] Super-Resolution Achieved.
```

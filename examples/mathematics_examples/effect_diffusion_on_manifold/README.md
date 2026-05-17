# Diffusion on a Manifold: Comonad x Monad

## Introduction

The heat equation is the canonical first PDE every engineering and physics student meets. It describes how anything that "spreads out" evolves over time: temperature in a metal rod, ink in water, smoke through a room, voltage on a leaky transmission line, pollutant concentration in a river, and the implied volatility surface in Black-Scholes option pricing. Same equation, different units.

Numerically, the explicit-Euler scheme used here is exactly what shows up in introductory computational physics, finite-difference CFD, image-processing blurring (a Gaussian blur is a few heat-equation steps), and even some forms of regularization in machine learning. Two things matter in practice: the stencil step (compute the Laplacian at every grid point), and the time step (advance the field by one tick). This example separates them cleanly. The stencil lives inside `extend`. The tick lives inside `bind`. A stability check sits between them so the simulation aborts cleanly if you push past the CFL limit rather than silently producing NaN soup.

The discrete heat equation on a 1D manifold uses two abstractions at once: `extend` for the spatial Laplacian, and `bind` for time-stepping inside the causal monad.

## How to Run

```bash
cargo run -p mathematics_examples --example effect_diffusion_on_manifold
```

## What It Demonstrates

The spatial operator and the temporal operator live in different layers of the same composition. `ManifoldWitness::extend` is a CoMonad operation; it walks every position of the field and reads neighbors. `CausalEffectPropagationProcessWitness::bind` is a Monad operation; it threads each new manifold through the next step.

The error path is wired in: if any vertex becomes non-finite (a sign of CFL violation), the step returns `fail(...)` and the chain short-circuits.

## Mathematical Content

The explicit-Euler 1D heat equation:

```
phi_{i, t+1} = phi_{i, t} + alpha * (phi_{i-1, t} + phi_{i+1, t} - 2 * phi_{i, t})
```

With Neumann boundaries (reflection at the ends), the total mass `sum phi` is conserved across steps. The example checks that invariant at the end.

## Key APIs

- `ManifoldWitness::extend` for one Laplacian sweep
- `ProcessWitness::bind` for one time step
- `effect_helpers::fail` for the stability gate

## Adaptation

- Raise `ALPHA` above 0.5 to trigger an instability and watch the short-circuit fire.
- Replace the 1D line with a 2D triangulation for a real PDE solver skeleton.
- Add a `Functor::fmap` to log a per-step energy norm.

# Hypersonic Dual-Time (2T) Tracking

## Avionics Background

This example tracks a Mach-10 glide vehicle by propagating its state linearly in a 6D conformal space and projecting
the result back to 3D, one `CausalFlow` pipeline (`predict -> observe -> derive`) per 100 Hz tick.

Hypersonic vehicles (Mach 5+) and Maneuvering Re-entry Vehicles (MaRVs) operate in regimes that defeat standard
tracking models. They use aerodynamic lift to perform high-G skips and turns, creating non-linear trajectories with
rapid "Jerk" (change in acceleration).
Traditional tracking filters (Kalman, EKF, Interacting Multiple Model) assume "benign" dynamics ($F = ma$). When a
target pulls 100G in a lateral turn, these filters suffer from **Model Mismatch**: lag, exploding covariance, or total
track loss.

## The Challenge

The task is to predict the future state of a "chaotic" target with bounded computation.

* **Non-Linearity**: The equations of motion in 3D are intractable closed-form.
* **Latency**: Solving these ODEs numerically introduces lag, which is fatal when intercepting a hypersonic threat.
* **Ambiguity**: Is the target turning, or just drag-decelerating?

## The DeepCausality Solution

This example applies the **Two-Time (2T) Physics** formalism of Itzhak Bars, in which complex 3D dynamics are often
"shadows" of simple motion in higher dimensions.

### 1. Conformal Phase Space (4, 2)

The example lifts the system from 3D Space + 1 Time ($R^{3,1}$) to a **6D Conformal Space** with signature $(4, 2)$:

* 4 Spatial Dimensions (+ + + +)
* 2 Time Dimensions (- -)
  This signature supports an $Sp(2, R)$ symmetry group, which unifies many dynamical systems.

### 2. Linear Propagation in 6D

In this 6D space, the particle is constrained to a null hypercone ($X^2 = 0$). Complex 3D forces (like inverse-square
central potentials or conformal acceleration) can appear as **linear free motion** or simple rotation in 6D:
$$ X(\tau) = e^{\mathcal{G}\tau} X(0) $$
The example performs the update on a `CausalMultiVector` with a constant generator, as one first-order `Euler` step per
tick ($X \leftarrow X + \mathcal{G}\,dt$). Each step costs a vector addition instead of a nonlinear ODE solve.

### 3. Shadow Projection (Gauge Fixing)

The "Observation" step projects the 6D state back to 3D by reading its `e1`, `e2`, `e3` components.

* **The Shadow**: The observed 3D world is a "gauge choice" (a slice) of the 6D state.
* **Result**: A linear update in 6D produces a 3D trajectory from which the `derive` step computes speed and G-load.
  The radar measurement update (`ConformalTracker::correct`) is a placeholder, so the example propagates without
  correction.

## Running the Example

```bash
cargo run -p avionics_examples --example hypersonic_2t
```

# Hypersonic Dual-Time (2T) Tracking

## Avionics Background
Hypersonic vehicles (Mach 5+) and Maneuvering Re-entry Vehicles (MaRVs) operate in regimes that defy standard tracking physics. They utilize aerodynamic lift to perform high-G skips and turns, creating highly non-linear trajectories with rapid "Jerk" (change in acceleration).
Traditional tracking filters (Kalman, EKF, Interacting Multiple Model) assume "benign" dynamics ($F = ma$). When a target pulls 100G in a lateral turn, these filters suffer from **Model Mismatch**, leading to lag, large covariance explosions, or total track loss.

## The Challenge
The challenge is to predict the future state of a "chaotic" target without infinite computational resources.
*   **Non-Linearity**: The equations of motion in 3D are intractable closed-form.
*   **Latency**: Solving these ODEs numerically introduces lag, which is fatal when intercepting a hypersonic threat.
*   **Ambiguity**: Is the target turning, or just drag-decelerating?

## The DeepCausality Solution
This example applies the cutting-edge **Two-Time (2T) Physics** formalism (pioneered by Itzhak Bars), which reveals that complex 3D dynamics are often "shadows" of simple motion in higher dimensions.

### 1. Conformal Phase Space (4, 2)
We lift the system from 3D Space + 1 Time ($R^{3,1}$) to a **6D Conformal Space** with signature $(4, 2)$:
*   4 Spatial Dimensions (+ + + +)
*   2 Time Dimensions (- -)
This signature supports an $Sp(2, R)$ symmetry group, which unifies many dynamical systems.

### 2. Linear Propagation in 6D
In this 6D space, the particle is constrained to a null hypercone ($X^2 = 0$). Remarkably, highly complex 3D forces (like inverse-square central potentials or conformal acceleration) can appear as **linear free motion** or simple rotation in 6D:
$$ X(\tau) = e^{\mathcal{G}\tau} X(0) $$
We use `CausalMultiVector` to perform this linear update, which is computationally equivalent to a matrix multiplication—**orders of magnitude faster** than numerical integration.

### 3. Shadow Projection (Gauge Fixing)
The "Observation" step projects the 6D state back to the familiar 3D reality.
*   **The Shadow**: The 3D world we see is just a "gauge choice" (a slice) of the 6D reality.
*   **Result**: The example demonstrates how a simple linear update in 6D produces a trajectory in 3D that accurately tracks distance and speed, potentially offering a **Zero-Lag** tracking capability for next-gen missile defense.

## Running the Example
```bash
cargo run -p avionics_examples --example hypersonic_2t
```

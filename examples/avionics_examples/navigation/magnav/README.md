# Advanced Drone MagNav (Magnetic Navigation)

## Avionics Background
This example corrects INS drift with a particle filter that matches magnetometer readings against a magnetic anomaly map.

Modern navigation systems rely on GNSS (GPS) for absolute positioning. GPS signals are weak (~ -160 dBW) and easily jammed or spoofed in contested environments. Inertial Navigation Systems (INS) provide autonomy but suffer from integration drift that grows without bound (typically >1 km per hour of flight for tactical-grade sensors). **Magnetic Navigation (MagNav)** corrects this drift with the Earth's crustal magnetic anomaly field, a stable, unjammable "fingerprint" of the terrain.

## The Challenge
The core engineering problem in MagNav is **real-time map matching** under uncertainty.
1.  **Non-Uniqueness**: Many locations may share the same reading (Perceptual Aliasing).
2.  **Non-Linearity**: The magnetic map is non-linear and "rough", which makes standard Extended Kalman Filters (EKF) diverge.
3.  **Sensor Noise**: Magnetometers are noisy and pick up the drone's own electronics.

The system must correlate noisy observations $z_t$ with a high-resolution grid map $h(x_t)$ to estimate the posterior distribution $P(x_t | z_{1:t})$.

## The DeepCausality Solution
The example implements a **Causal Particle Filter (Sequential Monte Carlo)**:

### 1. Map Storage (`CausalTensor`)
A `CausalTensor` stores the Magnetic Anomaly Map with contiguous memory and constant-time random access for each of the 1000 particles.
*   **Bilinear Interpolation**: `model.rs` samples between grid points to support continuous particle positions.

### 2. Causal Bayesian Update (`PropagatingEffect`)
The "Measurement Update" runs inside the `PropagatingEffect` monad.
*   **Decoupled Logic**: The observation ($z_t$) is a causal effect that binds to the state.
*   **Likelihood Calculation**:
    $$ w_t^{(i)} \propto w_{t-1}^{(i)} \cdot \exp\left(-\frac{(z_t - h(x_t^{(i)}))^2}{2\sigma^2}\right) $$
    Each weight update consumes only the current observation, so the data drives the probability in a causal chain.

### 3. Convergence & Resilience
*   **Initialization**: Particles start in a Gaussian cloud (σ = 200 m) around the a-priori fix.
*   **Convergence**: As the drone moves, the filter converges toward the true position (Ground Truth) by down-weighting particles that do not match the magnetic sequence.
*   **Recovery**: If all weights collapse to near zero, the filter resets them to uniform.

## Mathematical Details
*   **State Space**: $x_t = [p_x, p_y]^T$; the INS velocity $[v_x, v_y]$ enters as the control input of the constant-velocity motion model.
*   **Observation Model**: $z_t = \text{Map}(p_x, p_y) + \mathcal{N}(0, R)$
*   **Resampling**: Systematic resampling, triggered when the effective sample size falls below half the particle count, prevents particle degeneracy.

## Running the Example
```bash
cargo run -p avionics_examples --example magnav
```

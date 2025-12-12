# Advanced Drone MagNav (Magnetic Navigation)

## Avionics Background
Modern navigation systems rely heavily on GNSS (GPS) for absolute positioning. However, GPS signals are weak (~ -160 dBW), easily jammed, or spoofed in contested environments. Inertial Navigation Systems (INS) provide autonomy but suffer from integration drift that grows unbounded over time (typically drifting >1 km per hour of flight for tactical grade sensors). **Magnetic Navigation (MagNav)** offers a robust solution by using the Earth's crustal magnetic anomaly field—a stable, unique, and unjammable "fingerprint" of the terrain—to continuously correct this INS drift.

## The Challenge
The primary engineering challenge in MagNav is **real-time map matching** under uncertainty.
1.  **Non-Uniqueness**: The magnetic field is not unique; many locations may have the same reading (Perceptual Aliasing).
2.  **Non-Linearity**: The magnetic map is highly non-linear and "rough", making standard Extended Kalman Filters (EKF) diverge.
3.  **Sensor Noise**: Magnetometers are noisy and affected by the drone's own electronics.

The system must correlate noisy observations $z_t$ with a high-resolution grid map $h(x_t)$ to estimate the posterior distribution $P(x_t | z_{1:t})$.

## The DeepCausality Solution
DeepCausality implements a **Causal Particle Filter (Sequencial Monte Carlo)** to solve this problem efficiently:

### 1. Efficient Map Storage (`CausalTensor`)
We uses `CausalTensor` to store the Magnetic Anomaly Map. This provides optimized memory layout and fast random access for millions of particles.
*   **Bilinear Interpolation**: The `model.rs` implements fast sampling between grid points to support continuous particle positions.

### 2. Causal Bayesian Update (`PropagatingEffect`)
The core innovation is wrapping the "Measurement Update" in the `PropagatingEffect` monad.
*   **Decoupled Logic**: The observation ($z_t$) is treated as a causal effect that binds to the state.
*   **Likelihood Calculation**:
    $$ w_t^{(i)} \propto w_{t-1}^{(i)} \cdot \exp\left(-\frac{(z_t - h(x_t^{(i)}))^2}{2\sigma^2}\right) $$
    This ensures that the "Data" drives the "Probability" in a strictly causal chain, preventing future information leakage in simulations.

### 3. Convergence & Resilience
The example demonstrates a "Global Localization" problem:
*   **Initialization**: Particles are scattered with high uncertainty.
*   **Convergence**: As the drone simulates movement, the filter rapidly converges to the true position (Ground Truth) by eliminating particles that do not match the magnetic sequence.

## Mathematical Details
*   **State Space**: $x_t = [p_x, p_y, v_x, v_y]^T$
*   **Observation Model**: $z_t = \text{Map}(p_x, p_y) + \mathcal{N}(0, R)$
*   **Resampling**: Uses a "Low Variance" or "Systematic" resampling approach (simplified in this example) to prevent particle degeneracy.

## Running the Example
```bash
cargo run -p avionics_examples --example magnav
```

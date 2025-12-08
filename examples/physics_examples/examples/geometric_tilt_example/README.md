# Geometric Tilt Estimator with Adaptive Gravity Observer

This example demonstrates a robust IMU sensor fusion system using DeepCausality's monadic composition with Geometric Algebra.

## How to Run

From the root of the `deep_causality` project, run:

```bash
cargo run -p physics_examples --example geometric_tilt_example
```

---

## Engineering Value

1. **Eliminates Gimbal Lock**: Uses Geometric Algebra (Rotors) instead of Euler angles
2. **Dynamic Calibration**: Adaptive Kalman filter continuously refines gravity estimate
3. **Motion Rejection**: Detects linear acceleration and skips updates to avoid corruption
4. **Hardware Independence**: Pure causal kernel, portable across any platform

---

## Causal Chain

```text
[Step 1] Gyro Integration    → Predict orientation: R_new = R * exp(-0.5·Ω·dt)
                                       ↓
[Step 2] Motion Detection    → Check |accel| vs expected gravity
                                       ↓
[Step 3] Kalman Update       → Update gravity estimate (skip if motion detected)
                                       ↓
[Step 4] Tilt Correction     → Align body gravity with world reference
```

---

## Key Components

### Orientation Tracking (Rotors)

Instead of Euler angles (which suffer from gimbal lock), we use **Rotors** from Geometric Algebra:

```
R_new = R_old × (1 - 0.5 × Ω × dt)
```

where Ω is the angular velocity as a bivector.

### Adaptive Gravity Observer (Kalman Filter)

Distinguishes between tilting and linear acceleration:

- **Innovation**: `y = z - x_pred` (measurement vs prediction)
- **Kalman Gain**: `K = P × (P + R)^-1`
- **Adaptive R**: `R_eff = R_base × (1 + scale × |gyro|)`

When rapid rotation is detected, measurement trust is reduced.

### Motion Detection

If `|accel| - 9.81| > threshold`, the system assumes linear acceleration and skips the measurement update to avoid corrupting the gravity estimate.

### Tilt Correction

Uses Geometric Algebra to compute a correction rotor that aligns the estimated body gravity with the world reference:

```
R_correction = normalize(1 + g_ref × g_world_est)
```

---

## Tunable Parameters

See `config.rs` for adjustable constants:

| Parameter | Default | Purpose |
|-----------|---------|---------|
| `Q_DIAG` | 0.0 | Process noise (gravity drift) |
| `R_BASE` | 0.1 | Accelerometer noise variance |
| `MOTION_THRESHOLD` | 2.0 | Motion detection sensitivity (m/s²) |
| `GYRO_SCALE` | 2.0 | Adaptive R scaling factor |
| `TILT_CORRECTION_ALPHA` | 0.1 | Correction blending (0=smooth, 1=aggressive) |

---

## Reference

Based on Mohammad Javad Azadi's Reaction-Wheel Unicycle work:
- https://iamazadi.github.io/Porta.jl/dev/reactionwheelunicycle.html

# Summary
- **Context**: The MHD module defines newtype wrappers for physical quantities with validation constraints to enforce physical invariants.
- **Bug**: Four types (`LarmorRadius`, `DebyeLength`, `PlasmaFrequency`, `Conductivity`) derive `Default`, which creates a value of `0.0`, but their `new()` constructors reject `0.0` as invalid because these quantities must be strictly positive (`> 0`).
- **Actual vs. expected**: The `Default` implementation creates invalid instances that violate the documented physical constraints, bypassing the validation that `new()` enforces.
- **Impact**: Code can create physically invalid quantities through `Default::default()`, breaking type safety guarantees and potentially causing incorrect physics calculations or runtime errors when these invalid values are used.

# Code with bug

```rust
/// Larmor Radius ($r_L$). Gyroradius of a charged particle.
/// Unit: Meters (m). Constraint: > 0.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)] // <-- BUG 🔴 Default creates 0.0, violating > 0 constraint
pub struct LarmorRadius(f64);

impl LarmorRadius {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if val <= 0.0 { // <-- Rejects 0.0
            return Err(PhysicsError::new(
                PhysicsErrorEnum::PhysicalInvariantBroken("Larmor Radius must be positive".into()),
            ));
        }
        Ok(Self(val))
    }
    // ...
}
```

```rust
/// Debye Length ($\lambda_D$). Screening length in plasma.
/// Unit: Meters (m). Constraint: > 0.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)] // <-- BUG 🔴 Default creates 0.0, violating > 0 constraint
pub struct DebyeLength(f64);

impl DebyeLength {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if val <= 0.0 { // <-- Rejects 0.0
            return Err(PhysicsError::new(
                PhysicsErrorEnum::PhysicalInvariantBroken("Debye Length must be positive".into()),
            ));
        }
        Ok(Self(val))
    }
    // ...
}
```

```rust
/// Plasma Frequency ($\omega_{pe}$). Natural oscillation frequency.
/// Unit: Rad/s. Constraint: > 0.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)] // <-- BUG 🔴 Default creates 0.0, violating > 0 constraint
pub struct PlasmaFrequency(f64);

impl PlasmaFrequency {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if val <= 0.0 { // <-- Rejects 0.0
            return Err(PhysicsError::new(
                PhysicsErrorEnum::PhysicalInvariantBroken(
                    "Plasma Frequency must be positive".into(),
                ),
            ));
        }
        Ok(Self(val))
    }
    // ...
}
```

```rust
/// Electrical Conductivity ($\sigma$).
/// Unit: Siemens/m (S/m). Constraint: > 0.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)] // <-- BUG 🔴 Default creates 0.0, violating > 0 constraint
pub struct Conductivity(f64);

impl Conductivity {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if val <= 0.0 { // <-- Rejects 0.0
            return Err(PhysicsError::new(
                PhysicsErrorEnum::PhysicalInvariantBroken("Conductivity must be positive".into()),
            ));
        }
        Ok(Self(val))
    }
    // ...
}
```

# Evidence

## Example

Consider this scenario:

```rust
// Using Default to create a LarmorRadius
let radius: LarmorRadius = Default::default();
// radius.value() == 0.0

// But trying to create the same value explicitly fails
let result = LarmorRadius::new(0.0);
// result.is_err() == true
// Error: "Larmor Radius must be positive"
```

The `Default` trait creates an instance with `0.0`, but the documented constraint says "Constraint: > 0", and the `new()` constructor enforces this by rejecting `<= 0.0`. This is a contradiction - `Default` produces values that the type's own validation logic considers invalid.

## Inconsistency with own spec

### Reference spec

From `deep_causality_physics/src/mhd/quantities.rs`:

```rust
/// Larmor Radius ($r_L$). Gyroradius of a charged particle.
/// Unit: Meters (m). Constraint: > 0.
```

```rust
/// Debye Length ($\lambda_D$). Screening length in plasma.
/// Unit: Meters (m). Constraint: > 0.
```

```rust
/// Plasma Frequency ($\omega_{pe}$). Natural oscillation frequency.
/// Unit: Rad/s. Constraint: > 0.
```

```rust
/// Electrical Conductivity ($\sigma$).
/// Unit: Siemens/m (S/m). Constraint: > 0.
```

### Current code

All four types derive `Default`:
```rust
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct LarmorRadius(f64);
// ... same for DebyeLength, PlasmaFrequency, Conductivity
```

And all reject `0.0` in their constructors:
```rust
pub fn new(val: f64) -> Result<Self, PhysicsError> {
    if val <= 0.0 {
        return Err(PhysicsError::new(
            PhysicsErrorEnum::PhysicalInvariantBroken("... must be positive".into()),
        ));
    }
    Ok(Self(val))
}
```

### Contradiction

The documentation explicitly states "Constraint: > 0" (must be strictly positive), and the `new()` constructor enforces this with `if val <= 0.0`. However, the derived `Default` trait creates a value of `0.0`, which:
1. Violates the documented constraint
2. Would be rejected by `new()`
3. Represents an invalid physical state (you cannot have a Larmor radius of zero, a Debye length of zero, etc.)

## Failing test

### Test script

```rust
/*
 * Test to demonstrate that Default trait creates invalid physical quantities
 * that violate the physical constraints enforced by the new() constructor.
 */

use deep_causality_physics::{
    Conductivity, DebyeLength, LarmorRadius, PlasmaFrequency,
};

#[test]
fn test_larmor_radius_default_violates_constraint() {
    // LarmorRadius requires > 0 (must be positive)
    // But Default creates 0.0
    let default_val: LarmorRadius = Default::default();

    // Try to create the same value using new()
    let result = LarmorRadius::new(default_val.value());

    // This should fail because 0.0 is not positive
    assert!(result.is_err(),
        "LarmorRadius::new(0.0) should fail, but Default::default() creates 0.0");
}

#[test]
fn test_debye_length_default_violates_constraint() {
    // DebyeLength requires > 0 (must be positive)
    // But Default creates 0.0
    let default_val: DebyeLength = Default::default();

    // Try to create the same value using new()
    let result = DebyeLength::new(default_val.value());

    // This should fail because 0.0 is not positive
    assert!(result.is_err(),
        "DebyeLength::new(0.0) should fail, but Default::default() creates 0.0");
}

#[test]
fn test_plasma_frequency_default_violates_constraint() {
    // PlasmaFrequency requires > 0 (must be positive)
    // But Default creates 0.0
    let default_val: PlasmaFrequency = Default::default();

    // Try to create the same value using new()
    let result = PlasmaFrequency::new(default_val.value());

    // This should fail because 0.0 is not positive
    assert!(result.is_err(),
        "PlasmaFrequency::new(0.0) should fail, but Default::default() creates 0.0");
}

#[test]
fn test_conductivity_default_violates_constraint() {
    // Conductivity requires > 0 (must be positive)
    // But Default creates 0.0
    let default_val: Conductivity = Default::default();

    // Try to create the same value using new()
    let result = Conductivity::new(default_val.value());

    // This should fail because 0.0 is not positive
    assert!(result.is_err(),
        "Conductivity::new(0.0) should fail, but Default::default() creates 0.0");
}

fn main() {
    test_larmor_radius_default_violates_constraint();
    test_debye_length_default_violates_constraint();
    test_plasma_frequency_default_violates_constraint();
    test_conductivity_default_violates_constraint();
    println!("All tests passed - Default creates invalid physical quantities!");
}
```

### Test output

```
running 4 tests
test test_debye_length_default_violates_constraint ... ok
test test_conductivity_default_violates_constraint ... ok
test test_larmor_radius_default_violates_constraint ... ok
test test_plasma_frequency_default_violates_constraint ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

The tests pass, confirming that `Default::default()` creates values that would be rejected by `new()`.

# Full context

The `deep_causality_physics/src/mhd/quantities.rs` file defines newtype wrappers for physical quantities used in magnetohydrodynamics (MHD) simulations. These types are designed to enforce physical invariants through validation in their constructors.

The module contains eight quantity types:
1. `AlfvenSpeed` - Speed of magnetic waves (constraint: `>= 0`)
2. `PlasmaBeta` - Ratio of thermal to magnetic pressure (constraint: `>= 0`)
3. `MagneticPressure` - Energy density of magnetic field (constraint: `>= 0`)
4. `Diffusivity` - Magnetic diffusivity (constraint: `>= 0`)
5. **`LarmorRadius`** - Gyroradius of charged particles (constraint: `> 0`) 🔴
6. **`DebyeLength`** - Screening length in plasma (constraint: `> 0`) 🔴
7. **`PlasmaFrequency`** - Natural oscillation frequency (constraint: `> 0`) 🔴
8. **`Conductivity`** - Electrical conductivity (constraint: `> 0`) 🔴

The first four types (marked with `>= 0`) correctly allow `0.0` as a valid value, so their `Default` implementation is correct. However, the last four types (marked with `> 0`) require strictly positive values, making `0.0` invalid.

These quantities are used throughout the MHD module:
- `alfven_speed_kernel()` in `ideal.rs` creates `AlfvenSpeed` from magnetic field and density
- `magnetic_pressure_kernel()` in `ideal.rs` creates `MagneticPressure`
- `debye_length_kernel()` in `plasma.rs` creates `DebyeLength` from temperature and density
- `larmor_radius_kernel()` in `plasma.rs` creates `LarmorRadius` from mass, velocity, charge, and magnetic field
- The `wrappers.rs` file wraps these kernels in `PropagatingEffect` for use in the causal framework

The types are also used in test files across the codebase. The existing tests in `tests/mhd/quantities_tests.rs` actually test that `Default::default()` returns `0.0` (lines 142-163), but fail to recognize that this violates the physical constraints for types requiring `> 0`.

# Why has this bug gone undetected?

This bug has gone undetected for several reasons:

1. **Direct use is likely rare**: The codebase appears to primarily create these quantities through the validated constructor functions (`new()`) or computational kernels (like `debye_length_kernel()`, `larmor_radius_kernel()`), not through `Default::default()`.

2. **Tests validate the wrong behavior**: The existing tests in `tests/mhd/quantities_tests.rs` (lines 142-163) explicitly test that `Default::default()` returns `0.0`, treating this as expected behavior rather than a bug. These tests pass, giving false confidence that the implementation is correct.

3. **Validation is bypassed**: The `Default` trait bypasses the validation logic in `new()`. Code that uses `Default::default()` never triggers the error checking that would catch the invalid `0.0` value.

4. **Physical invalidity is subtle**: While `0.0` is physically meaningless for these quantities (you can't have a zero Larmor radius or zero conductivity in real plasma), using these invalid values in calculations might not immediately cause obvious errors - they might just produce incorrect results that could be mistaken for edge cases or numerical issues.

5. **Type system provides false safety**: Because the types can be created through `Default`, developers might assume they're valid, trusting the type system's guarantees without realizing the invariant is violated.

6. **No runtime usage detected**: The grep searches show no code currently using `Default::default()` for these types in production code (only in tests). This suggests the feature is present but unused, so the bug hasn't manifested in practice yet.

# Recommended fix

Remove the `Default` derive for the four types that require strictly positive values:

```rust
/// Larmor Radius ($r_L$). Gyroradius of a charged particle.
/// Unit: Meters (m). Constraint: > 0.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)] // <-- FIX 🟢 Removed Default
pub struct LarmorRadius(f64);
```

Apply the same fix to `DebyeLength`, `PlasmaFrequency`, and `Conductivity`.

Note: Keep `Default` for `AlfvenSpeed`, `PlasmaBeta`, `MagneticPressure`, and `Diffusivity` since their constraints (`>= 0`) correctly allow `0.0`.

The tests in `tests/mhd/quantities_tests.rs` (lines 142-163) that test `Default::default()` for these four types should also be removed, as they will no longer compile once `Default` is removed.

# Related bugs

This same pattern may exist in other physics modules. The file is located at `deep_causality_physics/src/mhd/quantities.rs`, and similar `quantities.rs` files exist in:
- `condensed/quantities.rs`
- `electromagnetism/quantities.rs`
- `materials/quantities.rs`
- `quantum/quantities.rs`
- `photonics/quantities.rs`
- `dynamics/quantities.rs`
- `nuclear/quantities.rs`
- `thermodynamics/quantities.rs`
- `relativity/quantities.rs`
- `fluids/quantities.rs`

Each of these should be audited to ensure that types with `> 0` constraints do not derive `Default`.

# Summary
- **Context**: The `kalman_filter_linear_kernel` function in `deep_causality_physics/src/dynamics/estimation.rs` implements the Kalman filter update (measurement correction) step, which is used for state estimation in physics applications like the geometric tilt estimator.
- **Bug**: Process noise covariance matrix Q is incorrectly added to the posterior covariance during the Kalman filter update step.
- **Actual vs. expected**: The standard Kalman filter update step should compute `P_new = (I - KH)P` (or Joseph form), but the implementation adds process noise to get `P_final = P_new + Q`, which violates the Kalman filter update equations.
- **Impact**: The posterior covariance will be systematically overestimated, leading to suboptimal Kalman gains in subsequent updates, degraded state estimation accuracy, and slower convergence of the filter.

# Code with bug
```rust
// 6. Process Noise Addition: P_final = P_new + Q
// We apply process noise here effectively preparing P for the next prediction step (or representing posterior uncertainty including process diffusion).
if p_new.shape() != process_noise.shape() {
    return Err(PhysicsError::new(
        crate::PhysicsErrorEnum::DimensionMismatch(format!(
            "Posterior covariance shape {:?} != process noise shape {:?}",
            p_new.shape(),
            process_noise.shape()
        )),
    ));
}
let p_final = p_new.add(process_noise); // <-- BUG 🔴 Q should not be added here

Ok((x_new, p_final))
```

The bug is at `deep_causality_physics/src/dynamics/estimation.rs:240-251`.

# Evidence

## Example

Consider a simple 1D Kalman filter update with the following inputs:
- Predicted state: `x_pred = 10.0`
- Predicted covariance: `P_pred = 1.0`
- Measurement: `z = 12.0`
- Measurement matrix: `H = 1.0` (identity)
- Measurement noise: `R = 1.0`
- Process noise: `Q = 0.5`

**Step-by-step calculation (standard Kalman update):**
1. Innovation: `y = z - Hx = 12.0 - 10.0 = 2.0`
2. Innovation covariance: `S = HPH' + R = 1.0*1.0*1.0 + 1.0 = 2.0`
3. Kalman gain: `K = PH'S^-1 = 1.0*1.0*0.5 = 0.5`
4. State update: `x_new = x + Ky = 10.0 + 0.5*2.0 = 11.0`
5. Covariance update (Joseph form): `P_new = (I - KH)P = (1.0 - 0.5*1.0)*1.0 = 0.5`

**Expected result:** `P_new = 0.5` (process noise Q is not used in update step)

**Actual result with bug:** `P_final = P_new + Q = 0.5 + 0.5 = 1.0`

The implementation incorrectly doubles the posterior covariance by adding Q.

## Inconsistency with own spec / docstring

### Reference spec / comment

From the function documentation at `deep_causality_physics/src/dynamics/estimation.rs:98-117`:

```rust
/// Standard Linear Kalman Filter Update Step.
///
/// Implements the discrete-time Kalman filter update equations:
///
/// 1. Innovation Residual: $\mathbf{y} = \mathbf{z} - \mathbf{H}\hat{\mathbf{x}}$
/// 2. Innovation Covariance: $\mathbf{S} = \mathbf{H}\mathbf{P}\mathbf{H}^T + \mathbf{R}$
/// 3. Optimal Kalman Gain: $\mathbf{K} = \mathbf{P}\mathbf{H}^T \mathbf{S}^{-1}$
/// 4. State Update: $\hat{\mathbf{x}}_{new} = \hat{\mathbf{x}} + \mathbf{K}\mathbf{y}$
/// 5. Covariance Update: $\mathbf{P}_{new} = (\mathbf{I} - \mathbf{K}\mathbf{H})\mathbf{P}$
///
/// # Arguments
/// * `x_pred` - Predicted state vector ($\hat{\mathbf{x}}$).
/// * `p_pred` - Predicted estimate covariance ($\mathbf{P}$).
/// * `measurement` - Observation vector ($\mathbf{z}$).
/// * `measurement_matrix` - Observation model ($\mathbf{H}$).
/// * `measurement_noise` - Observation noise covariance ($\mathbf{R}$).
/// * `_process_noise` - Process noise covariance (unused in update step, typically used in prediction).
```

The docstring explicitly states:
1. This is a "Standard Linear Kalman Filter **Update Step**"
2. Lists 5 equations, ending with covariance update as step 5
3. Parameter `process_noise` is documented as **"unused in update step, typically used in prediction"**

### Current code

The implementation at lines 240-251:
```rust
// 6. Process Noise Addition: P_final = P_new + Q
let p_final = p_new.add(process_noise);
Ok((x_new, p_final))
```

### Contradiction

The docstring lists 5 steps for the Kalman update and explicitly states that process noise is "unused in update step", yet the implementation adds a 6th step that uses the process noise parameter. This directly contradicts both the documented behavior and standard Kalman filter theory.

## Failing test

### Test script

```rust
#[test]
fn test_kalman_filter_process_noise_bug() {
    // This test demonstrates that process noise Q is incorrectly added during the UPDATE step.
    // In standard Kalman filter theory, Q should ONLY be added in the PREDICTION step.
    //
    // Setup: Simple 1D Kalman filter
    // State x = 10.0, Covariance P = 1.0
    // Measurement z = 12.0, H = 1.0, R = 1.0
    // Process noise Q = 0.5 (NON-ZERO to expose the bug)

    let x_pred = CausalTensor::new(vec![10.0], vec![1, 1]).unwrap();
    let p_pred = CausalTensor::new(vec![1.0], vec![1, 1]).unwrap();
    let measurement = CausalTensor::new(vec![12.0], vec![1, 1]).unwrap();
    let h = CausalTensor::new(vec![1.0], vec![1, 1]).unwrap();
    let r = CausalTensor::new(vec![1.0], vec![1, 1]).unwrap();
    let q = CausalTensor::new(vec![0.5], vec![1, 1]).unwrap(); // Non-zero Q

    let result = kalman_filter_linear_kernel(&x_pred, &p_pred, &measurement, &h, &r, &q);
    assert!(result.is_ok());

    let (_x_new, p_new) = result.unwrap();

    // Standard Kalman UPDATE equations:
    // y = z - Hx = 12 - 10 = 2
    // S = HPH' + R = 1*1*1 + 1 = 2
    // K = PH'S^-1 = 1*1*0.5 = 0.5
    // P_new = (I - KH)P = (1 - 0.5)*1 = 0.5
    //
    // EXPECTED: P_new should be 0.5 (without Q added)
    // ACTUAL: Implementation adds Q, so P_final = 0.5 + 0.5 = 1.0

    let expected_p_correct = 0.5; // Correct Kalman update (without Q)
    let actual_p = p_new.data()[0];

    // BUG: The function adds Q in lines 240-251 of estimation.rs
    // This violates standard Kalman filter theory.
    assert!(
        (actual_p - expected_p_correct).abs() < 1e-10,
        "BUG: Process noise Q was added during UPDATE step. Expected P={}, Got P={}. \
         Process noise should only be added in PREDICTION step, not UPDATE step.",
        expected_p_correct,
        actual_p
    );
}
```

### Test output

```
running 1 test
test dynamics::estimation_tests::test_kalman_filter_process_noise_bug ... FAILED

failures:

---- dynamics::estimation_tests::test_kalman_filter_process_noise_bug stdout ----

thread 'dynamics::estimation_tests::test_kalman_filter_process_noise_bug' (5368) panicked at deep_causality_physics/tests/dynamics/estimation_tests.rs:254:5:
BUG: Process noise Q was added during UPDATE step. Expected P=0.5, Got P=1. Process noise should only be added in PREDICTION step, not UPDATE step.
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    dynamics::estimation_tests::test_kalman_filter_process_noise_bug

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 536 filtered out; finished in 0.00s
```

# Full context

The `kalman_filter_linear_kernel` function is a core component of the physics library's state estimation capabilities. It's used in the geometric tilt estimator example (`examples/physics_examples/geometric_tilt_example/model.rs`) for fusing accelerometer and gyroscope data to estimate device orientation.

The Kalman filter is a two-step recursive algorithm:
1. **Prediction step**: Projects the state and covariance forward in time, adding process noise Q to account for model uncertainty: `P_pred = F*P*F' + Q`
2. **Update step**: Corrects the prediction using a measurement, producing a posterior estimate with reduced uncertainty

The function signature and parameter names (`x_pred`, `p_pred`) indicate this is the update step. The function is wrapped by `kalman_filter_linear` in `deep_causality_physics/src/dynamics/wrappers.rs` and exposed as a public API.

In the geometric tilt example, the function is called from the `kalman_update` causaloid at line 221 of `model.rs`. The example handles motion detection, and when motion is detected, it manually adds Q to propagate uncertainty (lines 185-190). This suggests the developers understood that Q should be added separately for prediction, but didn't realize the update function was also adding Q internally, leading to potential double-counting in certain code paths.

## External documentation

- [Kalman Filter - Wikipedia](https://en.wikipedia.org/wiki/Kalman_filter)

**Update (measurement correction) equations:**
```
Innovation (measurement residual):
  ỹ_k = z_k - H_k x̂_k|k-1

Innovation covariance:
  S_k = H_k P_k|k-1 H_k^T + R_k

Optimal Kalman gain:
  K_k = P_k|k-1 H_k^T S_k^-1

Updated state estimate:
  x̂_k|k = x̂_k|k-1 + K_k ỹ_k

Updated covariance estimate:
  P_k|k = (I - K_k H_k) P_k|k-1
```

Note: Process noise Q does not appear in any of the update equations.

**Predict (time update) equations:**
```
Predicted state estimate:
  x̂_k|k-1 = F_k x̂_k-1|k-1 + B_k u_k

Predicted covariance estimate:
  P_k|k-1 = F_k P_k-1|k-1 F_k^T + Q_k  ← Q is only used here
```

# Why has this bug gone undetected?

The bug has remained undetected for several reasons:

1. **Test suite uses Q = 0**: The existing test `test_kalman_filter_linear_kernel_identity` at line 24 of `deep_causality_physics/tests/dynamics/estimation_tests.rs` passes `q = CausalTensor::new(vec![0.0], vec![1, 1])` with a comment "Process noise (unused)". Since Q is zero, adding it has no effect, so the test passes even with the bug present.

2. **Subtle impact**: The bug doesn't cause crashes or obviously wrong results. It inflates the posterior covariance, which makes the filter more conservative (trusting predictions over measurements). In many scenarios, this might appear as slightly suboptimal performance rather than an obvious failure.

3. **Misleading comment**: The comment at line 241 says "We apply process noise here effectively preparing P for the next prediction step", which sounds plausible but is incorrect. The prediction step should handle its own process noise addition. Mixing update and prediction responsibilities violates the separation of concerns in the Kalman filter algorithm.

4. **Parameter naming**: The parameter is named `process_noise` rather than something like `_unused_process_noise`, so it's natural for implementers to feel they should use it, even though the docstring says it's unused.

5. **Real-world usage may compensate**: In the geometric tilt example, when motion is detected, the code manually adds Q and skips the update step. When no motion is detected, it calls the buggy function which also adds Q. The effects might partially cancel out or be tuned around through the other parameters (R, Q values), making the bug less noticeable in practice.

# Recommended fix

Remove the process noise addition from the update step. The function should return `(x_new, p_new)` directly after computing the Joseph form covariance update:

```rust
// 5. Covariance Update (Joseph form):
// P_new = (I - K H) P (I - K H)^T + K R K^T
// ... (existing Joseph form calculation) ...
let p_new = joseph_main.add(&krkt);

Ok((x_new, p_new)) // <-- FIX 🟢 Return directly without adding Q
```

Additionally, either:
- Remove the `process_noise` parameter entirely from the update step function, OR
- Rename it to `_process_noise` and keep it for API compatibility but don't use it

If a combined update-and-predict function is desired, it should be a separate function with clearer documentation about what it does.

# Summary
- **Context**: `HalfLife` is a newtype wrapper representing radioactive half-life in seconds, used in nuclear physics calculations throughout the `deep_causality_physics` crate.
- **Bug**: `HalfLife::new(0.0)` succeeds without error, allowing construction of an invalid value.
- **Actual vs. expected**: The constructor accepts zero, but `radioactive_decay_kernel` explicitly rejects zero half-life as a `Singularity` error, violating the invariant that successfully constructed values should be usable in the API.
- **Impact**: Users can construct `HalfLife` values that appear valid but cause runtime errors when used in physics calculations, breaking the type safety guarantees of the newtype wrapper pattern.

# Code with bug

`deep_causality_physics/src/nuclear/quantities.rs`:
```rust
impl HalfLife {
    /// Creates a new `HalfLife` instance.
    ///
    /// # Errors
    /// Returns `PhysicsError` if `val < 0.0`.
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if val < 0.0 {  // <-- BUG 🔴 Should be `val <= 0.0` to reject zero
            return Err(PhysicsError::new(
                PhysicsErrorEnum::PhysicalInvariantBroken("Negative HalfLife".into()),
            ));
        }
        Ok(Self(val))
    }
}
```

# Evidence

## Example

Consider attempting to calculate radioactive decay with a half-life of zero:

```rust
// Step 1: Construction succeeds
let half_life = HalfLife::new(0.0).unwrap(); // ✓ No error
assert_eq!(half_life.value(), 0.0);

// Step 2: Attempt to use in physics calculation
let n0 = AmountOfSubstance::new(1000.0).unwrap();
let time = Time::new(1.0).unwrap();
let result = radioactive_decay_kernel(&n0, &half_life, &time);

// Step 3: Usage fails
assert!(result.is_err()); // ✗ Singularity error!
```

The radioactive decay formula is `N(t) = N₀ × 2^(-t/t_half)`. When `t_half = 0`, this becomes division by zero: `2^(-t/0)`, which is mathematically undefined. The decay rate constant is `λ = ln(2)/t_half`, so zero half-life implies `λ = ∞` (infinite decay rate), which is a physical singularity.

## Inconsistency with own spec

### Reference spec

From `deep_causality_physics/src/nuclear/physics.rs`:
```rust
/// Calculates the remaining amount of a radioactive substance: $N(t) = N_0 \cdot 2^{-t / t_{1/2}}$.
///
/// # Errors
/// * `Singularity` - If `half_life` is zero (infinite decay rate).
pub fn radioactive_decay_kernel(
    n0: &AmountOfSubstance,
    half_life: &HalfLife,
    time: &Time,
) -> Result<AmountOfSubstance, PhysicsError> {
    if half_life.value() == 0.0 {
        return Err(PhysicsError::new(crate::PhysicsErrorEnum::Singularity(
            "Radioactive half-life cannot be zero".into(),
        )));
    }
    // ...
}
```

The documentation explicitly states that zero half-life causes a `Singularity` error because it represents an infinite decay rate.

### Current code

From `deep_causality_physics/src/nuclear/quantities.rs`:
```rust
pub fn new(val: f64) -> Result<Self, PhysicsError> {
    if val < 0.0 {
        return Err(PhysicsError::new(
            PhysicsErrorEnum::PhysicalInvariantBroken("Negative HalfLife".into()),
        ));
    }
    Ok(Self(val))
}
```

### Contradiction

The constructor only rejects negative values (`val < 0.0`), allowing zero to pass through. However, the physics module explicitly documents and enforces that zero is invalid. This creates a broken contract: the type system claims the value is valid (construction succeeded), but the physics API rejects it as a singularity.

## Failing test

### Test script
```rust
/*
 * Demonstration test for HalfLife zero value bug
 *
 * This test demonstrates that HalfLife::new(0.0) succeeds but creates
 * an invalid value that cannot be used in physics calculations.
 */

use deep_causality_physics::{AmountOfSubstance, HalfLife, Time, radioactive_decay_kernel, PhysicsErrorEnum};

#[test]
fn test_halflife_zero_violates_usability_invariant() {
    // Step 1: Construction succeeds without error
    let half_life_result = HalfLife::new(0.0);
    assert!(
        half_life_result.is_ok(),
        "HalfLife::new(0.0) currently succeeds - this is the bug!"
    );

    let half_life = half_life_result.unwrap();
    assert_eq!(half_life.value(), 0.0);

    // Step 2: Try to use it in radioactive_decay_kernel
    let n0 = AmountOfSubstance::new(1000.0).unwrap();
    let time = Time::new(1.0).unwrap();

    let decay_result = radioactive_decay_kernel(&n0, &half_life, &time);

    // Step 3: Usage fails with Singularity error
    assert!(
        decay_result.is_err(),
        "radioactive_decay_kernel rejects zero half-life"
    );

    match decay_result.unwrap_err().0 {
        PhysicsErrorEnum::Singularity(msg) => {
            assert!(msg.contains("half-life") || msg.contains("zero"));
        }
        other => panic!("Expected Singularity error, got {:?}", other),
    }

    // This proves the bug: A value that can be constructed successfully
    // should be usable in the API, but zero half-life violates this invariant.
}

#[test]
fn test_halflife_positive_works_correctly() {
    // Positive values work as expected in both construction and usage
    let half_life = HalfLife::new(100.0).unwrap();
    let n0 = AmountOfSubstance::new(1000.0).unwrap();
    let time = Time::new(100.0).unwrap();

    let result = radioactive_decay_kernel(&n0, &half_life, &time);
    assert!(result.is_ok(), "Positive half-life should work");
}
```

### Test output
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.22s
     Running tests/halflife_bug_demo.rs (target/debug/deps/halflife_bug_demo-ef5bee85b54bfcdd)

running 2 tests
test test_halflife_zero_violates_usability_invariant ... ok
test test_halflife_positive_works_correctly ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

The test passes, confirming the bug: `HalfLife::new(0.0)` succeeds, but the resulting value causes a `Singularity` error when used in `radioactive_decay_kernel`.

# Full context

The `HalfLife` type is a core component of the nuclear physics module in `deep_causality_physics`. It represents the time required for a radioactive substance to decay to half its initial amount, measured in seconds.

The type is used primarily in the `radioactive_decay_kernel` function, which implements the standard radioactive decay equation `N(t) = N₀ × 2^(-t/t_half)`. This kernel is also exposed through a causal wrapper `radioactive_decay` in the `wrappers` module.

The newtype wrapper pattern is used throughout this crate to enforce physical invariants at the type level. For example, `Mass`, `Energy`, `Temperature`, and `Pressure` all reject negative values in their constructors. The intent is that if construction succeeds, the value should be valid for all operations in the API.

However, `HalfLife` has a special constraint beyond just being non-negative: it must also be non-zero. This is because:
1. The decay equation involves division by half-life
2. The decay constant λ = ln(2)/t_half becomes infinite when t_half = 0
3. Zero half-life has no physical meaning (it would imply instantaneous complete decay)

The bug breaks this contract. Users can create seemingly valid `HalfLife` values that cause runtime errors when used in physics calculations, defeating the purpose of the newtype wrapper.

## External documentation

- [Half-life - Wikipedia](https://en.wikipedia.org/wiki/Half-life)
> "Half-life is the time required for a quantity (of substance) to reduce to half of its initial value. The term is commonly used in nuclear physics to describe how quickly unstable atoms undergo radioactive decay."

- [Radioactive Decay Law](https://www.nuclear-power.com/nuclear-power/reactor-physics/atomic-nuclear-physics/radioactive-decay/radioactive-decay-law/)
> "The radioactive decay law: N(t) = N₀e^(-λt), where λ is the decay constant. The relationship between the decay constant λ and the half-life t₁/₂ is: λ = ln(2)/t₁/₂"

This confirms that zero half-life would require λ = ln(2)/0 = ∞, which is mathematically undefined.

# Why has this bug gone undetected?

The bug has gone undetected for several reasons:

1. **Existing tests explicitly accept zero**: The test suite includes `test_half_life_new_zero` which asserts that `HalfLife::new(0.0).is_ok()`. This test validates the buggy behavior rather than catching it.

2. **Separated concerns**: The validation logic is split between the constructor (which only checks `< 0.0`) and the physics kernel (which checks `== 0.0`). Developers might assume that if construction succeeds, the value is fully valid.

3. **Edge case**: Zero is an uncommon value in practice. Most radioactive isotopes have positive half-lives ranging from microseconds to billions of years. Users typically work with realistic physical constants rather than edge cases like zero.

4. **Physics kernel protects against it**: The `radioactive_decay_kernel` has defensive validation that catches zero half-life, preventing actual incorrect calculations. This means the bug causes a (somewhat) clean error rather than silent corruption.

5. **Test coverage gap**: While there's a test confirming that zero half-life causes an error in `radioactive_decay_kernel` (`test_radioactive_decay_zero_half_life`), there's no test validating the end-to-end invariant that all successfully constructed values should be usable in the API.

# Recommended fix

Change the validation condition in `HalfLife::new` to reject zero:

```rust
pub fn new(val: f64) -> Result<Self, PhysicsError> {
    if val <= 0.0 {  // <-- FIX 🟢 Changed from `< 0.0` to `<= 0.0`
        return Err(PhysicsError::new(
            PhysicsErrorEnum::PhysicalInvariantBroken(
                "HalfLife must be positive (zero implies infinite decay rate)".into()
            ),
        ));
    }
    Ok(Self(val))
}
```

Additionally, update the test `test_half_life_new_zero` to expect an error:

```rust
#[test]
fn test_half_life_new_zero() {
    let hl = HalfLife::new(0.0);
    assert!(hl.is_err(), "Zero half-life should be rejected");  // <-- FIX 🟢
}
```

# Related bugs

While investigating this bug, I discovered similar issues with other quantity types:

1. **`AmountOfSubstance` accepts zero**: Like `HalfLife`, `AmountOfSubstance::new(0.0)` succeeds, but causes errors in `ideal_gas_law_kernel` which performs division by moles. This is tested in `test_ideal_gas_law_kernel_zero_moles_error`.

2. **`HalfLife` accepts NaN**: `HalfLife::new(f64::NAN)` succeeds, creating a `HalfLife` with value `NaN`. This is clearly invalid but passes validation because `NaN < 0.0` evaluates to `false`. Similar issues likely exist in all quantity types that only check for negative values.

3. **No checks for infinity**: `HalfLife::new(f64::INFINITY)` succeeds. While positive infinity might have physical meaning for stable isotopes (infinite half-life = no decay), this should be explicitly considered and documented if intentional.

# Summary
- **Context**: `PhaseAngle` is a physics quantity type representing phase angles in radians, used in both quantum mechanics (for quantum phase) and relativistic calculations (for rapidity, a hyperbolic angle).
- **Bug**: `PhaseAngle::new()` accepts non-finite values (NaN, Infinity, -Infinity) without validation.
- **Actual vs. expected**: The constructor returns `Ok(PhaseAngle(value))` for any f64 input, including NaN and infinities, but should return `Err(PhysicsError)` for non-finite values to prevent numerical instability.
- **Impact**: Non-finite phase angles can silently propagate through physics calculations, leading to incorrect results in quantum operations and relativistic transformations without any error indication.

# Code with bug
```rust
// deep_causality_physics/src/quantum/quantities.rs, lines 11-14
impl PhaseAngle {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        Ok(Self(val))  // <-- BUG 🔴 No validation - accepts NaN and infinities
    }
```

# Evidence

## Example

Consider this sequence of operations in `time_dilation_angle_kernel`:

1. Two spacetime vectors are normalized: `gamma = dot / denom`
2. Even with checks, floating-point edge cases or extremely large gamma values can occur
3. `eta = gamma.acosh()` is computed - this can produce:
    - `NaN` when gamma < 1.0 (though there's a check, floating-point noise could slip through)
    - `Infinity` when gamma is extremely large (e.g., `f64::MAX.acosh() = inf`)
4. `PhaseAngle::new(eta)` is called - **currently accepts these invalid values without error**

Concrete demonstration:
```rust
// acosh() can produce non-finite values:
let large_gamma = f64::MAX;
let eta = large_gamma.acosh();  // Returns Infinity
let phase = PhaseAngle::new(eta);  // Returns Ok(PhaseAngle(inf))
// Bug: This should fail but succeeds
```

## Inconsistency within the codebase

### Reference code
`deep_causality_physics/src/units/probability.rs`
```rust
impl Probability {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if !(0.0..=1.0).contains(&val) {
            return Err(PhysicsError::new(PhysicsErrorEnum::NormalizationError(
                format!("Probability must be between 0 and 1, got {}", val),
            )));
        }
        Ok(Self(val))
    }
}
```

`deep_causality_physics/src/units/time.rs`
```rust
impl Time {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if val < 0.0 {
            return Err(PhysicsError::new(
                PhysicsErrorEnum::PhysicalInvariantBroken(
                    "Time cannot be negative (relative time duration assumed positive)".into(),
                ),
            ));
        }
        Ok(Self(val))
    }
}
```

### Current code
`deep_causality_physics/src/quantum/quantities.rs`
```rust
impl PhaseAngle {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        Ok(Self(val))  // No validation
    }
}
```

### Contradiction
The codebase establishes a clear pattern: all physics quantity constructors validate their inputs and return appropriate errors for invalid values. `Probability` checks for range violations, `Time` checks for negative values, but `PhaseAngle` performs no validation whatsoever. This breaks the established pattern and allows non-finite values (NaN, Infinity) to silently propagate through physics calculations.

## Failing test

### Test script
```rust
// deep_causality_physics/tests/quantum/quantities_tests.rs

#[test]
fn test_phase_angle_nan_should_fail() {
    let angle = PhaseAngle::new(f64::NAN);
    // BUG: This currently returns Ok, but should return Err
    assert!(angle.is_ok(), "BUG: PhaseAngle::new(NaN) should return Err, but returns Ok");
}

#[test]
fn test_phase_angle_infinity_should_fail() {
    let angle = PhaseAngle::new(f64::INFINITY);
    // BUG: This currently returns Ok, but should return Err
    assert!(angle.is_ok(), "BUG: PhaseAngle::new(INFINITY) should return Err, but returns Ok");
}

#[test]
fn test_phase_angle_neg_infinity_should_fail() {
    let angle = PhaseAngle::new(f64::NEG_INFINITY);
    // BUG: This currently returns Ok, but should return Err
    assert!(angle.is_ok(), "BUG: PhaseAngle::new(NEG_INFINITY) should return Err, but returns Ok");
}
```

### Test output
```
running 6 tests
test quantum::quantities_tests::test_phase_angle_infinity_should_fail ... ok
test quantum::quantities_tests::test_phase_angle_default ... ok
test quantum::quantities_tests::test_phase_angle_into_f64 ... ok
test quantum::quantities_tests::test_phase_angle_nan_should_fail ... ok
test quantum::quantities_tests::test_phase_angle_neg_infinity_should_fail ... ok
test quantum::quantities_tests::test_phase_angle_new_valid ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 533 filtered out; finished in 0.00s
```

All three bug demonstration tests pass (assert that `.is_ok()` is true), confirming that `PhaseAngle::new()` incorrectly accepts NaN and infinity values.

# Full context

`PhaseAngle` is a fundamental physics quantity type used across multiple modules in the `deep_causality_physics` crate:

1. **Quantum Mechanics Module** (`quantum/`): Represents the phase angle in quantum states and wave functions, which is a critical component of quantum interference and superposition calculations.

2. **Relativity Module** (`relativity/spacetime.rs`): Used to represent rapidity (η), a hyperbolic angle in special relativity. The `time_dilation_angle_kernel` function computes rapidity from timelike vectors using the formula:
   ```
   η = acosh(γ) where γ = (t₁·t₂)/(|t₁||t₂|)
   ```
   The result is wrapped in a `PhaseAngle` via `PhaseAngle::new(eta)` at line 126.

3. **Wrapper Integration** (`relativity/wrappers.rs`): The `time_dilation_angle` function returns `PropagatingEffect<PhaseAngle>`, propagating phase angle values through the effect system.

The physics calculations in `time_dilation_angle_kernel` already include extensive numerical stability checks (lines 107-123) to prevent invalid inputs to `acosh()`. However, these checks are not exhaustive - extreme values could still slip through, and the `acosh()` function itself can produce non-finite outputs (e.g., `f64::MAX.acosh() = inf`). When this happens, the non-finite value is passed to `PhaseAngle::new()`, which silently accepts it.

This creates a gap in the defensive programming: the physics kernel code carefully validates inputs, but the output wrapper (`PhaseAngle`) provides no validation, allowing invalid values to enter the system's type-safe wrappers and propagate to downstream calculations.

# Why has this bug gone undetected?

This bug has remained undetected for several reasons:

1. **Defensive upstream checks**: The `time_dilation_angle_kernel` function (the primary caller of `PhaseAngle::new()`) includes extensive validation before computing the rapidity. It checks for:
    - Zero or non-finite denominators (line 107)
    - Invalid gamma values < 1.0 (lines 119-122)

   These checks catch most problematic cases before they reach `PhaseAngle::new()`.

2. **Limited extreme value testing**: The existing test suite only tests `PhaseAngle` with normal values (π, 0.0, 1.23). There are no tests with edge cases like extremely large values, NaN, or infinity.

3. **Infrequent edge cases**: In typical physics simulations, the values passed to `PhaseAngle::new()` are usually well-behaved results from mathematical operations that have already been validated upstream. True edge cases (like `acosh()` returning infinity due to overflow) would only occur in extreme scenarios not commonly tested.

4. **Silent propagation**: When a non-finite `PhaseAngle` value does occur, it doesn't cause an immediate crash. Instead, it silently propagates through calculations, producing incorrect results that might be attributed to other factors or might not be noticed in complex multi-step physics simulations.

5. **Type safety illusion**: The `PhaseAngle` type wrapper gives the false impression of safety - code using `PhaseAngle` assumes it contains a valid finite value, and the `Result<Self, PhysicsError>` return type suggests validation is occurring, when in fact it's not.

# Recommended fix

Add finite value validation to `PhaseAngle::new()`:

```rust
impl PhaseAngle {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if !val.is_finite() {  // <-- FIX 🟢
            return Err(PhysicsError::new(
                PhysicsErrorEnum::NumericalInstability(
                    format!("PhaseAngle must be finite, got {}", val)
                ),
            ));
        }
        Ok(Self(val))
    }
}
```

This validation:
- Aligns with the established pattern in other physics quantity types
- Provides a clear error message when non-finite values are detected
- Acts as a safety net for any edge cases that slip through upstream validation
- Uses the appropriate `PhysicsErrorEnum::NumericalInstability` variant for non-finite values
- Maintains the existing API contract (`Result<Self, PhysicsError>`)

# Summary
- **Context**: The `calculate_potential_divergence` method in the Maxwell solver computes the divergence of the electromagnetic vector potential, a key quantity for checking the Lorenz gauge condition in electromagnetic simulations.
- **Bug**: The function only extracts the scalar component (index 0) from the inner product result, but when inputs contain non-vector (mixed-grade) components, the inner product produces additional non-scalar terms that are silently ignored.
- **Actual vs. expected**: The function silently accepts mixed-grade multivectors and returns an incomplete scalar value, whereas it should either validate that inputs are pure grade-1 vectors or return the complete result.
- **Impact**: Users receive incorrect divergence calculations when gradient or potential contain numerical errors or non-vector components, with no warning that the result is incomplete, potentially causing silent failures in electromagnetic simulations where gauge conditions must be precisely satisfied.

# Code with bug

```rust
pub fn calculate_potential_divergence(
    gradient: &CausalMultiVector<f64>,
    potential: &CausalMultiVector<f64>,
) -> Result<f64, PhysicsError> {
    Self::validate_compatibility(gradient, potential)?;

    // L = d . A (Scalar part of geometric product)
    let da = gradient.inner_product(potential);
    let scalar = *da.get(0).unwrap_or(&0.0);  // <-- BUG 🔴 Only extracts index 0, ignores other components

    if !scalar.is_finite() {
        return Err(PhysicsError::new(PhysicsErrorEnum::NumericalInstability(
            "Non-finite potential divergence".into(),
        )));
    }

    Ok(scalar)
}
```

# Evidence

## Example

Consider a gradient with both vector and bivector components (which could arise from numerical errors, bugs in gradient calculation, or user error):

```
Gradient: e₁ + 0.5·e₁₂  (mixed grade: 1 and 2)
Potential: e₂ + 0.3·e₁₂  (mixed grade: 1 and 2)
```

According to the geometric algebra inner product formula for mixed-grade multivectors, the inner product is:
```
∇ · A = ⟨∇A⟩_{s-r}
```

When the inputs have multiple grades, this produces multiple output grades:
- The scalar part (grade 0): -0.15
- The vector part (grade 1): -0.3

**The bug**: The function returns only -0.15 (the scalar), silently discarding the vector component of -0.3, which is 200% the magnitude of the scalar being returned.

## Failing test

### Test script

```rust
/*
 * Failing test demonstrating the bug in calculate_potential_divergence
 */

use deep_causality_multivector::{CausalMultiVector, Metric, MultiVector};
use deep_causality_physics::MaxwellSolver;

fn main() {
    let metric = Metric::Minkowski(4);

    println!("=== FAILING TEST: Non-vector inputs should be rejected ===\n");

    // Create gradient with both vector and bivector components (invalid!)
    let mut d_data = vec![0.0; 16];
    d_data[2] = 1.0;  // e1 (vector - valid)
    d_data[6] = 0.5;  // e12 (bivector - INVALID for a gradient!)
    let gradient = CausalMultiVector::new(d_data, metric).unwrap();

    // Create potential with both vector and bivector components (invalid!)
    let mut a_data = vec![0.0; 16];
    a_data[4] = 1.0;  // e2 (vector - valid)
    a_data[6] = 0.3;  // e12 (bivector - INVALID for a potential!)
    let potential = CausalMultiVector::new(a_data, metric).unwrap();

    println!("Input gradient (should be pure vector, grade 1):");
    print_grades(&gradient);

    println!("\nInput potential (should be pure vector, grade 1):");
    print_grades(&potential);

    // Call the function - it should ERROR but it doesn't!
    let result = MaxwellSolver::calculate_potential_divergence(&gradient, &potential);

    match result {
        Ok(div) => {
            println!("\n❌ TEST FAILED!");
            println!("Function returned Ok({}) but should have rejected non-vector inputs", div);

            // Show what was ignored
            let inner = gradient.inner_product(&potential);
            println!("\nFull inner product result:");
            print_components(&inner);

            println!("\nThe function only returned the scalar part ({}) and ignored:", div);
            for (i, &val) in inner.data().iter().enumerate() {
                if i != 0 && val.abs() > 1e-10 {
                    println!("  - Index {} (grade {}): {}", i, i.count_ones(), val);
                }
            }

            std::process::exit(1);
        }
        Err(e) => {
            println!("\n✅ TEST PASSED!");
            println!("Function correctly rejected invalid inputs with error: {:?}", e);
            std::process::exit(0);
        }
    }
}

fn print_grades(mv: &CausalMultiVector<f64>) {
    let mut grades_found = std::collections::BTreeSet::new();
    for (i, &val) in mv.data().iter().enumerate() {
        if val.abs() > 1e-10 {
            grades_found.insert(i.count_ones());
        }
    }
    if grades_found.len() == 1 {
        println!("  Pure grade {} multivector ✓", grades_found.iter().next().unwrap());
    } else {
        println!("  Mixed grades: {:?} ✗", grades_found);
    }
}

fn print_components(mv: &CausalMultiVector<f64>) {
    for (i, &val) in mv.data().iter().enumerate() {
        if val.abs() > 1e-10 {
            println!("  Index {} (grade {}): {}", i, i.count_ones(), val);
        }
    }
}
```

### Test output

```
=== FAILING TEST: Non-vector inputs should be rejected ===

Input gradient (should be pure vector, grade 1):
  Mixed grades: {1, 2} ✗

Input potential (should be pure vector, grade 1):
  Mixed grades: {1, 2} ✗

❌ TEST FAILED!
Function returned Ok(-0.15) but should have rejected non-vector inputs

Full inner product result:
  Index 0 (grade 0): -0.15
  Index 4 (grade 1): -0.3

The function only returned the scalar part (-0.15) and ignored:
  - Index 4 (grade 1): -0.3
```

# Full context

The `calculate_potential_divergence` function is part of the `MaxwellSolver` struct in `deep_causality_physics/src/electromagnetism/solver.rs`. This solver implements Maxwell's equations using geometric algebra (GA), where electromagnetic quantities are represented as multivectors.

In the geometric algebra formulation of electromagnetism:
- The gradient ∇ and vector potential A should be **pure grade-1 vectors** (4-vectors in Minkowski spacetime)
- The divergence ∇·A is computed via the inner product, which for two pure vectors yields a **scalar** (grade 0)
- This divergence is used to check the Lorenz gauge condition, which requires ∇·A ≈ 0

The function is called from:
1. **Maxwell electromagnetic simulation example** (`examples/physics_examples/maxwell_example/model.rs`): Used in a causal chain to compute plane wave propagation and verify gauge conditions
2. **Unit tests** (`deep_causality_physics/tests/electromagnetism/solver_tests.rs`): All existing tests use pure vectors as inputs, so the bug has not been detected

The `calculate_field_tensor` and `calculate_current_density` functions in the same file use `grade_projection` after their respective products to extract only the relevant grade components, which helps ensure correct results even with impure inputs. In contrast, `calculate_potential_divergence` only checks index 0 without validating the grade structure.

## External documentation

- [Geometric Algebra for Physicists (Doran & Lasenby), Section 7.2.2](https://www.cambridge.org/core/books/geometric-algebra-for-physicists/FB8D3ACB76AB3AB10BA7F27505925091)
```
The electromagnetic vector potential A is a vector (grade-1 multivector) in spacetime.
The divergence ∇·A is the scalar part of ∇A, computed via the inner product.
For pure vectors a and b: a·b = ⟨ab⟩₀ (scalar grade projection of geometric product)
```

- [CausalMultiVector Inner Product Documentation](./deep_causality_multivector/src/traits/multi_vector.rs)
```rust
/// Computes the inner product (left contraction) $A \cdot B$ (or $A \rfloor B$).
///
/// The inner product of a grade $r$ multivector $A$ and a grade $s$ multivector $B$
/// is the grade $s-r$ part of their geometric product.
/// $$ A \cdot B = \langle AB \rangle_{s-r} $$
///
/// For basis blades $e_I$ and $e_J$, $e_I \cdot e_J$ is non-zero only if $I \subseteq J$.
fn inner_product(&self, rhs: &Self) -> Self
```

# Why has this bug gone undetected?

The bug has remained undetected for several reasons:

1. **All existing tests use pure vectors**: The unit tests in `solver_tests.rs` only test with pure grade-1 vectors (e.g., e₀, e₁), which produce only scalar results from the inner product. In these cases, extracting index 0 is correct.

2. **Normal usage is correct**: In typical electromagnetic simulations, gradients and potentials are constructed as pure vectors. The Maxwell example in `examples/physics_examples/maxwell_example/model.rs` constructs proper 4-vectors, so the bug doesn't manifest.

3. **Inner product behavior for vectors**: For two grade-1 vectors a and b, the inner product a·b correctly produces only a scalar (grade |1-1| = 0), so `da.get(0)` captures the complete result. The bug only appears when inputs accidentally or erroneously contain higher-grade components.

4. **No validation of input grades**: Unlike `calculate_field_tensor` (which extracts grade 2) and `calculate_current_density` (which extracts grade 1), this function assumes the inner product result is purely scalar without validating that assumption. If numerical errors, bugs elsewhere, or incorrect API usage cause mixed-grade inputs, the function silently returns incomplete results.

5. **The finite check is insufficient**: The function only validates that the scalar at index 0 is finite, but doesn't check whether other components of the inner product result are non-zero, which would indicate invalid inputs or an incomplete calculation.

# Recommended fix

Validate that both inputs are pure grade-1 vectors before computing the divergence:

```rust
pub fn calculate_potential_divergence(
    gradient: &CausalMultiVector<f64>,
    potential: &CausalMultiVector<f64>,
) -> Result<f64, PhysicsError> {
    Self::validate_compatibility(gradient, potential)?;

    // Validate that inputs are pure vectors (grade 1) // <-- FIX 🟢
    Self::validate_pure_grade(gradient, 1, "gradient")?;  // <-- FIX 🟢
    Self::validate_pure_grade(potential, 1, "potential")?;

    // L = d . A (inner product of vectors gives scalar)
    let da = gradient.inner_product(potential);
    let scalar = *da.get(0).unwrap_or(&0.0);

    if !scalar.is_finite() {
        return Err(PhysicsError::new(PhysicsErrorEnum::NumericalInstability(
            "Non-finite potential divergence".into(),
        )));
    }

    Ok(scalar)
}

// Helper to add to the impl block
fn validate_pure_grade(
    mv: &CausalMultiVector<f64>,
    expected_grade: u32,
    context: &str,
) -> Result<(), PhysicsError> {
    for (i, &val) in mv.data().iter().enumerate() {
        if val.abs() > 1e-10 {
            let grade = i.count_ones();
            if grade != expected_grade {
                return Err(PhysicsError::new(PhysicsErrorEnum::InvalidInput(
                    format!(
                        "{} must be pure grade {} multivector, but contains grade {} component at index {}",
                        context, expected_grade, grade, i
                    ),
                )));
            }
        }
    }
    Ok(())
}
```

Alternatively, if mixed-grade inputs should be supported, extract the full inner product result or validate that non-scalar components are negligible.

# Summary
- **Context**: The dynamics quantities module defines type-safe wrappers for physical quantities (Mass, Speed, Length, etc.) with validation to ensure physical invariants (e.g., non-negative values) are maintained.
- **Bug**: The validation logic for quantities that must be non-negative (Mass, Speed, Length, Area, Volume, MomentOfInertia, Frequency) fails to reject NaN and positive infinity values.
- **Actual vs. expected**: The `new()` methods accept NaN and infinity values, allowing them to pass validation, whereas these values should be rejected as physically meaningless and computationally hazardous.
- **Impact**: NaN and infinity values silently propagate through physics calculations, producing invalid results without raising errors, which can corrupt entire calculation chains and make debugging extremely difficult.

# Code with bug

```rust
impl Mass {
    /// Creates a new `Mass` instance.
    ///
    /// # Errors
    /// Returns `PhysicsError::PhysicalInvariantBroken` if `val < 0.0`.
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if val < 0.0 {  // <-- BUG 🔴 NaN < 0.0 is false, so NaN passes validation
            return Err(PhysicsError::new(
                PhysicsErrorEnum::PhysicalInvariantBroken(format!(
                    "Mass cannot be negative: {}",
                    val
                )),
            ));
        }
        Ok(Self(val))  // <-- BUG 🔴 INFINITY also passes since INFINITY >= 0.0 is true
    }
    // ...
}
```

The same bug pattern appears in:
- `Speed::new()` (line 52)
- `Length::new()` (line 149)
- `Area::new()` (line 181)
- `Volume::new()` (line 213)
- `MomentOfInertia::new()` (line 245)
- `Frequency::new()` (line 277)

# Evidence

## Example

Consider the following step-by-step variable values:

1. Create a Mass with NaN:
   ```rust
   let mass = Mass::new(f64::NAN)
   ```
    - `val = NaN`
    - Check: `NaN < 0.0` → `false` (per IEEE 754, NaN comparisons are always false except !=)
    - Validation passes ✗ (should fail)
    - Returns: `Ok(Mass(NaN))`

2. Use this Mass in orbital velocity calculation:
   ```rust
   let radius = Length::new(1e6).unwrap()
   let speed = orbital_velocity_kernel(&mass, &radius)
   ```
    - `gm = NEWTONIAN_CONSTANT_OF_GRAVITATION * NaN` → `NaN`
    - `v = sqrt(NaN / 1e6)` → `NaN`
    - `Speed::new(NaN)` → Check: `NaN < 0.0` → `false`
    - Returns: `Ok(Speed(NaN))`

3. The NaN propagates through all subsequent calculations, silently corrupting results.

Similarly for infinity:
1. Create a Mass with infinity:
   ```rust
   let mass = Mass::new(f64::INFINITY)
   ```
    - `val = INFINITY`
    - Check: `INFINITY < 0.0` → `false`
    - Validation passes ✗ (should fail)
    - Returns: `Ok(Mass(INFINITY))`

## Failing test

### Test script

```rust
/*
 * Test to verify that quantities properly reject NaN and infinity values
 */

use deep_causality_physics::{Mass, Speed, Length, Area, Volume, MomentOfInertia, Frequency};

#[test]
fn test_mass_rejects_nan() {
    let mass_nan = Mass::new(f64::NAN);
    assert!(mass_nan.is_err(), "Mass should reject NaN values");
}

#[test]
fn test_mass_rejects_infinity() {
    let mass_inf = Mass::new(f64::INFINITY);
    assert!(mass_inf.is_err(), "Mass should reject INFINITY values");
}

#[test]
fn test_speed_rejects_nan() {
    let speed_nan = Speed::new(f64::NAN);
    assert!(speed_nan.is_err(), "Speed should reject NaN values");
}

#[test]
fn test_speed_rejects_infinity() {
    let speed_inf = Speed::new(f64::INFINITY);
    assert!(speed_inf.is_err(), "Speed should reject INFINITY values");
}

#[test]
fn test_length_rejects_nan() {
    let length_nan = Length::new(f64::NAN);
    assert!(length_nan.is_err(), "Length should reject NaN values");
}

#[test]
fn test_length_rejects_infinity() {
    let length_inf = Length::new(f64::INFINITY);
    assert!(length_inf.is_err(), "Length should reject INFINITY values");
}

#[test]
fn test_area_rejects_nan() {
    let area_nan = Area::new(f64::NAN);
    assert!(area_nan.is_err(), "Area should reject NaN values");
}

#[test]
fn test_volume_rejects_nan() {
    let volume_nan = Volume::new(f64::NAN);
    assert!(volume_nan.is_err(), "Volume should reject NaN values");
}

#[test]
fn test_moment_of_inertia_rejects_nan() {
    let moi_nan = MomentOfInertia::new(f64::NAN);
    assert!(moi_nan.is_err(), "MomentOfInertia should reject NaN values");
}

#[test]
fn test_frequency_rejects_nan() {
    let freq_nan = Frequency::new(f64::NAN);
    assert!(freq_nan.is_err(), "Frequency should reject NaN values");
}
```

### Test output

```
   Compiling deep_causality_physics v0.1.1 (/home/user/deep_causality/deep_causality_physics)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.25s
     Running tests/test_nan_infinity.rs (target/debug/deps/test_nan_infinity-38e6a34d8cc79a9a)
error: test failed, to rerun pass `-p deep_causality_physics --test test_nan_infinity`


running 10 tests
test test_frequency_rejects_nan ... FAILED
test test_area_rejects_nan ... FAILED
test test_length_rejects_infinity ... FAILED
test test_length_rejects_nan ... FAILED
test test_mass_rejects_infinity ... FAILED
test test_mass_rejects_nan ... FAILED
test test_moment_of_inertia_rejects_nan ... FAILED
test test_speed_rejects_infinity ... FAILED
test test_speed_rejects_nan ... FAILED
test test_volume_rejects_nan ... FAILED

failures:

---- test_frequency_rejects_nan stdout ----

thread 'test_frequency_rejects_nan' (3503) panicked at deep_causality_physics/tests/test_nan_infinity.rs:64:5:
Frequency should reject NaN values
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- test_area_rejects_nan stdout ----

thread 'test_area_rejects_nan' (3502) panicked at deep_causality_physics/tests/test_nan_infinity.rs:46:5:
Area should reject NaN values

---- test_length_rejects_infinity stdout ----

thread 'test_length_rejects_infinity' (3504) panicked at deep_causality_physics/tests/test_nan_infinity.rs:40:5:
Length should reject INFINITY values

---- test_length_rejects_nan stdout ----

thread 'test_length_rejects_nan' (3505) panicked at deep_causality_physics/tests/test_nan_infinity.rs:34:5:
Length should reject NaN values

---- test_mass_rejects_infinity stdout ----

thread 'test_mass_rejects_infinity' (3506) panicked at deep_causality_physics/tests/test_nan_infinity.rs:16:5:
Mass should reject INFINITY values

---- test_mass_rejects_nan stdout ----

thread 'test_mass_rejects_nan' (3507) panicked at deep_causality_physics/tests/test_nan_infinity.rs:10:5:
Mass should reject NaN values

---- test_moment_of_inertia_rejects_nan stdout ----

thread 'test_moment_of_inertia_rejects_nan' (3508) panicked at deep_causality_physics/tests/test_nan_infinity.rs:58:5:
MomentOfInertia should reject NaN values

---- test_speed_rejects_infinity stdout ----

thread 'test_speed_rejects_infinity' (3509) panicked at deep_causality_physics/tests/test_nan_infinity.rs:28:5:
Speed should reject INFINITY values

---- test_speed_rejects_nan stdout ----

thread 'test_speed_rejects_nan' (3510) panicked at deep_causality_physics/tests/test_nan_infinity.rs:22:5:
Speed should reject NaN values

---- test_volume_rejects_nan stdout ----

thread 'test_volume_rejects_nan' (3511) panicked at deep_causality_physics/tests/test_nan_infinity.rs:52:5:
Volume should reject NaN values


failures:
    test_area_rejects_nan
    test_frequency_rejects_nan
    test_length_rejects_infinity
    test_length_rejects_nan
    test_mass_rejects_infinity
    test_mass_rejects_nan
    test_moment_of_inertia_rejects_nan
    test_speed_rejects_infinity
    test_speed_rejects_nan
    test_volume_rejects_nan

test result: FAILED. 0 passed; 10 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

# Full context

The quantities module (`deep_causality_physics/src/dynamics/quantities.rs`) provides type-safe wrappers for fundamental physical quantities used throughout the physics engine. These types are designed to enforce physical invariants at construction time, preventing invalid values from entering the system.

These quantity types are used extensively across the codebase:
- In `kinematics.rs`: `kinetic_energy_kernel()` uses Mass and velocity to calculate kinetic energy
- In `astro/mechanics.rs`: `orbital_velocity_kernel()` and `escape_velocity_kernel()` use Mass and Length to calculate velocities
- In `waves/general.rs`: Uses Speed for wave propagation calculations
- In `mhd/` modules: Uses various quantities for magnetohydrodynamics calculations
- Throughout other physics domains (quantum, relativity, thermodynamics, etc.)

The validation in the `new()` methods is the primary defense against invalid physical values entering the system. Once a quantity is created, its value is accessed via the `value()` method and used directly in calculations without further validation.

The codebase does include `is_finite()` checks in some calculation functions (e.g., `kinematics.rs:49`), but these are defensive checks for intermediate calculation results, not substitutes for input validation. The assumption is that properly validated quantities should not need re-validation.

## External documentation

- [IEEE 754 Floating Point Standard - NaN comparison behavior](https://en.wikipedia.org/wiki/IEEE_754#Comparison_predicates)
```
According to IEEE 754, all comparisons with NaN (except !=) return false:
- NaN < x is false
- NaN <= x is false
- NaN > x is false
- NaN >= x is false
- NaN == x is false
- NaN != x is true (including NaN != NaN)

This means a simple check like `if val < 0.0` will not catch NaN values.
```

- [Rust f64 documentation](https://doc.rust-lang.org/std/primitive.f64.html#method.is_finite)
```
pub const fn is_finite(self) -> bool

Returns true if this number is neither infinite nor NaN.

Examples:
let f = 7.0f64;
let inf = f64::INFINITY;
let neg_inf = f64::NEG_INFINITY;
let nan = f64::NAN;

assert!(f.is_finite());
assert!(!inf.is_finite());
assert!(!neg_inf.is_finite());
assert!(!nan.is_finite());
```

# Why has this bug gone undetected?

This bug has gone undetected for several reasons:

1. **Unusual input values**: In normal usage, physics calculations with valid inputs produce valid outputs. NaN typically only arises from:
    - Division by zero (usually caught earlier)
    - Square root of negative numbers (often prevented by validation)
    - Propagation from external sources (parsing errors, sensor failures)

   Most test cases use "happy path" values like `10.0`, `100.0`, etc.

2. **IEEE 754 comparison semantics**: The non-intuitive behavior of NaN comparisons (all return false) means that a check like `val < 0.0` appears correct at first glance. Developers naturally think "if it's not negative, it must be valid," but NaN is neither negative nor non-negative in the usual sense.

3. **Test coverage gap**: The existing test suite (`tests/dynamics/quantities_tests.rs`) tests:
    - Valid positive values
    - Zero values
    - Negative values (correctly rejected)
    - The `unchecked` constructors
    - Type conversions

   But it doesn't test edge cases like NaN, infinity, or negative infinity.

4. **Implicit assumption**: There's an implicit assumption that if a value comes from a calculation in Rust, it's probably finite. However, Rust doesn't prevent NaN/infinity creation (e.g., `0.0/0.0` produces NaN, `1.0/0.0` produces infinity).

5. **Rust's permissive float handling**: Unlike some languages, Rust allows NaN and infinity to propagate silently through calculations without exceptions, making bugs harder to detect through runtime errors.

6. **Silent propagation**: Once a NaN enters the system through these constructors, it propagates silently through all downstream calculations, never triggering an error. The final results are simply wrong (NaN), but no panic or error is raised, making the root cause hard to trace.

# Recommended fix

Add explicit validation for finite values in all quantity constructors that should only accept finite, non-negative values:

```rust
impl Mass {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if !val.is_finite() {  // <-- FIX 🟢 Reject NaN and infinity first
            return Err(PhysicsError::new(
                PhysicsErrorEnum::PhysicalInvariantBroken(format!(
                    "Mass must be finite: {}",
                    val
                )),
            ));
        }
        if val < 0.0 {  // <-- FIX 🟢 Now this check is meaningful
            return Err(PhysicsError::new(
                PhysicsErrorEnum::PhysicalInvariantBroken(format!(
                    "Mass cannot be negative: {}",
                    val
                )),
            ));
        }
        Ok(Self(val))
    }
}
```

Apply the same fix to: Speed, Length, Area, Volume, MomentOfInertia, and Frequency.

Note: Quantities that allow negative values (Acceleration, Force, Torque) should also add `is_finite()` checks, but without the negativity check:

```rust
impl Acceleration {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::new(
                PhysicsErrorEnum::PhysicalInvariantBroken(format!(
                    "Acceleration must be finite: {}",
                    val
                )),
            ));
        }
        Ok(Self(val))
    }
}
```

# Related bugs

This same pattern likely exists in quantity types in other physics modules:
- `deep_causality_physics/src/electromagnetism/quantities.rs`
- `deep_causality_physics/src/quantum/quantities.rs`
- `deep_causality_physics/src/thermodynamics/quantities.rs`
- `deep_causality_physics/src/relativity/quantities.rs`
- `deep_causality_physics/src/fluids/quantities.rs`
- `deep_causality_physics/src/materials/quantities.rs`
- `deep_causality_physics/src/photonics/quantities.rs`
- `deep_causality_physics/src/nuclear/quantities.rs`
- `deep_causality_physics/src/mhd/quantities.rs`
- `deep_causality_physics/src/condensed/quantities.rs`

Each should be audited for the same validation issue.

# Summary
- **Context**: The dynamics quantities module defines type-safe wrappers for physical quantities (Mass, Speed, Length, etc.) with validation to ensure physical invariants (e.g., non-negative values) are maintained.
- **Bug**: The validation logic for quantities that must be non-negative (Mass, Speed, Length, Area, Volume, MomentOfInertia, Frequency) fails to reject NaN and positive infinity values.
- **Actual vs. expected**: The `new()` methods accept NaN and infinity values, allowing them to pass validation, whereas these values should be rejected as physically meaningless and computationally hazardous.
- **Impact**: NaN and infinity values silently propagate through physics calculations, producing invalid results without raising errors, which can corrupt entire calculation chains and make debugging extremely difficult.

# Code with bug

```rust
impl Mass {
    /// Creates a new `Mass` instance.
    ///
    /// # Errors
    /// Returns `PhysicsError::PhysicalInvariantBroken` if `val < 0.0`.
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if val < 0.0 {  // <-- BUG 🔴 NaN < 0.0 is false, so NaN passes validation
            return Err(PhysicsError::new(
                PhysicsErrorEnum::PhysicalInvariantBroken(format!(
                    "Mass cannot be negative: {}",
                    val
                )),
            ));
        }
        Ok(Self(val))  // <-- BUG 🔴 INFINITY also passes since INFINITY >= 0.0 is true
    }
    // ...
}
```

The same bug pattern appears in:
- `Speed::new()` (line 52)
- `Length::new()` (line 149)
- `Area::new()` (line 181)
- `Volume::new()` (line 213)
- `MomentOfInertia::new()` (line 245)
- `Frequency::new()` (line 277)

# Evidence

## Example

Consider the following step-by-step variable values:

1. Create a Mass with NaN:
   ```rust
   let mass = Mass::new(f64::NAN)
   ```
    - `val = NaN`
    - Check: `NaN < 0.0` → `false` (per IEEE 754, NaN comparisons are always false except !=)
    - Validation passes ✗ (should fail)
    - Returns: `Ok(Mass(NaN))`

2. Use this Mass in orbital velocity calculation:
   ```rust
   let radius = Length::new(1e6).unwrap()
   let speed = orbital_velocity_kernel(&mass, &radius)
   ```
    - `gm = NEWTONIAN_CONSTANT_OF_GRAVITATION * NaN` → `NaN`
    - `v = sqrt(NaN / 1e6)` → `NaN`
    - `Speed::new(NaN)` → Check: `NaN < 0.0` → `false`
    - Returns: `Ok(Speed(NaN))`

3. The NaN propagates through all subsequent calculations, silently corrupting results.

Similarly for infinity:
1. Create a Mass with infinity:
   ```rust
   let mass = Mass::new(f64::INFINITY)
   ```
    - `val = INFINITY`
    - Check: `INFINITY < 0.0` → `false`
    - Validation passes ✗ (should fail)
    - Returns: `Ok(Mass(INFINITY))`

## Failing test

### Test script

```rust
/*
 * Test to verify that quantities properly reject NaN and infinity values
 */

use deep_causality_physics::{Mass, Speed, Length, Area, Volume, MomentOfInertia, Frequency};

#[test]
fn test_mass_rejects_nan() {
    let mass_nan = Mass::new(f64::NAN);
    assert!(mass_nan.is_err(), "Mass should reject NaN values");
}

#[test]
fn test_mass_rejects_infinity() {
    let mass_inf = Mass::new(f64::INFINITY);
    assert!(mass_inf.is_err(), "Mass should reject INFINITY values");
}

#[test]
fn test_speed_rejects_nan() {
    let speed_nan = Speed::new(f64::NAN);
    assert!(speed_nan.is_err(), "Speed should reject NaN values");
}

#[test]
fn test_speed_rejects_infinity() {
    let speed_inf = Speed::new(f64::INFINITY);
    assert!(speed_inf.is_err(), "Speed should reject INFINITY values");
}

#[test]
fn test_length_rejects_nan() {
    let length_nan = Length::new(f64::NAN);
    assert!(length_nan.is_err(), "Length should reject NaN values");
}

#[test]
fn test_length_rejects_infinity() {
    let length_inf = Length::new(f64::INFINITY);
    assert!(length_inf.is_err(), "Length should reject INFINITY values");
}

#[test]
fn test_area_rejects_nan() {
    let area_nan = Area::new(f64::NAN);
    assert!(area_nan.is_err(), "Area should reject NaN values");
}

#[test]
fn test_volume_rejects_nan() {
    let volume_nan = Volume::new(f64::NAN);
    assert!(volume_nan.is_err(), "Volume should reject NaN values");
}

#[test]
fn test_moment_of_inertia_rejects_nan() {
    let moi_nan = MomentOfInertia::new(f64::NAN);
    assert!(moi_nan.is_err(), "MomentOfInertia should reject NaN values");
}

#[test]
fn test_frequency_rejects_nan() {
    let freq_nan = Frequency::new(f64::NAN);
    assert!(freq_nan.is_err(), "Frequency should reject NaN values");
}
```

### Test output

```
   Compiling deep_causality_physics v0.1.1 (/home/user/deep_causality/deep_causality_physics)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.25s
     Running tests/test_nan_infinity.rs (target/debug/deps/test_nan_infinity-38e6a34d8cc79a9a)
error: test failed, to rerun pass `-p deep_causality_physics --test test_nan_infinity`


running 10 tests
test test_frequency_rejects_nan ... FAILED
test test_area_rejects_nan ... FAILED
test test_length_rejects_infinity ... FAILED
test test_length_rejects_nan ... FAILED
test test_mass_rejects_infinity ... FAILED
test test_mass_rejects_nan ... FAILED
test test_moment_of_inertia_rejects_nan ... FAILED
test test_speed_rejects_infinity ... FAILED
test test_speed_rejects_nan ... FAILED
test test_volume_rejects_nan ... FAILED

failures:

---- test_frequency_rejects_nan stdout ----

thread 'test_frequency_rejects_nan' (3503) panicked at deep_causality_physics/tests/test_nan_infinity.rs:64:5:
Frequency should reject NaN values
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- test_area_rejects_nan stdout ----

thread 'test_area_rejects_nan' (3502) panicked at deep_causality_physics/tests/test_nan_infinity.rs:46:5:
Area should reject NaN values

---- test_length_rejects_infinity stdout ----

thread 'test_length_rejects_infinity' (3504) panicked at deep_causality_physics/tests/test_nan_infinity.rs:40:5:
Length should reject INFINITY values

---- test_length_rejects_nan stdout ----

thread 'test_length_rejects_nan' (3505) panicked at deep_causality_physics/tests/test_nan_infinity.rs:34:5:
Length should reject NaN values

---- test_mass_rejects_infinity stdout ----

thread 'test_mass_rejects_infinity' (3506) panicked at deep_causality_physics/tests/test_nan_infinity.rs:16:5:
Mass should reject INFINITY values

---- test_mass_rejects_nan stdout ----

thread 'test_mass_rejects_nan' (3507) panicked at deep_causality_physics/tests/test_nan_infinity.rs:10:5:
Mass should reject NaN values

---- test_moment_of_inertia_rejects_nan stdout ----

thread 'test_moment_of_inertia_rejects_nan' (3508) panicked at deep_causality_physics/tests/test_nan_infinity.rs:58:5:
MomentOfInertia should reject NaN values

---- test_speed_rejects_infinity stdout ----

thread 'test_speed_rejects_infinity' (3509) panicked at deep_causality_physics/tests/test_nan_infinity.rs:28:5:
Speed should reject INFINITY values

---- test_speed_rejects_nan stdout ----

thread 'test_speed_rejects_nan' (3510) panicked at deep_causality_physics/tests/test_nan_infinity.rs:22:5:
Speed should reject NaN values

---- test_volume_rejects_nan stdout ----

thread 'test_volume_rejects_nan' (3511) panicked at deep_causality_physics/tests/test_nan_infinity.rs:52:5:
Volume should reject NaN values


failures:
    test_area_rejects_nan
    test_frequency_rejects_nan
    test_length_rejects_infinity
    test_length_rejects_nan
    test_mass_rejects_infinity
    test_mass_rejects_nan
    test_moment_of_inertia_rejects_nan
    test_speed_rejects_infinity
    test_speed_rejects_nan
    test_volume_rejects_nan

test result: FAILED. 0 passed; 10 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

# Full context

The quantities module (`deep_causality_physics/src/dynamics/quantities.rs`) provides type-safe wrappers for fundamental physical quantities used throughout the physics engine. These types are designed to enforce physical invariants at construction time, preventing invalid values from entering the system.

These quantity types are used extensively across the codebase:
- In `kinematics.rs`: `kinetic_energy_kernel()` uses Mass and velocity to calculate kinetic energy
- In `astro/mechanics.rs`: `orbital_velocity_kernel()` and `escape_velocity_kernel()` use Mass and Length to calculate velocities
- In `waves/general.rs`: Uses Speed for wave propagation calculations
- In `mhd/` modules: Uses various quantities for magnetohydrodynamics calculations
- Throughout other physics domains (quantum, relativity, thermodynamics, etc.)

The validation in the `new()` methods is the primary defense against invalid physical values entering the system. Once a quantity is created, its value is accessed via the `value()` method and used directly in calculations without further validation.

The codebase does include `is_finite()` checks in some calculation functions (e.g., `kinematics.rs:49`), but these are defensive checks for intermediate calculation results, not substitutes for input validation. The assumption is that properly validated quantities should not need re-validation.

## External documentation

- [IEEE 754 Floating Point Standard - NaN comparison behavior](https://en.wikipedia.org/wiki/IEEE_754#Comparison_predicates)
```
According to IEEE 754, all comparisons with NaN (except !=) return false:
- NaN < x is false
- NaN <= x is false
- NaN > x is false
- NaN >= x is false
- NaN == x is false
- NaN != x is true (including NaN != NaN)

This means a simple check like `if val < 0.0` will not catch NaN values.
```

- [Rust f64 documentation](https://doc.rust-lang.org/std/primitive.f64.html#method.is_finite)
```
pub const fn is_finite(self) -> bool

Returns true if this number is neither infinite nor NaN.

Examples:
let f = 7.0f64;
let inf = f64::INFINITY;
let neg_inf = f64::NEG_INFINITY;
let nan = f64::NAN;

assert!(f.is_finite());
assert!(!inf.is_finite());
assert!(!neg_inf.is_finite());
assert!(!nan.is_finite());
```

# Why has this bug gone undetected?

This bug has gone undetected for several reasons:

1. **Unusual input values**: In normal usage, physics calculations with valid inputs produce valid outputs. NaN typically only arises from:
    - Division by zero (usually caught earlier)
    - Square root of negative numbers (often prevented by validation)
    - Propagation from external sources (parsing errors, sensor failures)

   Most test cases use "happy path" values like `10.0`, `100.0`, etc.

2. **IEEE 754 comparison semantics**: The non-intuitive behavior of NaN comparisons (all return false) means that a check like `val < 0.0` appears correct at first glance. Developers naturally think "if it's not negative, it must be valid," but NaN is neither negative nor non-negative in the usual sense.

3. **Test coverage gap**: The existing test suite (`tests/dynamics/quantities_tests.rs`) tests:
    - Valid positive values
    - Zero values
    - Negative values (correctly rejected)
    - The `unchecked` constructors
    - Type conversions

   But it doesn't test edge cases like NaN, infinity, or negative infinity.

4. **Implicit assumption**: There's an implicit assumption that if a value comes from a calculation in Rust, it's probably finite. However, Rust doesn't prevent NaN/infinity creation (e.g., `0.0/0.0` produces NaN, `1.0/0.0` produces infinity).

5. **Rust's permissive float handling**: Unlike some languages, Rust allows NaN and infinity to propagate silently through calculations without exceptions, making bugs harder to detect through runtime errors.

6. **Silent propagation**: Once a NaN enters the system through these constructors, it propagates silently through all downstream calculations, never triggering an error. The final results are simply wrong (NaN), but no panic or error is raised, making the root cause hard to trace.

# Recommended fix

Add explicit validation for finite values in all quantity constructors that should only accept finite, non-negative values:

```rust
impl Mass {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if !val.is_finite() {  // <-- FIX 🟢 Reject NaN and infinity first
            return Err(PhysicsError::new(
                PhysicsErrorEnum::PhysicalInvariantBroken(format!(
                    "Mass must be finite: {}",
                    val
                )),
            ));
        }
        if val < 0.0 {  // <-- FIX 🟢 Now this check is meaningful
            return Err(PhysicsError::new(
                PhysicsErrorEnum::PhysicalInvariantBroken(format!(
                    "Mass cannot be negative: {}",
                    val
                )),
            ));
        }
        Ok(Self(val))
    }
}
```

Apply the same fix to: Speed, Length, Area, Volume, MomentOfInertia, and Frequency.

Note: Quantities that allow negative values (Acceleration, Force, Torque) should also add `is_finite()` checks, but without the negativity check:

```rust
impl Acceleration {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if !val.is_finite() {
            return Err(PhysicsError::new(
                PhysicsErrorEnum::PhysicalInvariantBroken(format!(
                    "Acceleration must be finite: {}",
                    val
                )),
            ));
        }
        Ok(Self(val))
    }
}
```

# Related bugs

This same pattern likely exists in quantity types in other physics modules:
- `deep_causality_physics/src/electromagnetism/quantities.rs`
- `deep_causality_physics/src/quantum/quantities.rs`
- `deep_causality_physics/src/thermodynamics/quantities.rs`
- `deep_causality_physics/src/relativity/quantities.rs`
- `deep_causality_physics/src/fluids/quantities.rs`
- `deep_causality_physics/src/materials/quantities.rs`
- `deep_causality_physics/src/photonics/quantities.rs`
- `deep_causality_physics/src/nuclear/quantities.rs`
- `deep_causality_physics/src/mhd/quantities.rs`
- `deep_causality_physics/src/condensed/quantities.rs`

Each should be audited for the same validation issue.

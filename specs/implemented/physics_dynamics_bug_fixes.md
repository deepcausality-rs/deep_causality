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


--

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

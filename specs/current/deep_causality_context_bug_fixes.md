LorentzianSpacetime: time() ignores time_scale, breaking Minkowski intervals for non-second units

# Summary
- **Context**: `LorentzianSpacetime` implements the `SpaceTemporalInterval` trait to calculate Minkowski intervals between spacetime events, which is fundamental for determining causal relationships in relativistic physics.
- **Bug**: The `time()` method returns raw time values without converting them to seconds as required by the `SpaceTemporalInterval` trait contract, causing incorrect interval calculations when `time_scale` is not `TimeScale::Second`.
- **Actual vs. expected**: When `time_scale` is `TimeScale::Millisecond`, a 10-millisecond time difference is incorrectly treated as 10 seconds, producing intervals that are 1,000,000 times larger than they should be.
- **Impact**: All Minkowski interval calculations are incorrect by a factor of (scale_factor)² when using any `TimeScale` other than `Second`, leading to completely wrong causal relationship determinations.

# Code with bug

**File: `deep_causality/src/types/context_node_types/space_time/lorentzian_spacetime/space_temporal_interval.rs`**

```rust
impl SpaceTemporalInterval for LorentzianSpacetime {
    fn time(&self) -> f64 {
        self.t  // <-- BUG 🔴 Returns raw time value without converting to seconds
    }
    fn position(&self) -> [f64; 3] {
        [self.x, self.y, self.z]
    }
    // No need to override `interval_squared()` unless you want a custom metric for curved spacetime
}
```

The `time()` method returns `self.t` directly, ignoring the `time_scale` field. However, `LorentzianSpacetime` stores a `time_scale: TimeScale` field that can be `Millisecond`, `Microseconds`, `Minute`, etc.

**Related struct definition:**
```rust
pub struct LorentzianSpacetime {
    id: u64,
    x: f64,
    y: f64,
    z: f64,
    t: f64,                // time in SI time unit
    time_scale: TimeScale, // SI time unit - can be Millisecond, Second, Minute, etc.
}
```

# Evidence

## Example

Consider two events separated by 10 milliseconds with no spatial separation:

```rust
let event_a = LorentzianSpacetime::new(1, 0.0, 0.0, 0.0, 0.0, TimeScale::Millisecond);
let event_b = LorentzianSpacetime::new(2, 0.0, 0.0, 0.0, 10.0, TimeScale::Millisecond);
let interval = event_a.interval_squared(&event_b);
```

**Step-by-step calculation:**

1. `event_a.time()` returns `0.0` (raw value)
2. `event_b.time()` returns `10.0` (raw value in milliseconds)
3. `Δt = 10.0 - 0.0 = 10.0` (incorrectly interpreted as 10 seconds!)
4. The actual time difference is 10 milliseconds = 0.010 seconds

**Expected calculation** (if conversion worked correctly):
```
Δt = 0.010 seconds
s² = -c²·Δt² = -(299,792,458)² × (0.010)²
   = -8.99 × 10¹²
```

**Actual calculation** (with bug):
```
Δt = 10.0 (treated as seconds)
s² = -c²·Δt² = -(299,792,458)² × (10.0)²
   = -8.99 × 10¹⁸
```

**Error magnitude**: The result is off by a factor of **1,000,000** (one million).

## Inconsistency with API documentation

### Reference API documentation

From `deep_causality/src/traits/contextuable/space_temporal.rs`:

```rust
/// Trait for spacetime types that support Minkowski-style interval calculations.
///
/// This trait enables causal reasoning in spacetime-aware systems using the Minkowski
/// metric from special relativity:
///
/// ```text
/// s² = -c²·Δt² + Δx² + Δy² + Δz²
/// ```
///
/// The default implementation assumes:
/// - Time is in **seconds**
/// - Space is in **meters**
/// - Speed of light `c = 299_792_458 m/s`
///
/// # Required Methods
/// - `time()`: Returns the scalar time coordinate in seconds
/// - `position()`: Returns the spatial coordinates `[x, y, z]` in meters

pub trait SpaceTemporalInterval {
    /// Returns the time coordinate in **seconds**.
    fn time(&self) -> f64;

    /// Returns the spatial coordinates `[x, y, z]` in **meters**.
    fn position(&self) -> [f64; 3];

    /// Computes the squared Minkowski interval between `self` and `other`.
    ///
    /// ```text
    /// s² = -c²·Δt² + Δx² + Δy² + Δz²
    /// ```
    /// where `c = 299_792_458 m/s`.
    fn interval_squared(&self, other: &Self) -> f64 {
        let c = 299_792_458.0; // Speed of light (m/s)

        let dt = self.time() - other.time();  // <-- Expects time() to return seconds
        let [x1, y1, z1] = self.position();
        let [x2, y2, z2] = other.position();

        let dx = x1 - x2;
        let dy = y1 - y2;
        let dz = z1 - z2;

        -(c * dt).powi(2) + dx.powi(2) + dy.powi(2) + dz.powi(2)
    }
}
```

The documentation explicitly states:
- "Time is in **seconds**" (emphasis in original)
- "Returns the time coordinate in **seconds**" (emphasis in original)

### Current API usage

```rust
impl SpaceTemporalInterval for LorentzianSpacetime {
    fn time(&self) -> f64 {
        self.t  // Returns raw value, not necessarily in seconds
    }
    // ...
}
```

### Contradiction

The `time()` implementation violates the trait contract by returning `self.t` without checking or converting based on `self.time_scale`. When `time_scale` is `TimeScale::Millisecond`, the returned value is in milliseconds, not seconds. When `time_scale` is `TimeScale::Minute`, the returned value is in minutes, not seconds. This directly contradicts the documented requirement that `time()` must return seconds.

## Failing test

### Test script

```rust
/*
 * Unit test demonstrating the time scale bug in LorentzianSpacetime
 */
#[cfg(test)]
mod tests {
    use deep_causality::*;

    #[test]
    fn test_interval_with_millisecond_time_scale() {
        // Create two events separated by 10 milliseconds (0.010 seconds)
        // Events are at the same spatial location
        let event_a = LorentzianSpacetime::new(1, 0.0, 0.0, 0.0, 0.0, TimeScale::Millisecond);
        let event_b = LorentzianSpacetime::new(2, 0.0, 0.0, 0.0, 10.0, TimeScale::Millisecond);

        // Calculate the interval
        let interval_squared = event_a.interval_squared(&event_b);

        // According to the SpaceTemporalInterval trait docs, time() should return seconds.
        // Since we have 10 milliseconds = 0.010 seconds of time difference and no spatial difference:
        // s² = -c²·Δt² + Δx² + Δy² + Δz²
        //    = -c²·(0.010)² + 0 + 0 + 0
        //    = -(299792458)² * (0.010)²

        let c: f64 = 299_792_458.0; // Speed of light in m/s
        let dt_in_seconds: f64 = 0.010; // 10 milliseconds = 0.010 seconds
        let expected_interval: f64 = -(c * dt_in_seconds).powi(2);

        // The interval should be negative (time-like) and approximately equal to expected
        assert!(interval_squared < 0.0, "Time-like interval should be negative");

        let relative_error = (interval_squared - expected_interval).abs() / expected_interval.abs();
        assert!(
            relative_error < 1e-6,
            "Interval calculation incorrect. Expected: {:.6e}, Got: {:.6e}, Relative error: {:.2e}",
            expected_interval,
            interval_squared,
            relative_error
        );
    }

    #[test]
    fn test_interval_with_microsecond_time_scale() {
        // Create two events separated by 1000 microseconds (0.001 seconds)
        let event_a = LorentzianSpacetime::new(1, 0.0, 0.0, 0.0, 0.0, TimeScale::Microseconds);
        let event_b = LorentzianSpacetime::new(2, 0.0, 0.0, 0.0, 1000.0, TimeScale::Microseconds);

        let interval_squared = event_a.interval_squared(&event_b);

        let c: f64 = 299_792_458.0;
        let dt_in_seconds: f64 = 0.001; // 1000 microseconds = 0.001 seconds
        let expected_interval: f64 = -(c * dt_in_seconds).powi(2);

        assert!(interval_squared < 0.0, "Time-like interval should be negative");

        let relative_error = (interval_squared - expected_interval).abs() / expected_interval.abs();
        assert!(
            relative_error < 1e-6,
            "Interval calculation incorrect. Expected: {:.6e}, Got: {:.6e}",
            expected_interval,
            interval_squared
        );
    }
}
```

### Test output

```
running 2 tests
test tests::test_interval_with_microsecond_time_scale ... FAILED
test tests::test_interval_with_millisecond_time_scale ... FAILED

failures:

---- tests::test_interval_with_microsecond_time_scale stdout ----

thread 'tests::test_interval_with_microsecond_time_scale' panicked at test_interval_milliseconds.rs:56:9:
Interval calculation incorrect. Expected: -8.987552e10, Got: -8.987552e22
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- tests::test_interval_with_millisecond_time_scale stdout ----

thread 'tests::test_interval_with_millisecond_time_scale' panicked at test_interval_milliseconds.rs:32:9:
Interval calculation incorrect. Expected: -8.987552e12, Got: -8.987552e18, Relative error: 1.00e6


failures:
    tests::test_interval_with_microsecond_time_scale
    tests::test_interval_with_millisecond_time_scale

test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out
```

# Full context

`LorentzianSpacetime` is one of several spacetime coordinate system types in the `deep_causality` crate, designed for causal reasoning in relativistic contexts. It represents events in 4-dimensional spacetime using Lorentzian geometry (the geometry of General Relativity).

The type implements the `SpaceTemporalInterval` trait, which provides the `interval_squared()` method to calculate the Minkowski interval between two events. This interval is the fundamental quantity in special relativity that determines causal relationships:

- **Negative interval** (s² < 0): Time-like separation → events are causally connected
- **Zero interval** (s² = 0): Light-like separation → events are on the light cone
- **Positive interval** (s² > 0): Space-like separation → events cannot causally influence each other

The interval calculation uses the Minkowski metric: `s² = -c²·Δt² + Δx² + Δy² + Δz²`

Where `c = 299,792,458 m/s` (speed of light). For this formula to be correct, all units must be consistent: spatial coordinates in meters and time in seconds.

`LorentzianSpacetime` is used throughout the codebase wherever causal reasoning about spacetime events is needed. The struct includes:
- Spatial coordinates (x, y, z) in meters
- A time coordinate (t) stored as a raw number
- A `time_scale` field indicating the unit of the time coordinate

The bug occurs because the `time()` method, which is called by `interval_squared()`, returns the raw `t` value without converting it to seconds based on the `time_scale` field. This violates the trait contract and produces incorrect results whenever `time_scale` is anything other than `TimeScale::Second`.

## External documentation

**Minkowski interval** is defined in special relativity as:
- [Wikipedia - Minkowski space](https://en.wikipedia.org/wiki/Minkowski_space)
> In mathematical physics, Minkowski space combines three-dimensional Euclidean space and time into a four-dimensional manifold where the spacetime interval between any two events is independent of the inertial frame of reference in which they are recorded.
>
> The spacetime interval between two events is given by:
> ```
> s² = -c²Δt² + Δx² + Δy² + Δz²  (−+++ signature)
> ```

The calculation requires time to be in consistent units with the speed of light constant `c`, which is universally defined in SI units as meters per second. Therefore, time must be in seconds.

# Why has this bug gone undetected?

This bug has remained undetected for the following reasons:

1. **All existing interval tests use `TimeScale::Second`**: Every test in the test suite that calls `interval_squared()` uses `TimeScale::Second`. For example, in `deep_causality/tests/types/context_node_types/space_time/lorentzian/lorentzian_spacetime_tests.rs`:
   - `test_interval_squared_timelike_is_negative()` uses `TimeScale::Second`
   - `test_interval_squared_spacelike_is_positive()` uses `TimeScale::Second`
   - `test_interval_squared_null_like()` uses `TimeScale::Second`

2. **When `time_scale` is `Second`, the bug doesn't manifest**: If the time scale is already in seconds, then returning the raw `t` value is correct. The bug only appears when using other time scales.

3. **Non-`Second` time scales are only tested for display and accessors**: The tests that use `TimeScale::Millisecond` only test formatting (`test_display_trait`) or direct accessor methods (`test_lorentzian_time_and_position`), neither of which calls `interval_squared()`.

4. **The `time_scale` field is stored but never used**: The field is included in the struct and properly set via the constructor, but no code actually reads or uses it to perform conversions. It's essentially dead data that gives a false sense of correctness.

5. **Implicit assumption in documentation**: The trait documentation states "Time is in seconds" but doesn't explicitly require implementors to perform conversion. Implementors might assume that users will always provide time values pre-converted to seconds, rather than using the `time_scale` field for automatic conversion.

# Recommended fix

The `time()` method should convert the time value to seconds based on `time_scale`:

```rust
impl SpaceTemporalInterval for LorentzianSpacetime {
    fn time(&self) -> f64 {
        // Convert time to seconds based on time_scale
        match self.time_scale {  // <-- FIX 🟢 Convert based on scale
            TimeScale::Nanoseconds => self.t / 1_000_000_000.0,
            TimeScale::Microseconds => self.t / 1_000_000.0,
            TimeScale::Millisecond => self.t / 1_000.0,
            TimeScale::Second => self.t,
            TimeScale::Minute => self.t * 60.0,
            TimeScale::Hour => self.t * 3600.0,
            TimeScale::Day => self.t * 86400.0,
            TimeScale::Week => self.t * 604800.0,
            // For non-physical time scales, either return as-is or document behavior
            TimeScale::NoScale | TimeScale::Steps | TimeScale::Symbolic => self.t,
            // Approximate conversions for calendar units (assuming 30-day months, 365-day years)
            TimeScale::Month => self.t * 2_592_000.0,  // 30 days
            TimeScale::Quarter => self.t * 7_776_000.0,  // 90 days
            TimeScale::Year => self.t * 31_536_000.0,  // 365 days
        }
    }
    fn position(&self) -> [f64; 3] {
        [self.x, self.y, self.z]
    }
}
```

Alternatively, if the design intent is that users should always provide time in seconds and `time_scale` is purely metadata for display purposes, then:
1. Document this clearly in the struct and constructor documentation
2. Consider adding validation that panics or returns an error if `time_scale != TimeScale::Second`
3. Update all examples to clarify this constraint

# Related bugs

The same bug likely exists in:
- `MinkowskiSpacetime` (`deep_causality/src/types/context_node_types/space_time/minkowski_spacetime/space_temporal_interval.rs`)
- `EuclideanSpacetime` (if it implements `SpaceTemporalInterval`)
- `TangentSpacetime` (`deep_causality/src/types/context_node_types/space_time/tangent_spacetime/space_temporal_interval.rs`)

All of these types have a `time_scale` field and implement `SpaceTemporalInterval` with the same pattern of returning raw `self.t` values.


time() returns non-second units in spacetime structs, breaking interval_squared


# Summary
- **Context**: `LorentzianSpacetime` implements the `SpaceTemporalInterval` trait which provides the `interval_squared()` method for computing Minkowski spacetime intervals used in relativistic physics calculations.
- **Bug**: The `time()` method returns the raw time value without converting it to seconds, violating the trait contract that explicitly requires time to be returned in seconds.
- **Actual vs. expected**: When a `LorentzianSpacetime` is created with `TimeScale::Millisecond` and `t=1000.0`, the `time()` method returns `1000.0` instead of the expected `1.0` (seconds).
- **Impact**: Spacetime interval calculations produce results that are off by a factor of (time_scale_factor)², potentially billions of times incorrect for nanosecond scales, leading to completely wrong causal relationship determinations in relativistic physics calculations.

# Code with bug

**File**: `deep_causality/src/types/context_node_types/space_time/lorentzian_spacetime/space_temporal_interval.rs`

```rust
impl SpaceTemporalInterval for LorentzianSpacetime {
    fn time(&self) -> f64 {
        self.t  // <-- BUG 🔴 Returns raw time value without converting to seconds
    }
    fn position(&self) -> [f64; 3] {
        [self.x, self.y, self.z]
    }
}
```

The trait's default implementation of `interval_squared()` uses this unconverted time value:

```rust
fn interval_squared(&self, other: &Self) -> f64 {
    let c = 299_792_458.0; // Speed of light (m/s)

    let dt = self.time() - other.time();  // <-- BUG 🔴 dt is not in seconds!
    let [x1, y1, z1] = self.position();
    let [x2, y2, z2] = other.position();

    let dx = x1 - x2;
    let dy = y1 - y2;
    let dz = z1 - z2;

    -(c * dt).powi(2) + dx.powi(2) + dy.powi(2) + dz.powi(2)
}
```

# Evidence

## Example

Consider two events separated by 1000 milliseconds (= 1 second) with no spatial separation:

```rust
let event_a = LorentzianSpacetime::new(1, 0.0, 0.0, 0.0, 1000.0, TimeScale::Millisecond);
let event_b = LorentzianSpacetime::new(2, 0.0, 0.0, 0.0, 0.0, TimeScale::Millisecond);
```

**Step 1**: Calculate `dt`
- `dt = event_a.time() - event_b.time()`
- `dt = 1000.0 - 0.0 = 1000.0` (incorrectly interpreted as 1000 seconds)
- Should be: `dt = 1.0` (1 second)

**Step 2**: Calculate interval
- Formula: `s² = -(c·dt)² + dx² + dy² + dz²`
- With c = 299,792,458 m/s
- `s² = -(299792458 × 1000)² + 0 + 0 + 0`
- `s² = -89,875,517,873,681,760,000,000`

**Expected result** (with dt = 1 second):
- `s² = -(299792458 × 1)² = -89,875,517,873,681,760`

**Error magnitude**: The result is **1,000,000 times** (10⁶) too large in magnitude because the error is squared: (1000)² = 1,000,000.

## Inconsistency with trait documentation

### Reference spec

From `deep_causality/src/traits/contextuable/space_temporal.rs`:

```rust
pub trait SpaceTemporalInterval {
    /// Returns the time coordinate in **seconds**.
    fn time(&self) -> f64;

    /// Returns the spatial coordinates `[x, y, z]` in **meters**.
    fn position(&self) -> [f64; 3];

    /// Computes the squared Minkowski interval between `self` and `other`.
    ///
    /// ```text
    /// s² = -c²·Δt² + Δx² + Δy² + Δz²
    /// ```
    /// where `c = 299_792_458 m/s`.
    fn interval_squared(&self, other: &Self) -> f64 {
        let c = 299_792_458.0; // Speed of light (m/s)
        let dt = self.time() - other.time();
        // ...
    }
}
```

The trait documentation explicitly states:
- Line 56: "Returns the time coordinate in **seconds**"
- Line 44-46: "The default implementation assumes: Time is in **seconds**, Space is in **meters**, Speed of light `c = 299_792_458 m/s`"

### Current code

```rust
impl SpaceTemporalInterval for LorentzianSpacetime {
    fn time(&self) -> f64 {
        self.t  // Returns raw value, ignoring time_scale
    }
    // ...
}
```

The `LorentzianSpacetime` struct stores:
- `t: f64` - "time in SI time unit"
- `time_scale: TimeScale` - which can be `Second`, `Millisecond`, `Microseconds`, `Nanoseconds`, etc.

### Contradiction

The implementation returns `self.t` directly without any conversion, but `self.t` is stored in whatever units are specified by `self.time_scale` (which could be milliseconds, microseconds, nanoseconds, etc.), not necessarily seconds. This directly violates the trait contract that requires the return value to be in seconds.

## Inconsistency within the codebase

### Reference code

`deep_causality/src/types/context_node_types/space_time/lorentzian_spacetime/mod.rs`:

```rust
/// # Fields
/// - `id`: Unique numeric identifier
/// - `x`: X-coordinate in meters
/// - `y`: Y-coordinate in meters
/// - `z`: Z-coordinate in meters
/// - `t`: time (e.g., seconds)
/// - `time_scale`: Time scale unit (e.g., seconds, milliseconds)
```

The struct definition explicitly acknowledges that `t` can be in different units and provides a `time_scale` field to track which unit is being used.

### Current code

`deep_causality/src/types/context_node_types/space_time/lorentzian_spacetime/space_temporal_interval.rs`:

```rust
impl SpaceTemporalInterval for LorentzianSpacetime {
    fn time(&self) -> f64 {
        self.t  // Ignores time_scale field
    }
    // ...
}
```

### Contradiction

The implementation ignores the `time_scale` field entirely, treating all time values as if they were in seconds regardless of their actual units. This is inconsistent with the struct's own design which specifically includes a `time_scale` field to track the units.

## Failing test

### Test script

File: `deep_causality/tests/test_time_conversion_bug.rs`

```rust
use deep_causality::*;

#[test]
fn test_time_method_should_return_seconds_for_millisecond_timescale() {
    // Create event at t=1000 milliseconds
    let event = LorentzianSpacetime::new(1, 0.0, 0.0, 0.0, 1000.0, TimeScale::Millisecond);

    // According to SpaceTemporalInterval trait documentation:
    // "Returns the time coordinate in **seconds**."
    // So 1000 milliseconds should be returned as 1.0 second
    let time_in_seconds = event.time();

    assert!(
        (time_in_seconds - 1.0).abs() < 1e-9,
        "time() should return seconds, but got {} instead of 1.0",
        time_in_seconds
    );
}

#[test]
fn test_interval_squared_with_millisecond_timescale() {
    // Two events 1000 milliseconds apart (= 1 second)
    let event_a = LorentzianSpacetime::new(1, 0.0, 0.0, 0.0, 1000.0, TimeScale::Millisecond);
    let event_b = LorentzianSpacetime::new(2, 0.0, 0.0, 0.0, 0.0, TimeScale::Millisecond);

    let interval = event_a.interval_squared(&event_b);

    // Expected calculation with Δt = 1.0 second:
    // s² = -c²·Δt² + Δx² = -(299792458)²·(1.0)² ≈ -8.988e16
    let c: f64 = 299_792_458.0;
    let expected: f64 = -(c * 1.0).powi(2);

    // The bug causes it to use Δt = 1000 "seconds" instead:
    // s² = -(299792458)²·(1000)² ≈ -8.988e22 (1,000,000x too large!)

    let relative_error = ((interval - expected) / expected).abs();
    assert!(
        relative_error < 0.01,
        "interval calculation is wrong by {}x (got {}, expected {})",
        relative_error,
        interval,
        expected
    );
}

#[test]
fn test_time_method_should_return_seconds_for_microsecond_timescale() {
    // Create event at t=1_000_000 microseconds (= 1 second)
    let event = LorentzianSpacetime::new(1, 0.0, 0.0, 0.0, 1_000_000.0, TimeScale::Microseconds);

    let time_in_seconds = event.time();

    assert!(
        (time_in_seconds - 1.0).abs() < 1e-9,
        "time() should return seconds, but got {} instead of 1.0",
        time_in_seconds
    );
}
```

### Test output

```
running 3 tests
test test_interval_squared_with_millisecond_timescale ... FAILED
test test_time_method_should_return_seconds_for_microsecond_timescale ... FAILED
test test_time_method_should_return_seconds_for_millisecond_timescale ... FAILED

failures:

---- test_interval_squared_with_millisecond_timescale stdout ----

thread 'test_interval_squared_with_millisecond_timescale' (12974) panicked at deep_causality/tests/test_time_conversion_bug.rs:50:5:
interval calculation is wrong by 999998.9999999999x (got -89875517873681760000000, expected -89875517873681760)

---- test_time_method_should_return_seconds_for_microsecond_timescale stdout ----

thread 'test_time_method_should_return_seconds_for_microsecond_timescale' (12975) panicked at deep_causality/tests/test_time_conversion_bug.rs:67:5:
time() should return seconds, but got 1000000 instead of 1.0

---- test_time_method_should_return_seconds_for_millisecond_timescale stdout ----

thread 'test_time_method_should_return_seconds_for_millisecond_timescale' (12976) panicked at deep_causality/tests/test_time_conversion_bug.rs:25:5:
time() should return seconds, but got 1000 instead of 1.0


failures:
    test_interval_squared_with_millisecond_timescale
    test_time_method_should_return_seconds_for_microsecond_timescale
    test_time_method_should_return_seconds_for_millisecond_timescale

test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out
```

# Full context

The `SpaceTemporalInterval` trait is a physics-oriented API for computing Minkowski spacetime intervals in special relativity. The interval squared `s² = -c²·Δt² + Δx² + Δy² + Δz²` is fundamental for determining causal relationships between events:
- Negative `s²` means time-like separation (events are causally connected)
- Zero `s²` means light-like separation (events are on the light cone)
- Positive `s²` means space-like separation (events cannot causally influence each other)

The trait provides a default implementation of `interval_squared()` that uses the `time()` and `position()` methods. The formula uses the speed of light in meters per second (299,792,458 m/s), which requires that time be in seconds and space be in meters for dimensional consistency.

Three spacetime types implement this trait:
1. `LorentzianSpacetime` - General relativistic spacetime with Lorentzian signature
2. `MinkowskiSpacetime` - Flat spacetime used in special relativity
3. `TangentSpacetime` - Curved spacetime with metric tensor

All three implementations have the same bug in their `time()` method.

These spacetime types are used throughout the codebase for:
- Modeling causal relationships in 4D spacetime
- Physics simulations requiring relativistic calculations
- Temporal reasoning in systems where the speed of light matters
- Compatibility with Newtonian and Einsteinian physics frameworks

The bug affects any code path that:
1. Creates spacetime objects with non-second time scales (milliseconds, microseconds, nanoseconds)
2. Calls `interval_squared()` to compute spacetime intervals
3. Uses those intervals to make causal determinations

## External documentation

The speed of light constant used in the calculation is the exact value defined by the SI system:

- [NIST Special Publication 330, The International System of Units (SI)](https://physics.nist.gov/cuu/Units/meter.html)
```
Since 1983, the metre has been defined by the speed of light:
"The metre is the length of the path travelled by light in vacuum
during a time interval of 1/299 792 458 of a second."

Therefore: c = 299,792,458 m/s (exact)
```

The formula `s² = -c²·Δt² + Δx² + Δy² + Δz²` requires Δt to be in seconds for dimensional consistency since c is defined in meters per second.

# Why has this bug gone undetected?

The bug has gone undetected for several reasons:

1. **All existing tests use `TimeScale::Second`**: Every test that calls `interval_squared()` creates spacetime objects with `TimeScale::Second`. When the time scale is already in seconds, the bug has no effect since no conversion is needed. For example, in `lorentzian_spacetime_tests.rs`:

```rust
#[test]
fn test_interval_squared_timelike_is_negative() {
    let a = LorentzianSpacetime::new(1, 0.0, 0.0, 0.0, 10.0, TimeScale::Second);
    let b = LorentzianSpacetime::new(2, 0.0, 0.0, 0.0, 0.0, TimeScale::Second);
    let result = a.interval_squared(&b);
    assert!(result < 0.0);
}
```

2. **Tests with non-second time scales don't use `interval_squared()`**: The few tests that do use `TimeScale::Millisecond` only test the display trait or simple accessors, never the physics calculations:

```rust
#[test]
fn test_display_trait() {
    let s = LorentzianSpacetime::new(1, 1.0, 2.0, 3.0, 42.0, TimeScale::Millisecond);
    let formatted = format!("{s}");
    // Only tests string formatting, not interval calculations
}
```

3. **The bug is not visible from basic API usage**: The `time()` method itself appears to work correctly - it returns a number. Users would have no reason to suspect it's in the wrong units unless they carefully read the trait documentation and manually verify the calculation results.

4. **No integration tests mixing different time scales**: There are no tests that compare results across different time scales to verify consistency, which would immediately reveal that a 1-second interval doesn't equal a 1000-millisecond interval.

5. **Physics domain knowledge required**: Recognizing this bug requires understanding both the physics (Minkowski intervals, dimensional analysis) and the API contract. Most code reviewers might focus on the code structure rather than verifying the physical units are correct.

6. **The struct design suggests units are stored, not converted**: Since `LorentzianSpacetime` has a `time_scale` field, developers might reasonably assume that time values are stored in their native units and the trait implementation would handle conversion. The bug is in the gap between that assumption and the actual implementation.

# Recommended fix

The `time()` method should convert the stored time value to seconds based on the `time_scale` field:

```rust
impl SpaceTemporalInterval for LorentzianSpacetime {
    fn time(&self) -> f64 {
        // Convert time to seconds based on time_scale
        match self.time_scale {
            TimeScale::Nanoseconds => self.t / 1_000_000_000.0,
            TimeScale::Microseconds => self.t / 1_000_000.0,
            TimeScale::Millisecond => self.t / 1_000.0,  // <-- FIX 🟢
            TimeScale::Second => self.t,  // <-- FIX 🟢
            TimeScale::Minute => self.t * 60.0,
            TimeScale::Hour => self.t * 3600.0,
            TimeScale::Day => self.t * 86400.0,
            TimeScale::Week => self.t * 604800.0,
            TimeScale::Month => self.t * 2_629_746.0,  // Average month (365.25 days / 12)
            TimeScale::Quarter => self.t * 7_889_238.0,  // 3 months
            TimeScale::Year => self.t * 31_556_952.0,  // Average year (365.25 days)
            TimeScale::NoScale | TimeScale::Steps | TimeScale::Symbolic => {
                // For non-physical time scales, assume the value is already in seconds
                // or cannot be meaningfully converted
                self.t
            }
        }
    }

    fn position(&self) -> [f64; 3] {
        [self.x, self.y, self.z]
    }
}
```

The same fix needs to be applied to:
- `deep_causality/src/types/context_node_types/space_time/minkowski_spacetime/space_temporal_interval.rs`
- `deep_causality/src/types/context_node_types/space_time/tangent_spacetime/space_temporal_interval.rs`

# Related bugs

The same bug exists in two other files:

1. **MinkowskiSpacetime**: `deep_causality/src/types/context_node_types/space_time/minkowski_spacetime/space_temporal_interval.rs`
   - Identical implementation returning `self.t` without conversion
   - Same trait contract violation
   - Same impact on interval calculations

2. **TangentSpacetime**: `deep_causality/src/types/context_node_types/space_time/tangent_spacetime/space_temporal_interval.rs`
   - Identical implementation returning `self.t` without conversion
   - Uses a custom `interval_squared()` with metric tensor, but still relies on buggy `time()` method
   - Same trait contract violation

All three implementations should be fixed together to maintain consistency across the codebase.


MinkowskiSpacetime Adjustable reads incorrect values with Array1D/2D/4D due to hardcoded 3D indexing

# Summary
- **Context**: The `MinkowskiSpacetime` type implements the `Adjustable` trait to allow updating or adjusting its x, y, z, and t coordinate values using data from an `ArrayGrid`.
- **Bug**: The implementation hardcodes `PointIndex::new3d` for indexing, which only works correctly with `Array3D` grids and produces incorrect results with `Array1D`, `Array2D`, or `Array4D` grids.
- **Actual vs. expected**: When using `Array1D` or `Array2D`, all four coordinates (x, y, z, t) read the same value from the grid instead of reading four distinct values; with `Array4D`, three coordinates get default values. The `Adjustable` trait documentation explicitly states that users can "select a 1, 2, 3, or 4 dimensional array grid", but only 3D works correctly.
- **Impact**: Users attempting to use `Array1D`, `Array2D`, or `Array4D` with `MinkowskiSpacetime.update()` or `adjust()` will silently receive incorrect results, potentially corrupting spacetime coordinate data in production systems without warning.

# Code with bug

From `deep_causality/src/types/context_node_types/space_time/minkowski_spacetime/adjustable.rs`:

```rust
fn update<const W: usize, const H: usize, const D: usize, const C: usize>(
    &mut self,
    array_grid: &ArrayGrid<f64, W, H, D, C>,
) -> Result<(), UpdateError> {
    // Create a 3D PointIndex for each of the updated x,y,z coordinates
    let p1 = PointIndex::new3d(0, 0, 0);  // <-- BUG 🔴 hardcoded new3d only works with Array3D
    let p2 = PointIndex::new3d(0, 0, 1);  // <-- BUG 🔴 incompatible with Array1D/2D/4D
    let p3 = PointIndex::new3d(0, 0, 2);  // <-- BUG 🔴 creates x=0, y=0, varying z
    let p4 = PointIndex::new3d(0, 0, 3);  // <-- BUG 🔴 wrong indexing pattern for other array types

    // Get the data at the index position from the array grid
    let new_x = array_grid.get(p1);
    let new_y = array_grid.get(p2);
    let new_z = array_grid.get(p3);
    let new_t = array_grid.get(p4);

    // ... validation and assignment ...
}
```

The same bug exists in the `adjust` method (lines 59-113).

# Evidence

## Example

### Scenario 1: Using Array1D (most natural for 4 sequential values)

**Array1D storage**: `[T; H]` uses only the x-coordinate for indexing: `self[p.x]`

**What the code does**:
```rust
// MinkowskiSpacetime creates these indices:
p1 = PointIndex::new3d(0, 0, 0)  // x=0, y=0, z=0
p2 = PointIndex::new3d(0, 0, 1)  // x=0, y=0, z=1
p3 = PointIndex::new3d(0, 0, 2)  // x=0, y=0, z=2
p4 = PointIndex::new3d(0, 0, 3)  // x=0, y=0, z=3
```

**Problem**: Array1D accesses `self[p.x]`, so:
- `p1` → `self[0]`
- `p2` → `self[0]` (same!)
- `p3` → `self[0]` (same!)
- `p4` → `self[0]` (same!)

**Result**: All four coordinates (x, y, z, t) get the same value from array position 0.

**Expected behavior**: Should read from positions 0, 1, 2, 3 by varying the x-coordinate.

### Scenario 2: Using Array2D

**Array2D storage**: `[[T; W]; H]` uses x and y coordinates: `self[p.y][p.x]`

**What the code does**: Creates indices with x=0, y=0, varying z (which Array2D ignores).

**Problem**: Array2D accesses `self[p.y][p.x] = self[0][0]` for all four indices.

**Result**: All four coordinates read from the same array cell `[0][0]`.

### Scenario 3: Using Array3D (the only working case)

**Array3D storage**: `[[[T; W]; H]; D]` uses pattern: `self[p.y][p.x][p.z]`

**What the code does**: Creates indices with x=0, y=0, varying z from 0 to 3.

**Result**: Accesses `self[0][0][0]`, `self[0][0][1]`, `self[0][0][2]`, `self[0][0][3]` ✓

This happens to work, but only because the z-coordinate varies and Array3D uses it as the innermost dimension.

## Inconsistency with trait documentation

### Reference spec

From `deep_causality/src/traits/adjustable/mod.rs`:

```rust
/// Depending on the type of node adjustment, select a 1, 2,3, or 4 dimensional array grid
/// that contains the transformation data to apply to the node.
```

### Contradiction

The trait documentation explicitly states that users can select 1D, 2D, 3D, or 4D array grids. However, the `MinkowskiSpacetime` implementation only works correctly with Array3D. Using Array1D or Array2D produces silently incorrect results, violating the documented contract.

## Inconsistency within the codebase

### Reference code: Array storage implementations

`deep_causality_data_structures/src/grid_type/storage_array_1d.rs`:
```rust
impl<T, const H: usize> Storage<T> for [T; H] {
    fn get(&self, p: PointIndex) -> &T {
        &self[p.x]  // Uses only x coordinate
    }
}
```

`deep_causality_data_structures/src/grid_type/storage_array_2d.rs`:
```rust
impl<T, const W: usize, const H: usize> Storage<T> for [[T; W]; H] {
    fn get(&self, p: PointIndex) -> &T {
        &self[p.y][p.x]  // Uses x and y coordinates
    }
}
```

`deep_causality_data_structures/src/grid_type/storage_array_3d.rs`:
```rust
impl<T, const W: usize, const H: usize, const D: usize> Storage<T> for [[[T; W]; H]; D] {
    fn get(&self, p: PointIndex) -> &T {
        &self[p.y][p.x][p.z]  // Uses x, y, and z coordinates
    }
}
```

### Current code

`deep_causality/src/types/context_node_types/space_time/minkowski_spacetime/adjustable.rs`:
```rust
let p1 = PointIndex::new3d(0, 0, 0);  // x=0, y=0, z=0
let p2 = PointIndex::new3d(0, 0, 1);  // x=0, y=0, z=1
let p3 = PointIndex::new3d(0, 0, 2);  // x=0, y=0, z=2
let p4 = PointIndex::new3d(0, 0, 3);  // x=0, y=0, z=3
```

### Contradiction

The indices created by `MinkowskiSpacetime` vary only the z-coordinate (keeping x=0, y=0). This pattern:
- **Fails with Array1D**: All reads access `self[0]` because Array1D only uses p.x, which is always 0
- **Fails with Array2D**: All reads access `self[0][0]` because Array2D uses p.y and p.x, which are both 0
- **Works with Array3D**: Accesses different positions because Array3D uses p.z as the innermost dimension
- **Partially fails with Array4D**: Would need proper t-coordinate values

The implementation assumes a specific array dimensionality (3D) despite the trait being designed to work with any dimensionality.

## Failing test

### Test script

```rust
/*
 * Test demonstrating the bug in MinkowskiSpacetime adjustable.rs
 */
use deep_causality::*;
use deep_causality_data_structures::{ArrayGrid, ArrayType, PointIndex};

#[test]
fn test_update_with_array1d_fails() {
    let mut s = MinkowskiSpacetime::new(1, 1.0, 2.0, 3.0, 4.0, TimeScale::Second);

    // Create Array1D - the most natural choice for 4 sequential values
    let grid: ArrayGrid<f64, 4, 4, 4, 4> = ArrayGrid::new(ArrayType::Array1D);

    // Set values at positions 0, 1, 2, 3 using new1d (natural for 1D arrays)
    grid.set(PointIndex::new1d(0), 10.0);
    grid.set(PointIndex::new1d(1), 20.0);
    grid.set(PointIndex::new1d(2), 30.0);
    grid.set(PointIndex::new1d(3), 40.0);

    // Perform update
    let result = s.update(&grid);
    assert!(result.is_ok());

    // EXPECTED: x=10.0, y=20.0, z=30.0, t=40.0
    // ACTUAL: x=10.0, y=10.0, z=10.0, t=10.0 (all read from array[0])
    assert_eq!(s.x(), 10.0);
    assert_eq!(s.y(), 20.0, "BUG: y should be 20.0 but reads from same index as x");
    assert_eq!(s.z(), 30.0, "BUG: z should be 30.0 but reads from same index as x");
    assert_eq!(*s.t(), 40.0, "BUG: t should be 40.0 but reads from same index as x");
}

#[test]
fn test_update_with_array2d_fails() {
    let mut s = MinkowskiSpacetime::new(1, 1.0, 2.0, 3.0, 4.0, TimeScale::Second);

    // Create Array2D
    let grid: ArrayGrid<f64, 4, 4, 4, 4> = ArrayGrid::new(ArrayType::Array2D);

    // Set values in a row (y=0, varying x)
    grid.set(PointIndex::new2d(0, 0), 10.0);
    grid.set(PointIndex::new2d(1, 0), 20.0);
    grid.set(PointIndex::new2d(2, 0), 30.0);
    grid.set(PointIndex::new2d(3, 0), 40.0);

    let result = s.update(&grid);
    assert!(result.is_ok());

    // EXPECTED: x=10.0, y=20.0, z=30.0, t=40.0
    // ACTUAL: x=10.0, y=10.0, z=10.0, t=10.0 (all read from array[0][0])
    assert_eq!(s.x(), 10.0);
    assert_eq!(s.y(), 20.0, "BUG: all values read from array[0][0]");
    assert_eq!(s.z(), 30.0, "BUG: all values read from array[0][0]");
    assert_eq!(*s.t(), 40.0, "BUG: all values read from array[0][0]");
}
```

### Test output

```
running 2 tests
test test_update_with_array1d_fails ... FAILED
test test_update_with_array2d_fails ... FAILED

failures:

---- test_update_with_array1d_fails stdout ----
thread 'test_update_with_array1d_fails' panicked at tests/test_minkowski_adjustable_bug.rs:24:5:
assertion `left == right` failed: BUG: y should be 20.0 but reads from same index as x
  left: 10.0
 right: 20.0

---- test_update_with_array2d_fails stdout ----
thread 'test_update_with_array2d_fails' panicked at tests/test_minkowski_adjustable_bug.rs:44:5:
assertion `left == right` failed: BUG: all values read from array[0][0]
  left: 10.0
 right: 20.0

failures:
    test_update_with_array1d_fails
    test_update_with_array2d_fails

test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out
```

# Full context

The `MinkowskiSpacetime` type represents a 4D spacetime event in special relativity, with three spatial coordinates (x, y, z) and one time coordinate (t). It is used throughout the DeepCausality framework as a context node type in causal graphs to model temporal-spatial relationships.

The `Adjustable` trait provides a mechanism to dynamically update or adjust these coordinate values using transformation data stored in an `ArrayGrid`. This is critical for scenarios where:
- Context graphs need global adjustments across millions of nodes
- Spacetime coordinates must be updated based on computed transformations
- Performance is critical (hence the use of fixed-size arrays instead of tensors)

The `ArrayGrid` type is specifically designed as a performance-optimized alternative to tensors, using fixed-size arrays with compile-time bounds checking. It supports 1D through 4D arrays via an enum type, allowing users to choose the appropriate dimensionality for their transformation data.

The bug occurs in the bridge between these two systems: the `MinkowskiSpacetime` implementation of `Adjustable` assumes a 3D array structure by hardcoding `PointIndex::new3d` calls with a specific indexing pattern (x=0, y=0, varying z). This breaks the abstraction and prevents the intended flexibility of the `Adjustable` trait.

Other spacetime types in the codebase have identical implementations with the same bug:
- `EuclideanSpacetime` (`deep_causality/src/types/context_node_types/space_time/euclidean_spacetime/adjustable.rs`)
- `LorentzianSpacetime` (`deep_causality/src/types/context_node_types/space_time/lorentzian_spacetime/adjustable.rs`)
- `TangentSpacetime` (`deep_causality/src/types/context_node_types/space_time/tangent_spacetime/adjustable.rs`)

The impact is significant because:
1. Users following the trait documentation will expect any array dimensionality to work
2. The bug produces incorrect results silently (no error is raised)
3. In production systems with millions of nodes, corrupted coordinate data could propagate throughout the causal graph
4. The bug affects a core feature (adjustability) that is explicitly documented and tested

## External documentation

### ArrayGrid README

From `deep_causality_data_structures/README_ArrayGrid.md`:

```markdown
# ArrayGrid - A Faster Tensor For Low Dimensional Data

ArrayGrid is an abstraction over scalars, vectors, and low dimensional matrices similar in idea to a tensor.
In contrast to a tensor, an ArrayGrid is limited to low dimensions (1 to 4), only allowing a scalar,
vector, or matrix type...

## Usage

Important details:

* All const generic parameters are required regardless of which ArrayType you are using
* To change the ArrayGrid type, just change the enum and you're good.
```

### Adjustable Trait Documentation

From `deep_causality/src/traits/adjustable/mod.rs`:

```rust
/// The default implementation does nothing to keep adjustment optional.
/// Override this method to implement a node adjustment when needed.
/// Depending on the type of node adjustment, select a 1, 2, 3, or 4 dimensional array grid
/// that contains the transformation data to apply to the node.
/// For a sample implementation, see src/types/context_types/node_types_adjustable
```

# Why has this bug gone undetected?

The bug has gone undetected for several reasons:

1. **Test bias**: All existing tests use `Array3D` and set values using the exact same indexing pattern as the implementation (`PointIndex::new3d(0, 0, z)`). The tests essentially replicate the bug rather than testing the intended behavior. See `deep_causality/tests/types/context_node_types/space_time/minkowski/adjustable_tests.rs`.

2. **Silent failure**: The bug doesn't raise any errors or panics - it produces plausible but incorrect results. All validation passes because it checks for finite values, which are satisfied even when the wrong values are read.

3. **Limited dimensionality testing**: No tests verify that Array1D, Array2D, or Array4D work correctly with spacetime types. The test suite only exercises Array3D, which happens to work due to the specific indexing pattern used.

4. **Documentation-implementation gap**: While the trait documentation promises support for 1D through 4D arrays, users likely default to the examples shown in tests (which all use Array3D), never discovering that other dimensionalities fail.

5. **Accidental correctness**: The use of `new3d(0, 0, z)` with Array3D works by accident - the z-coordinate happens to be the innermost dimension in the Array3D storage layout (`self[p.y][p.x][p.z]`), so varying z produces sequential access. This masks the underlying design problem.

6. **Low usage of flexibility**: Users may not be taking advantage of the documented ability to choose different array dimensionalities, instead following the patterns shown in the test suite. The feature is documented but perhaps rarely used in practice.

# Recommended fix

The implementation should detect the `ArrayGrid` variant at runtime and use appropriate indexing for each case:

```rust
fn update<const W: usize, const H: usize, const D: usize, const C: usize>(
    &mut self,
    array_grid: &ArrayGrid<f64, W, H, D, C>,
) -> Result<(), UpdateError> {
    // Use indices appropriate for linear access across all array types:
    // - Array1D: vary x (index 0, 1, 2, 3)
    // - Array2D: vary x along y=0 (or use appropriate 2D pattern)
    // - Array3D: vary z along x=0, y=0 (current pattern)
    // - Array4D: vary t along x=0, y=0, z=0

    // For simple sequential access, the most intuitive approach is:
    let p1 = PointIndex::new1d(0);  // <-- FIX 🟢
    let p2 = PointIndex::new1d(1);  // <-- FIX 🟢
    let p3 = PointIndex::new1d(2);
    let p4 = PointIndex::new1d(3);

    // This works with Array1D directly, and other types can access via x-coordinate
    // Alternatively, match on the array type and use type-specific indexing
}
```

Alternatively, the documentation should be updated to explicitly state that only Array3D is supported, though this would be a breaking change to the documented API contract.

# Related bugs

The following files have identical implementations with the same bug:
- `deep_causality/src/types/context_node_types/space_time/euclidean_spacetime/adjustable.rs`
- `deep_causality/src/types/context_node_types/space_time/lorentzian_spacetime/adjustable.rs`
- `deep_causality/src/types/context_node_types/space_time/tangent_spacetime/adjustable.rs`

Additionally, 3D space types (which have 3 coordinates instead of 4) may have a similar issue:
- `deep_causality/src/types/context_node_types/space/euclidean_space/adjustable.rs`
- `deep_causality/src/types/context_node_types/space/ecef_space/adjustable.rs`
- `deep_causality/src/types/context_node_types/space/ned_space/adjustable.rs`
- `deep_causality/src/types/context_node_types/space/geo_space/adjustable.rs`
- `deep_causality/src/types/context_node_types/space/quaternion_space/adjustable.rs`


MinkowskiSpacetime::time() ignores TimeScale and violates trait contract

# Summary
- **Context**: The `MinkowskiSpacetime` struct implements the `SpaceTemporalInterval` trait, which provides the `interval_squared()` method for computing Minkowski intervals between events in special relativity.
- **Bug**: The `time()` method returns the raw `t` field value without converting it from the stored `TimeScale` to seconds, violating the trait's contract.
- **Actual vs. expected**: The trait documentation explicitly states that `time()` must return time in seconds, but the implementation returns the time value in whatever `TimeScale` was used during construction (e.g., milliseconds, nanoseconds), causing interval calculations to be wrong by orders of magnitude.
- **Impact**: Minkowski interval calculations are completely incorrect when `TimeScale` is not `Second`, breaking causality analysis and spacetime physics computations for any non-second time units.

# Code with bug
```rust
impl SpaceTemporalInterval for MinkowskiSpacetime {
    fn time(&self) -> f64 {
        self.t  // <-- BUG 🔴 Returns raw time value without converting to seconds
    }
    fn position(&self) -> [f64; 3] {
        [self.x, self.y, self.z]
    }
}
```

From `deep_causality/src/types/context_node_types/space_time/minkowski_spacetime/mod.rs`:
```rust
pub struct MinkowskiSpacetime {
    id: u64,
    x: f64,
    y: f64,
    z: f64,
    t: f64,                // time in SI time unit
    time_scale: TimeScale, // SI time unit  <-- This field is completely ignored!
}
```

# Evidence

## Example

Consider two events that are physically 1 second apart in time and 1 meter apart in space:

**Case 1: Using `TimeScale::Second`**
```rust
let e1 = MinkowskiSpacetime::new(1, 0.0, 0.0, 0.0, 0.0, TimeScale::Second);
let e2 = MinkowskiSpacetime::new(2, 1.0, 0.0, 0.0, 1.0, TimeScale::Second);
let interval = e1.interval_squared(&e2);
// Result: -89875517873681760 (correct)
```

**Case 2: Same physical events using `TimeScale::Millisecond`**
```rust
let e1 = MinkowskiSpacetime::new(1, 0.0, 0.0, 0.0, 0.0, TimeScale::Millisecond);
let e2 = MinkowskiSpacetime::new(2, 1.0, 0.0, 0.0, 1000.0, TimeScale::Millisecond); // 1000ms = 1s
let interval = e1.interval_squared(&e2);
// Result: -89875517873681760000000 (WRONG - off by a factor of 1,000,000)
```

The implementation treats `1000.0` as `1000 seconds` instead of `1000 milliseconds = 1 second`.

For the Minkowski interval formula: `s² = -c²·Δt² + Δx² + Δy² + Δz²`

- With correct conversion: `Δt = 1.0 seconds`, so `s² = -(299792458)² × 1.0² + 1.0² ≈ -8.99×10¹⁶`
- With the bug: `Δt = 1000.0` (treated as seconds), so `s² = -(299792458)² × 1000² + 1.0² ≈ -8.99×10²²`

The bug causes the result to be **1 million times too large**, completely invalidating any causality analysis.

## Inconsistency with own spec

### Reference spec
From `deep_causality/src/traits/contextuable/space_temporal.rs`:
```rust
pub trait SpaceTemporalInterval {
    /// Returns the time coordinate in **seconds**.
    fn time(&self) -> f64;

    /// Returns the spatial coordinates `[x, y, z]` in **meters**.
    fn position(&self) -> [f64; 3];
```

The trait documentation explicitly states:
```
/// # Required Methods
/// - `time()`: Returns the scalar time coordinate in seconds
/// - `position()`: Returns the spatial coordinates `[x, y, z]` in meters
///
/// The default implementation assumes:
/// - Time is in **seconds**
/// - Space is in **meters**
/// - Speed of light `c = 299_792_458 m/s`
```

### Current code
From `deep_causality/src/types/context_node_types/space_time/minkowski_spacetime/space_temporal_interval.rs`:
```rust
impl SpaceTemporalInterval for MinkowskiSpacetime {
    fn time(&self) -> f64 {
        self.t  // Returns time in whatever TimeScale was used, NOT seconds
    }
    fn position(&self) -> [f64; 3] {
        [self.x, self.y, self.z]
    }
}
```

### Contradiction
The trait explicitly requires `time()` to return seconds, but the implementation returns `self.t` directly without any conversion based on `self.time_scale`. When a `MinkowskiSpacetime` is created with `TimeScale::Millisecond`, the `t` field contains milliseconds, not seconds, violating the trait contract.

## Inconsistency within the codebase

### Reference code
`deep_causality/src/types/context_node_types/space_time/lorentzian_spacetime/space_temporal_interval.rs`:
```rust
impl SpaceTemporalInterval for LorentzianSpacetime {
    fn time(&self) -> f64 {
        self.t  // Same bug exists here
    }
    fn position(&self) -> [f64; 3] {
        [self.x, self.y, self.z]
    }
}
```

### Current code
All spacetime implementations (`MinkowskiSpacetime`, `LorentzianSpacetime`) have identical implementations that ignore the `time_scale` field. This is a systematic issue across all spacetime types that implement `SpaceTemporalInterval`.

### Comparison
The bug affects multiple types consistently, suggesting this was a design oversight rather than an isolated implementation mistake. All implementations ignore their respective `time_scale` fields when implementing the `time()` method.

## Failing test

### Test script
```rust
/*
 * Unit test demonstrating the TimeScale bug in MinkowskiSpacetime::time()
 */

#[cfg(test)]
mod tests {
    use deep_causality::*;

    #[test]
    fn test_interval_with_millisecond_timescale() {
        // Define the same physical scenario in two different time units
        // Physical scenario: two events 1 second apart in time, 1 meter apart in space

        // Case 1: Using TimeScale::Second
        let e1_seconds = MinkowskiSpacetime::new(1, 0.0, 0.0, 0.0, 0.0, TimeScale::Second);
        let e2_seconds = MinkowskiSpacetime::new(2, 1.0, 0.0, 0.0, 1.0, TimeScale::Second);
        let interval_seconds = e1_seconds.interval_squared(&e2_seconds);

        // Case 2: Same physical scenario using TimeScale::Millisecond
        // 1 second = 1000 milliseconds
        let e1_millis = MinkowskiSpacetime::new(3, 0.0, 0.0, 0.0, 0.0, TimeScale::Millisecond);
        let e2_millis = MinkowskiSpacetime::new(4, 1.0, 0.0, 0.0, 1000.0, TimeScale::Millisecond);
        let interval_millis = e1_millis.interval_squared(&e2_millis);

        // The intervals should be the same since they represent the same physical scenario
        // Allow for floating point precision errors
        let epsilon = 1e-6;
        assert!(
            (interval_seconds - interval_millis).abs() < epsilon,
            "Intervals should be equal for the same physical events.\n\
             Got: seconds={}, millis={}",
            interval_seconds,
            interval_millis
        );
    }

    #[test]
    fn test_interval_with_nanosecond_timescale() {
        // Same physical scenario: two events 1 second apart in time, 1 meter apart in space

        // Case 1: Using TimeScale::Second
        let e1_seconds = MinkowskiSpacetime::new(1, 0.0, 0.0, 0.0, 0.0, TimeScale::Second);
        let e2_seconds = MinkowskiSpacetime::new(2, 1.0, 0.0, 0.0, 1.0, TimeScale::Second);
        let interval_seconds = e1_seconds.interval_squared(&e2_seconds);

        // Case 2: Using TimeScale::Nanoseconds
        // 1 second = 1_000_000_000 nanoseconds
        let e1_nanos = MinkowskiSpacetime::new(3, 0.0, 0.0, 0.0, 0.0, TimeScale::Nanoseconds);
        let e2_nanos = MinkowskiSpacetime::new(4, 1.0, 0.0, 0.0, 1_000_000_000.0, TimeScale::Nanoseconds);
        let interval_nanos = e1_nanos.interval_squared(&e2_nanos);

        // The intervals should be the same since they represent the same physical scenario
        let epsilon = 1e-6;
        assert!(
            (interval_seconds - interval_nanos).abs() < epsilon,
            "Intervals should be equal for the same physical events.\n\
             Got: seconds={}, nanos={}",
            interval_seconds,
            interval_nanos
        );
    }
}
```

### Test output
```
running 2 tests
test tests::test_interval_with_millisecond_timescale ... FAILED
test tests::test_interval_with_nanosecond_timescale ... FAILED

failures:

---- tests::test_interval_with_millisecond_timescale stdout ----

thread 'tests::test_interval_with_millisecond_timescale' (8222) panicked at test_interval_bug.rs:28:9:
Intervals should be equal for the same physical events.
Got: seconds=-89875517873681760, millis=-89875517873681760000000
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- tests::test_interval_with_nanosecond_timescale stdout ----

thread 'tests::test_interval_with_nanosecond_timescale' (8223) panicked at test_interval_bug.rs:54:9:
Intervals should be equal for the same physical events.
Got: seconds=-89875517873681760, nanos=-89875517873681770000000000000000000


failures:
    tests::test_interval_with_millisecond_timescale
    tests::test_interval_with_nanosecond_timescale

test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

The test clearly shows that the same physical scenario produces vastly different interval values depending on the `TimeScale` used, when they should produce identical results.

# Full context

The `SpaceTemporalInterval` trait is designed to compute Minkowski intervals for special relativity calculations. The trait provides a default implementation of `interval_squared()` that uses the formula:

```
s² = -c²·Δt² + Δx² + Δy² + Δz²
```

where `c = 299_792_458 m/s` (speed of light).

This trait is implemented by three spacetime types in the codebase:
1. `MinkowskiSpacetime` - flat spacetime for special relativity (affected by this bug)
2. `LorentzianSpacetime` - flat or curved spacetime for general relativity (affected by this bug)
3. `TangentSpacetime` - curved spacetime with custom metric tensor (not affected, as it overrides `interval_squared()`)

All three types store a `time_scale: TimeScale` field to indicate the time unit being used (seconds, milliseconds, microseconds, nanoseconds, etc.). However, the `SpaceTemporalInterval::time()` implementation ignores this field entirely.

The trait is used for:
- Determining causal relationships between events (time-like vs. space-like separation)
- Computing proper time intervals
- Identifying light-cone boundaries
- General relativistic causality analysis

When users create spacetime events with non-second time scales (which is common in high-precision physics simulations, sensor data processing, or quantum mechanics where nanosecond precision is needed), the interval calculations become incorrect by factors of 1000 (milliseconds), 1,000,000 (microseconds), or 1,000,000,000 (nanoseconds).

## External documentation

No external documentation applies directly, but the physics is standard:
- The Minkowski interval formula requires consistent units (SI units: meters and seconds)
- The speed of light constant used (299,792,458 m/s) is defined in SI units (meters per second)

# Why has this bug gone undetected?

The bug has gone undetected for several reasons:

1. **All existing tests use `TimeScale::Second`**: Every test for `interval_squared()` in the test suite uses `TimeScale::Second`, which makes the bug invisible:
   - `deep_causality/tests/types/context_node_types/space_time/minkowski/minkowski_spacetime_tests.rs` - all interval tests use `TimeScale::Second`
   - `deep_causality/tests/types/context_node_types/space_time/lorentzian/lorentzian_spacetime_tests.rs` - all interval tests use `TimeScale::Second`

2. **Tests with other TimeScales don't test intervals**: There are tests that create spacetime objects with `TimeScale::Millisecond` (e.g., line 33 in `minkowski_spacetime_tests.rs` and line 60 in `lorentzian_spacetime_tests.rs`), but these only test display/formatting functionality, not interval calculations.

3. **Systematic nature of the bug**: Since all spacetime implementations have the same bug, there's no correct reference implementation to compare against. The bug is consistent across the codebase.

4. **The TimeScale field appears optional**: Since the field is stored but never used in physics calculations, developers might assume it's purely for documentation or display purposes rather than being essential for correct computations.

5. **Limited real-world usage with non-second units**: If production code primarily uses `TimeScale::Second`, the bug would never manifest in practice, even though the API explicitly supports and documents other time scales.

# Recommended fix

The `time()` method must convert the stored time value to seconds based on the `time_scale` field:

```rust
impl SpaceTemporalInterval for MinkowskiSpacetime {
    fn time(&self) -> f64 {
        // Convert time to seconds based on time_scale  // <-- FIX 🟢
        match self.time_scale {
            TimeScale::Second => self.t,
            TimeScale::Millisecond => self.t / 1_000.0,
            TimeScale::Microseconds => self.t / 1_000_000.0,
            TimeScale::Nanoseconds => self.t / 1_000_000_000.0,
            TimeScale::Minute => self.t * 60.0,
            TimeScale::Hour => self.t * 3_600.0,
            TimeScale::Day => self.t * 86_400.0,
            TimeScale::Week => self.t * 604_800.0,
            // For other time scales, either panic or return self.t
            // depending on whether they should be supported
            _ => self.t,  // or panic!("Unsupported TimeScale for physics calculations")
        }
    }
    fn position(&self) -> [f64; 3] {
        [self.x, self.y, self.z]
    }
}
```

Alternatively, create a helper method on `TimeScale` to get the conversion factor to seconds, and use it across all implementations.

# Related bugs

The following files have identical bugs:
1. `deep_causality/src/types/context_node_types/space_time/lorentzian_spacetime/space_temporal_interval.rs` - same bug in `LorentzianSpacetime::time()`

Note: `TangentSpacetime` is not affected because it overrides `interval_squared()` with a custom implementation using a metric tensor, bypassing the default implementation that relies on `time()`.


update_node desynchronizes id_to_index_map when replacing a node with a different ID

# Summary
- **Context**: The `update_node` method in `ContextuableGraph` implementation maintains an `id_to_index_map` that translates logical `ContextoidId`s to physical graph indices.
- **Bug**: When `update_node` replaces a node with a new node that has a different ID, it fails to update the `id_to_index_map`, causing the map to become desynchronized with the actual graph state.
- **Actual vs. expected**: The map continues to point from the old ID to the index, but the node at that index now has a new ID; the map should be updated to remove the old ID mapping and add a new mapping for the new ID.
- **Impact**: Nodes become unreachable by their actual ID, can be accessed by stale IDs, cannot be removed by their actual ID, and duplicate IDs can be inadvertently created in the map.

# Code with bug

```rust
fn update_node(
    &mut self,
    node_id: ContextoidId,
    new_node: Contextoid<D, S, T, ST, SYM, VS, VT>,
) -> Result<(), ContextIndexError> {
    if let Some(&index_to_update) = self.id_to_index_map.get(&node_id) {
        self.base_context
            .update_node(index_to_update, new_node)  // <-- BUG 🔴 Updates the graph but not the id_to_index_map
            .map_err(|e| ContextIndexError(e.to_string()))
    } else {
        Err(ContextIndexError(format!(
            "Cannot update node. Contextoid with ID {node_id} not found in context"
        )))
    }
}
```

The bug occurs because:
1. The function retrieves the physical index from `id_to_index_map` using the old `node_id`
2. It updates the graph node at that index with `new_node` (which may have a different ID)
3. It never updates `id_to_index_map` to reflect the new node's ID
4. The map still contains `old_id -> index` but the node at `index` now has `new_id`

# Evidence

## Example

Consider this sequence:

1. **Initial state**: Add a node with ID 42
   - Graph: `index 0 -> Contextoid{id: 42}`
   - Map: `{42 -> 0}`

2. **Update operation**: Call `update_node(42, new_node_with_id_99)`
   - Graph after: `index 0 -> Contextoid{id: 99}`
   - Map after: `{42 -> 0}` (unchanged - this is the bug!)

3. **Broken invariants**:
   - `get_node_index_by_id(99)` returns `None` (should return `Some(0)`)
   - `get_node_index_by_id(42)` returns `Some(0)` (should return `None`)
   - `get_node(0).id()` returns `99` (correct)
   - **Invariant violation**: `map[old_id] -> index` but `graph[index].id() == new_id`

4. **Cascading failures**:
   - Cannot remove the node by its actual ID (99)
   - Can still "find" the node by a stale ID (42) that doesn't match
   - Can add another node with ID 42, creating duplicate map entries

## Inconsistency with own spec/documentation

### Reference: Git commit message

From commit `6b685a3e` that introduced `id_to_index_map`:

```
To support these operations safely, the Context struct now includes an
id_to_index_map. This map translates stable, logical ContextoidIds to
the volatile physical indices used by the underlying ultragraph,
preventing ID-desynchronization bugs.
```

### Current code

The `update_node` implementation updates the underlying graph but does not update the `id_to_index_map`:

```rust
fn update_node(&mut self, node_id: ContextoidId, new_node: Contextoid<...>) -> Result<(), ContextIndexError> {
    if let Some(&index_to_update) = self.id_to_index_map.get(&node_id) {
        self.base_context
            .update_node(index_to_update, new_node)  // Updates graph only
            .map_err(|e| ContextIndexError(e.to_string()))
        // Missing: updating id_to_index_map with new_node.id()
    } else {
        Err(ContextIndexError(...))
    }
}
```

### Contradiction

The `id_to_index_map` was explicitly added to "prevent ID-desynchronization bugs" by maintaining a consistent mapping between logical IDs and physical indices. However, `update_node` fails to maintain this consistency when the new node has a different ID than the old node, directly causing the ID-desynchronization bugs the map was meant to prevent.

## Inconsistency within the codebase

### Reference code: `add_node` method

`deep_causality/src/types/context_types/context_graph/contextuable_graph.rs:25-36`

```rust
fn add_node(
    &mut self,
    value: Contextoid<D, S, T, ST, SYM, VS, VT>,
) -> Result<usize, ContextIndexError> {
    let contextoid_id = value.id();  // Extract ID from the node
    let index = match self.base_context.add_node(value) {
        Ok(index) => index,
        Err(e) => return Err(ContextIndexError(e.to_string())),
    };
    self.id_to_index_map.insert(contextoid_id, index);  // Update map
    Ok(index)
}
```

### Reference code: `remove_node` method

`deep_causality/src/types/context_types/context_graph/contextuable_graph.rs:49-65`

```rust
fn remove_node(&mut self, node_id: ContextoidId) -> Result<(), ContextIndexError> {
    if let Some(&index_to_remove) = self.id_to_index_map.get(&node_id) {
        self.base_context
            .remove_node(index_to_remove)
            .map_err(|e| ContextIndexError(e.to_string()))?;

        // If successful, then remove the entry from our map to stay in sync.
        self.id_to_index_map.remove(&node_id);  // Update map

        Ok(())
    } else {
        Err(ContextIndexError(...))
    }
}
```

### Current code: `update_node` method

`deep_causality/src/types/context_types/context_graph/contextuable_graph.rs:67-81`

```rust
fn update_node(
    &mut self,
    node_id: ContextoidId,
    new_node: Contextoid<D, S, T, ST, SYM, VS, VT>,
) -> Result<(), ContextIndexError> {
    if let Some(&index_to_update) = self.id_to_index_map.get(&node_id) {
        self.base_context
            .update_node(index_to_update, new_node)
            .map_err(|e| ContextIndexError(e.to_string()))
        // Missing: any update to id_to_index_map
    } else {
        Err(ContextIndexError(...))
    }
}
```

### Contradiction

Both `add_node` and `remove_node` consistently maintain the `id_to_index_map` in sync with the underlying graph:
- `add_node` extracts the ID from the node being added and inserts the mapping
- `remove_node` explicitly removes the mapping with a comment "to stay in sync"

However, `update_node` breaks this pattern by modifying the graph without updating the map. This inconsistency violates the established invariant that the map should always accurately reflect the ID-to-index relationship of nodes in the graph.

## Failing test

### Test script

```rust
/*
 * Test to demonstrate the update_node bug where the id_to_index_map
 * becomes out of sync when a node is updated with a different ID.
 */

use deep_causality::{
    BaseContext, Context, Contextoid, ContextoidType, ContextuableGraph, Identifiable, Root,
};

#[test]
fn test_update_node_with_different_id_breaks_id_mapping() {
    let mut context: BaseContext = Context::with_capacity(1, "test context", 10);

    // Add a node with ID 42
    let original_id = 42;
    let original_contextoid = Contextoid::new(original_id, ContextoidType::Root(Root::new(original_id)));
    let index = context.add_node(original_contextoid).expect("Failed to add node");

    // Verify we can find the node by its ID
    let found_index = context.get_node_index_by_id(original_id);
    assert!(found_index.is_some(), "Should find the original node ID");
    assert_eq!(found_index.unwrap(), index);

    // Now update the node at that index with a NEW node that has a DIFFERENT ID
    let new_id = 99;
    let new_contextoid = Contextoid::new(new_id, ContextoidType::Root(Root::new(new_id)));

    context.update_node(original_id, new_contextoid).expect("Failed to update node");

    // BUG VERIFICATION: The id_to_index_map is now out of sync

    // Get the actual node at the index and check its ID
    let actual_node = context.get_node(index).expect("Node should exist at index");
    let actual_node_id = actual_node.id();

    // The actual node now has new_id
    assert_eq!(actual_node_id, new_id, "Node at index should have new ID");

    // But we can't find it by its actual ID in the mapping
    let found_new = context.get_node_index_by_id(new_id);
    assert!(found_new.is_none(), "BUG: Cannot find node by its actual new ID");

    // And we can still find it by the old ID that doesn't match
    let found_old = context.get_node_index_by_id(original_id);
    assert!(found_old.is_some(), "BUG: Can still find node by stale old ID");

    // This demonstrates the mapping is corrupted:
    // id_to_index_map[original_id] -> index
    // But graph[index].id() == new_id
    // And id_to_index_map[new_id] doesn't exist

    println!("BUG CONFIRMED: id_to_index_map is out of sync after update_node with different ID");
    println!("  - id_to_index_map still has: {} -> {}", original_id, found_old.unwrap());
    println!("  - But node at index {} has ID: {}", index, actual_node_id);
    println!("  - And id_to_index_map does NOT have: {} -> {}", new_id, index);
}

#[test]
fn test_update_node_prevents_removal_by_actual_id() {
    let mut context: BaseContext = Context::with_capacity(1, "test context", 10);

    let original_id = 42;
    let original_contextoid = Contextoid::new(original_id, ContextoidType::Root(Root::new(original_id)));
    context.add_node(original_contextoid).expect("Failed to add node");

    // Update with a different ID
    let new_id = 99;
    let new_contextoid = Contextoid::new(new_id, ContextoidType::Root(Root::new(new_id)));
    context.update_node(original_id, new_contextoid).expect("Failed to update node");

    // Try to remove by the new ID (the actual ID of the node) - this should work but doesn't
    let result = context.remove_node(new_id);
    assert!(result.is_err(), "BUG: Cannot remove node by its actual ID after update");
}

#[test]
fn test_update_node_allows_duplicate_id_insertion() {
    let mut context: BaseContext = Context::with_capacity(1, "test context", 10);

    let original_id = 42;
    let original_contextoid = Contextoid::new(original_id, ContextoidType::Root(Root::new(original_id)));
    let first_index = context.add_node(original_contextoid).expect("Failed to add node");

    // Update with a different ID
    let new_id = 99;
    let new_contextoid = Contextoid::new(new_id, ContextoidType::Root(Root::new(new_id)));
    context.update_node(original_id, new_contextoid).expect("Failed to update node");

    // Now try to add another node with the original ID
    // This should arguably fail (duplicate ID) or succeed
    // But the behavior is inconsistent because the map still has the old ID
    let another_node = Contextoid::new(original_id, ContextoidType::Root(Root::new(original_id)));
    let result = context.add_node(another_node);

    // If it succeeds, we now have TWO indices associated with the same ID
    // which violates the intended 1-to-1 mapping
    if let Ok(second_index) = result {
        println!("BUG: Added second node with same ID {} at different index {}", original_id, second_index);
        println!("  First was at index {}, second at index {}", first_index, second_index);

        // The map will have been overwritten, so now it points to the second index
        let found = context.get_node_index_by_id(original_id);
        assert!(found.is_some());
        // It now points to the second index, but the first index is orphaned
        assert_eq!(found.unwrap(), second_index, "Map was overwritten to point to second node");

        // The node at first_index still exists but is unreachable by ID
        let orphaned_node = context.get_node(first_index);
        assert!(orphaned_node.is_some(), "Original node still exists but is orphaned");
    }
}
```

### Test output

```
$ cargo test --package deep_causality --test bug_test_update_node_id_mismatch
   Compiling deep_causality v0.13.2 (/home/user/deep_causality/deep_causality)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 15.42s
     Running tests/bug_test_update_node_id_mismatch.rs (target/debug/deps/bug_test_update_node_id_mismatch-fa1e7c5dcd200323)

running 3 tests
test test_update_node_prevents_removal_by_actual_id ... ok
test test_update_node_allows_duplicate_id_insertion ... ok
test test_update_node_with_different_id_breaks_id_mapping ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

All three tests pass, which demonstrates that:
1. The `id_to_index_map` becomes desynchronized when `update_node` is called with a node having a different ID
2. A node cannot be removed by its actual ID after such an update
3. Duplicate IDs can be added to the system, orphaning the original node

# Full context

The `Context` struct maintains a graph of `Contextoid` nodes using the `ultragraph` library. The ultragraph uses integer indices to reference nodes, and these indices can change when nodes are removed (due to index reuse or compaction). To provide stable node references, the `Context` struct introduced the `id_to_index_map` field that maps logical `ContextoidId` values (which are stable, application-level identifiers stored in each `Contextoid`) to volatile physical indices in the underlying graph.

This mapping layer is critical for the generative system, which uses the context graph to store and manipulate causal model structures at runtime. The `Interpreter` component in `deep_causality/src/types/generative_types/interpreter.rs` executes operations like `UpdateContextoidInContext`, which relies on `update_node` to modify nodes in the context graph. When these operations provide a new node with a different ID, the bug causes the mapping to become corrupted.

The bug affects several operations:
- **Node lookup**: Nodes become unreachable by their actual ID via `get_node_index_by_id`
- **Node removal**: Attempting to remove a node by its actual ID fails because the map doesn't contain the new ID
- **Graph consistency**: The map can contain stale mappings that point to nodes with different IDs
- **ID uniqueness**: The system can inadvertently allow duplicate IDs when a new node is added with an old ID that's still in the map

The `ContextuableGraph` trait defines the interface for these operations and is implemented by `Context`. The trait methods `add_node`, `remove_node`, and `update_node` all accept or manipulate nodes with IDs, establishing an expectation that the implementation will maintain ID consistency.

## External documentation

- [ultragraph GraphMut trait documentation](https://github.com/deepcausality-rs/ultragraph/blob/main/src/traits/graph_mut.rs)

```rust
/// Trait for mutable graph operations.
pub trait GraphMut<N, W>: Graph<N> {
    // Node Mutation
    fn add_node(&mut self, node: N) -> Result<usize, GraphError>;
    fn update_node(&mut self, index: usize, node: N) -> Result<(), GraphError>;
    fn remove_node(&mut self, index: usize) -> Result<(), GraphError>;
    // ...
}
```

The ultragraph's `update_node` simply replaces the node at a given index with a new node. It is unaware of IDs and does not maintain any ID-to-index mapping. This responsibility falls entirely on the `Context` implementation.

# Why has this bug gone undetected?

This bug has likely gone undetected for several reasons:

1. **Uncommon usage pattern**: The bug only manifests when `update_node` is called with a new node that has a *different* ID than the original node. Most typical usage might update a node while preserving its ID (e.g., changing the node's data but keeping the same identifier), which wouldn't trigger the bug.

2. **New feature**: The `update_node` method and `id_to_index_map` were both added in the same commit (`6b685a3e` on July 1, 2025) as part of a "full CRUD for context and contextoids" feature. This is relatively recent code that may not have been exercised extensively in production scenarios yet.

3. **Indirect access**: The test file `deep_causality/tests/types/context_types/context_graph/contextuable_graph_tests.rs` only contains an error case test for `update_node` where the node doesn't exist. It doesn't test the successful update path, and certainly not with a different ID.

4. **Higher-level abstractions**: Most code interacts with the context through higher-level abstractions like the generative interpreter, which may typically generate update operations that preserve node IDs. The bug would only appear if the generative system produces an operation to replace a node with a completely different one.

5. **Subtle failure mode**: The bug doesn't cause immediate crashes or obvious errors. The system continues to function, but with degraded behavior (can't find nodes by ID, can remove by wrong ID, etc.). Without specific tests checking the ID mapping invariants, this could go unnoticed.

6. **Implementation oversight**: The commit message explicitly mentions preventing "ID-desynchronization bugs," but the `update_node` implementation appears to have been written without considering that the new node could have a different ID. The code pattern in `add_node` and `remove_node` shows the correct way to maintain the map, but this pattern wasn't followed for `update_node`.

# Recommended fix

The `update_node` method should maintain the `id_to_index_map` by:
1. Extracting the ID from the new node
2. If the new ID differs from the old ID:
   - Remove the old ID mapping
   - Insert a new mapping for the new ID
3. Updating the node in the underlying graph

Here's the corrected implementation:

```rust
fn update_node(
    &mut self,
    node_id: ContextoidId,
    new_node: Contextoid<D, S, T, ST, SYM, VS, VT>,
) -> Result<(), ContextIndexError> {
    if let Some(&index_to_update) = self.id_to_index_map.get(&node_id) {
        let new_node_id = new_node.id();  // <-- FIX 🟢 Extract new node's ID

        self.base_context
            .update_node(index_to_update, new_node)
            .map_err(|e| ContextIndexError(e.to_string()))?;

        // <-- FIX 🟢 Update id_to_index_map if ID changed
        if new_node_id != node_id {
            self.id_to_index_map.remove(&node_id);
            self.id_to_index_map.insert(new_node_id, index_to_update);
        }

        Ok(())
    } else {
        Err(ContextIndexError(format!(
            "Cannot update node. Contextoid with ID {node_id} not found in context"
        )))
    }
}
```

This fix ensures that the `id_to_index_map` always accurately reflects the actual IDs of nodes in the graph, maintaining the invariant that the map was designed to enforce.

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


--

# Summary
- **Context**: The `deep_causality_physics::fluids::mechanics` module provides core fluid mechanics calculations (`hydrostatic_pressure_kernel` and `bernoulli_pressure_kernel`) that accept strongly-typed physical quantities (Pressure, Density, Speed, Length) as inputs.
- **Bug**: The validation logic in the quantity types fails to reject NaN (Not-a-Number) values, allowing them to propagate through physics calculations undetected.
- **Actual vs. expected**: NaN values pass the validation check `if val < 0.0` (since `NaN < 0.0` evaluates to `false`), but physical quantities should only accept finite, valid numerical values.
- **Impact**: Invalid calculations with NaN results can succeed silently, producing meaningless output that appears valid but corrupts downstream physics simulations.

# Code with bug

The bug exists in the validation logic of quantity types used by `mechanics.rs`:

```rust
// From deep_causality_physics/src/fluids/quantities.rs
impl Pressure {
    pub fn new(val: f64) -> Result<Self, PhysicsError> {
        if val < 0.0 {  // <-- BUG 🔴 NaN < 0.0 evaluates to false, so NaN passes
            return Err(PhysicsError::new(
                PhysicsErrorEnum::PhysicalInvariantBroken("Negative Pressure".into()),
            ));
        }
        Ok(Self(val))
    }
}
```

The same flawed validation pattern appears in:
- `Density::new()` in `deep_causality_physics/src/fluids/quantities.rs:39`
- `Speed::new()` in `deep_causality_physics/src/dynamics/quantities.rs:52`
- `Length::new()` in `deep_causality_physics/src/dynamics/quantities.rs:149`

These types are used in `mechanics.rs`:

```rust
// From deep_causality_physics/src/fluids/mechanics.rs
pub fn hydrostatic_pressure_kernel(
    p0: &Pressure,    // <-- Can contain NaN
    density: &Density, // <-- Can contain NaN
    depth: &Length,   // <-- Can contain NaN
) -> Result<Pressure, PhysicsError> {
    let rho_g_h = density.value() * G * depth.value();
    let p_total = p0.value() + rho_g_h;  // <-- NaN propagates through calculation

    Pressure::new(p_total)  // <-- NaN passes validation again
}
```

# Evidence

## Example

Here's how NaN propagates through the system:

**Step 1**: Create a Pressure with NaN value
```rust
let nan_pressure = Pressure::new(f64::NAN);  // Should fail, but doesn't!
// Result: Ok(Pressure(NaN))
```

**Step 2**: Use it in hydrostatic pressure calculation
```rust
let result = hydrostatic_pressure_kernel(&nan_pressure, &density, &depth);
// Calculation: NaN + (1000.0 * 9.80665 * 10.0) = NaN
// Result: Ok(Pressure(NaN))
```

**Step 3**: The invalid result appears successful
```rust
match result {
    Ok(p) => println!("Pressure: {} Pa", p.value()),  // Prints "Pressure: NaN Pa"
    Err(e) => println!("Error: {:?}", e),              // Never executed
}
```

**Why NaN passes validation:**
- The check `if val < 0.0` evaluates to `false` for NaN
- In fact, NaN compared to any value always returns `false`:
    - `NaN < 0.0` → `false`
    - `NaN >= 0.0` → `false`
    - `NaN == 0.0` → `false`
- Therefore NaN bypasses the negative value check and is accepted

## Failing test

### Test script

```rust
/// Demonstrates NaN validation bug in mechanics.rs
use deep_causality_physics::{
    Density, Length, Pressure, Speed, bernoulli_pressure_kernel, hydrostatic_pressure_kernel,
};

fn main() {
    // Test 1: NaN pressure propagates through hydrostatic_pressure_kernel
    println!("Test 1: NaN in hydrostatic_pressure_kernel");
    let nan_p0 = Pressure::new(f64::NAN).expect("Pressure should reject NaN but doesn't");
    let density = Density::new(1000.0).unwrap();
    let depth = Length::new(10.0).unwrap();

    let result = hydrostatic_pressure_kernel(&nan_p0, &density, &depth);
    match result {
        Ok(p) => {
            assert!(p.value().is_nan(), "Output should be NaN");
            println!("  ❌ FAIL: NaN propagated undetected, output = {}", p.value());
        }
        Err(_) => println!("  ✓ PASS: Function rejected invalid input"),
    }

    // Test 2: NaN density propagates through calculation
    println!("\nTest 2: NaN density in hydrostatic_pressure_kernel");
    let p0 = Pressure::new(101325.0).unwrap();
    let nan_density = Density::new(f64::NAN).expect("Density should reject NaN but doesn't");
    let depth = Length::new(10.0).unwrap();

    let result = hydrostatic_pressure_kernel(&p0, &nan_density, &depth);
    match result {
        Ok(p) => {
            assert!(p.value().is_nan(), "Output should be NaN");
            println!("  ❌ FAIL: NaN propagated undetected, output = {}", p.value());
        }
        Err(_) => println!("  ✓ PASS: Function rejected invalid input"),
    }

    // Test 3: NaN speed in bernoulli_pressure_kernel
    println!("\nTest 3: NaN speed in bernoulli_pressure_kernel");
    let p1 = Pressure::new(100000.0).unwrap();
    let nan_v1 = Speed::new(f64::NAN).expect("Speed should reject NaN but doesn't");
    let h1 = Length::new(10.0).unwrap();
    let v2 = Speed::new(10.0).unwrap();
    let h2 = Length::new(5.0).unwrap();
    let density = Density::new(1000.0).unwrap();

    let result = bernoulli_pressure_kernel(&p1, &nan_v1, &h1, &v2, &h2, &density);
    match result {
        Ok(p) => {
            assert!(p.value().is_nan(), "Output should be NaN");
            println!("  ❌ FAIL: NaN propagated undetected, output = {}", p.value());
        }
        Err(_) => println!("  ✓ PASS: Function rejected invalid input"),
    }

    // Test 4: Infinity is also accepted (may or may not be desired)
    println!("\nTest 4: Infinity handling");
    let inf_pressure = Pressure::new(f64::INFINITY);
    match inf_pressure {
        Ok(p) => println!("  ⚠️  Pressure::new(∞) succeeded: {}", p.value()),
        Err(_) => println!("  ✓ Pressure::new(∞) rejected"),
    }
}
```

### Test output

```
Test 1: NaN in hydrostatic_pressure_kernel
  ❌ FAIL: NaN propagated undetected, output = NaN

Test 2: NaN density in hydrostatic_pressure_kernel
  ❌ FAIL: NaN propagated undetected, output = NaN

Test 3: NaN speed in bernoulli_pressure_kernel
  ❌ FAIL: NaN propagated undetected, output = NaN

Test 4: Infinity handling
  ⚠️  Pressure::new(∞) succeeded: inf
```

# Full context

The `deep_causality_physics` crate provides a comprehensive physics simulation library with strong typing to enforce physical invariants. According to the README:

> **Quantities** (`*::quantities`, `units::*`): Newtype wrappers (e.g., `Speed`, `Mass`, `Temperature`) that **enforce physical invariants** (e.g., mass cannot be negative) and type safety.

The `mechanics.rs` module contains two core fluid mechanics functions:

1. **`hydrostatic_pressure_kernel`**: Computes pressure at depth using formula P = P₀ + ρgh
    - Used in fluid simulations, reservoir modeling, submarine depth calculations
    - Exported through causal wrapper `hydrostatic_pressure()` in `wrappers.rs`

2. **`bernoulli_pressure_kernel`**: Applies Bernoulli's principle to compute pressure changes in flowing fluids
    - Formula: P₂ = P₁ + ½ρ(v₁² - v₂²) + ρg(h₁ - h₂)
    - Used in pipe networks, Venturi meters, aircraft wing analysis
    - Exported through causal wrapper `bernoulli_pressure()` in `wrappers.rs`

These functions are designed to be composed in the DeepCausality hyper-graph simulation engine using the `PropagatingEffect` monad, enabling complex multi-physics simulations.

The functions rely on type-safe quantity wrappers to prevent invalid inputs:
- `Pressure` (Pa) - Must be non-negative in absolute pressure contexts
- `Density` (kg/m³) - Must be non-negative
- `Speed` (m/s) - Must be non-negative (scalar magnitude)
- `Length` (m) - Must be non-negative

These types are created using constructors like `Pressure::new()` which validate the input values. However, the validation only checks for negative values using `if val < 0.0`, which fails to detect NaN.

The bug affects the entire physics crate as the same validation pattern is used across multiple modules:
- Fluids: `Pressure`, `Density`, `Viscosity`
- Dynamics: `Speed`, `Length`, `Mass`, `Area`, `Volume`, `MomentOfInertia`, `Frequency`
- Materials: Various material properties
- MHD: Magnetic field quantities
- Nuclear: Nuclear quantities
- And others throughout the crate

# Why has this bug gone undetected?

1. **No test coverage for invalid inputs**: The test suite in `deep_causality_physics/tests/fluids/mechanics_tests.rs` only tests valid, well-formed inputs. There are no tests checking boundary conditions, NaN, or Infinity values.

2. **NaN is rare in normal use**: In typical physics simulations with valid numerical inputs, NaN rarely occurs naturally. Users would need to encounter a calculation error (like 0/0 or ∞/∞) upstream for NaN to appear.

3. **Silent failure mode**: When NaN does occur, it propagates silently through calculations. The functions return `Ok(Pressure(NaN))` rather than erroring, so error handling code is never triggered. This makes debugging difficult because the problem manifests far downstream from its source.

4. **Type system provides false confidence**: The strong typing gives developers confidence that invalid values are caught at construction time. The presence of `Result<Self, PhysicsError>` suggests validation is thorough, masking the gap in NaN handling.

5. **IEEE 754 semantics are counterintuitive**: The behavior that `NaN < 0.0 == false` is technically correct per IEEE 754, but counterintuitive. Developers might reasonably assume the negative check catches all invalid values.

6. **Recent implementation**: The code was added relatively recently (commits 419182ff and 2ab7b206) as part of the initial physics crate implementation, so it hasn't been battle-tested in production scenarios where numerical instability might produce NaN values.

# Recommended fix

Replace the validation check with a proper finite check:

```rust
pub fn new(val: f64) -> Result<Self, PhysicsError> {
    if !val.is_finite() {  // <-- FIX 🟢 Rejects NaN and ±Infinity
        return Err(PhysicsError::new(
            PhysicsErrorEnum::PhysicalInvariantBroken(
                format!("Invalid value (must be finite): {}", val)
            ),
        ));
    }
    if val < 0.0 {  // <-- Keep negative check for physical invariant
        return Err(PhysicsError::new(
            PhysicsErrorEnum::PhysicalInvariantBroken(
                format!("Negative Pressure: {}", val)
            ),
        ));
    }
    Ok(Self(val))
}
```

Alternative single-check version:
```rust
pub fn new(val: f64) -> Result<Self, PhysicsError> {
    if !val.is_finite() || val < 0.0 {  // <-- FIX 🟢
        return Err(PhysicsError::new(
            PhysicsErrorEnum::PhysicalInvariantBroken(
                format!("Invalid Pressure (must be finite and non-negative): {}", val)
            ),
        ));
    }
    Ok(Self(val))
}
```

This fix should be applied to all quantity types with non-negative constraints throughout the crate.

For quantities that allow negative values (Force, Acceleration, Torque), the fix is simpler:
```rust
pub fn new(val: f64) -> Result<Self, PhysicsError> {
    if !val.is_finite() {
        return Err(PhysicsError::new(
            PhysicsErrorEnum::PhysicalInvariantBroken(
                format!("Invalid value (must be finite): {}", val)
            ),
        ));
    }
    Ok(Self(val))
}
```

# Related bugs

The same validation bug exists in quantity types across the entire `deep_causality_physics` crate:

- `deep_causality_physics/src/fluids/quantities.rs`: `Pressure`, `Density`, `Viscosity`
- `deep_causality_physics/src/dynamics/quantities.rs`: `Mass`, `Speed`, `Length`, `Area`, `Volume`, `MomentOfInertia`, `Frequency`
- `deep_causality_physics/src/materials/quantities.rs`: Material property types
- `deep_causality_physics/src/mhd/quantities.rs`: MHD quantity types
- `deep_causality_physics/src/nuclear/quantities.rs`: Nuclear quantity types
- `deep_causality_physics/src/condensed/quantities.rs`: Condensed matter types
- `deep_causality_physics/src/units/temperature.rs`: Temperature type

Each file uses the same `if val < 0.0` pattern that fails to catch NaN.


--

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


--

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

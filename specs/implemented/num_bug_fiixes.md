# Summary
- **Context**: The `powi` method in `deep_causality_num/src/complex/complex_number/ops.rs` computes integer powers of complex numbers using exponentiation by squaring.
- **Bug**: For negative exponents with absolute value greater than 1, `powi` returns the inverse of the original number instead of the inverse of the number raised to the positive power.
- **Actual vs. expected**: `(2 + 0i)^-2` returns `0.5` instead of `0.25`; `(1 + i)^-2` returns `0.5 - 0.5i` instead of `0 - 0.5i`.
- **Impact**: Any computation using complex numbers raised to negative powers (except -1) produces mathematically incorrect results, potentially affecting physics simulations, quantum computations, and signal processing algorithms.

# Code with bug
```rust
pub fn powi(&self, n: i32) -> Self {
    if n == 0 {
        return Self::one();
    }
    let mut res = Self::one();
    let mut base = *self;
    let mut n_abs = n.abs();

    while n_abs > 0 {
        if n_abs % 2 == 1 {
            res *= base;
        }
        base = base * base;
        n_abs /= 2;
    }

    if n < 0 { self._inverse_impl() } else { res } // <-- BUG 🔴 should be res._inverse_impl()
}
```

# Evidence

## Example

Let's trace through `(2 + 0i)^-2`:

1. **Input**: `self = (2 + 0i)`, `n = -2`
2. **Initialize**: `res = (1 + 0i)`, `base = (2 + 0i)`, `n_abs = 2`
3. **Loop iteration 1**: `n_abs = 2` (even), so skip `res *= base`
    - `base = base * base = (4 + 0i)`
    - `n_abs = 1`
4. **Loop iteration 2**: `n_abs = 1` (odd), so `res *= base`
    - `res = (1 + 0i) * (4 + 0i) = (4 + 0i)`
    - `base = base * base = (16 + 0i)`
    - `n_abs = 0`
5. **Exit loop**: `res = (4 + 0i)` (this is `self^2`)
6. **Return**: Since `n < 0`, returns `self._inverse_impl() = (0.5 + 0i)` ❌

**Expected**: Should return `res._inverse_impl() = 1/(4 + 0i) = (0.25 + 0i)` ✅

Similarly for `(1 + i)^-2`:
- After the loop: `res = (1 + i)^2 = (0 + 2i)`
- Bug returns: `(1 + i)^-1 = (0.5 - 0.5i)` ❌
- Should return: `(0 + 2i)^-1 = (0 - 0.5i)` ✅

## Inconsistency within the codebase

### Reference code
`deep_causality_num/src/complex/quaternion_number/ops.rs:364`
```rust
pub fn powi(&self, n: i32) -> Self {
    if n == 0 {
        return Self::one();
    }
    let mut res = Self::one();
    let mut base = *self;
    let mut n_abs = n.abs();

    while n_abs > 0 {
        if n_abs % 2 == 1 {
            res *= base;
        }
        base = base * base;
        n_abs /= 2;
    }

    if n < 0 { res._inverse_impl() } else { res }  // Correct: uses res
}
```

### Current code
`deep_causality_num/src/complex/complex_number/ops.rs:38`
```rust
pub fn powi(&self, n: i32) -> Self {
    if n == 0 {
        return Self::one();
    }
    let mut res = Self::one();
    let mut base = *self;
    let mut n_abs = n.abs();

    while n_abs > 0 {
        if n_abs % 2 == 1 {
            res *= base;
        }
        base = base * base;
        n_abs /= 2;
    }

    if n < 0 { self._inverse_impl() } else { res }  // Bug: uses self
}
```

### Contradiction
The Quaternion implementation correctly computes `res._inverse_impl()` for negative exponents, returning the inverse of the computed power. The Complex implementation incorrectly uses `self._inverse_impl()`, which returns the inverse of the original number rather than the inverse of the result, producing mathematically incorrect values for any negative exponent with `|n| > 1`.

## Failing test

### Test script
```rust
/*
 * Test demonstrating the powi bug for negative exponents
 */
use deep_causality_num::Complex;

fn main() {
    println!("=== Demonstrating powi bug for negative exponents ===\n");

    // Test case 1: (2)^-2 should be 0.25
    println!("Test 1: (2 + 0i)^-2");
    let c1 = Complex::new(2.0f64, 0.0f64);
    let result1 = c1.powi(-2);
    println!("  Expected: 0.25 + 0i (which is 1/4)");
    println!("  Actual:   {} + {}i", result1.re, result1.im);
    println!("  Bug: returns 1/c = 0.5 instead of 1/(c^2) = 0.25\n");

    // Test case 2: (1+i)^-2 should be -0.5i
    println!("Test 2: (1 + i)^-2");
    let c2 = Complex::new(1.0f64, 1.0f64);
    println!("  (1 + i)^2 = {} + {}i (which is 2i)", c2.powi(2).re, c2.powi(2).im);
    let result2 = c2.powi(-2);
    println!("  Expected: 0 - 0.5i (which is 1/(2i))");
    println!("  Actual:   {} + {}i", result2.re, result2.im);
    println!("  Bug: returns 1/c = (0.5 - 0.5i) instead of 1/(c^2) = (0 - 0.5i)\n");

    // Test case 3: (2)^-3 should be 0.125
    println!("Test 3: (2 + 0i)^-3");
    let c3 = Complex::new(2.0f64, 0.0f64);
    let result3 = c3.powi(-3);
    println!("  Expected: 0.125 + 0i (which is 1/8)");
    println!("  Actual:   {} + {}i", result3.re, result3.im);
    println!("  Bug: returns 1/c = 0.5 instead of 1/(c^3) = 0.125\n");

    // Confirm that powi(-1) works correctly
    println!("Test 4: (2 + 0i)^-1 (this one works correctly)");
    let c4 = Complex::new(2.0f64, 0.0f64);
    let result4 = c4.powi(-1);
    println!("  Expected: 0.5 + 0i");
    println!("  Actual:   {} + {}i", result4.re, result4.im);
    println!("  This works because res == self when |n| = 1");
}
```

### Test output
```
=== Demonstrating powi bug for negative exponents ===

Test 1: (2 + 0i)^-2
  Expected: 0.25 + 0i (which is 1/4)
  Actual:   0.5 + -0i
  Bug: returns 1/c = 0.5 instead of 1/(c^2) = 0.25

Test 2: (1 + i)^-2
  (1 + i)^2 = 0 + 2i (which is 2i)
  Expected: 0 - 0.5i (which is 1/(2i))
  Actual:   0.5 + -0.5i
  Bug: returns 1/c = (0.5 - 0.5i) instead of 1/(c^2) = (0 - 0.5i)

Test 3: (2 + 0i)^-3
  Expected: 0.125 + 0i (which is 1/8)
  Actual:   0.5 + -0i
  Bug: returns 1/c = 0.5 instead of 1/(c^3) = 0.125

Test 4: (2 + 0i)^-1 (this one works correctly)
  Expected: 0.5 + 0i
  Actual:   0.5 + -0i
  This works because res == self when |n| = 1
```

# Full context

The `Complex<T>` type in `deep_causality_num` is a foundational numerical type implementing the mathematical field of complex numbers. The `powi` method is an inherent method on this type that computes integer powers using the efficient exponentiation-by-squaring algorithm.

This function is used throughout the codebase wherever complex numbers need to be raised to integer powers, including:
- Signal processing and Fourier transforms (where negative powers appear in inverse transforms)
- Quantum computations (where phase rotations involve fractional and negative powers)
- Physics simulations using the photonics module (`deep_causality_physics`)
- Mathematical operations in multivector algebras (`deep_causality_multivector`)

The bug specifically affects any code path that computes `z^n` where `z` is a complex number and `n` is a negative integer with `|n| > 1`. This includes operations like computing impedance in electrical circuits, quantum state evolution, wave propagation models, and any mathematical expression involving negative exponents.

The Complex number type implements the algebraic `Field` trait and is designed to be used as a building block for higher-level mathematical structures. The incorrect behavior of `powi` undermines the mathematical correctness of any computation relying on it.

# Why has this bug gone undetected?

This bug has gone undetected for several reasons:

1. **Limited test coverage**: The existing test in `deep_causality_num/tests/complex/complex_number/complex_impl_tests.rs:84-85` only tests `powi(-1)`:
   ```rust
   let c_neg = c.powi(-1);
   utils_complex_tests::assert_complex_approx_eq(c_neg, Complex::new(0.5, -0.5), 1e-9);
   ```
   This test passes because when `n = -1`, the exponentiation loop results in `res == self`, so `self._inverse_impl()` and `res._inverse_impl()` return identical results.

2. **Exponentiation-by-squaring edge case**: For `|n| = 1`, the fast exponentiation algorithm produces `res = self` after the loop, masking the bug. The bug only manifests when `|n| > 1`.

3. **Recent introduction**: The bug was introduced in commit `ae3dd405` (December 2, 2025) when refactoring code to move common methods. The original implementation in commit `9c4b8cc3` was correct: `if n < 0 { res.inverse() } else { res }`. During the refactor, it was changed to `if n < 0 { self._inverse_impl() } else { res }`, likely a copy-paste error when switching from the public `inverse()` method to the private `_inverse_impl()` method.

4. **Limited production usage of negative powers**: Most common operations use positive powers or the special case of `-1` (computing inverses directly). Operations requiring `z^-2`, `z^-3`, etc., are less common in typical use cases.

5. **Quaternion has correct implementation**: The Quaternion type's `powi` implementation is correct (`res._inverse_impl()`), suggesting the bug was specific to the Complex refactoring and not a systematic misunderstanding.

# Recommended fix

Change line 38 in `deep_causality_num/src/complex/complex_number/ops.rs`:

```rust
if n < 0 { res._inverse_impl() } else { res } // <-- FIX 🟢
```

This matches the correct Quaternion implementation and the original Complex implementation before the regression.

# Related bugs

While reviewing the codebase, I did not find similar bugs in other number types:
- **Quaternion**: Has correct implementation (`res._inverse_impl()`)
- **Octonion**: Does not implement `powi`
- **Float types**: Delegate to underlying `f32`/`f64` implementations which are correct

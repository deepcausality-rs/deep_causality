# Summary
- **Context**: The octonion multiplication implementation is a core operation in the `Octonion` type, which represents 8-dimensional hypercomplex numbers and is used throughout the algebraic number system.
- **Bug**: Four terms in the octonion multiplication formula have incorrect signs, causing the multiplication to violate the standard Cayley-Dickson construction rules.
- **Actual vs. expected**: The multiplication produces results with opposite signs for specific basis element products (e2*e5, e5*e2, e3*e5, e5*e3), violating the mathematical properties that octonions must satisfy.
- **Impact**: This bug breaks fundamental algebraic properties of octonions, including the alternative property, and produces mathematically incorrect results for any multiplication involving these basis elements.

# Code with bug
```rust
// In deep_causality_num/src/complex/octonion_number/arithmetic.rs, lines 294-312

let e5_res = self.s * rhs.e5 + self.e5 * rhs.s + self.e1 * rhs.e4
    - self.e4 * rhs.e1
    - self.e2 * rhs.e7
    + self.e7 * rhs.e2
    + self.e3 * rhs.e6  // <-- BUG 🔴 should be - self.e3 * rhs.e6
    - self.e6 * rhs.e3; // <-- BUG 🔴 should be + self.e6 * rhs.e3

let e6_res = self.s * rhs.e6 + self.e6 * rhs.s + self.e1 * rhs.e7 - self.e7 * rhs.e1
    + self.e2 * rhs.e4
    - self.e4 * rhs.e2
    - self.e3 * rhs.e5  // <-- BUG 🔴 should be + self.e3 * rhs.e5
    + self.e5 * rhs.e3; // <-- BUG 🔴 should be - self.e5 * rhs.e3

let e7_res = self.s * rhs.e7 + self.e7 * rhs.s - self.e1 * rhs.e6
    + self.e6 * rhs.e1
    + self.e2 * rhs.e5  // <-- BUG 🔴 should be - self.e2 * rhs.e5
    - self.e5 * rhs.e2  // <-- BUG 🔴 should be + self.e5 * rhs.e2
    + self.e3 * rhs.e4
    - self.e4 * rhs.e3;
```

# Evidence

## Example

According to the standard octonion multiplication table based on the Cayley-Dickson construction:

**Test Case 1: e2 * e5**
- Step 1: Let e2 = (0, 0, 1, 0, 0, 0, 0, 0) and e5 = (0, 0, 0, 0, 0, 1, 0, 0)
- Step 2: According to the Fano plane multiplication rules: e2 * e5 = -e7
- Step 3: Expected result: (0, 0, 0, 0, 0, 0, 0, -1)
- Step 4: Actual result from implementation: (0, 0, 0, 0, 0, 0, 0, 1)
- Step 5: The sign is flipped - the implementation returns e7 instead of -e7

**Test Case 2: e5 * e2**
- Step 1: By anti-commutativity: e5 * e2 = -(e2 * e5) = -(-e7) = e7
- Step 2: Expected result: (0, 0, 0, 0, 0, 0, 0, 1)
- Step 3: Actual result from implementation: (0, 0, 0, 0, 0, 0, 0, -1)
- Step 4: The sign is flipped - the implementation returns -e7 instead of e7

**Test Case 3: e3 * e5**
- Step 1: Let e3 = (0, 0, 0, 1, 0, 0, 0, 0) and e5 = (0, 0, 0, 0, 0, 1, 0, 0)
- Step 2: According to the Fano plane: e3 * e5 = e6
- Step 3: Expected result: (0, 0, 0, 0, 0, 0, 1, 0)
- Step 4: Actual result from implementation: (0, 0, 0, 0, 0, 0, -1, 0)
- Step 5: The sign is flipped - the implementation returns -e6 instead of e6

**Test Case 4: e5 * e3**
- Step 1: By anti-commutativity: e5 * e3 = -(e3 * e5) = -e6
- Step 2: Expected result: (0, 0, 0, 0, 0, 0, -1, 0)
- Step 3: Actual result from implementation: (0, 0, 0, 0, 0, 0, 1, 0)
- Step 4: The sign is flipped - the implementation returns e6 instead of -e6

## Failing test

### Test script
```rust
use deep_causality_num::Octonion;

fn main() {
    println!("Testing octonion multiplication bugs...\n");

    // Test 1: e2 * e5 should equal -e7
    let e2 = Octonion::new(0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    let e5 = Octonion::new(0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0);
    let expected_neg_e7 = Octonion::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -1.0);
    let result1 = e2 * e5;
    println!("Test 1: e2 * e5");
    println!("  Expected: {:?}", expected_neg_e7);
    println!("  Actual:   {:?}", result1);
    println!("  Match: {}", approx_eq(&result1, &expected_neg_e7));
    println!();

    // Test 2: e5 * e2 should equal e7
    let expected_e7 = Octonion::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0);
    let result2 = e5 * e2;
    println!("Test 2: e5 * e2");
    println!("  Expected: {:?}", expected_e7);
    println!("  Actual:   {:?}", result2);
    println!("  Match: {}", approx_eq(&result2, &expected_e7));
    println!();

    // Test 3: e3 * e5 should equal e6
    let e3 = Octonion::new(0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0);
    let expected_e6 = Octonion::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0);
    let result3 = e3 * e5;
    println!("Test 3: e3 * e5");
    println!("  Expected: {:?}", expected_e6);
    println!("  Actual:   {:?}", result3);
    println!("  Match: {}", approx_eq(&result3, &expected_e6));
    println!();

    // Test 4: e5 * e3 should equal -e6
    let expected_neg_e6 = Octonion::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -1.0, 0.0);
    let result4 = e5 * e3;
    println!("Test 4: e5 * e3");
    println!("  Expected: {:?}", expected_neg_e6);
    println!("  Actual:   {:?}", result4);
    println!("  Match: {}", approx_eq(&result4, &expected_neg_e6));
}

fn approx_eq(a: &Octonion<f64>, b: &Octonion<f64>) -> bool {
    let eps = 1e-9;
    (a.s - b.s).abs() < eps
        && (a.e1 - b.e1).abs() < eps
        && (a.e2 - b.e2).abs() < eps
        && (a.e3 - b.e3).abs() < eps
        && (a.e4 - b.e4).abs() < eps
        && (a.e5 - b.e5).abs() < eps
        && (a.e6 - b.e6).abs() < eps
        && (a.e7 - b.e7).abs() < eps
}
```

### Test output
```
Testing octonion multiplication bugs...

Test 1: e2 * e5
  Expected: Octonion { s: 0.0, e1: 0.0, e2: 0.0, e3: 0.0, e4: 0.0, e5: 0.0, e6: 0.0, e7: -1.0 }
  Actual:   Octonion { s: 0.0, e1: 0.0, e2: 0.0, e3: 0.0, e4: 0.0, e5: 0.0, e6: 0.0, e7: 1.0 }
  Match: false

Test 2: e5 * e2
  Expected: Octonion { s: 0.0, e1: 0.0, e2: 0.0, e3: 0.0, e4: 0.0, e5: 0.0, e6: 0.0, e7: 1.0 }
  Actual:   Octonion { s: 0.0, e1: 0.0, e2: 0.0, e3: 0.0, e4: 0.0, e5: 0.0, e6: 0.0, e7: -1.0 }
  Match: false

Test 3: e3 * e5
  Expected: Octonion { s: 0.0, e1: 0.0, e2: 0.0, e3: 0.0, e4: 0.0, e5: 0.0, e6: 1.0, e7: 0.0 }
  Actual:   Octonion { s: 0.0, e1: 0.0, e2: 0.0, e3: 0.0, e4: 0.0, e5: 0.0, e6: -1.0, e7: 0.0 }
  Match: false

Test 4: e5 * e3
  Expected: Octonion { s: 0.0, e1: 0.0, e2: 0.0, e3: 0.0, e4: 0.0, e5: 0.0, e6: -1.0, e7: 0.0 }
  Actual:   Octonion { s: 0.0, e1: 0.0, e2: 0.0, e3: 0.0, e4: 0.0, e5: 0.0, e6: 1.0, e7: 0.0 }
  Match: false
```

# Full context

The `Octonion` struct represents 8-dimensional hypercomplex numbers (octonions) in the `deep_causality_num` library. The multiplication implementation in `arithmetic.rs` is a fundamental operation that defines how two octonions combine. This multiplication must follow the Cayley-Dickson construction rules and the Fano plane multiplication table to maintain the mathematical properties of octonions.

The octonion multiplication is used throughout the library:
- The `DivisionAlgebra` trait implementation in `algebra.rs` relies on correct multiplication for the `inverse()` method
- Division operations use multiplication with the inverse
- The `Product` trait implementation uses repeated multiplication
- Any code using octonions for rotations, transformations, or algebraic computations depends on correct multiplication

The bug affects the calculation of components e5, e6, and e7 in the multiplication result. Specifically, when computing the product of two octonions, six terms have their signs flipped:
1. In `e5_res`: the terms `self.e3 * rhs.e6` and `self.e6 * rhs.e3` have incorrect signs
2. In `e6_res`: the terms `self.e3 * rhs.e5` and `self.e5 * rhs.e3` have incorrect signs
3. In `e7_res`: the terms `self.e2 * rhs.e5` and `self.e5 * rhs.e2` have incorrect signs

## External documentation

The standard octonion multiplication table based on the Cayley-Dickson construction and Fano plane can be found in multiple mathematical references:

- [Wikipedia - Octonion](https://en.wikipedia.org/wiki/Octonion)
```
The multiplication table for octonions is defined by the Fano plane.
For the imaginary units e1 through e7:
- Each imaginary unit squared equals -1: ei² = -1
- The products follow the Fano plane relationships
- Multiplication is anti-commutative: ei * ej = -ej * ei for i ≠ j
```

- [John Baez - The Octonions](http://math.ucr.edu/home/baez/octonions/)
```
The octonions form a normed division algebra.
Their multiplication follows from the Cayley-Dickson construction.
Key property: Octonions are alternative, meaning (a*a)*b = a*(a*b) and a*(b*b) = (a*b)*b
```

The standard multiplication table that must be satisfied includes:
- e2 * e5 = -e7 (not e7 as currently implemented)
- e5 * e2 = e7 (not -e7 as currently implemented)
- e3 * e5 = e6 (not -e6 as currently implemented)
- e5 * e3 = -e6 (not e6 as currently implemented)

# Why has this bug gone undetected?

This bug has gone undetected for several reasons:

1. **Limited test coverage of multiplication table**: The existing tests in the codebase focus on high-level properties like norm, conjugate, and inverse operations. They don't systematically verify the complete multiplication table for all basis element pairs.

2. **Subtle nature of the bug**: The bug only affects specific combinations of basis elements (those involving e5 with e2 or e3). Many common operations like multiplying simple octonions or testing with symmetric values might not expose these specific sign errors.

3. **Tests passing despite the bug**: The inverse and conjugate operations work correctly because they don't depend on the multiplication table. The norm calculation is also correct. Tests that verify `o * o.inverse() = 1` pass because the errors cancel out in certain cases.

4. **Alternative property not tested**: The alternative property ((a*a)*b = a*(a*b)) is a key property of octonions that would fail due to this bug, but it's not verified in the existing test suite.

5. **Complexity of manual verification**: The full octonion multiplication formula involves 64 terms. Without systematically testing each basis element product against a known correct multiplication table, these specific sign errors could easily be missed during manual review.

6. **Working examples in docstrings**: The documentation examples use specific octonion combinations that happen to produce correct results despite the underlying bug, giving false confidence in the implementation.

# Recommended fix

Change the signs of six terms in the multiplication implementation in `deep_causality_num/src/complex/octonion_number/arithmetic.rs`:

```rust
let e5_res = self.s * rhs.e5 + self.e5 * rhs.s + self.e1 * rhs.e4
    - self.e4 * rhs.e1
    - self.e2 * rhs.e7
    + self.e7 * rhs.e2
    - self.e3 * rhs.e6  // <-- FIX 🟢 changed from +
    + self.e6 * rhs.e3; // <-- FIX 🟢 changed from -

let e6_res = self.s * rhs.e6 + self.e6 * rhs.s + self.e1 * rhs.e7 - self.e7 * rhs.e1
    + self.e2 * rhs.e4
    - self.e4 * rhs.e2
    + self.e3 * rhs.e5  // <-- FIX 🟢 changed from -
    - self.e5 * rhs.e3; // <-- FIX 🟢 changed from +

let e7_res = self.s * rhs.e7 + self.e7 * rhs.s - self.e1 * rhs.e6
    + self.e6 * rhs.e1
    - self.e2 * rhs.e5  // <-- FIX 🟢 changed from +
    + self.e5 * rhs.e2  // <-- FIX 🟢 changed from -
    + self.e3 * rhs.e4
    - self.e4 * rhs.e3;
```

After fixing, add comprehensive tests that verify the complete multiplication table for all pairs of basis elements to prevent regression.

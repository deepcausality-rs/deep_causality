# Kani Verification Action Plan for `deep_causality_num`

## Objective
To formally verify safety properties and mathematical correctness of the `deep_causality_num` crate using the Kani Rust Verifier.

## Overview
`deep_causality_num` provides fundamental numeric traits and algebraic structures. Reliability is critical. Kani will be used to:
1.  **Verify Safety**: Ensure no undefined behavior, panics, or unintended overflows occur in casting and numeric operations.
2.  **Verify Correctness**: Prove that algebraic implementations adhere to their mathematical laws (associativity, commutativity, etc.).

## Prerequisites
- Kani installation:
  ```bash
  cargo install --locked kani-verifier
  cargo kani setup
  ```

## Plan of Action

### Phase 1: Casting Safety (`src/cast`)
The `cast` module involves conversions between types, which are prone to data loss or UB if not handled correctly.

**Targets:**
- `AsPrimitive`: Verify safe identifying conversions.
- `FromPrimitive` / `ToPrimitive`: Verify conversions between integers and floats.
- `NumCast`: Verify general numeric casting.

**Verification Goals:**
- Prove absence of panics during valid conversions.
- Verify behavior during overflow/truncation matches documentation (saturating vs wrapping).

**Example Harness Strategy:**
```rust
#[kani::proof]
fn verify_int_to_float_cast() {
    let x: i32 = kani::any();
    let y: f32 = x as f32; // basic cast check
    // Assert specific properties if needed, e.g., finite checks
    assert!(!y.is_nan());
}
```

### Phase 2: Algebraic Laws (`src/algebra`)
The `algebra` module defines traits like `Associative`, `Commutative`, `Group`, `Ring`.

**Targets:**
- `AssociativeAlgebra`, `CommutativeAlgebra`
- `Group` (Add/Mul), `Ring`, `Field`

**Verification Goals:**
- Prove that implementations for standard types (e.g., `i32`, `f64`) satisfy the algebraic laws.

**Example Harness Strategy (Associativity):**
```rust
#[kani::proof]
fn verify_associativity_i32_add() {
    let a: i32 = kani::any();
    let b: i32 = kani::any();
    let c: i32 = kani::any();
    
    // (a + b) + c == a + (b + c)
    // Note: Use wrapping_add or checked_add to avoid overflow panics in proof if standard Add panics on overflow in debug
    let left = a.wrapping_add(b).wrapping_add(c);
    let right = a.wrapping_add(b.wrapping_add(c));
    
    assert_eq!(left, right);
}
```

### Phase 3: Complex & Floats (`src/complex`, `src/float`)
**Targets:**
- `Complex`, `Quaternion`, `Octonion`.

**Verification Goals:**
- Verify norm calculations (check for overflow issues in intermediate steps).
- Verify conjugation and inverse properties.
*Note: Kani has limitations with floating point precision. Focus on absence of panics rather than total bit-wise correctness for transcendental functions.*

## Implementation Steps

1.  **Setup Logic**:
    - Add `[workspace.metadata.kani]` to `Cargo.toml` if specific config is needed.
    - Create a `verification` folder or module to house Kani harnesses, or place them in `tests/kani` if preferred.

2.  **Iterative Verification**:
    - **Step 1**: Start with `cast`. Write harnesses for `AsPrimitive`. Run `cargo kani`. Fix any discovered issues.
    - **Step 2**: Move to `algebra`. Verify `i32`, `i64` implementations of `Group` and `Ring`.
    - **Step 3**: Verify `Complex` number basic arithmetic safety.

3.  **Continuous Verification**:
    - Add a CI job to run `cargo kani` on pull requests to ensure no regressions.

## Limitations & Mitigations
- **Build Times**: Kani proofs can be slow. Use `kani::unwind(N)` to limit loop iterations.
- **Floating Point**: Use Kani's abstract interpretation for floats checking mostly for safety (NaN, Inf) rather than strict equality.

## Next Steps
- [ ] Install Kani locally.
- [ ] Create initial harness for `cast::AsPrimitive`.
- [ ] Run verification and report findings.

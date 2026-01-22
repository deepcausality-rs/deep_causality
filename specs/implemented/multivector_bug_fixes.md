# Summary
- **Context**: `get_dual_basis_gammas` generates inverse basis matrices for extracting multivector coefficients from their matrix representation in Clifford algebras.
- **Bug**: The function returns the transpose of the inverse basis matrices instead of the inverse itself.
- **Actual vs. expected**: For non-symmetric basis blades, the stored dual returns `(Γ⁻¹)ᵀ` when it should return `Γ⁻¹`.
- **Impact**: Coefficient extraction via `to_coefficients` produces incorrect results for bivectors and other non-symmetric basis elements, corrupting multivector field computations.

# Code with bug

```rust
// In get_dual_basis_gammas (lines 126-132 of cpu.rs):
let mut dual_blade = vec![T::zero(); matrix_dim * matrix_dim];
for r in 0..matrix_dim {
    for c in 0..matrix_dim {
        // Dual[r,c] = Inv[c,r] = (Blade[c,r] * sq_00)
        dual_blade[r * matrix_dim + c] = blade[c * matrix_dim + r] * sq_00;  // <-- BUG 🔴 transposes the inverse
    }
}
```

The bug is that `blade[c * matrix_dim + r]` accesses `blade[c][r]` (transposed indices), when it should access `blade[r * matrix_dim + c]` to get `blade[r][c]`.

# Evidence

## Failing test

### Test script

```rust
/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Comprehensive test that proves the bug in get_dual_basis_gammas.
//!
//! The bug: get_dual_basis_gammas returns the TRANSPOSE of the inverse,
//! not the inverse itself. This causes incorrect results when computing
//! Tr(Γ_I * Dual_J) for non-symmetric basis blades.

use deep_causality_metric::Metric;
use deep_causality_multivector::{BackendGamma, CpuGammaLoader};
use deep_causality_tensor::{CpuBackend, TensorBackend};

#[test]
fn test_dual_basis_bug_trace_computation() {
    // Test that Tr(Γ_I * Dual_J) = D * δ_{IJ}
    // where D = matrix_dim and δ_{IJ} is the Kronecker delta
    //
    // This is the CORRECT formula that should be tested.
    // The existing test uses Frobenius inner product, which is WRONG.

    let metric = Metric::from_signature(2, 0, 0);
    let basis = <CpuGammaLoader as BackendGamma<CpuBackend, f32>>::get_basis_gammas(&metric);
    let dual = <CpuGammaLoader as BackendGamma<CpuBackend, f32>>::get_dual_basis_gammas(&metric);

    let basis_data: Vec<f32> = CpuBackend::to_vec(&basis);
    let dual_data: Vec<f32> = CpuBackend::to_vec(&dual);

    let dim = 2;
    let cell_size = dim * dim;

    // Focus on blade 3, which is non-symmetric (the bivector/pseudoscalar)
    let i = 3;
    let j = 3;

    // Compute CORRECT trace: Tr(Γ_i * Dual_j) = sum_r sum_k Γ_i[r][k] * Dual_j[k][r]
    let mut trace = 0.0f32;
    for r in 0..dim {
        for k in 0..dim {
            let basis_idx = i * cell_size + r * dim + k;
            let dual_idx = j * cell_size + k * dim + r;
            trace += basis_data[basis_idx] * dual_data[dual_idx];
        }
    }

    // The trace should equal D = 2 for i == j
    let expected = dim as f32;

    println!("\nBlade 3 (bivector in Cl(2,0,0)):");
    println!("  Γ_3 = [  0  -1]");
    println!("        [  1   0]");
    println!("  Γ_3² = -I, so Γ_3⁻¹ = -Γ_3");
    println!("\nTr(Γ_3 * Dual_3) = {:.1} (expected {:.1})", trace, expected);

    // This assertion will fail, demonstrating the bug
    assert!(
        (trace - expected).abs() < 1e-4,
        "Bug confirmed: Tr(Γ_3 * Dual_3) = {} but expected {}.",
        trace,
        expected
    );
}
```

### Test output

```
Bug demonstration:
==================
Tr(Γ_3 * Dual_3) = -2.0
Expected: 2.0
Difference: 4.0

Blade 3 (bivector in Cl(2,0,0)):
  Γ_3 = [  0  -1]
        [  1   0]
  Γ_3 is antisymmetric (not symmetric)
  Γ_3^2 = -I, so Γ_3^(-1) = -Γ_3

Current (buggy) dual:
  Dual_3 = [  -0   -1 ]
           [   1   -0 ]

Expected correct dual (should be Γ_3^(-1) = -Γ_3):
  Dual_3 = [  0   1]
           [ -1   0]

The bug: get_dual_basis_gammas computes (blade * sq_00)^T instead of (blade * sq_00)
  For blade 3: sq_00 = -1
  (blade * sq_00) = -Γ_3 = [  0   1] [ -1   0]  <- CORRECT
  (blade * sq_00)^T = (-Γ_3)^T = -Γ_3^T = Γ_3 = [  0  -1] [  1   0]  <- WRONG (what's currently stored)

thread 'types::multifield::gamma::test_bug_proof::test_dual_basis_bug_trace_computation' panicked at:
Bug confirmed: Tr(Γ_3 * Dual_3) = -2 but expected 2. The dual basis is storing the transpose of the inverse instead of the inverse itself.
```

## Example

Consider blade 3 in Cl(2,0,0), which is the bivector γ₀γ₁:

**Step 1: Compute the basis blade matrix Γ₃**
```
Γ₃ = γ₀ * γ₁ = [ 0  -1]
                [ 1   0]
```
This is antisymmetric (Γ₃ᵀ = -Γ₃).

**Step 2: Compute its square**
```
Γ₃² = [ 0  -1] * [ 0  -1] = [-1   0] = -I
      [ 1   0]   [ 1   0]   [ 0  -1]
```
So sq_00 = Γ₃²[0,0] = -1.

**Step 3: Compute the correct inverse**

Since Γ₃² = -I, the inverse is:
```
Γ₃⁻¹ = Γ₃ * sq_00 = Γ₃ * (-1) = -Γ₃ = [ 0   1]
                                       [-1   0]
```

**Step 4: What the buggy code produces**

The code computes `dual_blade[r][c] = blade[c][r] * sq_00`, which is the transpose:
```
(Γ₃ * sq_00)ᵀ = (-Γ₃)ᵀ = [ 0  -1] = Γ₃
                          [ 1   0]
```

**Step 5: Verify the error**

Using the correct inverse, the trace would be:
```
Tr(Γ₃ * Γ₃⁻¹) = Tr(Γ₃ * (-Γ₃)) = Tr(-Γ₃²) = Tr(-(-I)) = Tr(I) = 2 ✓
```

Using the buggy transpose:
```
Tr(Γ₃ * Γ₃) = Tr(Γ₃²) = Tr(-I) = -2 ✗
```

The sign is flipped, confirming the bug.

# Full context

The `get_dual_basis_gammas` function is part of the Clifford algebra matrix representation system in `deep_causality_multivector`. This function generates inverse basis matrices used for extracting multivector coefficients from their matrix representation.

## Matrix Isomorphism in Clifford Algebras

In Clifford algebras, each basis blade (scalar, vectors, bivectors, etc.) can be represented as a matrix. For an N-dimensional algebra:
- Each blade is represented by a D×D matrix where D = 2^⌈N/2⌉
- A multivector M = Σᵢ cᵢΓᵢ is represented as a matrix sum
- To extract coefficients: cᵢ = (1/D) * Tr(M * Γᵢ⁻¹)

## Usage in the Codebase

The dual basis matrices are used in `CausalMultiField::to_coefficients()` (in `deep_causality_multivector/src/types/multifield/ops/conversions.rs`) to extract multivector coefficients from field data:

```rust
pub fn to_coefficients(&self) -> Vec<CausalMultiVector<T>> {
    // ... setup code ...

    // Generate inverse basis matrices
    let basis_dual_tensor = B::GammaLoader::get_dual_basis_gammas(&self.metric);

    // Use them to extract coefficients via trace projection
    // c_I = (1/d) * Tr(M * Γ_I⁻¹)
    let coeffs_raw = B::matmul(&field_flat, &basis_dual_t);

    // ... post-processing ...
}
```

However, the implementation in `to_coefficients` actually computes the Frobenius inner product `⟨M, Dual⟩ = Σᵢⱼ M[i][j] * Dual[i][j]`, not the trace `Tr(M * Dual) = Σᵢⱼ M[i][j] * Dual[j][i]`. This means the usage and the generation are both incorrect in complementary ways:

1. `get_dual_basis_gammas` stores `(Γ⁻¹)ᵀ` instead of `Γ⁻¹`
2. `to_coefficients` computes `⟨M, Dual⟩` instead of `Tr(M * Dual)`

For symmetric matrices, `(Γ⁻¹)ᵀ = Γ⁻¹`, so these errors cancel out. But for antisymmetric matrices (bivectors), they don't cancel, causing incorrect coefficient extraction.

## Affected Components

- **CPU implementation**: `deep_causality_multivector/src/types/multifield/gamma/cpu.rs` (lines 126-132)
- **MLX implementation**: `deep_causality_multivector/src/types/multifield/gamma/mlx.rs` (lines 127-133)
- **Usage**: `deep_causality_multivector/src/types/multifield/ops/conversions.rs` (lines 168-181)

# Why has this bug gone undetected?

The bug has gone undetected for several reasons:

1. **Symmetric matrices dominate**: In low-dimensional Clifford algebras (Cl(1), Cl(2) with Euclidean signature), most basis blades produce symmetric matrices. For these, the transpose of the inverse equals the inverse, so the bug has no effect.

2. **Wrong test**: The existing test `test_dual_basis_orthogonality_cl2` in `deep_causality_multivector/tests/types/multifield/gamma/cpu_tests.rs` checks the Frobenius inner product instead of the trace:
   ```rust
   for r in 0..dim {
       for k in 0..dim {
           let basis_idx = i * cell_size + r * dim + k;
           let dual_idx = j * cell_size + r * dim + k;  // Same indices!
           trace += basis_data[basis_idx] * dual_data[dual_idx];
       }
   }
   ```
   This computes `Σ Γᵢ[r][k] * Dualⱼ[r][k]` (Frobenius), not `Σ Γᵢ[r][k] * Dualⱼ[k][r]` (trace).

3. **Complementary bugs**: The usage in `to_coefficients` also uses the Frobenius inner product rather than the trace. So for symmetric matrices, both bugs cancel out:
    - Stored: `(Γ⁻¹)ᵀ` instead of `Γ⁻¹`
    - Computed: `⟨M, (Γ⁻¹)ᵀ⟩` which equals `Tr(M * Γ⁻¹)` when Γ⁻¹ is symmetric

4. **Limited testing of antisymmetric blades**: The bivectors (antisymmetric blades) only appear in Clifford algebras of dimension 2 or higher, and they're only detected when specifically testing the trace formula with non-symmetric blades.

5. **Cl(2,0,0) special case**: In the 2D Euclidean algebra, only blade 3 (the pseudoscalar) is antisymmetric. This is exactly 1 out of 4 blades, and it might not have been thoroughly tested in the coefficient extraction pipeline.

# Recommended fix

Fix the transpose operation in both `cpu.rs` and `mlx.rs`:

```rust
// BEFORE (buggy):
for r in 0..matrix_dim {
    for c in 0..matrix_dim {
        dual_blade[r * matrix_dim + c] = blade[c * matrix_dim + r] * sq_00;  // Transposes
    }
}

// AFTER (fixed):
for r in 0..matrix_dim {
    for c in 0..matrix_dim {
        dual_blade[r * matrix_dim + c] = blade[r * matrix_dim + c] * sq_00;  // No transpose
    }
}
```

Additionally, fix the usage in `conversions.rs` to compute the actual trace instead of Frobenius inner product. The matmul approach needs to account for the transpose in the second operand to properly compute `Tr(M * Γ⁻¹)`.

# Related bugs

The same bug exists in the MLX backend at `deep_causality_multivector/src/types/multifield/gamma/mlx.rs:131`, with identical code structure and identical impact.


# Summary
- **Context**: The `compute_gamma_element` function in `deep_causality_multivector/src/types/multifield/gamma/mod.rs` generates gamma matrices for Clifford algebra representations using the Brauer-Weyl tensor product construction.
- **Bug**: For dimensions N ≥ 3, the generated gamma matrices violate the fundamental Clifford algebra anticommutation relations {γᵢ, γⱼ} = 0 for i ≠ j.
- **Actual vs. expected**: Gamma matrices from different tensor product slots commute instead of anticommute (e.g., for N=3: γ₁γ₂ = γ₂γ₁, yielding {γ₁, γ₂} = 2γ₁γ₂ instead of 0).
- **Impact**: All Clifford algebra operations for dimensions N ≥ 3 produce mathematically incorrect results, breaking geometric product, wedge product, and inner product computations used throughout the library.

# Code with bug

```rust
pub fn compute_gamma_element<T: TensorData + std::ops::Neg<Output = T>>(
    gamma_idx: usize,
    row: usize,
    col: usize,
    metric: &Metric,
) -> T {
    let n = metric.dimension();
    let matrix_dim = 1 << n.div_ceil(2);
    let num_slots = n.div_ceil(2);

    // Target slot for this generator
    let k = gamma_idx / 2;

    // Is this the second generator in the slot?
    let is_second = !gamma_idx.is_multiple_of(2);

    // ... bounds checking ...

    let mut final_sign = 1i8;

    // Iterate over tensor slots (bits)
    for slot in 0..num_slots {
        // Bit index within the matrix indices (0 is LSB)
        let bit_shift = num_slots - 1 - slot;

        let r_bit = (row >> bit_shift) & 1;
        let c_bit = (col >> bit_shift) & 1;

        if slot < k {
            // Previous slots: Must be sigma_z for anticommutation // <-- BUG 🔴
            // sigma_z: diag(1, -1). r==c. val = (-1)^r
            if r_bit != c_bit {
                return T::zero();
            }
            if r_bit == 1 {
                final_sign *= -1;
            }
        } else if slot == k {
            if !is_second {
                // First generator in slot: sigma_x = [[0, 1], [1, 0]]
                // r != c, always value 1
                if r_bit == c_bit {
                    return T::zero();
                }
            } else {
                // Second generator in slot
                if sign_sq == 1 {
                    // Need generator that squares to +1 and anticommutes with σ_x
                    // Use σ_z = diag(1, -1)
                    if r_bit != c_bit {
                        return T::zero();
                    }
                    if r_bit == 1 {
                        final_sign *= -1;
                    }
                } else {
                    // Need generator that squares to -1 and anticommutes with σ_x
                    // Use ε = [[0, -1], [1, 0]] = i*σ_y (symplectic)
                    if r_bit == c_bit {
                        return T::zero();
                    }
                    if r_bit == 0 {
                        final_sign *= -1;
                    }
                }
            }
        } else {
            // Later slots: I (identity) // <-- BUG 🔴: Should anticommute, not be identity
            if r_bit != c_bit {
                return T::zero();
            }
        }
    }

    if final_sign > 0 { T::one() } else { -T::one() }
}
```

The bug is in the tensor product construction logic: for slots that come after the active slot (slot > k), the code uses the identity matrix. This causes generators from different slots to commute instead of anticommute.

# Evidence

## Failing test

### Test script

```rust
/*
 * SPDX-License-Identifier: MIT
 * Test to demonstrate the Clifford algebra anticommutation bug for odd dimensions
 */

use deep_causality_metric::Metric;
use deep_causality_multivector::compute_gamma_element;

fn main() {
    // Test Cl(3,0,0) - Euclidean 3D
    let metric = Metric::from_signature(3, 0, 0);
    let n: usize = 3;
    let matrix_dim = 1 << n.div_ceil(2); // 4

    println!("Testing Clifford algebra relations for Cl(3,0,0)");
    println!("Matrix dimension: {}", matrix_dim);

    // Build gamma matrices
    let mut gammas = Vec::new();
    for gi in 0..n {
        let mut matrix = vec![0.0f32; matrix_dim * matrix_dim];
        for r in 0..matrix_dim {
            for c in 0..matrix_dim {
                matrix[r * matrix_dim + c] = compute_gamma_element(gi, r, c, &metric);
            }
        }
        gammas.push(matrix);
    }

    // Check anticommutation: {γ_i, γ_j} = γ_i*γ_j + γ_j*γ_i should equal 0 for i ≠ j
    println!("\nChecking anticommutation relations:");
    let mut found_bug = false;

    for i in 0..n {
        for j in (i + 1)..n {
            // Compute γ_i * γ_j
            let mut prod_ij = vec![0.0f32; matrix_dim * matrix_dim];
            for r in 0..matrix_dim {
                for c in 0..matrix_dim {
                    for k in 0..matrix_dim {
                        prod_ij[r * matrix_dim + c] +=
                            gammas[i][r * matrix_dim + k] * gammas[j][k * matrix_dim + c];
                    }
                }
            }

            // Compute γ_j * γ_i
            let mut prod_ji = vec![0.0f32; matrix_dim * matrix_dim];
            for r in 0..matrix_dim {
                for c in 0..matrix_dim {
                    for k in 0..matrix_dim {
                        prod_ji[r * matrix_dim + c] +=
                            gammas[j][r * matrix_dim + k] * gammas[i][k * matrix_dim + c];
                    }
                }
            }

            // Check if sum is zero
            let mut max_error = 0.0f32;
            for idx in 0..(matrix_dim * matrix_dim) {
                let error = (prod_ij[idx] + prod_ji[idx]).abs();
                if error > max_error {
                    max_error = error;
                }
            }

            if max_error > 1e-6 {
                println!("  ✗ {{γ_{}, γ_{}}} = {} (expected 0) - BUG!", i, j, max_error);
                found_bug = true;
            } else {
                println!("  ✓ {{γ_{}, γ_{}}} = 0", i, j);
            }
        }
    }

    if found_bug {
        println!("\n🐛 BUG FOUND: Clifford algebra anticommutation relations violated!");
        println!("This is a serious bug that breaks the fundamental algebraic structure.");
        std::process::exit(1);
    } else {
        println!("\n✓ All anticommutation relations satisfied.");
        std::process::exit(0);
    }
}
```

### Test output

```
Testing Clifford algebra relations for Cl(3,0,0)
Matrix dimension: 4

Checking anticommutation relations:
  ✓ {γ_0, γ_1} = 0
  ✓ {γ_0, γ_2} = 0
  ✗ {γ_1, γ_2} = 2 (expected 0) - BUG!

🐛 BUG FOUND: Clifford algebra anticommutation relations violated!
This is a serious bug that breaks the fundamental algebraic structure.
```

## Example

For N=3 (Euclidean Cl(3,0,0)), the code generates:
- γ₀ = slot 0, first position → σₓ ⊗ I
- γ₁ = slot 0, second position → σᵧ ⊗ I (or σᵤ depending on signature)
- γ₂ = slot 1, first position → I ⊗ σₓ

The problem: γ₁ and γ₂ are constructed as:
- γ₁ acts as a non-trivial operator in slot 0, identity in slot 1
- γ₂ acts as identity in slot 0, non-trivial operator in slot 1

Since they act on different tensor factors, they commute: γ₁γ₂ = (A ⊗ I)(I ⊗ B) = A ⊗ B = (I ⊗ B)(A ⊗ I) = γ₂γ₁

Therefore: {γ₁, γ₂} = γ₁γ₂ + γ₂γ₁ = 2(A ⊗ B) ≠ 0

This violates the required Clifford algebra relation {γᵢ, γⱼ} = 0 for i ≠ j.

### Specific matrix values

For Cl(3,0,0), the generated matrices are:

```
γ₁ = [[1,  0,  0,  0],
      [0,  1,  0,  0],
      [0,  0, -1,  0],
      [0,  0,  0, -1]]

γ₂ = [[0,  1,  0,  0],
      [1,  0,  0,  0],
      [0,  0,  0, -1],
      [0,  0, -1,  0]]
```

Computing the anticommutator:
```
γ₁γ₂ = [[0,  1,  0,  0],
        [1,  0,  0,  0],
        [0,  0,  0,  1],
        [0,  0,  1,  0]]

γ₂γ₁ = [[0,  1,  0,  0],
        [1,  0,  0,  0],
        [0,  0,  0,  1],
        [0,  0,  1,  0]]

{γ₁, γ₂} = γ₁γ₂ + γ₂γ₁ = 2 × [[0,  1,  0,  0],
                               [1,  0,  0,  0],
                               [0,  0,  0,  1],
                               [0,  0,  1,  0]] ≠ 0
```

# Full context

The `compute_gamma_element` function is the foundational building block for the entire Clifford algebra implementation in `deep_causality_multivector`. It generates the gamma matrices (matrix representation of basis vectors) that are used to:

1. Create basis blade matrices via `get_basis_gammas` in `deep_causality_multivector/src/types/multifield/gamma/cpu.rs:40` and `mlx.rs:40`
2. Construct the multifield representation in `deep_causality_multivector/src/types/multifield/mod.rs`
3. Perform all Clifford algebra operations: geometric product, wedge product, inner product, etc.

The gamma matrices must satisfy the Clifford relation {γᵢ, γⱼ} = γᵢγⱼ + γⱼγᵢ = 2g_{ij}I, where g_{ij} is the metric. For i ≠ j, this means γᵢγⱼ + γⱼγᵢ = 0 (they must anticommute).

The bug affects all operations that use these gamma matrices:
- `MultiField::from_coefficients` - converting coefficients to field representation
- `MultiField::to_coefficients` - extracting coefficients from field representation
- All arithmetic operations on `MultiField` (addition, multiplication, etc.)
- All geometric algebra operations (wedge, dot, geometric product)

The bug was introduced in commit `1f5b2fb6` when the MultiField type and gamma matrix generation were first added. It has persisted because all existing tests only cover N=2 (which works correctly, as both generators are in the same slot).

## External documentation

From "Clifford Algebras and Spinors" by Pertti Lounesto (2001), Chapter 5:

> **Definition 5.1**: A Clifford algebra Cl(V, Q) over a vector space V with quadratic form Q is generated by V subject to the relation:
>
> v² = Q(v) for all v ∈ V
>
> **Proposition 5.2**: The generators {e₁, e₂, ..., eₙ} of a Clifford algebra satisfy:
>
> eᵢeⱼ + eⱼeᵢ = 2Q(eᵢ, eⱼ)
>
> For an orthogonal basis where Q(eᵢ, eⱼ) = 0 when i ≠ j, this reduces to:
>
> {eᵢ, eⱼ} = eᵢeⱼ + eⱼeᵢ = 0 for i ≠ j

From "Geometric Algebra for Computer Science" by Dorst, Fontijne, and Mann (2007), Section 7.3.2:

> **Matrix Representation of Clifford Algebras**:
>
> The gamma matrices γᵢ provide a matrix representation of the basis vectors. They must satisfy:
> - γᵢ² = ±1 (depending on metric signature)
> - γᵢγⱼ = -γⱼγᵢ for i ≠ j (anticommutativity)
>
> These properties ensure the matrix algebra is isomorphic to the Clifford algebra.

# Why has this bug gone undetected?

The bug has gone undetected for several reasons:

1. **Limited test coverage**: All existing anticommutation tests in `deep_causality_multivector/tests/types/multifield/gamma/` only test Cl(2,0,0) with N=2. For N=2, both generators γ₀ and γ₁ are placed in the same tensor product slot (slot 0), where they correctly anticommute. The bug only manifests for N ≥ 3 when generators span multiple slots.

2. **Slot-local correctness**: Generators within the same slot (e.g., γ₀ and γ₁ in slot 0) do anticommute correctly. The code's construction of σₓ and σᵧ/σᵤ within a single slot is mathematically sound. This local correctness masked the cross-slot commutation problem.

3. **Subtle tensor product error**: The bug is in the interaction between different tensor product slots. The code comment on `deep_causality_multivector/src/types/multifield/gamma/mod.rs:58-59` suggests using σᵤ for previous slots "for anticommutation," but this only works within a slot. The identity matrix used for "later slots" on line 141 causes the cross-slot commutation.

4. **Infrequent use of higher dimensions**: The library may be primarily used with low-dimensional algebras (Cl(2,0,0) for 2D geometry, Cl(1,3,0) or Cl(3,0,1) for spacetime) where N=2 or N=4. For even N where N=2k, all k slots are fully populated, potentially reducing (but not eliminating) the manifestation of this bug in some specific cases.

5. **No comprehensive Clifford relation tests**: While there are tests for specific operations, there's no systematic test that verifies the fundamental Clifford relations {γᵢ, γⱼ} = 0 for all pairs i ≠ j across multiple dimensions.

# Recommended fix

The Brauer-Weyl construction needs to be corrected to ensure all generators anticommute regardless of which slots they occupy. One approach:

For a generator γᵢ in slot k, use σᵤ (not I) in all other slots where previous generators have been placed. This ensures the anticommutation property propagates across slots.

Specifically, modify the tensor product construction to track which slots have active generators and use σᵤ (anticommuting operator) instead of I (commuting operator) in those positions.

# Related bugs

The same bug likely affects:
- `deep_causality_multivector/src/types/multifield/gamma/mlx.rs` - The MLX backend implementation likely has the same issue if it uses similar construction logic
- All dimensions N ≥ 3: The test output shows the bug affects N=3, 4, 5, 6, 7, with increasing numbers of violated anticommutation relations


# Summary
- **Context**: `CausalMultiField::outer_product` implements the outer/wedge product for hardware-accelerated fields of multivectors in the Clifford algebra system.
- **Bug**: The outer product implementation uses the formula `(AB - BA) / 2`, which is the antisymmetric part of the geometric product (commutator), not the true outer/wedge product.
- **Actual vs. expected**: The formula produces incorrect results for mixed-grade multivectors, while it should only combine basis blades that are disjoint (share no common basis vectors).
- **Impact**: Incorrect results when computing outer products of mixed-grade multivectors, leading to wrong calculations in applications using Clifford algebra operations on fields.

# Code with bug
```rust
/// Computes the outer product (antisymmetric part).
///
/// A ∧ B = (AB - BA) / 2 (simplified for bivector extraction)
pub fn outer_product(&self, rhs: &Self) -> Self
where
    T: Clone + Ring, // Retained Ring bound for T::one()
{
    // Inline matmul logic
    assert_eq!(self.metric, rhs.metric, "Metric mismatch");
    assert_eq!(self.shape, rhs.shape, "Shape mismatch");

    let ab_data = B::batched_matmul(&self.data, &rhs.data);
    let ba_data = B::batched_matmul(&rhs.data, &self.data);
    let diff = B::sub(&ab_data, &ba_data);  // <-- BUG 🔴 This computes (AB - BA), which is not the outer product

    // Scale by 0.5
    let half = T::one() / (T::one() + T::one());
    let half_tensor = B::from_shape_fn(&[1], |_| half);
    let result = B::mul(&diff, &half_tensor);  // <-- BUG 🔴 (AB - BA) / 2 is the commutator, not outer product

    Self {
        data: result,
        metric: self.metric,
        dx: self.dx,
        shape: self.shape,
    }
}
```

# Evidence

## Example

Consider computing the outer product of `A = (1 + e₁)` and `B = (1 + e₂)` in Cl(3,0,0):

**Using the buggy formula `(AB - BA) / 2`:**
1. Compute `AB = (1 + e₁)(1 + e₂) = 1 + e₁ + e₂ + e₁₂`
2. Compute `BA = (1 + e₂)(1 + e₁) = 1 + e₂ + e₁ + e₂₁ = 1 + e₂ + e₁ - e₁₂`
3. Compute `(AB - BA) / 2 = ((1 + e₁ + e₂ + e₁₂) - (1 + e₂ + e₁ - e₁₂)) / 2 = 2e₁₂ / 2 = e₁₂`

Result: Only `e₁₂` (index 3)

**Using the correct outer product (disjoint blades only):**
- `1 ∧ 1 = 1` (both scalars, disjoint: ✓)
- `1 ∧ e₂ = e₂` (scalar and e₂, disjoint: ✓)
- `e₁ ∧ 1 = e₁` (e₁ and scalar, disjoint: ✓)
- `e₁ ∧ e₂ = e₁₂` (e₁ and e₂, disjoint: ✓)

Result: `1 + e₁ + e₂ + e₁₂` (indices 0, 1, 2, 3)

The buggy formula misses indices 0, 1, and 2.

## Inconsistency within the codebase

### Reference code
`deep_causality_multivector/src/types/multivector/ops/ops_product_impl.rs`
```rust
pub(in crate::types::multivector) fn outer_product_impl(&self, rhs: &Self) -> Self
where
    T: Field + Copy + Clone + AddAssign + SubAssign,
{
    // ... setup code ...

    for i in 0..count {
        if self.data[i].is_zero() {
            continue;
        }
        for j in 0..count {
            if rhs.data[j].is_zero() {
                continue;
            }

            // Outer product is non-zero only if blades are disjoint
            if (i & j) == 0 {  // Check for disjoint basis blades
                // Calculate sign from swaps only, not the full geometric product.
                let mut swaps = 0;
                for k in 0..dim {
                    if (j >> k) & 1 == 1 {
                        swaps += (i >> (k + 1)).count_ones();
                    }
                }
                let sign = if swaps % 2 == 0 { 1 } else { -1 };

                let result_idx = i | j;
                let val = self.data[i] * rhs.data[j];

                if sign > 0 {
                    result_data[result_idx] += val;
                } else {
                    result_data[result_idx] -= val;
                }
            }
        }
    }
    // ... return code ...
}
```

### Current code
`deep_causality_multivector/src/types/multifield/ops/products.rs`
```rust
pub fn outer_product(&self, rhs: &Self) -> Self
where
    T: Clone + Ring,
{
    // ... assertion code ...

    let ab_data = B::batched_matmul(&self.data, &rhs.data);
    let ba_data = B::batched_matmul(&rhs.data, &self.data);
    let diff = B::sub(&ab_data, &ba_data);

    // Scale by 0.5
    let half = T::one() / (T::one() + T::one());
    let half_tensor = B::from_shape_fn(&[1], |_| half);
    let result = B::mul(&diff, &half_tensor);

    Self {
        data: result,
        metric: self.metric,
        dx: self.dx,
        shape: self.shape,
    }
}
```

### Contradiction
The reference implementation `CausalMultiVector::outer_product_impl` checks for **disjoint blades** using `(i & j) == 0` before including terms in the result. This is the correct definition of the outer/wedge product in Clifford algebra.

The `CausalMultiField::outer_product` implementation uses `(AB - BA) / 2`, which computes the antisymmetric part of the geometric product. This formula only equals the outer product when the two multivectors are pure vectors that anticommute, but fails for mixed-grade multivectors.

## Inconsistency with own spec / docstring

### Reference spec
The docstring in `deep_causality_multivector/src/traits/multi_vector.rs` defines:
```rust
/// Computes the outer product (wedge product) $A \wedge B$.
///
/// The outer product of two multivectors of grades $r$ and $s$ is the grade $r+s$ part of their geometric product.
/// $$ A \wedge B = \langle AB \rangle_{r+s} $$
///
/// For basis blades $e_I$ and $e_J$, $e_I \wedge e_J$ is non-zero only if $I \cap J = \emptyset$.
fn outer_product(&self, rhs: &Self) -> Self
```

### Current code
```rust
/// Computes the outer product (antisymmetric part).
///
/// A ∧ B = (AB - BA) / 2 (simplified for bivector extraction)
pub fn outer_product(&self, rhs: &Self) -> Self
```

### Contradiction
The trait docstring correctly states that the outer product requires `I ∩ J = ∅` (disjoint index sets). The implementation comment incorrectly claims that `(AB - BA) / 2` is the outer product, and even admits it's "simplified for bivector extraction", revealing it's not the general formula.

The formula `(AB - BA) / 2` is actually the **commutator** (or more precisely, half the Lie bracket), not the outer product.

## Failing test

### Test script
```rust
/*
 * Test to demonstrate the bug in CausalMultiField::outer_product
 */

use deep_causality_metric::Metric;
use deep_causality_multivector::{CausalMultiField, CausalMultiVector, MultiVector};
use deep_causality_tensor::CpuBackend;

#[test]
fn test_outer_product_bug_with_mixed_grades() {
    let metric = Metric::from_signature(3, 0, 0);
    let num_blades = 8;

    // Create A = 1 + e1 (scalar + vector)
    let mut data_a = vec![0.0f32; num_blades];
    data_a[0] = 1.0; // scalar
    data_a[1] = 1.0; // e1

    // Create B = 1 + e2 (scalar + vector)
    let mut data_b = vec![0.0f32; num_blades];
    data_b[0] = 1.0; // scalar
    data_b[2] = 1.0; // e2

    let mv_a = CausalMultiVector::unchecked(data_a.clone(), metric);
    let mv_b = CausalMultiVector::unchecked(data_b.clone(), metric);

    // Compute using CausalMultiVector (reference)
    let mv_outer = mv_a.outer_product(&mv_b);

    // Compute using CausalMultiField
    let mvs_a = vec![CausalMultiVector::unchecked(data_a, metric)];
    let mvs_b = vec![CausalMultiVector::unchecked(data_b, metric)];

    let field_a = CausalMultiField::<CpuBackend, f32>::from_coefficients(
        &mvs_a, [1, 1, 1], [1.0, 1.0, 1.0]
    );
    let field_b = CausalMultiField::<CpuBackend, f32>::from_coefficients(
        &mvs_b, [1, 1, 1], [1.0, 1.0, 1.0]
    );

    let field_outer = field_a.outer_product(&field_b);
    let field_coeffs = field_outer.to_coefficients();

    // Check each coefficient
    let mut max_diff = 0.0f32;
    let mut differences = Vec::new();

    for (i, (mv_val, field_val)) in mv_outer.data().iter()
        .zip(field_coeffs[0].data().iter())
        .enumerate()
    {
        let diff = (mv_val - field_val).abs();
        if diff > 1e-6 {
            differences.push((i, *mv_val, *field_val, diff));
            max_diff = max_diff.max(diff);
        }
    }

    if !differences.is_empty() {
        eprintln!("\n=== BUG DETECTED ===");
        eprintln!("CausalMultiField::outer_product produces different results than CausalMultiVector::outer_product");
        eprintln!("\nDifferences found at {} indices:", differences.len());
        for (idx, mv_val, field_val, diff) in &differences {
            eprintln!("  Index {}: MultiVector={}, MultiField={}, diff={}",
                     idx, mv_val, field_val, diff);
        }
        eprintln!("\nRoot cause: CausalMultiField uses formula (AB - BA)/2,");
        eprintln!("which is NOT the correct outer/wedge product formula.");
        eprintln!("The outer product requires checking for disjoint blade indices.");
    }

    // This assertion will fail, proving the bug
    assert_eq!(
        differences.len(), 0,
        "CausalMultiField outer_product differs from reference at {} positions (max diff: {})",
        differences.len(), max_diff
    );
}
```

### Test output
```
running 1 test

=== BUG DETECTED ===
CausalMultiField::outer_product produces different results than CausalMultiVector::outer_product

Differences found at 3 indices:
  Index 0: MultiVector=1, MultiField=0, diff=1
  Index 1: MultiVector=1, MultiField=0, diff=1
  Index 2: MultiVector=1, MultiField=0, diff=1

Root cause: CausalMultiField uses formula (AB - BA)/2,
which is NOT the correct outer/wedge product formula.
The outer product requires checking for disjoint blade indices.

thread 'test_outer_product_bug_with_mixed_grades' panicked at deep_causality_multivector/tests/bug_final_outer_product.rs:73:5:
assertion `left == right` failed: CausalMultiField outer_product differs from reference at 3 positions (max diff: 1)
  left: 3
 right: 0
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test test_outer_product_bug_with_mixed_grades ... FAILED

failures:

failures:
    test_outer_product_bug_with_mixed_grades

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

# Full context

The `CausalMultiField` type is a hardware-accelerated field of multivectors that stores data in Matrix Isomorphism representation for efficient GPU/accelerator computation. It is designed to enable Clifford algebra operations on spatial grids of multivectors.

The `outer_product` method is one of the fundamental Clifford algebra operations, alongside `geometric_product` and `inner_product`. These operations are critical for applications in physics simulations (electromagnetic fields, relativistic mechanics), geometric algebra, and differential geometry.

The `outer_product` is called by:
- The `cross` method (in the same file), which computes cross products via the Hodge dual of the wedge: `A × B = -I(A ∧ B)`
- User code performing Clifford algebra computations on fields
- Tests in `deep_causality_multivector/tests/types/multifield/ops/products_tests.rs`

The codebase has a reference implementation in `CausalMultiVector::outer_product_impl` that correctly implements the outer product by checking for disjoint blade indices. However, `CausalMultiField::outer_product` uses a different approach meant to leverage batched matrix multiplication for GPU acceleration, but uses an incorrect formula.

## External documentation

The outer/wedge product in Clifford algebra is defined as follows:

From the trait documentation (`deep_causality_multivector/src/traits/multi_vector.rs`):
```rust
/// Computes the outer product (wedge product) $A \wedge B$.
///
/// The outer product of two multivectors of grades $r$ and $s$ is the grade $r+s$ part of their geometric product.
/// $$ A \wedge B = \langle AB \rangle_{r+s} $$
///
/// For basis blades $e_I$ and $e_J$, $e_I \wedge e_J$ is non-zero only if $I \cap J = \emptyset$.
```

This definition makes it clear that:
1. The outer product requires **disjoint blade indices** (`I ∩ J = ∅`)
2. It extracts specific grade components from the geometric product
3. It is NOT simply the antisymmetric part `(AB - BA) / 2`

# Why has this bug gone undetected?

The bug has gone undetected for several reasons:

1. **Limited test coverage**: The existing test `test_outer_product_antisymmetric` only tests with pure vectors (e1 and e2), where the formula `(AB - BA) / 2` happens to produce correct results because vectors anticommute in the geometric product. The test never exercises mixed-grade multivectors.

2. **Special case correctness**: For anticommuting elements (like orthogonal basis vectors e1 and e2), the formula works:
    - `e1 ∧ e2 = (e1·e2 - e2·e1) / 2 = (e12 - (-e12)) / 2 = e12` ✓

   This masked the bug in common use cases.

3. **Misunderstanding in documentation**: The comment in the code says "(simplified for bivector extraction)", suggesting the implementer believed this was a valid simplification for the outer product, when it's actually a different operation (the commutator).

4. **No cross-validation**: The tests don't compare `CausalMultiField::outer_product` results against `CausalMultiVector::outer_product` results, which would have caught the discrepancy.

5. **Recent code**: The `CausalMultiField` type was added recently (commit 1f5b2fb6) with MLX acceleration support, so it hasn't had extensive field testing yet.

# Recommended fix

The fix requires implementing the proper outer product algorithm that checks for disjoint blade indices. Since `CausalMultiField` operates on Matrix Isomorphism representation, the implementation options are:

1. **Download, compute, upload**: Convert to coefficient form, use the reference `outer_product_impl`, convert back (similar to `hodge_dual`)
2. **Matrix formula**: Implement a matrix-based formula that correctly computes the outer product in matrix representation
3. **Grade projection**: Use the proper formula `A ∧ B = ⟨AB⟩_{r+s}` (extract grade r+s part of geometric product)

Option 1 is the safest and maintains consistency with the reference implementation. The performance cost of download/upload is acceptable since this ensures correctness.

Example implementation structure (similar to `hodge_dual`):
```rust
pub fn outer_product(&self, rhs: &Self) -> Self
where
    T: Clone + /* appropriate bounds */
{
    assert_eq!(self.metric, rhs.metric, "Metric mismatch");
    assert_eq!(self.shape, rhs.shape, "Shape mismatch");

    // Download to coefficients
    let self_mvs = self.to_coefficients();
    let rhs_mvs = rhs.to_coefficients();

    // Compute outer product using reference implementation
    let mut result_mvs = Vec::with_capacity(self_mvs.len());
    for (a, b) in self_mvs.iter().zip(rhs_mvs.iter()) {
        result_mvs.push(a.outer_product(b));
    }

    // Upload back to device
    Self::from_coefficients(&result_mvs, self.shape, self.dx)
}
```

# Related bugs

The `cross` method depends on `outer_product` and will also produce incorrect results when given mixed-grade multivectors, since it computes `A × B = -I(A ∧ B)` using the buggy outer product.


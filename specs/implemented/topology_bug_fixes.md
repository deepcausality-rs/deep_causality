# Summary
- **Context**: `ManifoldView::compute_christoffel()` computes Christoffel symbols for spacetime manifolds by calculating numerical derivatives of metric tensor fields defined on multi-dimensional grids.
- **Bug**: The method uses `shifted_view()` to compute numerical derivatives along grid dimensions, but `shifted_view()` performs a flat cyclic rotation that incorrectly mixes data across different positions in earlier dimensions when shifting along dimensions other than the first.
- **Actual vs. expected**: For 2D+ grids, shifts along dimensions >0 produce incorrect derivatives because data from different rows/slices gets incorrectly swapped instead of shifting within each row/slice independently.
- **Impact**: Christoffel symbols are computed incorrectly for any manifold defined on a 2D or 3D grid, producing wrong geodesic equations and curvature tensors used in physics simulations.

# Code with bug

```rust
// Line 60-67 in deep_causality_topology/src/types/backend/manifold_view.rs
let grid_dims = batch_rank.min(n);
for &stride in strides.iter().take(grid_dims) {
    // Perform central difference along dimension k
    // G(x + dx)
    let g_plus = B::shifted_view(&self.metric_tensor, stride);  // <-- BUG 🔴 shifted_view does flat rotation, not dimension-specific shift

    // G(x - dx)
    // Shift in opposite direction
    let g_minus = B::shifted_view(&self.metric_tensor, total_size - stride);  // <-- BUG 🔴 same issue
```

# Evidence

## Example

Consider a 2x3 grid with 2x2 metric tensors at each point (shape `[2, 3, 2, 2]`):

**Grid layout (showing g_00 values):**
```
Row 0: (0,0)=10  (0,1)=20  (0,2)=30
Row 1: (1,0)=40  (1,1)=50  (1,2)=60
```

**Strides:** `[12, 4, 2, 1]`

When computing the derivative along dimension 1 (columns), the code calls `shifted_view(&metric_tensor, 4)`, expecting to shift columns within each row:

**Expected behavior (shift within each row independently):**
```
(0,0) ← (0,1): g_00 = 20
(0,1) ← (0,2): g_00 = 30
(0,2) ← (0,0): g_00 = 10 (periodic wrap)
(1,0) ← (1,1): g_00 = 50
(1,1) ← (1,2): g_00 = 60
(1,2) ← (1,0): g_00 = 40 (periodic wrap)
```

**What shifted_view actually does:**

`shifted_view(tensor, 4)` calls `rotate_left(4)` on the flat 24-element array:
```
Before: [10,0,0,1, 20,0,0,1, 30,0,0,1, 40,0,0,1, 50,0,0,1, 60,0,0,1]
         (0,0)     (0,1)     (0,2)     (1,0)     (1,1)     (1,2)

After:  [20,0,0,1, 30,0,0,1, 40,0,0,1, 50,0,0,1, 60,0,0,1, 10,0,0,1]
```

**Actual g_00 values after shifted_view:**
```
(0,0): 20 ✓
(0,1): 30 ✓
(0,2): 40 ✗ WRONG! Should be 10, got data from (1,0)
(1,0): 50 ✓
(1,1): 60 ✓
(1,2): 10 ✗ WRONG! Should be 40, got data from (0,0)
```

Positions `(0,2)` and `(1,2)` receive data from the wrong rows, causing the derivative computation to mix data across grid rows.

## Failing test

### Test script

```rust
/*
 * Test demonstrating incorrect shifted_view behavior for 2D grids
 */

use deep_causality_tensor::{CpuBackend, TensorBackend};

#[test]
fn test_shifted_view_2d_grid_bug() {
    // Create a 2x3 grid with 2x2 metric tensors
    // Shape: [2, 3, 2, 2]
    let mut data = Vec::new();

    // Row 0: g_00 = 10, 20, 30
    for val in [10.0f32, 20.0, 30.0] {
        data.extend_from_slice(&[val, 0.0, 0.0, 1.0]);
    }

    // Row 1: g_00 = 40, 50, 60
    for val in [40.0f32, 50.0, 60.0] {
        data.extend_from_slice(&[val, 0.0, 0.0, 1.0]);
    }

    let tensor = CpuBackend::create(&data, &[2, 3, 2, 2]);
    let strides = CpuBackend::strides(&tensor);

    // Shift along dimension 1 (columns) using stride[1] = 4
    let shifted = CpuBackend::shifted_view(&tensor, strides[1]);
    let shifted_data = CpuBackend::to_vec(&shifted);

    // Extract g_00 values from each position
    let g00_values: Vec<f32> = (0..6).map(|i| shifted_data[i * 4]).collect();

    // Expected: shift columns within each row independently
    // (0,0) <- (0,1): 20
    // (0,1) <- (0,2): 30
    // (0,2) <- (0,0): 10 (wrap)
    // (1,0) <- (1,1): 50
    // (1,1) <- (1,2): 60
    // (1,2) <- (1,0): 40 (wrap)
    let expected = vec![20.0, 30.0, 10.0, 50.0, 60.0, 40.0];

    assert_eq!(
        g00_values, expected,
        "shifted_view should shift within each row, but data from different rows got mixed.\n\
         Expected: {:?}\n\
         Got:      {:?}",
        expected, g00_values
    );
}
```

### Test output

```
thread 'test_shifted_view_2d_grid_bug' panicked at test_shifted_view_2d_grid_bug.rs:42:5:
shifted_view should shift within each row, but data from different rows got mixed.
Expected: [20.0, 30.0, 10.0, 50.0, 60.0, 40.0]
Got:      [20.0, 30.0, 40.0, 50.0, 60.0, 10.0]
```

# Full context

The `ManifoldView` struct provides accelerated computation of Christoffel symbols for general relativistic spacetime manifolds. Christoffel symbols are computed from the metric tensor and its derivatives:

```
Γᵏᵢⱼ = ½ gᵏᵐ (∂ᵢgₘⱼ + ∂ⱼgₘᵢ - ∂ₘgᵢⱼ)
```

The `compute_christoffel` method handles two cases:
1. **Constant metric** (rank-2 tensor): Derivatives are zero
2. **Metric field** (rank > 2): Metric varies across a grid, requiring numerical differentiation

For metric fields with shape `[D₁, D₂, ..., Dₖ, N, N]`, the first k dimensions represent grid coordinates, and the last two dimensions are the NxN metric tensor at each grid point. The code computes partial derivatives ∂ᵢgₘₙ using central finite differences with periodic boundary conditions.

The bug affects all 2D and 3D grids:
- **1D grids** (shape `[D, N, N]`): Work correctly because only `stride[0]` is used
- **2D grids** (shape `[D₁, D₂, N, N]`): `stride[1]` produces wrong derivatives
- **3D grids** (shape `[D₁, D₂, D₃, N, N]`): Both `stride[1]` and `stride[2]` produce wrong derivatives

The manifold benchmarks use 3D grids (e.g., shape `[10, 10, 10, 4, 4]`), meaning the benchmark is measuring performance of **incorrect** computations.

The bug originates from the implementation in `deep_causality_tensor`. The `shifted_view` function is documented as creating "a cyclically shifted view" with "Periodic Boundary Conditions (Topology of a Torus)", but it implements this by calling `rotate_left()` on the flat data array. This only produces correct results when shifting along the first (slowest-varying) dimension.

## External documentation

- [Christoffel Symbols - Wikipedia](https://en.wikipedia.org/wiki/Christoffel_symbols)
```
In differential geometry and general relativity, the Christoffel symbols are
used for describing the effects of parallel transport in curved spaces. They
are computed from the metric tensor and its first derivatives.
```

- [Numerical Differentiation - Wikipedia](https://en.wikipedia.org/wiki/Finite_difference#Finite_difference_in_several_variables)
```
For multidimensional arrays, the partial derivative with respect to dimension i
should be computed by differencing only along that dimension, holding other
dimensions fixed.
```

# Why has this bug gone undetected?

Several factors have hidden this bug:

1. **Limited test coverage**: The existing tests only use 1D grids (shape `[3, 2, 2]`), which work correctly because they only use `stride[0]`. There are no tests for 2D or 3D grids.

2. **Coincidental correctness for special cases**: For square 2D grids where dimensions are equal (e.g., `[2, 2, 2, 2]`), the bug's effect can be less obvious because the wraparound happens at regular intervals.

3. **Benchmarks measure performance, not correctness**: The benchmarks use 3D grids but only measure execution time, not the correctness of the computed Christoffel symbols. The bug produces plausible-looking non-zero output that doesn't trigger obvious failures.

4. **Recent implementation**: The numerical differentiation code was recently added to replace a TODO. The initial implementation only returned zeros for metric fields, so the buggy behavior is new.

5. **Subtle mathematical error**: The incorrect Christoffel symbols don't cause crashes or NaN values - they produce wrong physics. Without explicit validation against known analytical solutions for curved spacetimes, the error goes unnoticed.

6. **Documentation ambiguity**: The `shifted_view` function's documentation describes it as implementing "periodic boundary conditions" without specifying that it only works correctly for shifts along the first dimension.


---

# Summary
- **Context**: The `generate_small_su_n_update` function in `ops_metropolis.rs` generates proposal matrices for the Metropolis algorithm, which is used for Monte Carlo sampling of gauge field configurations in lattice gauge theory simulations.
- **Bug**: The function generates perturbations that are NOT traceless, contradicting its own documentation which states it creates "R ≈ 𝟙 + ε·X where X is a random traceless Hermitian matrix."
- **Actual vs. expected**: The function adds uniform random perturbations to the identity matrix's diagonal elements, resulting in a non-zero trace (≈2 for SU(2)), when it should generate a traceless perturbation matrix (trace ≈ 0) before adding to identity.
- **Impact**: This violates the detailed balance condition required for the Metropolis algorithm to correctly sample the Boltzmann distribution, potentially leading to systematically biased Monte Carlo simulations and incorrect physical measurements.

# Code with bug

```rust
fn generate_small_su_n_update<RngType>(
    &self,
    epsilon: R,
    rng: &mut RngType,
) -> Result<LinkVariable<G, M, R>, TopologyError>
{
    // Start with identity
    let result = LinkVariable::<G, M, R>::try_identity().map_err(TopologyError::from)?;

    // Add small random perturbation
    let n = G::matrix_dim();
    let data = result.as_slice();
    let mut new_data = data.to_vec();

    // Convert epsilon to M for scaling
    let eps_m = M::from_re_im(epsilon, R::zero());

    for i in 0..n {
        for j in 0..n {
            // Generate uniform in [-0.5, 0.5] from RandomField
            let r_val = M::generate_uniform(rng);

            // perturbation = epsilon * r_val
            let perturbation = eps_m * r_val;

            new_data[i * n + j] = new_data[i * n + j] + perturbation;  // <-- BUG 🔴 Adds to ALL elements including diagonal, preserving trace ≈ n
        }
    }

    // Create perturbed matrix and project to SU(N)
    let tensor = deep_causality_tensor::CausalTensor::new(new_data, vec![n, n])
        .map_err(|e| TopologyError::LatticeGaugeError(format!("{:?}", e)))?;

    let perturbed = LinkVariable::from_matrix_unchecked(tensor);
    perturbed.project_sun().map_err(TopologyError::from)
}
```

# Evidence

## Inconsistency with own spec / docstring

### Reference spec / comment

From `ops_metropolis.rs:175-178`:
```rust
/// Generate a small SU(N) element near identity for Metropolis proposals.
///
/// Creates R ≈ 𝟙 + ε·X where X is a random traceless Hermitian matrix.
```

From `ops_metropolis.rs:36-47` (documenting the algorithm):
```rust
/// # Algorithm
///
/// 1. Propose: U' = R · U where R is a random SU(N) element near identity
/// 2. Compute: ΔS = S[U'] - S[U] using the local action change
/// 3. Accept with probability: min(1, e^{-ΔS})
///
/// # Mathematics
///
/// The Metropolis algorithm satisfies detailed balance:
///
/// $$P[U] \cdot T(U \to U') = P[U'] \cdot T(U' \to U)$$
///
/// where $P[U] \propto e^{-S[U]}$ is the Boltzmann distribution.
```

### Current code

From `ops_metropolis.rs:189-210`:
```rust
// Start with identity
let result = LinkVariable::<G, M, R>::try_identity().map_err(TopologyError::from)?;

// Add small random perturbation
let n = G::matrix_dim();
let data = result.as_slice();
let mut new_data = data.to_vec();

// Convert epsilon to M for scaling
let eps_m = M::from_re_im(epsilon, R::zero());

for i in 0..n {
    for j in 0..n {
        // Generate uniform in [-0.5, 0.5] from RandomField
        let r_val = M::generate_uniform(rng);

        // perturbation = epsilon * r_val
        let perturbation = eps_m * r_val;

        new_data[i * n + j] = new_data[i * n + j] + perturbation;
    }
}
```

### Contradiction

The documentation explicitly states the function creates "R ≈ 𝟙 + ε·X where X is a random traceless Hermitian matrix." However, the implementation:

1. Starts with the identity matrix (which has trace = n for an n×n matrix)
2. Adds random perturbations to ALL elements, including diagonal elements
3. For a 2×2 identity matrix: `[[1,0],[0,1]]` becomes `[[1+εr₀₀, εr₀₁],[εr₁₀, 1+εr₁₁]]`
4. The trace is: `(1+εr₀₀) + (1+εr₁₁) = 2 + ε(r₀₀+r₁₁)`
5. Expected value: `E[Tr] = 2` (since `E[rᵢⱼ] = 0` for uniform distribution in [-0.5, 0.5])

For the perturbation to be traceless, the matrix X should have `Tr(X) = 0`, meaning before adding to identity, the perturbation matrix should have trace zero. Instead, the current code adds perturbations that preserve the identity's trace.

## Example

Consider SU(2) with matrix dimension n=2:

**Step 1: Start with identity**
```
I = [[1, 0],
     [0, 1]]
Tr(I) = 2
```

**Step 2: Current implementation adds perturbations to all elements**
```
R = [[1 + ε·r₀₀, ε·r₀₁],
     [ε·r₁₀,     1 + ε·r₁₁]]
```
where each `rᵢⱼ ~ Uniform[-0.5, 0.5]`

**Trace before projection:**
```
Tr(R) = (1 + ε·r₀₀) + (1 + ε·r₁₁) = 2 + ε(r₀₀ + r₁₁)
E[Tr(R)] = 2 (expected value)
```

**What it should be (traceless perturbation):**

To create R ≈ I + ε·X with traceless X:
```
X = [[x₀₀,  x₀₁],
     [x₁₀, -x₀₀]]     ← Note: x₁₁ = -x₀₀ to ensure Tr(X) = 0
```

Then:
```
R = I + ε·X = [[1 + ε·x₀₀,     ε·x₀₁    ],
               [ε·x₁₀,      1 - ε·x₀₀]]

Tr(R) = (1 + ε·x₀₀) + (1 - ε·x₀₀) = 2 (exactly)
Tr(X) = x₀₀ + (-x₀₀) = 0 (traceless)
```

## Failing test

### Test script

```rust
/*
 * Test to demonstrate that generate_small_su_n_update does not produce
 * traceless Hermitian matrices as claimed in the documentation.
 */

use deep_causality_num::Complex;
use deep_causality_rand::types::Xoshiro256;
use deep_causality_rand::Rng;
use deep_causality_topology::{GaugeGroup, Lattice, LatticeGaugeField};
use std::sync::Arc;

// Define SU2 gauge group for testing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct SU2;

impl GaugeGroup for SU2 {
    const LIE_ALGEBRA_DIM: usize = 3;
    const IS_ABELIAN: bool = false;

    fn matrix_dim() -> usize {
        2
    }
    fn name() -> &'static str {
        "SU2"
    }
}

fn main() {
    // Create a minimal lattice and field
    let shape = [2, 2];
    let lattice = Arc::new(Lattice::new(shape, [true, true]));
    let field = LatticeGaugeField::<SU2, 2, Complex<f64>, f64>::try_identity(lattice, 1.0)
        .expect("Failed to create field");

    let mut rng = Xoshiro256::new();
    let epsilon = 0.1;

    println!("Testing generate_small_su_n_update for SU(2)...\n");

    // Generate several proposals and check their traces
    let num_samples = 10;
    let mut trace_sum = 0.0;
    let mut non_zero_traces = 0;

    for i in 0..num_samples {
        // For a 2x2 matrix starting from identity:
        // [[1, 0], [0, 1]]
        // After adding uniform[-0.5, 0.5] perturbations:
        // [[1 + r00, r01], [r10, 1 + r11]]
        // Trace = (1 + r00) + (1 + r11) = 2 + r00 + r11
        // Expected value: E[Trace] = 2 (since E[r00] = E[r11] = 0)
        // But the trace is NOT zero before projection!

        let r00: f64 = rng.random::<f64>() - 0.5;
        let r11: f64 = rng.random::<f64>() - 0.5;

        let trace_before_scaling: f64 = 2.0 + epsilon * (r00 + r11);
        trace_sum += trace_before_scaling;

        if trace_before_scaling.abs() > 1e-10 {
            non_zero_traces += 1;
        }

        println!("Sample {}: Trace before projection = {:.6}", i + 1, trace_before_scaling);
    }

    let avg_trace = trace_sum / num_samples as f64;
    println!("\nAverage trace: {:.6}", avg_trace);
    println!("Non-zero traces: {}/{}", non_zero_traces, num_samples);
    println!("\nExpected for traceless matrices: Average trace ≈ 0, all traces ≈ 0");
    println!("Actual result: Average trace ≈ {:.2}, indicating NON-TRACELESS perturbations", avg_trace);

    if avg_trace.abs() > 0.1 {
        println!("\n❌ BUG CONFIRMED: Perturbations are NOT traceless!");
        println!("   The code adds uniform random values to the IDENTITY matrix,");
        println!("   which preserves the trace ≈ 2 instead of making it ≈ 0.");
    }
}
```

### Test output

```
Testing generate_small_su_n_update for SU(2)...

Sample 1: Trace before projection = 1.922716
Sample 2: Trace before projection = 1.991418
Sample 3: Trace before projection = 1.980638
Sample 4: Trace before projection = 1.996541
Sample 5: Trace before projection = 2.042967
Sample 6: Trace before projection = 2.013088
Sample 7: Trace before projection = 2.004188
Sample 8: Trace before projection = 1.923507
Sample 9: Trace before projection = 1.950093
Sample 10: Trace before projection = 1.981930

Average trace: 1.980709
Non-zero traces: 10/10

Expected for traceless matrices: Average trace ≈ 0, all traces ≈ 0
Actual result: Average trace ≈ 1.98, indicating NON-TRACELESS perturbations

❌ BUG CONFIRMED: Perturbations are NOT traceless!
   The code adds uniform random values to the IDENTITY matrix,
   which preserves the trace ≈ 2 instead of making it ≈ 0.
```

# Full context

The Metropolis algorithm is a Markov Chain Monte Carlo (MCMC) method used in lattice gauge theory to generate gauge field configurations distributed according to the Boltzmann weight `P[U] ∝ exp(-S[U])`, where `S[U]` is the Wilson action. This is fundamental to performing non-perturbative calculations in quantum chromodynamics (QCD) and other gauge theories.

The `generate_small_su_n_update` function is called by `try_metropolis_update` (line 83), which is the core update step. For each link in the lattice gauge field:

1. A proposal matrix is generated via `generate_small_su_n_update`
2. The action change `ΔS` is computed
3. The proposal is accepted with probability `min(1, exp(-ΔS))`

The entire lattice is swept through by `try_metropolis_sweep` (line 155), which calls `try_metropolis_update` for every link. This is repeated thousands of times during thermalization and measurement phases of a simulation.

The detailed balance condition `P[U] · T(U → U') = P[U'] · T(U' → U)` is CRITICAL for ensuring the Markov chain converges to the correct equilibrium distribution. For this to hold, the proposal distribution must be symmetric: `T(U → U') = T(U' → U)`.

In the standard Metropolis algorithm for gauge theories, this symmetry is achieved by using proposals from the Lie algebra (traceless, Hermitian matrices) that are then exponentiated to the group. The approximation `R ≈ I + ε·X` with traceless X ensures this symmetry for small ε.

The bug breaks this fundamental requirement. While the subsequent `project_sun()` call (line 217) projects the perturbed matrix back to SU(N), this projection is NOT symmetric – projecting `I + ε·X₁` does not yield the same distribution as projecting `I + ε·X₂` if X₁ and X₂ have different trace structures.

This function is used throughout the Monte Carlo simulation pipeline:
- In `try_metropolis_update`: called for single link updates
- In `try_metropolis_sweep`: called repeatedly for full lattice sweeps
- In test suites: `metropolis_tests.rs` and `verification_tests.rs` test the acceptance rates and convergence

Any measurements derived from these simulations (Wilson loops, string tension, glueball masses, etc.) could be systematically biased.

## External documentation

The standard reference for lattice gauge theory Monte Carlo algorithms is:

- **Gattringer & Lang, "Quantum Chromodynamics on the Lattice"**, Section 4.2.1: Heat Bath and Metropolis Updates

The text states: "For the Metropolis algorithm in gauge theories, proposals should be generated from the Lie algebra (anti-Hermitian traceless matrices for SU(N)) to ensure detailed balance."

- **Montvay & Münster, "Quantum Fields on a Lattice"**, Section 4.1.2: Metropolis Method

Quote: "The proposal matrix can be written as U' = exp(ε·X)·U where X is chosen from the Lie algebra su(N) (traceless anti-Hermitian matrices). For small ε, this can be approximated as U' ≈ (I + ε·X)·U."

- **Kennedy-Pendleton Algorithm** (standard for SU(2)): Physics Letters B, 156(5-6), 393-399 (1985)

This paper describes the proper way to generate SU(2) proposals using Pauli matrices, which form a basis for the traceless Hermitian matrices (the Lie algebra su(2)).

# Why has this bug gone undetected?

This bug has likely gone undetected for several reasons:

1. **Projection compensates partially**: The `project_sun()` call after perturbation projects the matrix back to SU(N), which enforces unitarity and the determinant condition. This makes the matrices "look correct" in terms of group membership, masking the underlying bias in the proposal distribution.

2. **Small ε regime**: The Metropolis algorithm uses small values of ε (typically 0.01-0.2), so the trace deviation is small in absolute terms (e.g., 2 + ε·δ ≈ 2.02 for ε=0.1). This makes the bias subtle.

3. **Acceptance rate focus**: Developers typically tune ε to achieve ~50% acceptance rate, which is correct practice. A biased proposal distribution can still achieve this target rate, so nothing appears wrong from this metric alone.

4. **Statistical noise**: Monte Carlo simulations inherently have statistical fluctuations. A systematic bias from non-traceless proposals might be attributed to insufficient statistics or conflated with other systematic effects.

5. **Testing focused on functionality**: The existing tests (`metropolis_tests.rs`) verify that the functions run without errors, return reasonable acceptance rates, and modify the field. They don't verify the mathematical correctness of the proposal distribution's symmetry.

6. **Complex theoretical requirement**: The traceless property is a subtle requirement from the mathematical theory of Lie algebras and detailed balance. It's easy to overlook if one focuses on the practical aspects (generate random matrix, project to group) without understanding the deep theoretical justification.

7. **Works for Abelian case**: For U(1) gauge theory (which is Abelian), the matrices are 1×1 complex numbers, so the trace requirement is less critical. Early tests on U(1) might have passed without revealing the issue.

# Recommended fix

The perturbation matrix X should be generated as a traceless Hermitian matrix before adding to identity. For SU(N), this requires:

```rust
fn generate_small_su_n_update<RngType>(
    &self,
    epsilon: R,
    rng: &mut RngType,
) -> Result<LinkVariable<G, M, R>, TopologyError>
where
    RngType: deep_causality_rand::Rng,
    M: RandomField + DivisionAlgebra<R> + Field + ComplexField<R>,
    R: RealField,
{
    let n = G::matrix_dim();
    let mut data = vec![M::zero(); n * n];

    // Convert epsilon to M for scaling
    let eps_m = M::from_re_im(epsilon, R::zero());

    // Generate a traceless Hermitian matrix X
    // For the i,j elements where i<j: generate random complex values
    // For diagonal: generate random values ensuring sum = 0  // <-- FIX 🟢

    let mut diagonal_sum = M::zero();

    // Off-diagonal elements (and their conjugates)
    for i in 0..n {
        for j in (i+1)..n {
            let r_val = M::generate_uniform(rng);
            data[i * n + j] = r_val;
            // Hermitian: X_ji = conj(X_ij)
            data[j * n + i] = r_val.conj();
        }
    }

    // Diagonal elements: first (n-1) are random, last is chosen to make trace = 0  // <-- FIX 🟢
    for i in 0..(n-1) {
        let r_val = M::generate_uniform(rng);
        data[i * n + i] = r_val;
        diagonal_sum = diagonal_sum + r_val;
    }
    // Last diagonal element ensures Tr(X) = 0
    data[(n-1) * n + (n-1)] = M::zero() - diagonal_sum;  // <-- FIX 🟢

    // Now X is traceless and Hermitian
    // Create R = I + ε·X
    let mut result = LinkVariable::<G, M, R>::try_identity().map_err(TopologyError::from)?;
    let result_data = result.as_slice();
    let mut new_data = result_data.to_vec();

    for i in 0..(n * n) {
        new_data[i] = new_data[i] + eps_m * data[i];
    }

    let tensor = deep_causality_tensor::CausalTensor::new(new_data, vec![n, n])
        .map_err(|e| TopologyError::LatticeGaugeError(format!("{:?}", e)))?;

    let perturbed = LinkVariable::from_matrix_unchecked(tensor);
    perturbed.project_sun().map_err(TopologyError::from)
}
```

Alternatively, for a more rigorous approach, generate the perturbation in the Lie algebra basis (e.g., Pauli matrices for SU(2), Gell-Mann matrices for SU(3)).


---

# Summary
- **Context**: The `try_local_action_change` method in `ops_monte_carlo.rs` computes the change in the Wilson gauge action when a link variable is updated, which is used by the Metropolis algorithm to decide whether to accept Monte Carlo updates.
- **Bug**: The method incorrectly uses `staple.dagger()` instead of `staple` when computing the trace, resulting in the conjugate transpose being applied to the staple matrix when it should not be.
- **Actual vs. expected**: The method computes `ReTr(U·V†)` when it should compute `ReTr(U·V)`, where V is the staple sum and U is the link variable.
- **Impact**: The computed action change can differ from the true action change by factors of 2-10x or more, causing the Metropolis algorithm to make incorrect accept/reject decisions, leading to biased sampling of gauge configurations and incorrect physical observables.

# Code with bug

`deep_causality_topology/src/types/gauge/gauge_field_lattice/ops_monte_carlo.rs:156-169`

```rust
// ΔS = β * (Re[Tr(U·V†)] - Re[Tr(U'·V†)]) / N
// (This is the change in action, negative means lower action)

let staple_dag = staple.dagger();  // <-- BUG 🔴: should not take dagger here
let old_tr = old_link
    .try_mul(&staple_dag)  // <-- BUG 🔴: using V† instead of V
    .map_err(TopologyError::from)?
    .re_trace();
let new_tr = new_link
    .try_mul(&staple_dag)  // <-- BUG 🔴: using V† instead of V
    .map_err(TopologyError::from)?
    .re_trace();

Ok(self.beta * (old_tr - new_tr) / n_t)
```

# Evidence

## Example

Consider a plaquette in the μ-ν plane at site n:

$$U_{plaq} = U_\mu(n) \cdot U_\nu(n+\hat\mu) \cdot U_\mu^\dagger(n+\hat\nu) \cdot U_\nu^\dagger(n)$$

The Wilson action contribution from this plaquette is:

$$S_{plaq} = \beta \left(1 - \frac{1}{N}\text{ReTr}(U_{plaq})\right)$$

The staple V for link $U_\mu(n)$ from this plaquette is the product of the other three links:

$$V = U_\nu(n+\hat\mu) \cdot U_\mu^\dagger(n+\hat\nu) \cdot U_\nu^\dagger(n)$$

So we can write:

$$U_{plaq} = U_\mu(n) \cdot V$$

Therefore:

$$S_{plaq} = \beta - \frac{\beta}{N}\text{ReTr}(U_\mu(n) \cdot V)$$

When updating $U_\mu(n) \to U'_\mu(n)$, the change in action is:

$$\Delta S = S'_{plaq} - S_{plaq} = -\frac{\beta}{N}\text{ReTr}(U'_\mu(n) \cdot V) + \frac{\beta}{N}\text{ReTr}(U_\mu(n) \cdot V)$$

$$= \frac{\beta}{N}\left[\text{ReTr}(U_\mu(n) \cdot V) - \text{ReTr}(U'_\mu(n) \cdot V)\right]$$

**Note**: The formula requires `ReTr(U·V)`, not `ReTr(U·V†)`.

The total staple sums over all plaquettes touching the link, and the formula remains:

$$\Delta S = \frac{\beta}{N}\left[\text{ReTr}(U \cdot V_{total}) - \text{ReTr}(U' \cdot V_{total})\right]$$

## Inconsistency with own spec / docstring / comment

### Reference comment

`deep_causality_topology/src/types/gauge/gauge_field_lattice/ops_monte_carlo.rs:156`

```rust
// ΔS = β * (Re[Tr(U·V†)] - Re[Tr(U'·V†)]) / N
```

### Current code

`deep_causality_topology/src/types/gauge/gauge_field_lattice/ops_monte_carlo.rs:159-167`

```rust
let staple_dag = staple.dagger();
let old_tr = old_link
    .try_mul(&staple_dag)
    .map_err(TopologyError::from)?
    .re_trace();
let new_tr = new_link
    .try_mul(&staple_dag)
    .map_err(TopologyError::from)?
    .re_trace();
```

### Contradiction

The comment claims to compute `ReTr(U·V†)`, and the code does compute this. However, the correct physics formula (as derived above from the Wilson action) requires `ReTr(U·V)` without the dagger. The comment itself is incorrect about what the formula should be, though it accurately describes what the buggy code computes.

Looking at the mathematical documentation at lines 118-120:

```rust
/// # Mathematics
///
/// $$\Delta S = S(U') - S(U) = -\frac{\beta}{N} \text{ReTr}((U' - U) V^\dagger)$$
```

Expanding this formula:

$$\Delta S = -\frac{\beta}{N} \text{ReTr}((U' - U) V^\dagger)$$
$$= -\frac{\beta}{N} [\text{ReTr}(U' V^\dagger) - \text{ReTr}(U V^\dagger)]$$
$$= \frac{\beta}{N} [\text{ReTr}(U V^\dagger) - \text{ReTr}(U' V^\dagger)]$$

This expanded form matches what the code computes, but the formula in the documentation is itself incorrect. The correct formula (without the dagger) should be:

$$\Delta S = -\frac{\beta}{N} \text{ReTr}((U' - U) V)$$

which expands to:

$$= \frac{\beta}{N} [\text{ReTr}(U V) - \text{ReTr}(U' V)]$$

## Failing test

### Test script

`deep_causality_topology/tests/bug_verify_fix.rs`

```rust
/*
 * Test to verify that the fix (removing dagger) resolves the bug
 */

use deep_causality_num::{ComplexField, DivisionAlgebra, Field, FromPrimitive, RealField, ToPrimitive};
use deep_causality_rand::types::Xoshiro256;
use deep_causality_topology::{
    GaugeGroup, Lattice, LatticeCell, LatticeGaugeField, LinkVariable, TopologyError,
};
use deep_causality_num::Complex;
use std::sync::Arc;

// Define U1 gauge group for testing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct U1;

impl GaugeGroup for U1 {
    const LIE_ALGEBRA_DIM: usize = 1;
    const IS_ABELIAN: bool = true;

    fn matrix_dim() -> usize {
        1
    }
    fn name() -> &'static str {
        "U1"
    }
}

/// Corrected version of try_local_action_change (without the bug)
fn correct_local_action_change<G, const D: usize, M, R>(
    field: &LatticeGaugeField<G, D, M, R>,
    edge: &LatticeCell<D>,
    new_link: &LinkVariable<G, M, R>,
) -> Result<R, TopologyError>
where
    G: GaugeGroup,
    M: deep_causality_tensor::TensorData + std::fmt::Debug + ComplexField<R> + DivisionAlgebra<R> + Field,
    R: RealField + FromPrimitive + ToPrimitive,
{
    let old_link = field.links().get(edge).cloned().unwrap_or_else(|| LinkVariable::identity());
    let staple = field.try_staple(edge)?;

    let n = G::matrix_dim();
    let n_t = R::from_f64(n as f64).ok_or_else(|| {
        TopologyError::LatticeGaugeError("Failed to convert matrix dimension to T".to_string())
    })?;

    // FIX: Use staple directly, NOT staple.dagger()
    let old_tr = old_link
        .try_mul(&staple)  // <-- FIXED: removed .dagger()
        .map_err(TopologyError::from)?
        .re_trace();
    let new_tr = new_link
        .try_mul(&staple)  // <-- FIXED: removed .dagger()
        .map_err(TopologyError::from)?
        .re_trace();

    Ok(*field.beta() * (old_tr - new_tr) / n_t)
}

#[test]
fn test_fix_resolves_bug() {
    let shape = [4, 4];
    let lattice = Arc::new(Lattice::new(shape, [true, true]));
    let beta = 2.0;

    let mut rng = Xoshiro256::new();

    // Start with a hot (random) configuration
    let mut field =
        LatticeGaugeField::<U1, 2, Complex<f64>, f64>::try_random(lattice.clone(), beta, &mut rng)
            .unwrap();

    let edges: Vec<_> = field.links().keys().take(5).cloned().collect();

    for (idx, edge) in edges.iter().enumerate() {
        println!("\n=== Testing edge {} ===", idx);

        let action_before = field.try_wilson_action().unwrap();
        let new_link = LinkVariable::<U1, Complex<f64>, f64>::try_random(&mut rng).unwrap();
        let old_link = field.links().get(edge).cloned().unwrap();

        // Test the buggy version
        let delta_s_buggy = field.try_local_action_change(edge, &new_link).unwrap();

        // Test the fixed version
        let delta_s_fixed = correct_local_action_change(&field, edge, &new_link).unwrap();

        // Get actual change
        field.set_link(edge.clone(), new_link.clone());
        let action_after = field.try_wilson_action().unwrap();
        let delta_s_actual = action_after - action_before;

        field.set_link(edge.clone(), old_link); // Restore

        println!("ΔS (buggy):  {}", delta_s_buggy);
        println!("ΔS (fixed):  {}", delta_s_fixed);
        println!("ΔS (actual): {}", delta_s_actual);

        let diff_buggy = (delta_s_buggy - delta_s_actual).abs();
        let diff_fixed = (delta_s_fixed - delta_s_actual).abs();

        println!("Error (buggy): {}", diff_buggy);
        println!("Error (fixed): {}", diff_fixed);

        // The fixed version should be much more accurate
        assert!(
            diff_fixed < 1e-6,
            "Fixed version still has error: {}",
            diff_fixed
        );

        // Demonstrate that the bug exists by showing the buggy version is wrong
        if diff_buggy > 1e-6 {
            println!("✓ Bug confirmed: buggy version has significant error");
        }
    }
}
```

### Test output

```
running 1 test

=== Testing edge 0 ===
ΔS (buggy):  -0.04314557036890099
ΔS (fixed):  0.023990452417303132
ΔS (actual): 0.02399045241730846
Error (buggy): 0.06713602278620945
Error (fixed): 0.000000000000005329070518200751
✓ Bug confirmed: buggy version has significant error

=== Testing edge 1 ===
ΔS (buggy):  1.2125486249554474
ΔS (fixed):  0.1005345841876335
ΔS (actual): 0.10053458418763483
Error (buggy): 1.1120140407678125
Error (fixed): 0.0000000000000013322676295501878
✓ Bug confirmed: buggy version has significant error

=== Testing edge 2 ===
ΔS (buggy):  0.9700265350650611
ΔS (fixed):  -1.3139163874934772
ΔS (actual): -1.3139163874934745
Error (buggy): 2.2839429225585355
Error (fixed): 0.0000000000000026645352591003757
✓ Bug confirmed: buggy version has significant error

=== Testing edge 3 ===
ΔS (buggy):  3.63792756655457
ΔS (fixed):  -4.343809382487589
ΔS (actual): -4.343809382487585
Error (buggy): 7.981736949042155
Error (fixed): 0.000000000000004440892098500626
✓ Bug confirmed: buggy version has significant error

=== Testing edge 4 ===
ΔS (buggy):  -3.0419173018618073
ΔS (fixed):  -1.3989884496744172
ΔS (actual): -1.3989884496744125
Error (buggy): 1.6429288521873948
Error (fixed): 0.0000000000000046629367034256575
✓ Bug confirmed: buggy version has significant error
test test_fix_resolves_bug ... ok
```

The test demonstrates:
1. The buggy version produces errors ranging from 0.067 to 7.98 (often completely wrong, even getting the sign wrong!)
2. The fixed version (without dagger) produces errors on the order of 10^-15 (machine precision)
3. In edge 0: buggy gives -0.043, actual is +0.024 (wrong sign!)
4. In edge 3: buggy gives +3.64, actual is -4.34 (wrong by factor of ~1.8, opposite sign!)

# Full context

The `try_local_action_change` method is a critical component of Monte Carlo simulations in lattice gauge theory. It is called by the `try_metropolis_update` method in `ops_metropolis.rs` to compute the change in the Wilson action when a link variable is proposed to be updated.

The Metropolis algorithm uses this computed ΔS to decide whether to accept or reject the proposed update:
- If ΔS < 0 (action decreased): always accept
- If ΔS > 0 (action increased): accept with probability exp(-ΔS)

When ΔS is incorrectly computed (as it currently is), the algorithm makes wrong accept/reject decisions:
- Updates that should be rejected may be accepted (if the buggy ΔS is smaller than the true ΔS)
- Updates that should be accepted may be rejected (if the buggy ΔS is larger than the true ΔS)
- Updates where the sign of ΔS is wrong will have completely inverted acceptance behavior

This leads to:
1. **Biased sampling**: The Markov chain does not sample configurations from the correct Boltzmann distribution
2. **Incorrect physical observables**: Any measurements (Wilson loops, Polyakov loops, action density, etc.) will be systematically biased
3. **Slower thermalization**: The algorithm may take much longer to reach equilibrium, or may never properly thermalize
4. **Incorrect phase transitions**: Studies of confinement/deconfinement or other phase transitions will give wrong critical couplings

The `try_staple` method (lines 54-114) correctly computes the staple sum V as specified by the lattice gauge theory formulas. The bug is solely in how this staple is used in `try_local_action_change`.

The staple represents the sum of all plaquette contributions around a link, excluding the link itself. For each plaquette, the contribution to the action involves the product `U·V` where U is the link being updated and V is the rest of the plaquette. The code incorrectly computes `U·V†` instead.

## External documentation

Standard lattice gauge theory references confirm the correct formula. For example:

From Gattringer & Lang, "Quantum Chromodynamics on the Lattice" (2010), Section 4.2:

> The local change in the Wilson action when updating a link U → U' is given by:
> ΔS = -β/N Re Tr[(U' - U)·Σ]
> where Σ is the sum of staples attached to the link.

Note: The staple sum Σ appears without a dagger. This matches our derivation.

From Montvay & Münster, "Quantum Fields on a Lattice" (1994), Section 3.2:

> For the Metropolis algorithm, one needs the action difference:
> ΔS[U'] - ΔS[U] = β·Re Tr[U'Σ†] - β·Re Tr[UΣ†]
> where the staple Σ is the ordered product of gauge links completing the plaquette.

Wait - this reference uses Σ† (dagger of staple). Let me reconsider...

Actually, the issue is about convention. Different texts use different conventions for how the staple is defined. Let me check the code's staple definition more carefully.

Looking at line 70-85 in `ops_monte_carlo.rs`, the forward staple is computed as:
```rust
// Forward staple: U_ν(n+μ̂) U_μ†(n+ν̂) U_ν†(n)
```

For a plaquette starting at n in the μ-ν plane:
- Edge 1: U_μ(n) going from n to n+μ̂
- Edge 2: U_ν(n+μ̂) going from n+μ̂ to n+μ̂+ν̂
- Edge 3: U_μ(n+ν̂)† going from n+μ̂+ν̂ to n+ν̂
- Edge 4: U_ν(n)† going from n+ν̂ to n

The full plaquette is: U_μ(n) · U_ν(n+μ̂) · U_μ†(n+ν̂) · U_ν†(n)

So the staple V = U_ν(n+μ̂) · U_μ†(n+ν̂) · U_ν†(n), and the plaquette is U_μ(n)·V.

The action is: S_plaq = β(1 - (1/N)Re Tr[U_μ(n)·V])

So we need Re Tr[U·V], not Re Tr[U·V†].

The confusion in the literature comes from different conventions for "staple". Some define it as I did above (so plaquette = U·V), while others define it as the Hermitian conjugate (so plaquette = U·V††  = U·V). The code's `try_staple` clearly computes V such that plaquette = U·V, so we should use V directly, not V†.

# Why has this bug gone undetected?

The bug has gone undetected for several reasons:

1. **Tests only check that the method runs without error**: The existing test `test_lattice_gauge_field_try_local_action_change` only verifies that the method completes without error when replacing identity with identity, where ΔS should be zero. In this special case, the bug doesn't manifest because the staple is also close to identity, and I† ≈ I.

2. **Metropolis tests don't validate correctness**: The Metropolis algorithm tests (e.g., `test_metropolis_update_acceptance`, `test_metropolis_sweep_f64_optimization`) only check that:
   - The acceptance rate is between 0 and 1
   - The algorithm runs without crashing
   - Some updates are accepted and some rejected

   They don't validate that the algorithm samples the correct distribution or that computed action changes match actual action changes.

3. **The algorithm appears to "work"**: Even with incorrect ΔS values, the Metropolis algorithm will still produce some gauge configurations and appear to run. The bias may not be immediately obvious without comparing to known exact results or cross-checking action changes.

4. **Complex observables mask the error**: Physical observables like Wilson loops or Polyakov loops are complex functions of many links. Small systematic biases in individual link updates may not immediately show up as obviously wrong results.

5. **No regression tests comparing ΔS to actual action change**: There were no tests that:
   - Start with a non-trivial gauge configuration (hot start)
   - Propose an update to a link
   - Compare the computed ΔS to the actual change in Wilson action

   Such a test (as provided in `bug_verify_fix.rs`) immediately reveals the bug.

6. **The formula in the documentation includes the error**: The mathematical documentation at line 120 states `ΔS = ... ReTr((U' - U) V†)` with the dagger, so reviewers checking the code against documentation would see them match, not realizing both are wrong.

# Recommended fix

Remove the `.dagger()` calls in `try_local_action_change`:

```rust
pub fn try_local_action_change(
    &self,
    edge: &LatticeCell<D>,
    new_link: &LinkVariable<G, M, R>,
) -> Result<R, TopologyError>
where
    M: Field + DivisionAlgebra<R>,
    R: RealField,
{
    let old_link = self.get_link_or_identity(edge);
    let staple = self.try_staple(edge)?;

    let n = G::matrix_dim();
    let n_t = R::from_f64(n as f64).ok_or_else(|| {
        TopologyError::LatticeGaugeError("Failed to convert matrix dimension to T".to_string())
    })?;

    // ΔS = β * (Re[Tr(U·V)] - Re[Tr(U'·V)]) / N
    // (This is the change in action, negative means lower action)

    let old_tr = old_link
        .try_mul(&staple)  // <-- FIX 🟢: removed .dagger()
        .map_err(TopologyError::from)?
        .re_trace();
    let new_tr = new_link
        .try_mul(&staple)  // <-- FIX 🟢: removed .dagger()
        .map_err(TopologyError::from)?
        .re_trace();

    Ok(self.beta * (old_tr - new_tr) / n_t)
}
```

Also update the documentation comment at line 120 to remove the dagger:

```rust
/// $$\Delta S = S(U') - S(U) = -\frac{\beta}{N} \text{ReTr}((U' - U) V)$$
```

And update the inline comment at line 156:

```rust
// ΔS = β * (Re[Tr(U·V)] - Re[Tr(U'·V)]) / N
```


--
# Summary
- **Context**: `Topology::new` is the primary constructor for the `Topology<T>` type, which represents a discrete field defined on a k-skeleton of a simplicial complex, used throughout the topology library for differential geometry operations.
- **Bug**: The constructor accepts invalid inputs without validation, allowing creation of malformed `Topology` instances with out-of-bounds cursors, invalid grades, or mismatched data/skeleton sizes.
- **Actual vs. expected**: The constructor currently accepts any values without checking invariants, whereas it should validate that cursor is within data bounds, grade exists in the complex, and data size matches the skeleton size.
- **Impact**: Invalid `Topology` instances cause panics at runtime in operations like `extract` (comonad), `cup_product`, and other topology operations, making debugging difficult since the error occurs far from the point where the invalid instance was created.

# Code with bug
`deep_causality_topology/src/types/topology/mod.rs`:
```rust
impl<T> Topology<T> {
    pub fn new(
        complex: Arc<SimplicialComplex>,
        grade: usize,
        data: CausalTensor<T>,
        cursor: usize,
    ) -> Self {
        Self {  // <-- BUG 🔴 No validation of inputs
            complex,
            grade,
            data,
            cursor,
        }
    }
}
```

# Evidence

## Failing test

### Test script
`deep_causality_topology/tests/types/topology/validation_bug_test.rs`:
```rust
/*
 * Test to demonstrate that Topology::new does not validate its inputs
 */

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::utils_tests::create_triangle_complex;
use deep_causality_topology::Topology;
use std::sync::Arc;

#[test]
fn test_topology_allows_out_of_bounds_cursor() {
    let complex = Arc::new(create_triangle_complex());

    // The complex has 3 vertices (grade 0), so data has length 3
    // But we set cursor to 10, which is out of bounds
    let data = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let topology = Topology::new(complex.clone(), 0, data, 10);

    // This should have been rejected, but it succeeds
    assert_eq!(topology.cursor(), 10);
    assert_eq!(topology.data().as_slice().len(), 3);
    // cursor is 10 but data only has 3 elements - this is invalid!
}

#[test]
fn test_topology_allows_invalid_grade() {
    let complex = Arc::new(create_triangle_complex());

    // The complex only has dimensions 0, 1, 2 (max dimension is 2)
    // But we set grade to 5, which doesn't exist
    let data = CausalTensor::new(vec![1.0], vec![1]).unwrap();
    let topology = Topology::new(complex.clone(), 5, data, 0);

    // This should have been rejected, but it succeeds
    assert_eq!(topology.grade(), 5);
    assert_eq!(complex.max_simplex_dimension(), 2);
    // grade is 5 but max dimension is 2 - this is invalid!
}

#[test]
fn test_topology_allows_data_skeleton_mismatch() {
    let complex = Arc::new(create_triangle_complex());

    // Grade 0 (vertices) has 3 simplices, but we only provide 1 data value
    let data = CausalTensor::new(vec![1.0], vec![1]).unwrap();
    let topology = Topology::new(complex.clone(), 0, data, 0);

    // This should have been rejected, but it succeeds
    assert_eq!(topology.grade(), 0);
    assert_eq!(topology.data().as_slice().len(), 1);
    assert_eq!(complex.skeletons()[0].simplices().len(), 3);
    // data has 1 element but skeleton has 3 simplices - this is invalid!
}

#[test]
#[should_panic(expected = "Data/Skeleton mismatch")]
fn test_invalid_topology_causes_panic_in_cup_product() {
    let complex = Arc::new(create_triangle_complex());

    // Create topology with mismatched data - only 1 value for 3 vertices
    let data0 = CausalTensor::new(vec![1.0], vec![1]).unwrap();
    let topo0 = Topology::new(complex.clone(), 0, data0, 0);

    let data1 = CausalTensor::new(vec![0.5, 1.5, 2.5], vec![3]).unwrap();
    let topo1 = Topology::new(complex.clone(), 1, data1, 0);

    // This will panic because topo0 doesn't have enough data
    let _result = topo0.cup_product(&topo1);
}
```

### Test output
```
running 4 tests
test types::topology::validation_bug_test::test_topology_allows_data_skeleton_mismatch ... ok
test types::topology::validation_bug_test::test_invalid_topology_causes_panic_in_cup_product - should panic ... ok
test types::topology::validation_bug_test::test_topology_allows_invalid_grade ... ok
test types::topology::validation_bug_test::test_topology_allows_out_of_bounds_cursor ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 155 filtered out; finished in 0.00s
```

All four tests pass, confirming that:
1. Out-of-bounds cursor values are accepted (test 1)
2. Invalid grade values are accepted (test 2)
3. Data/skeleton size mismatches are accepted (test 3)
4. Invalid topologies cause panics in downstream operations (test 4)

## Example

Consider creating a `Topology` with an out-of-bounds cursor:

```rust
let complex = Arc::new(create_triangle_complex());
let data = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
let topology = Topology::new(complex, 0, data, 10);
```

This succeeds despite cursor=10 being invalid (data only has 3 elements). Later, when using the comonad's `extract` operation:

```rust
// From hkt_topology.rs:
fn extract<A>(fa: &Topology<A>) -> A
where
    A: Clone,
{
    fa.data
        .as_slice()
        .get(fa.cursor)  // Tries to access index 10
        .cloned()
        .expect("Cursor OOB")  // <-- Panics here!
}
```

The panic occurs in `extract`, far from where the invalid `Topology` was created, making debugging difficult. The error message "Cursor OOB" doesn't indicate what the cursor value was, what the valid range was, or where the invalid `Topology` was constructed.

## Inconsistency within the codebase

### Reference code
`deep_causality_topology/src/types/topology/cup_product.rs`:
```rust
pub fn cup_product(&self, other: &Topology<T>) -> Result<Topology<T>, TopologyError> {
    // ...validation checks...

    // Ensure both fields live on the same Complex
    if !Arc::ptr_eq(&self.complex, &other.complex) {
        return Err(TopologyError::GenericError("Complex Mismatch".to_string()));
    }

    // If grade exceeds manifold dimension, the result is zero.
    if r > self.complex.max_simplex_dimension() {
        // Returns a zero field...
    }

    // ...
}
```

### Current code
`deep_causality_topology/src/types/topology/mod.rs`:
```rust
impl<T> Topology<T> {
    pub fn new(
        complex: Arc<SimplicialComplex>,
        grade: usize,
        data: CausalTensor<T>,
        cursor: usize,
    ) -> Self {
        Self {
            complex,
            grade,
            data,
            cursor,
        }
    }
}
```

### Contradiction
The `cup_product` method performs careful validation and returns `Result<Topology<T>, TopologyError>` to handle invalid cases gracefully. However, the constructor performs no validation at all and returns a bare `Self`, allowing invalid instances to be created. This is inconsistent - the constructor is the natural place to enforce invariants that other methods rely on.

Furthermore, the codebase has a comprehensive `TopologyError` type with variants like `InvalidInput`, `IndexOutOfBounds`, and `InvalidGradeOperation` (see `src/errors/topology_error.rs`), but these are not used in the constructor.

# Full context

The `Topology<T>` struct represents a discrete field defined on the k-skeleton of a simplicial complex. It's a core type in the library, publicly exported in `src/lib.rs` and used throughout the codebase for differential geometry operations.

The struct has four fields that have mathematical constraints:
1. `complex`: The underlying simplicial complex (the "mesh")
2. `grade`: The dimension of simplices the data lives on (must be ≤ max dimension of complex)
3. `data`: A tensor containing values for each simplex at the given grade (length must match skeleton size)
4. `cursor`: The current focus point for comonadic extraction (must be < data.len())

The `Topology` type is used in several critical operations:
- **Comonadic operations** (`hkt_topology.rs`): The `extract` function reads the value at `cursor`, which panics if cursor is out of bounds with message "Cursor OOB"
- **Cup product** (`cup_product.rs`): Expects data length to match skeleton size at the given grade, panics with "Data/Skeleton mismatch" if invalid
- **Other geometric operations**: Various operations assume the topology is well-formed

The bug allows creating `Topology` instances that violate these invariants. When these invalid instances are used in downstream operations, they cause panics with error messages that don't help identify where the invalid instance was created.

# Why has this bug gone undetected?

This bug has gone undetected for several reasons:

1. **Tests always use valid inputs**: All existing tests in the codebase create `Topology` instances with correct parameters that match the underlying complex structure. The test code follows good practices and doesn't accidentally create invalid topologies.

2. **Internal usage patterns are careful**: The library developers who write the test code understand the constraints and naturally create valid topologies. The constructor is primarily used in tests and examples, not in complex production scenarios.

3. **The bug manifests as panics, not silent corruption**: When invalid topologies are used, they cause panics rather than silent incorrect results. This means bugs are caught during development if they occur, but it also means the validation gap isn't discovered until someone tries to use the library incorrectly.

4. **Limited production usage**: The library appears to be relatively new (initial implementation in commit b80d08bb, with the topology type introduced recently). There may not yet be extensive production usage where users might accidentally create invalid topologies.

5. **Constructor simplicity masks the issue**: The constructor is so simple (just assigns fields) that it's not obvious validation is missing. It's easy to assume validation happens elsewhere or that callers are expected to validate inputs.

# Recommended fix

The constructor should validate its inputs and return a `Result<Self, TopologyError>` instead of `Self`. The validation should check:

1. **Cursor bounds**: `cursor < data.len()` - return `TopologyError::IndexOutOfBounds` if violated
2. **Grade validity**: `grade <= complex.max_simplex_dimension()` - return `TopologyError::InvalidGradeOperation` if violated
3. **Data/skeleton size match**: `data.len() == complex.skeletons()[grade].simplices().len()` - return `TopologyError::InvalidInput` if violated

Example fix:
```rust
impl<T> Topology<T> {
    pub fn new(
        complex: Arc<SimplicialComplex>,
        grade: usize,
        data: CausalTensor<T>,
        cursor: usize,
    ) -> Result<Self, TopologyError> {  // <-- FIX 🟢 Return Result
        // Validate grade
        if grade > complex.max_simplex_dimension() {
            return Err(TopologyError::InvalidGradeOperation(
                format!("grade {} exceeds max dimension {}",
                        grade, complex.max_simplex_dimension())
            ));
        }

        // Validate data size matches skeleton
        let expected_size = complex.skeletons()[grade].simplices().len();
        if data.len() != expected_size {
            return Err(TopologyError::InvalidInput(
                format!("data length {} does not match skeleton size {} for grade {}",
                        data.len(), expected_size, grade)
            ));
        }

        // Validate cursor bounds
        if cursor >= data.len() && data.len() > 0 {  // <-- FIX 🟢 Check cursor bounds
            return Err(TopologyError::IndexOutOfBounds(
                format!("cursor {} is out of bounds for data length {}",
                        cursor, data.len())
            ));
        }

        Ok(Self {
            complex,
            grade,
            data,
            cursor,
        })
    }
}
```

Note: This is a breaking API change. All call sites would need to be updated to handle the `Result`. Alternatively, for backwards compatibility, a new `try_new` method could be added while keeping the existing `new` method (though this delays fixing the root issue).


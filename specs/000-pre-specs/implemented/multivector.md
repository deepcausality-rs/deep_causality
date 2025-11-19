# Multivector Specification

# `Multivector<T>` Data Type for Clifford Algebra in Rust

This document specifies the `Multivector<T>` data type, a fundamental structure for representing elements within a Clifford Algebra, tailored for applications in theoretical physics

This specification is derived from the requirements for building advancd
computational causality models, where multivectors are essential for
representing physical states and their transformations under various
symmetry groups.

## 1. Multivector

A multivector is an element of a Clifford Algebra Cl<sub>P,Q,R</sub>(ℝ). The algebra is built upon a vector space of dimension `N = P + Q + R`, with a basis of vectors `{e₁, e₂, ..., eₙ}` satisfying:
- `eᵢ² = +1` for `1 ≤ i ≤ P`
- `eᵢ² = -1` for `P < i ≤ P + Q`
- `eᵢ² = 0`   for `P + Q < i ≤ N`
- `eᵢeⱼ = -eⱼeᵢ` for `i ≠ j`

A multivector `A` is a linear combination of basis blades `eᵢ`:

`A = ∑ᵢ aᵢeᵢ`

where `I` is an ordered subset of `{1, 2, ..., N}` and `aᵢ` are scalar coefficients. The basis blade `eᵢ` is the geometric product of the basis vectors corresponding to the indices in `I`. There are `2ᴺ` basis blades in total.

The `Multivector` data type is represented by a struct that stores these `2ᴺ` coefficients. We use a const generic `P`, `Q`, and `R` to define the signature of the algebra at compile time.

### Rust Definition

```rust
// ============================================================================
// 1. Metric Signature Definition
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Metric {
    /// All basis vectors square to +1.
    /// Signature: (N, 0, 0)
    Euclidean(usize),

    /// All basis vectors square to -1.
    /// Signature: (0, N, 0)
    AntiEuclidean(usize),

    /// Standard Relativistic Spacetime.
    /// Convention: e0^2 = +1, all others -1.
    /// Signature: (1, N-1, 0)
    Minkowski(usize),

    /// Projective Geometric Algebra (PGA).
    /// Convention: e0^2 = 0 (degenerate), others +1.
    /// Often used as R(3,0,1) for 3D graphics.
    /// Signature: (N-1, 0, 1) where the *first* vector is the zero vector.
    PGA(usize),

    /// Explicit generic signature Cl(p, q, r).
    /// Order of generators assumed: 
    /// First p are (+), next q are (-), last r are (0).
    Generic { p: usize, q: usize, r: usize },

    /// Fully arbitrary signature defined by bitmasks.
    /// dim: Total dimension
    /// neg_mask: bit is 1 if e_i^2 = -1
    /// zero_mask: bit is 1 if e_i^2 = 0
    /// (If both bits are 0, default is +1)
    Custom { dim: usize, neg_mask: u64, zero_mask: u64 },
}

impl Metric {
    /// Returns the total dimension of the vector space (N = P + Q + R)
    pub fn dimension(&self) -> usize {
        match self {
            Metric::Euclidean(d) 
            | Metric::AntiEuclidean(d) 
            | Metric::Minkowski(d) 
            | Metric::PGA(d) => *d,
            Metric::Generic { p, q, r } => p + q + r,
            Metric::Custom { dim, .. } => *dim,
        }
    }

    /// Returns the value of the basis vector squared: e_i * e_i
    /// Possible return values: 1, -1, or 0.
    /// i is the 0-indexed generator index (0..N-1).
    pub fn sign_of_sq(&self, i: usize) -> i32 {
        match self {
            Metric::Euclidean(_) => 1,
            
            Metric::AntiEuclidean(_) => -1,
            
            // Minkowski: e0 is Time (+), e1..eN are Space (-)
            Metric::Minkowski(_) => if i == 0 { 1 } else { -1 },
            
            // PGA: e0 is Origin/Horizon (0), e1..eN are Euclidean (+)
            Metric::PGA(_) => if i == 0 { 0 } else { 1 },

            Metric::Generic { p, q, r: _ } => {
                if i < *p {
                    1
                } else if i < p + *q {
                    -1
                } else {
                    0 // The last r dimensions are degenerate
                }
            },

            Metric::Custom { dim: _, neg_mask, zero_mask } => {
                let is_zero = (zero_mask >> i) & 1 == 1;
                if is_zero { return 0; }

                let is_neg = (neg_mask >> i) & 1 == 1;
                if is_neg { return -1; }

                1 // Default to positive
            }
        }
    }
}

// You might need to add this helper to Metric to support Monadic growth.
impl Metric {
    /// Merges two metrics during a Tensor Product (Monad bind).
    /// e.g., Euclidean(2) + Euclidean(2) -> Euclidean(4)
    pub fn tensor_product(&self, other: &Self) -> Self {
        use Metric::*;
        let dim_a = self.dimension();
        let dim_b = other.dimension();
        
        // Simplified merging logic for the demo. 
        // In a full lib, you'd bitwise-merge the signatures of Custom/Generic.
        match (self, other) {
            (Euclidean(a), Euclidean(b)) => Euclidean(a + b),
            (AntiEuclidean(a), AntiEuclidean(b)) => AntiEuclidean(a + b),
            // Mixing signatures or using Minkowski defaults to a Generic construction
            // where we append the B dimensions after A.
            _ => {
                // For Furey's paper (Euclidean/Complex chain), Euclidean is primary.
                // Fallback to Custom or Generic for complex mixes would go here.
                // Returning Euclidean sum as a safe default for "Generic" size growth:
                Euclidean(dim_a + dim_b)
            }
        }
    }
}

```

```rust
// ============================================================================
// 2. The CausalMultiVector Struct
// ============================================================================

/// A MultiVector in Cl(p,q).
/// Data is stored in a flat vector of size 2^n.
/// Indexing is based on bitmaps: index 3 (binary 011) is e1^e2.
#[derive(Debug, Clone, PartialEq)]
pub struct CausalMultiVector<T> {
    data: Vec<T>,
    metric: Metric,
}

#[derive(Debug)]
pub struct CausalMultiVectorError {
    inner: CausalMultiVectorErrorInner,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CausalMultiVectorErrorInner {
    DimensionMismatch { expected: usize, found: usize },
    DataLengthMismatch { expected: usize, found: usize },
    ZeroMagnitude,
    MetricMismatch { left: Metric, right: Metric },
}

impl fmt::Display for CausalMultiVectorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.inner {
            CausalMultiVectorErrorInner::DimensionMismatch { expected, found } => {
                write!(f, "Dimension mismatch: expected {}, found {}", expected, found)
            }
            CausalMultiVectorErrorInner::DataLengthMismatch { expected, found } => {
                write!(f, "Data length mismatch: expected {}, found {}", expected, found)
            }
            CausalMultiVectorErrorInner::ZeroMagnitude => {
                write!(f, "Operation requires non-zero magnitude (e.g., inverse of zero)")
            }
            CausalMultiVectorErrorInner::MetricMismatch { left, right } => {
                write!(f, "Metric mismatch between operands: {:?} vs {:?}", left, right)
            }
        }
    }
}

impl std::error::Error for CausalMultiVectorError {}

impl CausalMultiVectorError {
    pub fn dimension_mismatch(expected: usize, found: usize) -> Self {
        Self { inner: CausalMultiVectorErrorInner::DimensionMismatch { expected, found } }
    }
    
    pub fn data_length_mismatch(expected: usize, found: usize) -> Self {
        Self { inner: CausalMultiVectorErrorInner::DataLengthMismatch { expected, found } }
    }

    pub fn zero_magnitude() -> Self {
        Self { inner: CausalMultiVectorErrorInner::ZeroMagnitude }
    }

    pub fn metric_mismatch(left: Metric, right: Metric) -> Self {
        Self { inner: CausalMultiVectorErrorInner::MetricMismatch { left, right } }
    }
}

impl<T> CausalMultiVector<T> {

  /// Create a new MultiVector from raw coefficients.
    /// Ensure data length is exactly 2^dim.
    pub fn new(data: Vec<T>, metric: Metric) -> Result<Self, CausalMultiVectorError> {
        let dim = metric.dimension();
        let expected_len = 1 << dim;
        if data.len() != expected_len {
            return Err(CausalMultiVectorError::data_length_mismatch(expected_len, data.len()));
        }
        Ok(Self { data, metric })
    }

    /// Create a scalar multivector (grade 0)
    pub fn scalar(val: T, metric: Metric) -> Self {
        let size = 1 << metric.dimension();
        let mut data = vec![T::zero(); size];
        data[0] = val;
        Self { data, metric }
    }

    /// Get a specific component by basis blade bitmap.
    /// e.g., get(3) returns coefficient for e1^e2.
    pub fn get(&self, idx: usize) -> Option<&T> {
        self.data.get(idx)
    }

}
```

## 1.1 Reference algabras 

The `CausalMultiVector` is a **Dynamic Clifford Algebra** implementation. Because the signature is stored in the `Metric` struct (runtime) rather than as const generics (compile-time), a single type `CausalMultiVector<f64>` can express **any non-degenerate Clifford Algebra $Cl(p, q)$**.

If you use complex numbers (`CausalMultiVector<Complex64>`), it can express any complex Clifford Algebra $Cl_{\mathbb{C}}(n)$.


### 1. The "How Many?"
Technically, it can express **countably infinite** algebras, limited only by system RAM (since size scales as $2^N$).

However, strictly based on your `Metric` enum (which supports $+1$ and $-1$ squares via `Custom` and standard variants, but seemingly not $0$ squares*), it can express the entire family of **Non-Degenerate Clifford Algebras**:

$$ Cl(p, q) \quad \text{where } p+q = N $$

*   **Total distinct algebras for dimension $N$:** There are $N+1$ distinct signatures for real algebras of dimension $N$.
*   **Field Flexibility:** By swapping `T` between `f64` and `Complex64`, you double the utility.


#### A. The Base Types
First, alias the underlying field types for readability.

```rust
use deep_causality_num::{ComplexNumber};

// The building blocks
pub type Complex64 = ComplexNumber<f64>;
pub type RealMultiVector = CausalMultiVector<f64>;
pub type ComplexMultiVector = CausalMultiVector<Complex64>;
pub type PGA3D = CausalMultiVector<f64>;
```

#### B. The Division Algebras (Standard Math)
These are the building blocks of numbers.

```rust
impl RealMultiVector {
    /// Cl(0, 1): Isomorphic to Complex Numbers C
    /// Basis: {1, e1} where e1^2 = -1 (acts as i)
    pub fn new_complex_number(real: f64, imag: f64) -> Self {
        let data = vec![real, imag];
        Self::new(data, Metric::NonEuclidean(1)).unwrap()
    }

    /// Cl(0, 2): Isomorphic to Quaternions H
    /// Basis: {1, e1, e2, e12}. e1^2 = e2^2 = -1.
    pub fn new_quaternion(w: f64, x: f64, y: f64, z: f64) -> Self {
        let data = vec![w, x, y, z];
        Self::new(data, Metric::NonEuclidean(2)).unwrap()
    }
    
    /// Cl(1, 0): Isomorphic to Split-Complex (Hyperbolic) Numbers
    /// Basis: {1, e1} where e1^2 = +1 (acts as j)
    pub fn new_split_complex(a: f64, b: f64) -> Self {
        let data = vec![a, b];
        Self::new(data, Metric::Euclidean(1)).unwrap()
    }
}
```

#### C. The Physics Algebras (Spacetime)
These are the algebras used for Relativistic Physics and Electromagnetism.

```rust
impl RealMultiVector {
    /// Cl(3, 0): Algebra of Physical Space (APS) / Pauli Algebra
    /// Used for non-relativistic quantum mechanics (Pauli Matrices).
    pub fn new_aps_vector(data: Vec<f64>) -> Self {
        // Dimension 3, Euclidean Metric (+ + +)
        Self::new(data, Metric::Euclidean(3)).unwrap()
    }

    /// Cl(1, 3): Space-Time Algebra (STA) / Dirac Algebra
    /// Used for Special Relativity and Maxwell's Equations.
    /// Metric: (+ - - -)
    pub fn new_spacetime_vector(data: Vec<f64>) -> Self {
        // Dimension 4, Minkowski Metric (+ - - -)
        Self::new(data, Metric::Minkowski(4)).unwrap()
    }
    
    /// Cl(4, 1): Conformal Geometric Algebra (CGA)
    /// Used for computer graphics and advanced robotics.
    /// 5 Dimensions. Custom Metric needed for (+ + + + -) or specific CGA mapping.
    pub fn new_cga_vector(data: Vec<f64>) -> Self {
        // 5 Dimensions.
        // Mask: We want one dimension to be negative. 
        // Let's say index 4 (the 5th one) is negative. Mask = 10000 binary = 16.
        Self::new(data, Metric::Custom(5, 16)).unwrap()
    }
}
```

#### D. The "Furey / Dixon" Algebras (Particle Physics)
These correspond to the requirements of the paper "An Algebraic Roadmap of Particle Theories".

```rust
impl ComplexMultiVector {
    /// Cl_C(2): Complex Quaternions / Pauli Algebra over C
    /// Isomorphic to Biquaternions.
    pub fn new_complex_pauli(data: Vec<Complex64>) -> Self {
        Self::new(data, Metric::Euclidean(2)).unwrap()
    }

    /// Cl_C(6): The algebra acting on Octonions (via Left Multiplication)
    /// As described in the paper (Eq 21).
    pub fn new_octonion_operator(data: Vec<Complex64>) -> Self {
        Self::new(data, Metric::Euclidean(6)).unwrap()
    }

    /// Cl_C(10): The Grand Unified Algebra (Spin(10))
    /// This is the container for the entire roadmap in the paper.
    pub fn new_gut_algebra(data: Vec<Complex64>) -> Self {
        // 10 Dimensions, 1024 Complex Coefficients.
        Self::new(data, Metric::Euclidean(10)).unwrap()
    }
}
```

#### E. 3D Projective Geometric Algebra
Signature: R(3, 0, 1) -> 3 Euclidean, 0 Negative, 1 Zero
 
```rust
// 3D Projective Geometric Algebra
// Signature: R(3, 0, 1) -> 3 Euclidean, 0 Negative, 1 Zero
pub type PGA3D = CausalMultiVector<f64>;

impl PGA3D {
    pub fn new_point(x: f64, y: f64, z: f64) -> Self {
        // In PGA, points are tri-vectors (dual)
        // p = x*e032 + y*e013 + z*e021 + e123
        // But purely for type definition:
        let data = vec![0.0; 16]; 
        Self::new(data, Metric::PGA(4)).unwrap()
    }

    pub fn translator(x: f64, y: f64, z: f64) -> Self {
        // A translator is 1 + (d/2)e_inf
        // This requires the degenerate metric e_inf^2 = 0
        // ...
    }
}
```


## 2. Operations

### 2.0. Utils


You must also update the **multiplication logic** in your `CausalMultiVector` implementation. When a basis vector squares to 0, the entire term in the geometric product must vanish. Use the calculate_basis_product function to handle this.

Here is the critical update for the `geometric_product` helper function:

```rust
impl<T> CausalMultiVector<T> {
    // ... (previous code) ...

    // Updated helper: Basis Blade Multiplication Logic
    fn calculate_basis_product(a_map: usize, b_map: usize, metric: &Metric) -> (i32, usize) {
        let mut sign = 1;
        
        // 1. Calculate Sign from Swaps (Canonical Reordering)
        // (Same as your existing logic)
        let mut a_temp = a_map;
        let mut swaps = 0;
        for i in 0..metric.dimension() {
            if (b_map >> i) & 1 == 1 {
                let higher_bits_in_a = (a_temp >> (i + 1)).count_ones();
                swaps += higher_bits_in_a;
            }
        }
        if swaps % 2 != 0 { sign *= -1; }

        // 2. Calculate Sign from Metric (Squaring generators)
        // This is the CRITICAL UPDATE for degenerate metrics
        let intersection = a_map & b_map;
        for i in 0..metric.dimension() {
            if (intersection >> i) & 1 == 1 {
                let sq_sign = metric.sign_of_sq(i);
                
                // If any generator in the intersection squares to 0, 
                // the whole term is annihilated.
                if sq_sign == 0 {
                    return (0, 0); 
                }
                
                sign *= sq_sign;
            }
        }

        // 3. Resulting Bitmap (XOR)
        let result_map = a_map ^ b_map;

        (sign, result_map)
    }
}
```

### What new algebras can this express?

With the addition of $R$ (degenerate dimensions), your library is now **Universal**. It can express the standard "Big Three" of Geometric Algebra:

| Algebra Name | Signature | Rust Construction | Utility |
| :--- | :--- | :--- | :--- |
| **Standard 3D** | $Cl(3,0,0)$ | `Metric::Euclidean(3)` | Physics, Normals |
| **Spacetime (STA)** | $Cl(1,3,0)$ | `Metric::Minkowski(4)` | Relativity, Electrodynamics |
| **Projective (PGA)** | $Cl(3,0,1)$ | `Metric::PGA(4)` | **Robotics, Computer Graphics** |



### 2.1. Addition

#### Mathematical Definition

The addition of two multivectors `A` and `B` is defined as the sum of their corresponding coefficients.

If `A = ∑ᵢ aᵢeᵢ` and `B = ∑ᵢ bᵢeᵢ`, then:
`A + B = ∑ᵢ (aᵢ + bᵢ)eᵢ`

#### Rust Implementation

```rust
impl<T> Add for CausalMultiVector<T>
where
    T: Add<Output = T> + Copy + Clone + PartialEq,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        if self.metric != rhs.metric {
            panic!("Dimension mismatch in addition: {:?} vs {:?}", self.metric, rhs.metric);
        }
        let data = self.data.iter()
            .zip(rhs.data.iter())
            .map(|(a, b)| *a + *b)
            .collect();
        
        Self { data, metric: self.metric }
    }
}
```

### 2.2. Subtraction

#### Mathematical Definition

The subtraction of two multivectors `A` and `B` is defined as the difference of their corresponding coefficients.

If `A = ∑ᵢ aᵢeᵢ` and `B = ∑ᵢ bᵢeᵢ`, then:
`A - B = ∑ᵢ (aᵢ - bᵢ)eᵢ`

#### Rust Implementation

```rust
impl<T> Sub for CausalMultiVector<T>
where
    T: Sub<Output = T> + Copy + Clone + PartialEq,
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        if self.metric != rhs.metric {
            panic!("Dimension mismatch in subtraction: {:?} vs {:?}", self.metric, rhs.metric);
        }
        let data = self.data.iter()
            .zip(rhs.data.iter())
            .map(|(a, b)| *a - *b)
            .collect();
        
        Self { data, metric: self.metric }
    }
}
```

### 2.3. Scalar Multiplication and Division

#### Mathematical Definition

The multiplication of a multivector `A` by a scalar `s` results in a new multivector where each coefficient is multiplied by `s`.

If `A = ∑ᵢ aᵢeᵢ`, then:
`sA = ∑ᵢ (saᵢ)eᵢ`

Division is defined similarly.

#### Rust Implementation

```rust
impl<T> Mul<T> for CausalMultiVector<T>
where
    T: Mul<Output = T> + Copy + Clone,
{
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        let data = self.data.iter().map(|a| *a * rhs).collect();
        Self { data, metric: self.metric }
    }
}

impl<T> Div<T> for CausalMultiVector<T>
where
    T: Div<Output = T> + Copy + Clone,
{
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        let data = self.data.iter().map(|a| *a / rhs).collect();
        Self { data, metric: self.metric }
    }
}
```

### 2.4. Geometric Product

#### Mathematical Definition

The geometric product is the fundamental product of Clifford algebra. The product of two multivectors `A = ∑ᵢ aᵢeᵢ` and `B = ∑ⱼ bⱼeⱼ` is given by:

`AB = ∑ᵢⱼ aᵢbⱼ(eᵢeⱼ)`

The product of two basis blades `eᵢ` and `eⱼ` simplifies to `eᵢeⱼ = sign(I, J) * metric(I, J) * e_{I∆J}`, where `∆` is the symmetric difference.
- `metric(I, J)` is the product of `eₖ²` for all `k` in `I ∩ J`.
- `sign(I, J)` is `(-1)ˢ` where `s` is the number of swaps required to order the vectors in `eᵢeⱼ`. `s = ∑_{k∈J} |{l∈I | l > k}|`.

#### Rust Implementation

We implement the geometric product by overriding the `Mul` operator for `Multivector` types.

```rust
impl<T> Mul for CausalMultiVector<T>
where
    T: Num + Copy + Clone + AddAssign + SubAssign,
{
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        if self.metric != rhs.metric {
            panic!("Dimension mismatch in geometric product");
        }
        
        let dim = self.metric.dimension();
        let count = 1 << dim;
        let mut result_data = vec![T::zero(); count];
        
        for i in 0..count {
            if self.data[i].is_zero() { continue; }
            for j in 0..count {
                if rhs.data[j].is_zero() { continue; }
                
                // Use the helper defined in the struct impl
                let (sign, result_idx) = Self::calculate_basis_product(i, j, &self.metric);
                
                if sign != 0 {
                    let val = self.data[i] * rhs.data[j];
                    if sign > 0 {
                        result_data[result_idx] += val;
                    } else {
                        result_data[result_idx] -= val;
                    }
                }
            }
        }
        
        Self { data: result_data, metric: self.metric }
    }
}
```

### 2.5. Outer Product (Wedge Product)

#### Mathematical Definition

The outer product of two multivectors of grades `r` and `s` is the grade `r + s` part of their geometric product.

`A ∧ B = ⟨AB⟩_{r+s}`

For two basis blades `eᵢ` and `eⱼ`:
`eᵢ ∧ eⱼ = sign * e_{I∪J}` if `I ∩ J = ∅`, and `0` otherwise. The sign is the same reordering sign as in the geometric product.

#### Rust Implementation

```rust
impl<T> CausalMultiVector<T>
where
    T: Num + Copy + Clone + AddAssign + SubAssign,
{
    pub fn outer_product(&self, rhs: &Self) -> Self {
        if self.metric != rhs.metric { panic!("Metric mismatch"); }
        
        let dim = self.metric.dimension();
        let count = 1 << dim;
        let mut result_data = vec![T::zero(); count];
        
        for i in 0..count {
            if self.data[i].is_zero() { continue; }
            for j in 0..count {
                if rhs.data[j].is_zero() { continue; }
                
                // Outer product is non-zero only if blades are disjoint
                if (i & j) == 0 {
                    let (sign, result_idx) = Self::calculate_basis_product(i, j, &self.metric);
                    // Note: calculate_basis_product includes metric sign, but for disjoint blades
                    // metric sign is always +1 (empty intersection). 
                    // It only calculates reordering sign.
                    
                    if sign != 0 {
                        let val = self.data[i] * rhs.data[j];
                        if sign > 0 {
                            result_data[result_idx] += val;
                        } else {
                            result_data[result_idx] -= val;
                        }
                    }
                }
            }
        }
        Self { data: result_data, metric: self.metric }
    }
}
```

### 2.6. Inner Product (Left Contraction)

#### Mathematical Definition

The inner product (left contraction) of a grade `r` multivector `A` and a grade `s` multivector `B` is the grade `s - r` part of their geometric product.

`A ⋅ B = ⟨AB⟩_{s-r}`

For two basis blades `eᵢ` and `eⱼ`:
`eᵢ ⋅ eⱼ = sign * metric * e_{I∆J}` if `I ⊆ J`, and `0` otherwise.

#### Rust Implementation

```rust
impl<T> CausalMultiVector<T>
where
    T: Num + Copy + Clone + AddAssign + SubAssign,
{
    pub fn inner_product(&self, rhs: &Self) -> Self {
        if self.metric != rhs.metric { panic!("Metric mismatch"); }
        
        let dim = self.metric.dimension();
        let count = 1 << dim;
        let mut result_data = vec![T::zero(); count];
        
        for i in 0..count {
            if self.data[i].is_zero() { continue; }
            for j in 0..count {
                if rhs.data[j].is_zero() { continue; }
                
                // Left contraction requires I subset J
                if (i & j) == i {
                    let (sign, result_idx) = Self::calculate_basis_product(i, j, &self.metric);
                    
                    if sign != 0 {
                        let val = self.data[i] * rhs.data[j];
                        if sign > 0 {
                            result_data[result_idx] += val;
                        } else {
                            result_data[result_idx] -= val;
                        }
                    }
                }
            }
        }
        Self { data: result_data, metric: self.metric }
    }
}
```

### 2.7. Grade Projection

#### Mathematical Definition

The grade projection operation, `⟨A⟩ₖ`, extracts the grade-`k` part of a multivector `A`. This results in a new multivector containing only the components corresponding to basis blades of grade `k`.

`⟨A⟩ₖ = ∑_{I | |I|=k} aᵢeᵢ`

#### Rust Implementation

```rust
impl<T> CausalMultiVector<T>
where
    T: Num + Copy + Clone,
{
    pub fn grade_projection(&self, k: u32) -> Self {
        let mut result_data = vec![T::zero(); self.data.len()];
        for (i, val) in self.data.iter().enumerate() {
            if i.count_ones() == k {
                result_data[i] = *val;
            }
        }
        Self { data: result_data, metric: self.metric }
    }
}
```

### 2.8. Reversion (Reverse)

#### Mathematical Definition

The reverse of a multivector $A$, denoted as $\tilde{A}$ (or $A^\dagger$), reverses the order of vectors in each basis blade.
For a basis blade $e_A$ of grade $k$, $\tilde{e}_A = (-1)^{k(k-1)/2} e_A$.

$\tilde{A} = \sum_{k=0}^N (-1)^{k(k-1)/2} \langle A \rangle_k$

#### Rust Implementation

```rust
impl<T> CausalMultiVector<T>
where
    T: Num + Copy + Clone + Neg<Output = T>,
{
    pub fn reversion(&self) -> Self {
        let mut result_data = vec![T::zero(); self.data.len()];
        for (i, val) in self.data.iter().enumerate() {
            let grade = i.count_ones();
            let sign_power = (grade * (grade - 1)) / 2;
            if sign_power % 2 == 1 {
                result_data[i] = -(*val);
            } else {
                result_data[i] = *val;
            }
        }
        Self { data: result_data, metric: self.metric }
    }
}
```

### 2.9. Squared Magnitude (Norm Squared)

#### Mathematical Definition

The squared magnitude (or squared norm) of a multivector $A$ is the scalar part of the geometric product of $A$ with its reverse $\tilde{A}$.

$||A||^2 = \langle A \tilde{A} \rangle_0$

#### Rust Implementation

```rust
impl<T> CausalMultiVector<T>
where
    T: Num + Copy + Clone + AddAssign + SubAssign + Neg<Output = T>,
{
    pub fn squared_magnitude(&self) -> T {
        let reverse = self.reversion();
        // We can optimize by only calculating the scalar part of the product
        // But for simplicity/correctness, let's use the full product
        let product = self.clone() * reverse; 
        product.data[0] // Scalar part
    }
}
```

### 2.10. Inverse

#### Mathematical Definition

The inverse of a multivector $A$, denoted $A^{-1}$, satisfies $A A^{-1} = 1$.
For versors, $A^{-1} = \frac{\tilde{A}}{||A||^2}$.

#### Rust Implementation

```rust
impl<T> CausalMultiVector<T>
where
    T: Float + AddAssign + SubAssign + Neg<Output = T>,
{
    pub fn inverse(&self) -> Result<Self, CausalMultiVectorError> {
        let sq_mag = self.squared_magnitude();
        if sq_mag.is_zero() {
            Err(CausalMultiVectorError::zero_magnitude())
        } else {
            let reverse = self.reversion();
            Ok(reverse / sq_mag)
        }
    }
}
```

### 2.11. Pseudoscalar

#### Mathematical Definition

The pseudoscalar $I$ is the highest grade element of the algebra.

$I = e_1 e_2 \dots e_N$

#### Rust Implementation

```rust
impl<T> CausalMultiVector<T>
where
    T: Num + Copy + Clone,
{
    pub fn pseudoscalar(metric: Metric) -> Self {
        let dim = metric.dimension();
        let size = 1 << dim;
        let mut data = vec![T::zero(); size];
        data[size - 1] = T::one();
        Self { data, metric }
    }
}
```

### 2.12. Dual

#### Mathematical Definition

The dual of a multivector $A$, denoted $A^*$, is defined as $A^* = A I^{-1}$.

#### Rust Implementation

```rust
impl<T> CausalMultiVector<T>
where
    T: Float + AddAssign + SubAssign + Neg<Output = T>,
{
    pub fn dual(&self) -> Result<Self, CausalMultiVectorError> {
        let i = Self::pseudoscalar(self.metric);
        // The dual A* = A I^-1.
        // We propagate the error if I is not invertible (e.g. degenerate metric with I^2=0).
        let i_inv = i.inverse()?;
        Ok(self.clone() * i_inv)
    }
}
```

## 3. Higher Kinded Types (HKT)

In the context of a **Clifford Algebra**, the implementations have specific geometric interpretations:
1.  **Applicative (`pure`)**: Lifts a value into a **Scalar** (Grade 0) MultiVector. This is the identity element for the Tensor Product.
2.  **Applicative (`apply`)**: Performs element-wise application. If the function is a Scalar (wrapped via `pure`), it broadcasts across the target vector (scalar multiplication).
3.  **Monad (`bind`)**: This is the most powerful operation. In this context, it implements the **Tensor Product (Kronecker Product)**. It replaces every coefficient of the input algebra with an entire copy of the output algebra, effectively "stacking" dimensions (e.g., $\mathbb{R} \to \mathbb{C} \to \mathbb{H}$).

### 3.0. HKT 


```rust
use deep_causality_haft::*;

pub struct CausalMultiVectorWitness;

impl HKT for CausalMultiVectorWitness {
    // We wrap the MultiVector in the Type
    type Type<A> = CausalMultiVector<A>;
}
```

### 3.1. Funnctor 

```rust

impl Functor<CausalMultiVectorWitness> for CausalMultiVectorWitness {
    fn fmap<A, B, F>(fa: CausalMultiVector<A>, mut f: F) -> CausalMultiVector<B>
    where
        F: FnMut(A) -> B,
        // Constraints for B to be a valid MultiVector component
        B: Clone + Copy + Zero + One + Add<Output=B> + Sub<Output=B> + Mul<Output=B> + Neg<Output=B> + PartialEq,
    {
        let new_data = fa.data.into_iter().map(|x| f(x)).collect();
        CausalMultiVector {
            data: new_data,
            metric: fa.metric,
        }
    }
}
```

### 3.2. Applicative

```rust
impl Applicative<CausalMultiVectorWitness> for CausalMultiVectorWitness {
    /// Lifts a value into a Scalar MultiVector (Dimension 0).
    /// This acts as the unit for the algebra.
    fn pure<T>(value: T) -> CausalMultiVector<T> {
        // Dimension 0 = 2^0 = 1 element.
        CausalMultiVector {
            data: vec![value],
            metric: Metric::Euclidean(0),
        }
    }

    /// Applies a wrapped function to a wrapped value.
    /// Supports broadcasting: if the function is a Scalar (pure), applies to all elements.
    fn apply<A, B, Func>(f_ab: CausalMultiVector<Func>, f_a: CausalMultiVector<A>) -> CausalMultiVector<B>
    where
        Func: FnMut(A) -> B,
        A: Clone,
        // We need bounds on B to construct the result vector
        B: Clone + Copy + Zero + One + Add<Output=B> + Sub<Output=B> + Mul<Output=B> + Neg<Output=B> + PartialEq,
    {
        // Case 1: Broadcast (Scalar Function applied to Vector)
        // e.g. pure(|x| x*2).apply(quaternion)
        if f_ab.data.len() == 1 {
            let mut func = f_ab.data.into_iter().next().unwrap();
            let new_data = f_a.data.into_iter().map(|a| func(a)).collect();
            return CausalMultiVector {
                data: new_data,
                metric: f_a.metric,
            };
        }

        // Case 2: Element-wise (Zip)
        // Metrics must match to have meaningful geometric application
        if f_ab.data.len() != f_a.data.len() {
            panic!("Applicative::apply shape mismatch: {:?} vs {:?}", f_ab.metric, f_a.metric);
        }

        let new_data = f_ab.data.into_iter()
            .zip(f_a.data.into_iter())
            .map(|(mut f, a)| f(a))
            .collect();

        CausalMultiVector {
            data: new_data,
            metric: f_a.metric,
        }
    }
}
```

### 3.3. Monad

```rust
impl Monad<CausalMultiVectorWitness> for CausalMultiVectorWitness {
    /// The Monadic Bind for MultiVectors corresponds to the Tensor Product.
    /// It iterates over the input basis; for every coefficient `a`, it generates
    /// a new algebra `B` via `f(a)`, effectively nesting the vector spaces.
    ///
    /// Spin(10) construction relies on this: R -> C -> H -> O structure implies
    /// Tensor Product chaining.
    fn bind<A, B, Func>(m_a: CausalMultiVector<A>, mut f: Func) -> CausalMultiVector<B>
    where
        Func: FnMut(A) -> CausalMultiVector<B>,
        // Trait bounds required for the inner types of the result
        B: Clone + Copy + Zero + One + Add<Output=B> + Sub<Output=B> + Mul<Output=B> + Neg<Output=B> + PartialEq,
    {
        if m_a.data.is_empty() {
            return CausalMultiVector::new(vec![], Metric::Euclidean(0)).unwrap(); 
        }

        // 1. Probe the function to determine the inner metric structure
        // We clone the first element just to get the shape of the result.
        // (Assuming A is Clone, which implies Copy/Clone usually for scalars).
        // If A isn't strictly Clone in the generic def, this is tricky, 
        // but for standard scalars (f64/Complex), it is.
        // NOTE: For strict safety, we'd run the first iteration separate.
        // Here we assume homogeneous structure returned by f.
        
        // We will flatten the result into a single data vector.
        // This corresponds to the Kronecker Product of the vectors.
        let mut result_data = Vec::new();
        let mut resulting_metric = Metric::Euclidean(0);
        let mut first_run = true;

        for a in m_a.data {
            let inner_mv = f(a);
            
            if first_run {
                // Calculate the combined metric: Outer + Inner
                resulting_metric = m_a.metric.tensor_product(&inner_mv.metric);
                // Reserve memory: OuterSize * InnerSize
                let total_size = m_a.data.len() * inner_mv.data.len(); // Not possible here since m_a.data is moved.
                // (Approximation of reserve)
                result_data.reserve(inner_mv.data.len() * 10); 
                first_run = false;
            }

            // Append the inner vector's data to the flat array
            result_data.extend(inner_mv.data);
        }

        CausalMultiVector {
            data: result_data,
            metric: resulting_metric
        }
    }
}
```

## 4. Misc System Traits


### 4.1. Display 

```rust
impl<T: fmt::Display> fmt::Display for CausalMultiVector<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MV(dim={}) [", self.metric.dimension())?;
        for (i, val) in self.data.iter().enumerate() {
            if i > 0 { write!(f, ", ")?; }
            write!(f, "{}:{}", i, val)?;
        }
        write!(f, "]")
    }
}
```


## 5. Example usage

```rust
use deep_causality_num::{ComplexNumber};

fn main() {
    println!("=== CausalMultiVector for Particle Physics ===");

    // 1. Define the Space
    // Furey uses Cl(2n). Let's do Cl(2) (Pauli Algebra equivalent) for brevity.
    // Dimension 2. Euclidean metric.
    let metric = Metric::Euclidean(2);
    
    // 2. Create Basis Vectors
    // Size is 2^2 = 4 components.
    // Indices: 0=scalar, 1=e1, 2=e2, 3=e1e2
    
    // Create Generator e1
    let mut e1_data = vec![ComplexNumber::new(0.0, 0.0); 4];
    e1_data[1] = ComplexNumber::new(1.0, 0.0); // Index 1 is e1
    let e1 = CausalMultiVector::new(e1_data, metric).unwrap();

    // Create Generator e2
    let mut e2_data = vec![ComplexNumber::new(0.0, 0.0); 4];
    e2_data[2] = ComplexNumber::new(1.0, 0.0); // Index 2 is e2
    let e2 = CausalMultiVector::new(e2_data, metric).unwrap();

    println!("Generator e1: {}", e1);
    println!("Generator e2: {}", e2);

    // 3. Geometric Product
    // e1 * e2 should be e12 (Index 3)
    let e12 = &e1 * &e2;
    println!("Geometric Product e1*e2: {}", e12);
    
    // Verify e2 * e1 = -e12
    let e21 = &e2 * &e1;
    println!("Geometric Product e2*e1: {}", e21);

    // 4. Commutator (Lie Algebra action)
    // [e1, e2] = e1e2 - e2e1 = e12 - (-e12) = 2*e12
    let comm = e1.commutator(&e2).unwrap();
    println!("Commutator [e1, e2]: {}", comm);

    // 5. Complex Coefficients (The Paper uses 'i')
    // Let's create i*e1
    let i_complex = ComplexNumber::new(0.0, 1.0);
    let i_e1 = CausalMultiVectorWitness::fmap(e1.clone(), |c| c * i_complex);
    println!("Complex Generator i*e1: {}", i_e1);

    // 6. Grading (Extracting Bivectors)
    // The commutator result should be pure bivector
    let bivector_part = comm.grade_projection(2);
    println!("Grade 2 part of commutator: {}", bivector_part);
    
    // 7. Reversion (Used for Hermitian Conjugate in paper)
    // (e1e2)† = e2e1 = -e1e2
    let rev = e12.reversion();
    println!("Reversion of e12: {}", rev);
}
```


#### Example: Defining 3D PGA (Projective Geometric Algebra)

PGA is magic for engineering because it represents points, lines, and planes uniformly, and translations are just rotations around a line at infinity ($e_0$).

```rust
// 3D Projective Geometric Algebra
// Signature: R(3, 0, 1) -> 3 Euclidean, 0 Negative, 1 Zero
pub type PGA3D = CausalMultiVector<f64>;

impl PGA3D {
    pub fn new_point(x: f64, y: f64, z: f64) -> Self {
        // In PGA, points are tri-vectors (dual)
        // p = x*e032 + y*e013 + z*e021 + e123
        // But purely for type definition:
        let data = vec![0.0; 16]; 
        Self::new(data, Metric::PGA(4)).unwrap()
    }

    pub fn translator(x: f64, y: f64, z: f64) -> Self {
        // A translator is 1 + (d/2)e_inf
        // This requires the degenerate metric e_inf^2 = 0
        // ...
    }
}
```



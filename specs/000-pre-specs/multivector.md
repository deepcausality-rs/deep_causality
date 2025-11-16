# Multivector Specification

# `Multivector<T>` Data Type for Clifford Algebra in Rust

This document specifies the `Multivector<T>` data type, a fundamental structure for representing elements within a Clifford Algebra, tailored for applications in theoretical physics

This specification is derived from the requirements for building advancd
computational causality models, where multivectors are essential for
representing physical states and their transformations under various
symmetry groups.

## 1. `Multivector<T, const P: usize, const Q: usize, const R: usize>` Data Type

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
use std::ops::{Add, Sub, Mul, Div, Neg, AddAssign, SubAssign, MulAssign, DivAssign, Index, IndexMut};
use num_traits::{Num, Float};

/// The total dimension of the underlying vector space.
const N: usize = P + Q + R;
/// The total number of basis blades in the algebra, which is 2^N.
const COUNT: usize = 1 << N;

/// A multivector in a Clifford Algebra Cl(P, Q, R).
/// T: The numeric type of the coefficients (e.g., f64).
/// P: The number of basis vectors that square to +1.
/// Q: The number of basis vectors that square to -1.
/// R: The number of basis vectors that square to 0.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Multivector<T, const P: usize, const Q: usize, const R: usize>
where
    T: Num + Copy + Clone,
{
    /// The coefficients of the 2^N basis blades.
    /// The index of the array corresponds to the basis blade.
    /// E.g., for N=3, index 5 (binary 101) corresponds to blade e₁e₃.
    pub coefficients: [T; COUNT],
}

impl<T, const P: usize, const Q: usize, const R: usize> Multivector<T, P, Q, R>
where
    T: Num + Copy + Clone,
{
    /// Creates a new multivector with all coefficients set to zero.
    pub fn zero() -> Self {
        Self {
            coefficients: [T::zero(); COUNT],
        }
    }

    /// Creates a new multivector from an array of coefficients.
    pub fn new(coefficients: [T; COUNT]) -> Self {
        Self { coefficients }
    }
}
```

## 2. Operations

### 2.1. Addition

#### Mathematical Definition

The addition of two multivectors `A` and `B` is defined as the sum of their corresponding coefficients.

If `A = ∑ᵢ aᵢeᵢ` and `B = ∑ᵢ bᵢeᵢ`, then:
`A + B = ∑ᵢ (aᵢ + bᵢ)eᵢ`

#### Rust Implementation

```rust
impl<T, const P: usize, const Q: usize, const R: usize> Add for Multivector<T, P, Q, R>
where
    T: Num + Copy + Clone,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut result_coeffs = [T::zero(); COUNT];
        for i in 0..COUNT {
            result_coeffs[i] = self.coefficients[i] + rhs.coefficients[i];
        }
        Self::new(result_coeffs)
    }
}

impl<T, const P: usize, const Q: usize, const R: usize> AddAssign for Multivector<T, P, Q, R>
where
    T: Num + Copy + Clone,
{
    fn add_assign(&mut self, rhs: Self) {
        for i in 0..COUNT {
            self.coefficients[i] = self.coefficients[i] + rhs.coefficients[i];
        }
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
impl<T, const P: usize, const Q: usize, const R: usize> Sub for Multivector<T, P, Q, R>
where
    T: Num + Copy + Clone,
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut result_coeffs = [T::zero(); COUNT];
        for i in 0..COUNT {
            result_coeffs[i] = self.coefficients[i] - rhs.coefficients[i];
        }
        Self::new(result_coeffs)
    }
}

impl<T, const P: usize, const Q: usize, const R: usize> SubAssign for Multivector<T, P, Q, R>
where
    T: Num + Copy + Clone,
{
    fn sub_assign(&mut self, rhs: Self) {
        for i in 0..COUNT {
            self.coefficients[i] = self.coefficients[i] - rhs.coefficients[i];
        }
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
impl<T, const P: usize, const Q: usize, const R: usize> Mul<T> for Multivector<T, P, Q, R>
where
    T: Num + Copy + Clone,
{
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        let mut result_coeffs = [T::zero(); COUNT];
        for i in 0..COUNT {
            result_coeffs[i] = self.coefficients[i] * rhs;
        }
        Self::new(result_coeffs)
    }
}

impl<T, const P: usize, const Q: usize, const R: usize> MulAssign<T> for Multivector<T, P, Q, R>
where
    T: Num + Copy + Clone,
{
    fn mul_assign(&mut self, rhs: T) {
        for i in 0..COUNT {
            self.coefficients[i] = self.coefficients[i] * rhs;
        }
    }
}

impl<T, const P: usize, const Q: usize, const R: usize> Div<T> for Multivector<T, P, Q, R>
where
    T: Num + Copy + Clone,
{
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        let mut result_coeffs = [T::zero(); COUNT];
        for i in 0..COUNT {
            result_coeffs[i] = self.coefficients[i] / rhs;
        }
        Self::new(result_coeffs)
    }
}

impl<T, const P: usize, const Q: usize, const R: usize> DivAssign<T> for Multivector<T, P, Q, R>
where
    T: Num + Copy + Clone,
{
    fn div_assign(&mut self, rhs: T) {
        for i in 0..COUNT {
            self.coefficients[i] = self.coefficients[i] / rhs;
        }
    }
}```

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
impl<T, const P: usize, const Q: usize, const R: usize> Mul for Multivector<T, P, Q, R>
where
    T: Float + AddAssign,
{
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut result = Self::zero();
        let self_coeffs = self.coefficients;
        let rhs_coeffs = rhs.coefficients;

        for i in 0..COUNT {
            if self_coeffs[i].is_zero() {
                continue;
            }
            for j in 0..COUNT {
                if rhs_coeffs[j].is_zero() {
                    continue;
                }

                let i_blade = i;
                let j_blade = j;

                // Calculate the sign from reordering
                let mut temp_j = j_blade;
                let mut swaps = 0;
                for bit_i in 0..N {
                    if (i_blade >> bit_i) & 1 == 1 {
                        swaps += temp_j.count_ones();
                    }
                    temp_j >>= 1;
                }

                let mut sign = if swaps % 2 == 0 { T::one() } else { -T::one() };

                // Calculate the sign from the metric
                let common_blades = i_blade & j_blade;
                for bit in 0..N {
                    if (common_blades >> bit) & 1 == 1 {
                        if bit < P {
                            // e_i^2 = 1, no sign change
                        } else if bit < P + Q {
                            sign = -sign; // e_i^2 = -1
                        } else {
                            sign = T::zero(); // e_i^2 = 0
                            break;
                        }
                    }
                }
                
                if !sign.is_zero() {
                    let result_blade = i_blade ^ j_blade;
                    let value = self_coeffs[i] * rhs_coeffs[j] * sign;
                    result.coefficients[result_blade] = result.coefficients[result_blade] + value;
                }
            }
        }
        result
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
impl<T, const P: usize, const Q: usize, const R: usize> Multivector<T, P, Q, R>
where
    T: Float + AddAssign,
{
    pub fn outer_product(self, rhs: Self) -> Self {
        let mut result = Self::zero();
        let self_coeffs = self.coefficients;
        let rhs_coeffs = rhs.coefficients;
        
        for i in 0..COUNT {
            if self_coeffs[i].is_zero() {
                continue;
            }
            for j in 0..COUNT {
                if rhs_coeffs[j].is_zero() {
                    continue;
                }

                if (i & j) == 0 { // I ∩ J = ∅
                    let i_blade = i;
                    let j_blade = j;

                    // Calculate the sign from reordering
                    let mut temp_j = j_blade;
                    let mut swaps = 0;
                    for bit_i in 0..N {
                        if (i_blade >> bit_i) & 1 == 1 {
                            swaps += temp_j.count_ones();
                        }
                        temp_j >>= 1;
                    }
                    
                    let sign = if swaps % 2 == 0 { T::one() } else { -T::one() };
                    
                    let result_blade = i | j; // I ∪ J
                    let value = self_coeffs[i] * rhs_coeffs[j] * sign;
                    result.coefficients[result_blade] = result.coefficients[result_blade] + value;
                }
            }
        }
        result
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
impl<T, const P: usize, const Q: usize, const R: usize> Multivector<T, P, Q, R>
where
    T: Float + AddAssign,
{
    pub fn inner_product(self, rhs: Self) -> Self {
        let mut result = Self::zero();
        let self_coeffs = self.coefficients;
        let rhs_coeffs = rhs.coefficients;

        for i in 0..COUNT {
            if self_coeffs[i].is_zero() {
                continue;
            }
            for j in 0..COUNT {
                if rhs_coeffs[j].is_zero() {
                    continue;
                }
                
                let i_blade = i;
                let j_blade = j;
                
                if (i_blade & j_blade) == i_blade { // I ⊆ J
                    // Calculate the sign from reordering
                    let mut temp_j = j_blade;
                    let mut swaps = 0;
                    for bit_i in 0..N {
                        if (i_blade >> bit_i) & 1 == 1 {
                            swaps += temp_j.count_ones();
                        }
                        temp_j >>= 1;
                    }

                    let mut sign = if swaps % 2 == 0 { T::one() } else { -T::one() };

                    // Calculate the sign from the metric
                    let common_blades = i_blade & j_blade;
                    for bit in 0..N {
                        if (common_blades >> bit) & 1 == 1 {
                             if bit < P {
                                // e_i^2 = 1
                            } else if bit < P + Q {
                                sign = -sign; // e_i^2 = -1
                            } else {
                                sign = T::zero(); // e_i^2 = 0
                                break;
                            }
                        }
                    }
                    
                    if !sign.is_zero() {
                        let result_blade = i_blade ^ j_blade;
                        let value = self_coeffs[i] * rhs_coeffs[j] * sign;
                        result.coefficients[result_blade] = result.coefficients[result_blade] + value;
                    }
                }
            }
        }
        result
    }
}
```

### 2.7. Grade Projection

#### Mathematical Definition

The grade projection operation, `⟨A⟩ₖ`, extracts the grade-`k` part of a multivector `A`. This results in a new multivector containing only the components corresponding to basis blades of grade `k`.

`⟨A⟩ₖ = ∑_{I | |I|=k} aᵢeᵢ`

#### Rust Implementation

```rust
impl<T, const P: usize, const Q: usize, const R: usize> Multivector<T, P, Q, R>
where
    T: Num + Copy + Clone,
{
    /// Extracts the grade-k part of the multivector.
    pub fn grade(&self, k: u32) -> Self {
        let mut result_coeffs = [T::zero(); COUNT];
        for i in 0..COUNT {
            if i.count_ones() == k {
                result_coeffs[i] = self.coefficients[i];
            }
        }
        Self::new(result_coeffs)
    }
}
```
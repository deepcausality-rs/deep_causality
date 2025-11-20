# `Octonion<F>` Data Type and Operations in Rust

This document specifies the `Octonion<F>` data type, the third of the normed division algebras over the real numbers, following Complex numbers and Quaternions. Octonions are an 8-dimensional extension of quaternions. They are notable for being neither commutative nor associative. 

This specification provides the exact Rust data type, a trait defining its fundamental operations, and concrete implementations. Each operation is first defined mathematically and then translated faithfully into Rust code.

## 1. `OctonionNumber<F>` Trait

This trait defines the essential operations for an octonion, ensuring a consistent API in line with `ComplexNumber<F>`.

### Rust Trait Definition

```rust
use std::ops::{Add, Sub, Mul, Div, Rem, Neg, AddAssign, SubAssign, MulAssign, DivAssign, Sum, Product};
use num_traits::Float;

pub trait OctonionNumber<F>: Sized
where
    F: Float,
    Self: Add<Output = Self>
        + Sub<Output = Self>
        + Mul<Output = Self>
        + Div<Output = Self>
        + Neg<Output = Self>
        + Sum
        + Product
        + PartialEq
        + Copy
        + Clone,
{
    /// Returns the scalar (real) part of the octonion (e₀).
    fn scalar(&self) -> F;

    /// Returns the vector part of the octonion (e₁ to e₇) as a new octonion.
    fn vector(&self) -> Self;

    /// Computes the squared norm (magnitude squared) of the octonion.
    /// For an octonion O = e₀ + e₁i + e₂j + e₃k + e₄l + e₅il + e₆jl + e₇kl,
    /// the squared norm is e₀² + e₁² + e₂² + e₃² + e₄² + e₅² + e₆² + e₇².
    /// This is more efficient than `norm()` if only relative magnitudes are needed.
    fn norm_sqr(&self) -> F;

    /// Computes the norm (magnitude) of the octonion.
    /// This is the square root of the sum of the squares of its components.
    fn norm(&self) -> F;

    /// Computes the conjugate of the octonion.
    /// The conjugate is obtained by negating the seven vector components (e₁ to e₇).
    fn conj(&self) -> Self;

    /// Computes the multiplicative inverse of the octonion.
    /// The inverse is defined as `conj(O) / norm_sqr(O)`.
    /// Returns `None` if the octonion is zero (and thus has no inverse).
    fn inverse(&self) -> Option<Self>;
}
```

## 2. `Octonion<F>` Struct

This struct represents an octonion using its eight components.

### Rust Struct Definition
```rust
/// Represents an octonion with a scalar part (e0) and a 7-dimensional vector part (e1-e7).
///
/// Octonions are an 8-dimensional non-associative number system that extends quaternions.
///
/// # Fields
///
/// * `e0`: The scalar component.
/// * `e1`-`e7`: The seven vector components.
#[derive(Copy, Clone, PartialEq, Default, Debug)]
pub struct Octonion<F>
where
    F: Float,
{
    pub e0: F, // Scalar part
    pub e1: F, // Vector part 1
    pub e2: F, // Vector part 2
    pub e3: F, // Vector part 3
    pub e4: F, // Vector part 4
    pub e5: F, // Vector part 5
    pub e6: F, // Vector part 6
    pub e7: F, // Vector part 7
}

impl<F> Octonion<F>
where
    F: Float,
{
    /// Creates a new octonion from its eight components.
    pub fn new(e0: F, e1: F, e2: F, e3: F, e4: F, e5: F, e6: F, e7: F) -> Self {
        Self { e0, e1, e2, e3, e4, e5, e6, e7 }
    }

    /// Creates an octonion with all components set to zero.
    pub fn zero() -> Self {
        Self {
            e0: F::zero(), e1: F::zero(), e2: F::zero(), e3: F::zero(),
            e4: F::zero(), e5: F::zero(), e6: F::zero(), e7: F::zero(),
        }
    }

    /// Creates the multiplicative identity octonion (1).
    pub fn one() -> Self {
        Self {
            e0: F::one(), e1: F::zero(), e2: F::zero(), e3: F::zero(),
            e4: F::zero(), e5: F::zero(), e6: F::zero(), e7: F::zero(),
        }
    }
}
```

## 3. Operations

### 3.1. Addition and Subtraction

#### Mathematical Definition

Addition and subtraction of octonions are performed component-wise. Given two octonions `A = ∑ aᵢeᵢ` and `B = ∑ bᵢeᵢ`:

`A ± B = ∑ (aᵢ ± bᵢ)eᵢ`

#### Rust Implementation

```rust
impl<F> Add for Octonion<F>
where
    F: Float,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            e0: self.e0 + rhs.e0, e1: self.e1 + rhs.e1, e2: self.e2 + rhs.e2, e3: self.e3 + rhs.e3,
            e4: self.e4 + rhs.e4, e5: self.e5 + rhs.e5, e6: self.e6 + rhs.e6, e7: self.e7 + rhs.e7,
        }
    }
}

impl<F> Sub for Octonion<F>
where
    F: Float,
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            e0: self.e0 - rhs.e0, e1: self.e1 - rhs.e1, e2: self.e2 - rhs.e2, e3: self.e3 - rhs.e3,
            e4: self.e4 - rhs.e4, e5: self.e5 - rhs.e5, e6: self.e6 - rhs.e6, e7: self.e7 - rhs.e7,
        }
    }
}

// Implement AddAssign and SubAssign for completeness
impl<F: Float> AddAssign for Octonion<F> {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl<F: Float> SubAssign for Octonion<F> {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}
```

### 3.2. Negation

#### Mathematical Definition

Negation is performed component-wise. For an octonion `A = ∑ aᵢeᵢ`:

`-A = ∑ (-aᵢ)eᵢ`

#### Rust Implementation

```rust
impl<F> Neg for Octonion<F>
where
    F: Float,
{
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            e0: -self.e0, e1: -self.e1, e2: -self.e2, e3: -self.e3,
            e4: -self.e4, e5: -self.e5, e6: -self.e6, e7: -self.e7,
        }
    }
}
```

### 3.3. Scalar Multiplication and Division

#### Mathematical Definition

Multiplication and division by a scalar `s` are performed component-wise. For an octonion `A = ∑ aᵢeᵢ`:

`sA = ∑ (s * aᵢ)eᵢ`
`A/s = ∑ (aᵢ / s)eᵢ`

#### Rust Implementation
```rust
impl<F> Mul<F> for Octonion<F>
where
    F: Float,
{
    type Output = Self;

    fn mul(self, rhs: F) -> Self::Output {
        Self {
            e0: self.e0 * rhs, e1: self.e1 * rhs, e2: self.e2 * rhs, e3: self.e3 * rhs,
            e4: self.e4 * rhs, e5: self.e5 * rhs, e6: self.e6 * rhs, e7: self.e7 * rhs,
        }
    }
}

impl<F> Div<F> for Octonion<F>
where
    F: Float,
{
    type Output = Self;

    fn div(self, rhs: F) -> Self::Output {
        Self {
            e0: self.e0 / rhs, e1: self.e1 / rhs, e2: self.e2 / rhs, e3: self.e3 / rhs,
            e4: self.e4 / rhs, e5: self.e5 / rhs, e6: self.e6 / rhs, e7: self.e7 / rhs,
        }
    }
}
```

### 3.4. Geometric Product (Multiplication)

#### Mathematical Definition

The octonion product is noncommutative and nonassociative. Given two octonions `A = a₀ + **a**` and `B = b₀ + **b**`, where `a₀` and `b₀` are scalar parts and **a** and **b** are vector parts:

`AB = (a₀b₀ - **a** ⋅ **b**) + (a₀**b** + b₀**a** + **a** × **b**)`

- `**a** ⋅ **b**` is the standard Euclidean dot product of the 7D vector parts.
- `**a** × **b**` is the non-associative octonion cross product, defined by the Fano plane multiplication table:

| ×  | e₁ | e₂ | e₃ | e₄ | e₅ | e₆ | e₇ |
|:---|:---|:---|:---|:---|:---|:---|:---|
| **e₁** | -1 | e₄ | e₇ | -e₂ | e₆ | -e₅ | -e₃ |
| **e₂** | -e₄| -1 | e₅ | e₁ | -e₃ | e₇ | -e₆ |
| **e₃** | -e₇| -e₅| -1 | e₆ | e₂ | -e₄ | e₁ |
| **e₄** | e₂ | -e₁| -e₆| -1 | e₇ | e₃ | -e₅ |
| **e₅** | -e₆| e₃ | -e₂| -e₇| -1 | e₁ | e₄ |
| **e₆** | e₅ | -e₇| e₄ | -e₃| -e₁| -1 | e₂ |
| **e₇** | e₃ | e₆ | -e₁| e₅ | -e₄| -e₂| -1 |

#### Rust Implementation

```rust
impl<F> Mul for Octonion<F>
where
    F: Float,
{
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        // Deconstruct into scalar and vector parts for clarity
        let a0 = self.e0;
        let a1 = self.e1; let a2 = self.e2; let a3 = self.e3; let a4 = self.e4;
        let a5 = self.e5; let a6 = self.e6; let a7 = self.e7;

        let b0 = rhs.e0;
        let b1 = rhs.e1; let b2 = rhs.e2; let b3 = rhs.e3; let b4 = rhs.e4;
        let b5 = rhs.e5; let b6 = rhs.e6; let b7 = rhs.e7;

        // Scalar part: a₀b₀ - a ⋅ b
        let s = a0 * b0 - (a1 * b1 + a2 * b2 + a3 * b3 + a4 * b4 + a5 * b5 + a6 * b6 + a7 * b7);

        // Vector part: a₀b + b₀a + a × b
        let v1 = a0 * b1 + b0 * a1 + (a2 * b4 - a4 * b2 + a3 * b7 - a7 * b3 + a6 * b5 - a5 * b6);
        let v2 = a0 * b2 + b0 * a2 + (a3 * b5 - a5 * b3 + a1 * b4 - b1 * a4 + a7 * b6 - a6 * b7);
        let v3 = a0 * b3 + b0 * a3 + (a1 * b7 - a7 * b1 + a2 * b5 - b2 * a5 + a4 * b6 - a6 * b4);
        let v4 = a0 * b4 + b0 * a4 + (a1 * b2 - b1 * a2 + a3 * b6 - b3 * a6 + a5 * b7 - a7 * b5);
        let v5 = a0 * b5 + b0 * a5 + (a1 * b6 - a6 * b1 + a2 * b3 - b2 * a3 + a4 * b7 - b4 * a7);
        let v6 = a0 * b6 + b0 * a6 + (a1 * b5 - b1 * a5 + a2 * b7 - b2 * a7 + a3 * b4 - b3 * a4);
        let v7 = a0 * b7 + b0 * a7 + (a1 * b3 - b1 * a3 + a2 * b6 - b2 * a6 + a3 * b5 - b5 * a3);

        Self::new(s, v1, v2, v3, v4, v5, v6, v7)
    }
}
```

### 3.5. Division

#### Mathematical Definition

Division of octonion `A` by `B` is defined as multiplication by the inverse. Since the product is non-associative, we define this as right-multiplication:

`A / B = A * B⁻¹` where `B⁻¹ = conj(B) / norm_sqr(B)`

#### Rust Implementation

```rust
impl<F> Div for Octonion<F>
where
    F: Float,
{
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        match rhs.inverse() {
            Some(inv) => self * inv,
            None => {
                // Handle division by zero. Depending on the context,
                // this might return NaN, infinity, or panic.
                // Here, we create an octonion of NaNs.
                let nan = F::nan();
                Self::new(nan, nan, nan, nan, nan, nan, nan, nan)
            }
        }
    }
}
```

## 4. `OctonionNumber<F>` Trait Implementation

Finally, we implement the `OctonionNumber` trait for our `Octonion` struct.

### Rust Implementation

```rust
impl<F> OctonionNumber<F> for Octonion<F>
where
    F: Float,
{
    fn scalar(&self) -> F {
        self.e0
    }

    fn vector(&self) -> Self {
        Self {
            e0: F::zero(),
            e1: self.e1, e2: self.e2, e3: self.e3, e4: self.e4,
            e5: self.e5, e6: self.e6, e7: self.e7,
        }
    }

    fn norm_sqr(&self) -> F {
          self.e0 * self.e0 + self.e1 * self.e1 + self.e2 * self.e2 + self.e3 * self.e3
        + self.e4 * self.e4 + self.e5 * self.e5 + self.e6 * self.e6 + self.e7 * self.e7
    }

    fn norm(&self) -> F {
        self.norm_sqr().sqrt()
    }

    fn conj(&self) -> Self {
        Self {
            e0: self.e0,
            e1: -self.e1, e2: -self.e2, e3: -self.e3, e4: -self.e4,
            e5: -self.e5, e6: -self.e6, e7: -self.e7,
        }
    }

    fn inverse(&self) -> Option<Self> {
        let norm_sq = self.norm_sqr();
        if norm_sq.is_zero() {
            None
        } else {
            Some(self.conj() / norm_sq)
        }
    }
}
```
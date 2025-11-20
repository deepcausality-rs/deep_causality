# `Octonion<F>` Data Type and Operations in Rust

This document specifies the `Octonion<F>` data type, the third of the normed division algebras over the real numbers, following Complex numbers and Quaternions. Octonions are an 8-dimensional extension of quaternions. They are notable for being neither commutative nor associative. 

This specification provides the exact Rust data type, a trait defining its fundamental operations, and concrete implementations. Each operation is first defined mathematically and then translated faithfully into Rust code.

## 1. `OctonionNumber<F>` Trait

This trait defines the essential operations for an octonion, ensuring a consistent API in line with `QuaternionNumber<F>`.

### Rust Trait Definition

```rust
use std::ops::{Add, Sub, Mul, Div, Rem, Neg, AddAssign, SubAssign, MulAssign, DivAssign, Sum, Product};
use crate::prelude::*;

pub trait OctonionNumber<F>: Num + Sized
where
    F: Float,
    Self: Add<Output = Self>
        + Sub<Output = Self>
        + Mul<Output = Self>
        + Div<Output = Self>
        + Rem<Output = Self>
        + Neg<Output = Self>
        + Sum
        + Product
        + PartialEq
        + Copy
        + Clone,
{
    fn conjugate(&self) -> Self;
    fn norm_sqr(&self) -> F;
    fn norm(&self) -> F;
    fn normalize(&self) -> Self;
    fn inverse(&self) -> Self;
    fn dot(&self, other: &Self) -> F;
}
```

## 2. `Octonion<F>` Struct

This struct represents an octonion using its eight components.

### Rust Struct Definition
```rust
use std::ops::{Add, Div, Mul, Neg, Rem, Sub, Sum, Product};

#[derive(Copy, Clone, PartialEq, Default)]
pub struct Octonion<F>
where
    F: Float,
    Self: Add<Output = Self>
        + Sub<Output = Self>
        + Mul<Output = Self>
        + Div<Output = Self>
        + Neg<Output = Self>
        + Rem<Output = Self>
        + Sum
        + Product,
{
    pub s: F,  // Scalar part
    pub e1: F, // Vector part 1
    pub e2: F, // Vector part 2
    pub e3: F, // Vector part 3
    pub e4: F, // Vector part 4
    pub e5: F, // Vector part 5
    pub e6: F, // Vector part 6
    pub e7: F, // Vector part 7
}

// Marker trait to ensure all Num requirements are implemented.
impl<F: Float> Num for Octonion<F> {}
```

## 3. Trait Implementations

### 3.1. Constructors

```rust
impl<F> Octonion<F>
where
    F: Float,
{
    /// Creates a new octonion from its eight components.
    pub fn new(s: F, e1: F, e2: F, e3: F, e4: F, e5: F, e6: F, e7: F) -> Self {
        Self { s, e1, e2, e3, e4, e5, e6, e7 }
    }

    /// Returns the identity octonion (1 + 0e₁ + ... + 0e₇).
    pub fn identity() -> Self {
        Self {
            s: F::one(),
            e1: F::zero(), e2: F::zero(), e3: F::zero(),
            e4: F::zero(), e5: F::zero(), e6: F::zero(), e7: F::zero(),
        }
    }
}
```

### 3.2. Identity Traits (`Zero`, `One`)

```rust
// Zero
impl<F: Float> Zero for Octonion<F> {
    fn zero() -> Self {
        Octonion::new(F::zero(), F::zero(), F::zero(), F::zero(), F::zero(), F::zero(), F::zero(), F::zero())
    }

    fn is_zero(&self) -> bool {
        self.s.is_zero() && self.e1.is_zero() && self.e2.is_zero() && self.e3.is_zero() &&
        self.e4.is_zero() && self.e5.is_zero() && self.e6.is_zero() && self.e7.is_zero()
    }
}

// ConstZero
impl<F: Float + ConstZero> ConstZero for Octonion<F> {
    const ZERO: Self = Octonion {
        s: F::ZERO, e1: F::ZERO, e2: F::ZERO, e3: F::ZERO,
        e4: F::ZERO, e5: F::ZERO, e6: F::ZERO, e7: F::ZERO,
    };
}

// One
impl<F: Float> One for Octonion<F> {
    fn one() -> Self {
        Octonion::new(F::one(), F::zero(), F::zero(), F::zero(), F::zero(), F::zero(), F::zero(), F::zero())
    }

    fn is_one(&self) -> bool {
        self.s.is_one() && self.e1.is_zero() && self.e2.is_zero() && self.e3.is_zero() &&
        self.e4.is_zero() && self.e5.is_zero() && self.e6.is_zero() && self.e7.is_zero()
    }
}

// ConstOne
impl<F: Float + ConstOne + ConstZero> ConstOne for Octonion<F> {
    const ONE: Self = Octonion {
        s: F::ONE,
        e1: F::ZERO, e2: F::ZERO, e3: F::ZERO,
        e4: F::ZERO, e5: F::ZERO, e6: F::ZERO, e7: F::ZERO,
    };
}
```

### 3.3. Arithmetic Traits (`Add`, `Sub`, `Mul`, `Div`, `Rem`)

```rust
// Add
impl<F: Float> Add for Octonion<F> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            s: self.s + rhs.s, e1: self.e1 + rhs.e1, e2: self.e2 + rhs.e2, e3: self.e3 + rhs.e3,
            e4: self.e4 + rhs.e4, e5: self.e5 + rhs.e5, e6: self.e6 + rhs.e6, e7: self.e7 + rhs.e7,
        }
    }
}

// Sub
impl<F: Float> Sub for Octonion<F> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            s: self.s - rhs.s, e1: self.e1 - rhs.e1, e2: self.e2 - rhs.e2, e3: self.e3 - rhs.e3,
            e4: self.e4 - rhs.e4, e5: self.e5 - rhs.e5, e6: self.e6 - rhs.e6, e7: self.e7 - rhs.e7,
        }
    }
}

// Mul (Octonion Product)
impl<F: Float> Mul for Octonion<F> {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        let s = self.s * rhs.s - (self.e1 * rhs.e1 + self.e2 * rhs.e2 + self.e3 * rhs.e3 + self.e4 * rhs.e4 + self.e5 * rhs.e5 + self.e6 * rhs.e6 + self.e7 * rhs.e7);
        let e1 = self.s * rhs.e1 + rhs.s * self.e1 + self.e2 * rhs.e4 - self.e4 * rhs.e2 - self.e3 * rhs.e7 + self.e7 * rhs.e3 + self.e5 * rhs.e6 - self.e6 * rhs.e5;
        let e2 = self.s * rhs.e2 + rhs.s * self.e2 - self.e1 * rhs.e4 + self.e4 * rhs.e1 + self.e3 * rhs.e5 - self.e5 * rhs.e3 - self.e6 * rhs.e7 + self.e7 * rhs.e6;
        let e3 = self.s * rhs.e3 + rhs.s * self.e3 + self.e1 * rhs.e7 - self.e7 * rhs.e1 - self.e2 * rhs.e5 + self.e5 * rhs.e2 + self.e4 * rhs.e6 - self.e6 * rhs.e4;
        let e4 = self.s * rhs.e4 + rhs.s * self.e4 + self.e1 * rhs.e2 - self.e2 * rhs.e1 - self.e3 * rhs.e6 + self.e6 * rhs.e3 - self.e5 * rhs.e7 + self.e7 * rhs.e5;
        let e5 = self.s * rhs.e5 + rhs.s * self.e5 - self.e1 * rhs.e6 + self.e6 * rhs.e1 + self.e2 * rhs.e3 - self.e3 * rhs.e2 + self.e4 * rhs.e7 - self.e7 * rhs.e4;
        let e6 = self.s * rhs.e6 + rhs.s * self.e6 + self.e1 * rhs.e5 - self.e5 * rhs.e1 - self.e2 * rhs.e7 + self.e7 * rhs.e2 - self.e3 * rhs.e4 + self.e4 * rhs.e3;
        let e7 = self.s * rhs.e7 + rhs.s * self.e7 - self.e1 * rhs.e3 + self.e3 * rhs.e1 + self.e2 * rhs.e6 - self.e6 * rhs.e2 - self.e4 * rhs.e5 + self.e5 * rhs.e4;
        Self::new(s, e1, e2, e3, e4, e5, e6, e7)
    }
}

// Mul (Scalar)
impl<F: Float> Mul<F> for Octonion<F> {
    type Output = Self;
    fn mul(self, scalar: F) -> Self {
        Octonion {
            s: self.s * scalar, e1: self.e1 * scalar, e2: self.e2 * scalar, e3: self.e3 * scalar,
            e4: self.e4 * scalar, e5: self.e5 * scalar, e6: self.e6 * scalar, e7: self.e7 * scalar,
        }
    }
}

// Div
#[allow(clippy::suspicious_arithmetic_impl)]
impl<F: Float> Div for Octonion<F> {
    type Output = Self;
    fn div(self, other: Self) -> Self {
        self * other.inverse()
    }
}

// Div (Scalar)
impl<F: Float> Div<F> for Octonion<F> {
    type Output = Self;
    fn div(self, scalar: F) -> Self {
        let inv_scalar = F::one() / scalar;
        self * inv_scalar
    }
}

// Rem
impl<F: Float> Rem for Octonion<F> {
    type Output = Self;
    fn rem(self, _other: Self) -> Self {
        self // Placeholder
    }
}
```

### 3.4. Assignment Traits (`AddAssign`, `SubAssign`, `MulAssign`, `DivAssign`)

```rust
// AddAssign
impl<F: Float + AddAssign> AddAssign for Octonion<F> {
    fn add_assign(&mut self, other: Self) {
        self.s += other.s; self.e1 += other.e1; self.e2 += other.e2; self.e3 += other.e3;
        self.e4 += other.e4; self.e5 += other.e5; self.e6 += other.e6; self.e7 += other.e7;
    }
}

// SubAssign
impl<F: Float + SubAssign> SubAssign for Octonion<F> {
    fn sub_assign(&mut self, other: Self) {
        self.s -= other.s; self.e1 -= other.e1; self.e2 -= other.e2; self.e3 -= other.e3;
        self.e4 -= other.e4; self.e5 -= other.e5; self.e6 -= other.e6; self.e7 -= other.e7;
    }
}

// MulAssign
impl<F: Float + MulAssign> MulAssign for Octonion<F> {
    fn mul_assign(&mut self, other: Self) {
        *self = *self * other;
    }
}

// DivAssign
impl<F: Float + DivAssign> DivAssign for Octonion<F> {
    fn div_assign(&mut self, other: Self) {
        *self = *self / other;
    }
}
```

### 3.5. Iterator Traits (`Sum`, `Product`)

```rust
// Sum
impl<F: Float> Sum for Octonion<F> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Octonion::zero(), |acc, x| acc + x)
    }
}

// Product
impl<F: Float> Product for Octonion<F> {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Octonion::one(), |acc, x| acc * x)
    }
}
```

### 3.6. Other Traits (`Neg`, `PartialOrd`, `Display`, `Debug`)

```rust
// Neg
impl<F: Float> Neg for Octonion<F> {
    type Output = Self;
    fn neg(self) -> Self {
        Octonion {
            s: -self.s, e1: -self.e1, e2: -self.e2, e3: -self.e3,
            e4: -self.e4, e5: -self.e5, e6: -self.e6, e7: -self.e7,
        }
    }
}

// PartialOrd
impl<F: Float> PartialOrd for Octonion<F> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // Lexicographical comparison, not mathematically standard for octonions.
        self.s.partial_cmp(&other.s)
            .and_then(|ord| if ord == Ordering::Equal { self.e1.partial_cmp(&other.e1) } else { Some(ord) })
            .and_then(|ord| if ord == Ordering::Equal { self.e2.partial_cmp(&other.e2) } else { Some(ord) })
            .and_then(|ord| if ord == Ordering::Equal { self.e3.partial_cmp(&other.e3) } else { Some(ord) })
            .and_then(|ord| if ord == Ordering::Equal { self.e4.partial_cmp(&other.e4) } else { Some(ord) })
            .and_then(|ord| if ord == Ordering::Equal { self.e5.partial_cmp(&other.e5) } else { Some(ord) })
            .and_then(|ord| if ord == Ordering::Equal { self.e6.partial_cmp(&other.e6) } else { Some(ord) })
            .and_then(|ord| if ord == Ordering::Equal { self.e7.partial_cmp(&other.e7) } else { Some(ord) })
    }
}

// Display
impl<F: Float + Display> Display for Octonion<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.s)?;
        write!(f, " {} {}e₁", if self.e1 < F::zero() { "-" } else { "+" }, self.e1.abs())?;
        write!(f, " {} {}e₂", if self.e2 < F::zero() { "-" } else { "+" }, self.e2.abs())?;
        write!(f, " {} {}e₃", if self.e3 < F::zero() { "-" } else { "+" }, self.e3.abs())?;
        write!(f, " {} {}e₄", if self.e4 < F::zero() { "-" } else { "+" }, self.e4.abs())?;
        write!(f, " {} {}e₅", if self.e5 < F::zero() { "-" } else { "+" }, self.e5.abs())?;
        write!(f, " {} {}e₆", if self.e6 < F::zero() { "-" } else { "+" }, self.e6.abs())?;
        write!(f, " {} {}e₇", if self.e7 < F::zero() { "-" } else { "+" }, self.e7.abs())
    }
}

// Debug
impl<F: Float + Debug> Debug for Octonion<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Octonion")
            .field("s", &self.s)
            .field("e1", &self.e1).field("e2", &self.e2).field("e3", &self.e3)
            .field("e4", &self.e4).field("e5", &self.e5).field("e6", &self.e6)
            .field("e7", &self.e7).finish()
    }
}
```

### 3.7. Primitive Conversion Traits

```rust
// FromPrimitive
impl<F: Float> FromPrimitive for Octonion<F> {
    fn from_f64(n: f64) -> Option<Self> { F::from(n).map(|f| Self::new(f, F::zero(), F::zero(), F::zero(), F::zero(), F::zero(), F::zero(), F::zero())) }
    // ... Implement for all other primitive types (i8, u8, etc.)
}

// ToPrimitive
impl<F: Float> ToPrimitive for Octonion<F> {
    fn to_f64(&self) -> Option<f64> { self.s.to_f64() }
    // ... Implement for all other primitive types
}

// AsPrimitive
impl<F: Float, T> AsPrimitive<T> for Octonion<F>
where
    F: AsPrimitive<T>,
    T: 'static + Copy + NumCast,
{
    fn as_(self) -> T {
        self.s.as_()
    }
}

// NumCast
impl<F: Float> NumCast for Octonion<F> {
    fn from<T: ToPrimitive>(n: T) -> Option<Self> {
        F::from(n).map(|f| Self::new(f, F::zero(), F::zero(), F::zero(), F::zero(), F::zero(), F::zero(), F::zero()))
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
    fn conjugate(&self) -> Self {
        Self {
            s: self.s,
            e1: -self.e1, e2: -self.e2, e3: -self.e3, e4: -self.e4,
            e5: -self.e5, e6: -self.e6, e7: -self.e7,
        }
    }

    fn norm_sqr(&self) -> F {
          self.s * self.s + self.e1 * self.e1 + self.e2 * self.e2 + self.e3 * self.e3
        + self.e4 * self.e4 + self.e5 * self.e5 + self.e6 * self.e6 + self.e7 * self.e7
    }

    fn norm(&self) -> F {
        self.norm_sqr().sqrt()
    }

    fn normalize(&self) -> Self {
        let n = self.norm();
        if n.is_zero() { *self } else { *self / n }
    }

    fn inverse(&self) -> Self {
        let n_sqr = self.norm_sqr();
        if n_sqr.is_zero() {
            let nan = F::nan();
            Self::new(nan, nan, nan, nan, nan, nan, nan, nan)
        } else {
            self.conjugate() / n_sqr
        }
    }

    fn dot(&self, other: &Self) -> F {
        self.s * other.s + self.e1 * other.e1 + self.e2 * other.e2 + self.e3 * other.e3 +
        self.e4 * other.e4 + self.e5 * other.e5 + self.e6 * other.e6 + self.e7 * other.e7
    }
}
```
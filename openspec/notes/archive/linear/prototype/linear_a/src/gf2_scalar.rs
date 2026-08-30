//! The `Matrix<F2>`-over-a-new-`Field`-impl route that qcl-gaps.md G-01 rejects.
//!
//! This file exists to MEASURE the cost of that route against the real tower,
//! not to endorse it. Everything below is required before `Gf2` satisfies
//! `deep_causality_algebra::Field`.

use core::ops::{Add, Div, DivAssign, Mul, Neg, Sub};
use deep_causality_algebra::{
    AbelianGroup, Annihilating, Associative, Commutative, Distributive, Invertible,
};
use deep_causality_num::{One, Zero};

/// The two-element field 𝔽₂ = {0, 1}, one element per `bool`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Gf2(pub bool);

impl Gf2 {
    pub const ZERO: Self = Gf2(false);
    pub const ONE: Self = Gf2(true);
}

// ---- operators (8 impls) ----

impl Add for Gf2 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Gf2(self.0 ^ rhs.0)
    }
}

impl Sub for Gf2 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Gf2(self.0 ^ rhs.0)
    }
}

impl Neg for Gf2 {
    type Output = Self;
    fn neg(self) -> Self {
        self // char 2: -x == x
    }
}

impl Mul for Gf2 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Gf2(self.0 & rhs.0)
    }
}

impl Div for Gf2 {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        // 1/1 = 1, 0/1 = 0. Division by 0 is undefined in any field; the tower
        // has no fallible division, so this mirrors the float convention of
        // returning a junk value rather than panicking.
        Gf2(self.0 & rhs.0)
    }
}

impl DivAssign for Gf2 {
    fn div_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

// ---- identities (2 impls) ----

impl Zero for Gf2 {
    fn zero() -> Self {
        Gf2(false)
    }
    fn is_zero(&self) -> bool {
        !self.0
    }
}

impl One for Gf2 {
    fn one() -> Self {
        Gf2(true)
    }
    fn is_one(&self) -> bool {
        self.0
    }
}

// ---- law markers (6 impls, all newly mandatory after the tower PR) ----

impl Commutative for Gf2 {}
impl Associative for Gf2 {}
impl Distributive for Gf2 {}
impl Annihilating for Gf2 {}
impl Invertible for Gf2 {}
// `AbelianGroup` is blanket-implemented only for `T: Num + Neg + Clone`
// (deep_causality_algebra/src/algebra/field_real.rs:39). `Gf2` deliberately does
// not implement `Num`, so the blanket does not fire and does not overlap.
impl AbelianGroup for Gf2 {}

#[cfg(test)]
mod tests {
    use super::*;
    use deep_causality_algebra::Field;

    fn assert_is_field<F: Field>() {}

    #[test]
    fn gf2_satisfies_the_real_field_trait() {
        assert_is_field::<Gf2>();
    }

    #[test]
    fn gf2_arithmetic() {
        assert_eq!(Gf2::ONE + Gf2::ONE, Gf2::ZERO);
        assert_eq!(Gf2::ONE * Gf2::ONE, Gf2::ONE);
        assert_eq!(Gf2::ONE / Gf2::ONE, Gf2::ONE);
        assert_eq!(-Gf2::ONE, Gf2::ONE);
    }
}

/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Real;

mod algebra;
mod arithmetic;
mod conjugate_scalar;
mod convert;
mod default;
mod display;
mod from_primitive;
mod identity;
mod real;

/// A dual number `a + b·ε` where the infinitesimal `ε` satisfies `ε² = 0`.
///
/// Dual numbers are the type-based primitive for **forward-mode automatic
/// differentiation**. Evaluating any function composed from the arithmetic and
/// elementary operations on `Dual::variable(x0)` (which is `x0 + 1·ε`) yields
/// `f(x0)` in the real part and `f'(x0)` in the `ε` part, exact to machine
/// precision — the derivative falls out of the trait impls by the chain rule.
///
/// `Dual<T>` is built over `T: Real` (the analytic real-scalar trait), not
/// `RealField`: a dual's component needs the elementary functions but never a field
/// inverse. `Dual<T>` itself implements [`Real`](crate::Real) — so a dual is a
/// first-class analytic scalar that drops into any `Real`-generic code and **nests**
/// (`Dual<Dual<T>>` gives second derivatives) — but it does **not** implement
/// [`Field`](deep_causality_algebra::Field)/[`RealField`](deep_causality_algebra::RealField), because `ε` is a zero
/// divisor (`ε·ε = 0`) and has no multiplicative inverse.
///
/// # No struct-level bound
///
/// The struct is generic over `T` and carries **no** bound, matching `Complex`, `Quaternion`,
/// `CausalTensor` and `CausalMultiVector`. The struct says what may be *stored*; the impls say what
/// may be *computed*. Every arithmetic, analytic and algebra-tower impl names `T: Real` itself, so
/// `Dual<f32>` and `Dual<f64>` behave exactly as before, and the nesting above is unaffected: it
/// rests on `impl<T: Real + Div<Output = T>> Real for Dual<T>`, which keeps its own bound.
///
/// The bound had to go for the functor layer to exist at all. A struct bound is a well-formedness
/// obligation on the *type*, so it propagates into the HKT projection `type Type<T> = Dual<T>` and
/// makes `Dual<()>` and `Dual<(A, B)>` ill-formed — `Real`'s domain is neither closed under `×`
/// nor possessed of the unit object, so `unit` and `zip` had no well-formed return type. See
/// [`DualWitness`](crate::DualWitness).
///
/// # Examples
///
/// ```
/// use deep_causality_num_dual::Dual;
///
/// // f(x) = x³ + 2x, evaluated with its derivative at x = 3.
/// let x = Dual::variable(3.0_f64);
/// let y = x * x * x + x + x;
/// assert_eq!(y.value(), 27.0 + 6.0); // 3³ + 2·3 = 33
/// assert_eq!(y.derivative(), 27.0 + 2.0); // 3·3² + 2 = 29
/// ```
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub struct Dual<T> {
    /// The real part `a` — the function value.
    pub re: T,
    /// The infinitesimal coefficient `b` — the derivative carried in the `ε` channel.
    pub du: T,
}

impl<T: Real> Dual<T> {
    /// Constructs a dual number `re + du·ε` from both components.
    #[inline]
    pub fn new(re: T, du: T) -> Self {
        Self { re, du }
    }

    /// Constructs a constant `re + 0·ε` (a value with zero derivative).
    #[inline]
    pub fn constant(re: T) -> Self {
        Self { re, du: T::zero() }
    }

    /// Constructs the differentiation seed `re + 1·ε` (the independent variable).
    #[inline]
    pub fn variable(re: T) -> Self {
        Self { re, du: T::one() }
    }

    /// Returns the real part `a` — the function value `f(x0)`.
    #[inline]
    pub fn value(&self) -> T {
        self.re
    }

    /// Returns the infinitesimal coefficient `b` — the derivative `f'(x0)`.
    #[inline]
    pub fn derivative(&self) -> T {
        self.du
    }
}

/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Numeric integration as a **Layer-2 operator over functions** — the complement of
//! the `autodiff` surface, which sits right beside it.
//!
//! Integration is *not* the mirror of the [`Dual`](crate::Dual) differentiation type
//! and cannot be: `∫` is a non-local functional over an interval, has no chain rule,
//! and (by Liouville) is not closed in the elementary functions, so there is no
//! "anti-dual" number whose arithmetic accumulates an integral. The faithful
//! realization is an operator that *consumes a function*:
//!
//! - [`Integrator`](integrator::Integrator) — a stepper for `y' = f(y)`, with
//!   [`Euler`](euler::Euler) and [`Rk4`](rk4::Rk4) implementations, generic over any
//!   module-valued state (`Add` + scalar `Mul`), so accuracy is a one-word type swap.
//! - [`quadrature`](quadrature::quadrature) — composite Simpson over a closed-form
//!   integrand. Being generic over [`Real`](crate::Real), it runs over `Dual` too,
//!   giving differentiate-under-the-integral (the Leibniz bridge) for free: the real
//!   part is `∫f(x,θ)dx`, the `ε` part is `d/dθ ∫f(x,θ)dx`.
//!
//! Differentiation (Layer-1, a functor on the scalar) and integration (Layer-2, an
//! operator on functions) meet via Leibniz, not as dual types.

pub mod euler;
pub mod integrator;
pub mod quadrature;
pub mod rk4;

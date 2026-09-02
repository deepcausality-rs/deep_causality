/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The decision form: what every QCL check returns.
//!
//! The four consumers of the quantum crate do not share a computation. They share a way of
//! deciding, and the crate had already shipped its canonical shape in `CommutatorCheck`: a
//! measured quantity, the threshold it was compared against, their ratio, and a verdict, with the
//! report counting how many items were examined. [`Check`] and [`CheckReport`] generalise that
//! pair over the item identifier, and [`Tolerance`] names the policies the thresholds come from.
//!
//! Two obligations follow from the shape and are enforced here rather than at each call site.
//! **A boolean is never the return type**: `worst_margin() == Some(0.87)` states an acceptance and
//! its remaining headroom, and `true` states neither. **The count is not decoration**: a check
//! that examined nothing has not agreed with you, and a report of zero items reads as vacuous,
//! distinguishable from an acceptance that looked at something.

pub(crate) mod check;
pub(crate) mod tolerance;

pub use check::*;
pub use tolerance::*;

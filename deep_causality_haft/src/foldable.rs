/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::HKT;

/// Foldable: Abstracts over types that can be reduced to a single summary value.
///
/// Generic over the HKT witness `F`.
pub trait Foldable<F: HKT> {
    /// Reduces the elements of the structure to a single value by applying a function.
    ///
    /// This is equivalent to a left-fold (foldl) operation.
    fn fold<A, B, Func>(fa: F::Type<A>, init: B, f: Func) -> B
    where
        Func: FnMut(B, A) -> B;
}

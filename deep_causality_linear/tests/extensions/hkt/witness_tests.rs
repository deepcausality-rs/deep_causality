/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! What the witnesses are at runtime.
//!
//! That each witness *projects* to its container is settled by the build, so it is pinned in
//! `src/traits/tower_pins.rs` rather than asserted here — a test of it would pass by compiling and
//! report nothing the build had not already refused.

use deep_causality_linear::{CsrMatrixWitness, DenseMatrixWitness, DenseVectorWitness};

#[test]
fn test_the_witnesses_are_zero_sized_and_defaultable() {
    // A witness is a stand-in for a type constructor, so it carries no data.
    assert_eq!(core::mem::size_of::<DenseMatrixWitness>(), 0);
    assert_eq!(core::mem::size_of::<DenseVectorWitness>(), 0);
    assert_eq!(core::mem::size_of::<CsrMatrixWitness>(), 0);
    let _ = DenseMatrixWitness;
    let _ = DenseVectorWitness;
}

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
fn test_the_witnesses_are_zero_sized() {
    // A witness is a stand-in for a type constructor, so it carries no data.
    assert_eq!(core::mem::size_of::<DenseMatrixWitness>(), 0);
    assert_eq!(core::mem::size_of::<DenseVectorWitness>(), 0);
    assert_eq!(core::mem::size_of::<CsrMatrixWitness>(), 0);
}

#[test]
fn test_the_witnesses_are_defaultable() {
    // `Default` has to be *invoked* to be covered. The earlier form of this test wrote
    // `let _ = DenseMatrixWitness;`, a unit-struct literal that needs no `Default` impl at all,
    // so deleting `Default` from the derives left it green.
    // Routed through a `Default`-bounded generic. A bare `Witness::default()` would catch a
    // missing impl just as well, since it resolves through the trait; the generic form is here
    // because clippy's `default_constructed_unit_structs` fires on the direct call and suggests
    // the bare literal, which would *not* catch it.
    fn defaulted<T: Default>() -> T {
        T::default()
    }
    assert_eq!(defaulted::<DenseMatrixWitness>(), DenseMatrixWitness);
    assert_eq!(defaulted::<DenseVectorWitness>(), DenseVectorWitness);
    assert_eq!(defaulted::<CsrMatrixWitness>(), CsrMatrixWitness);
}

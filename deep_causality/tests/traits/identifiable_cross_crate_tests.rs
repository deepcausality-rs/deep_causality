/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `Identifiable` lives in `deep_causality_core` because it is implemented on both sides of the
//! context/causal split. This is the only crate that can see both, so the cross-crate assertion
//! belongs here: one bound, two crates, no conversion.

use deep_causality::utils_test::test_utils::get_test_causaloid_deterministic_true;
use deep_causality_context::utils_test::test_utils::get_base_context;
use deep_causality_context::{BaseContextoid, ContextoidType, Root};
use deep_causality_core::{Identifiable, IdentificationValue};

/// One bound, satisfied by implementors from either crate.
fn read_id<T: Identifiable>(item: &T) -> IdentificationValue {
    item.id()
}

#[test]
fn test_identifiable_spans_context_and_causal_crates() {
    // Context side: a Contextoid from deep_causality_context.
    let contextoid = BaseContextoid::new(7, ContextoidType::Root(Root::new(7)));

    // Causal side: a Causaloid from deep_causality.
    let causaloid = get_test_causaloid_deterministic_true();

    assert_eq!(read_id(&contextoid), 7);
    assert_eq!(read_id(&causaloid), causaloid.id());
}

#[test]
fn test_context_itself_is_identifiable_through_core() {
    let context = get_base_context();
    assert_eq!(read_id(&context), context.id());
}

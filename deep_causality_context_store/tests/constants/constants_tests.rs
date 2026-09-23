/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The expected value 1 is the first record version, fixed by the crate's own contract (a
//! snapshot written under 1 restores under 1).
//!
//! Corner cases (rows A to K): all n/a (a constant has no inputs).
use deep_causality_context_store::RECORD_VERSION;

#[test]
fn test_record_version_is_the_first() {
    assert_eq!(RECORD_VERSION, 1);
}

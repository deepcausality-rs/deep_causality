/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The gate builder: verdicts. The printed line format is exercised by every example's
//! recorded output; these tests pin the boolean contract.

use deep_causality_cfd::Gates;

#[test]
fn all_passing_gates_finish_true() {
    let ok = Gates::new("unit")
        .gate("first", true, "detail a".to_string())
        .gate("second", true, "detail b".to_string())
        .finish();
    assert!(ok);
}

#[test]
fn one_failing_gate_finishes_false() {
    let ok = Gates::new("unit")
        .gate("first", true, "detail".to_string())
        .gate("broken", false, "detail".to_string())
        .finish();
    assert!(!ok);
}

#[test]
fn an_empty_gate_set_passes_vacuously() {
    assert!(Gates::new("unit").finish());
}

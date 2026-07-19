/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::FlowBranch;

#[test]
fn test_flow_branch_variants_distinct() {
    assert_ne!(FlowBranch::Subsonic, FlowBranch::Supersonic);
    assert_eq!(FlowBranch::Subsonic, FlowBranch::Subsonic);
}

#[test]
fn test_flow_branch_copy_and_debug() {
    let b = FlowBranch::Supersonic;
    let c = b; // Copy
    assert_eq!(b, c);
    assert!(!format!("{b:?}").is_empty());
}

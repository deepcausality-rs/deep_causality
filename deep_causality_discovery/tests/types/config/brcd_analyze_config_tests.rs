/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_discovery::BrcdAnalyzeConfig;

#[test]
fn test_new_and_getter() {
    let c = BrcdAnalyzeConfig::new(7);
    assert_eq!(c.top_k(), 7);
}

#[test]
fn test_default_top_k() {
    assert_eq!(BrcdAnalyzeConfig::default().top_k(), 5);
}

#[test]
fn test_display() {
    let s = format!("{}", BrcdAnalyzeConfig::new(3));
    assert!(s.contains("BrcdAnalyzeConfig"));
    assert!(s.contains("top_k"));
}

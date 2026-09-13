/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_quantum::SemanticsPath;

#[test]
fn test_names() {
    assert_eq!(SemanticsPath::Exact.name(), "exact");
    assert_eq!(SemanticsPath::Numeric.name(), "numeric");
    assert_ne!(SemanticsPath::Exact, SemanticsPath::Numeric);
}

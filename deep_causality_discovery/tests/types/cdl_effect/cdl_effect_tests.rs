/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_discovery::CdlError;
use deep_causality_discovery::{CdlBuilder, CdlEffect};
use deep_causality_haft::LogSize;

#[test]
fn test_cdl_builder_pure() {
    let eff = CdlBuilder::pure(42);
    assert!(eff.inner.is_ok());
    assert_eq!(eff.inner.unwrap(), 42);
    assert!(eff.warnings.is_empty());
}

#[test]
fn test_cdl_bind_success() {
    let eff = CdlBuilder::pure(10);
    // Use .bind() method on struct
    let res = eff.bind(|x| CdlBuilder::pure(x * 2));
    assert_eq!(res.inner.unwrap(), 20);
}

#[test]
fn test_cdl_bind_failure() {
    let eff: CdlEffect<i32> = CdlEffect {
        inner: Err(CdlError::AnalyzeError(
            deep_causality_discovery::AnalyzeError::AnalysisFailed("Test".into()),
        )),
        warnings: Default::default(),
    };

    // Binding on error should verify skip
    let res = eff.bind(|x| CdlBuilder::pure(x * 2));
    assert!(res.inner.is_err());
}

#[test]
fn test_warning_accumulation() {
    use deep_causality_haft::LogAddEntry;

    // First effect has a warning
    let mut eff1 = CdlBuilder::pure(1);
    eff1.warnings.add_entry("Warning 1");

    // Bind function produces effect with another warning
    let res = eff1.bind(|x| {
        let mut eff2 = CdlBuilder::pure(x + 1);
        eff2.warnings.add_entry("Warning 2");
        eff2
    });

    assert_eq!(res.inner.unwrap(), 2);
    assert_eq!(res.warnings.len(), 2);
    // Since Vec append wraps, order might depend on impl. usually append adds to end.
    // Check contents logic if needed, but len is good proxy.
}

#[test]
fn test_print_results() {
    // Just ensure it satisfies Display logic and doesn't panic
    let eff = CdlBuilder::pure("Success");
    eff.print_results(); // stdout capture? just running it.
}

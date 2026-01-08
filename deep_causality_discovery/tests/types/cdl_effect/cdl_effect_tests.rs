/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
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
}

#[test]
fn test_fmap() {
    use deep_causality_discovery::{CdlEffectWitness, CdlWarningLog};
    use deep_causality_haft::Functor;

    // Functor instance is CdlEffectWitness<CdlError, CdlWarningLog>

    let eff = CdlBuilder::pure(10);
    let mapped = CdlEffectWitness::<CdlError, CdlWarningLog>::fmap(eff, |x| x * 2);

    assert_eq!(mapped.inner.unwrap(), 20);
    assert!(mapped.warnings.is_empty());
}

#[test]
fn test_applicative_apply() {
    use deep_causality_discovery::{CdlEffectWitness, CdlWarningLog};
    use deep_causality_haft::{Applicative, LogAddEntry};

    let mut eff_fn = CdlBuilder::pure(|x: i32| x + 10);
    eff_fn.warnings.add_entry("FnWarning");

    let mut eff_val = CdlBuilder::pure(5);
    eff_val.warnings.add_entry("ValWarning");

    let applied = CdlEffectWitness::<CdlError, CdlWarningLog>::apply(eff_fn, eff_val);

    assert_eq!(applied.inner.unwrap(), 15);
    assert_eq!(applied.warnings.len(), 2); // Warnings combined
}

#[test]
fn test_print_results() {
    // Just ensure it satisfies Display logic and doesn't panic
    let eff = CdlBuilder::pure("Success");
    eff.print_results();

    let err_eff: CdlEffect<i32> = CdlEffect {
        inner: Err(CdlError::AnalyzeError(
            deep_causality_discovery::AnalyzeError::AnalysisFailed("Fail".into()),
        )),
        warnings: Default::default(),
    };
    err_eff.print_results();
}

#[test]
fn test_print_warnings() {
    use deep_causality_haft::LogAddEntry;
    let mut eff = CdlBuilder::pure(42);
    eff.warnings.add_entry("Beware");
    eff.print_warnings();
}

#[test]
fn test_clone_debug() {
    let eff = CdlBuilder::pure(42);
    let cloned = eff.clone();
    assert_eq!(cloned.inner.unwrap(), 42);

    let output = format!("{:?}", eff);
    assert!(output.contains("CdlEffect"));
    assert!(output.contains("Ok(42)"));
}

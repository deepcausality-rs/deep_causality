/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algorithms::brcd::brcd_error::{BrcdError, BrcdErrorEnum};
use std::error::Error;

/// Every failure case, including both parameterized variants.
fn all_variants() -> Vec<BrcdErrorEnum> {
    use BrcdErrorEnum::*;
    vec![
        EmptyData,
        DimensionMismatch,
        NotACpdag,
        NotAcyclic,
        ClassTooLarge { bound: 1024 },
        SingularSystem,
        InvalidTransformDomain,
        YeojohnsonUnsupported,
        ZeroCardinality,
        NonPositiveConcentration,
        StateOutOfRange,
        NodeOutOfBounds,
        ConfigSpaceTooLarge { edges: 30 },
    ]
}

#[test]
fn every_enum_variant_renders_a_nonempty_message() {
    for v in all_variants() {
        assert!(!format!("{v}").is_empty(), "empty display for {v:?}");
    }
}

#[test]
fn outer_error_wraps_the_inner_message() {
    let err = BrcdError(BrcdErrorEnum::EmptyData);
    let text = format!("{err}");
    assert!(text.starts_with("BRCD error: "));
    assert!(text.contains("no observations were supplied"));
}

#[test]
fn parameterized_variants_print_their_payload() {
    let class = format!("{}", BrcdErrorEnum::ClassTooLarge { bound: 1024 });
    assert!(class.contains("1024"));
    let space = format!("{}", BrcdErrorEnum::ConfigSpaceTooLarge { edges: 30 });
    assert!(space.contains("30"));
}

#[test]
fn kind_returns_the_inner_case() {
    let err = BrcdError(BrcdErrorEnum::NotAcyclic);
    assert_eq!(err.kind(), &BrcdErrorEnum::NotAcyclic);
}

#[test]
fn errors_are_cloneable_and_comparable() {
    let a = BrcdError(BrcdErrorEnum::ClassTooLarge { bound: 8 });
    assert_eq!(a, a.clone());
    assert_ne!(a, BrcdError(BrcdErrorEnum::ClassTooLarge { bound: 9 }));
    assert_ne!(
        BrcdError(BrcdErrorEnum::EmptyData),
        BrcdError(BrcdErrorEnum::DimensionMismatch)
    );
}

#[test]
fn implements_std_error_with_no_source() {
    let err = BrcdError(BrcdErrorEnum::SingularSystem);
    let dyn_err: &dyn Error = &err;
    assert!(dyn_err.source().is_none());
    assert!(!format!("{err:?}").is_empty());
}

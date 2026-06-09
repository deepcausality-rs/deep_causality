/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algorithms::brcd::BrcdResult;
use deep_causality_algorithms::surd::SurdResult;

/// The algorithm-specific result of a CDL discovery run.
///
/// A closed enum (no dynamic dispatch) that lets the SURD and BRCD sub-pipelines
/// converge on one analysis/finalize tail. Each `*_analyze` step wraps its
/// concrete result into the matching variant; the report and any future
/// consumer dispatch by exhaustive match, so adding an algorithm is a
/// compile-checked change.
///
/// Only `Debug` is derived: `SurdResult<T>` is neither `Clone` nor `PartialEq`,
/// and the converged state (`WithAnalysis`) and `CdlReport` are `Debug`-only, so
/// no stronger bound is needed.
///
/// The `Surd` payload is boxed because `SurdResult<T>` (many state maps) is far
/// larger than `BrcdResult<T>`; boxing keeps the enum's variants balanced in size.
#[derive(Debug)]
pub enum CdlDiscoveryOutcome<T> {
    /// A SURD synergistic/unique/redundant decomposition.
    Surd(Box<SurdResult<T>>),
    /// A BRCD ranked-candidate root-cause posterior.
    Brcd(BrcdResult<T>),
}

/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::cdl_effect::CdlEffectWitness;
use crate::{CDL, CdlConfig, CdlEffect, CdlError, CdlWarningLog, NoData};
use deep_causality_haft::{Effect3, Pure};

/// Entry point for constructing and running a CDL pipeline.
///
/// `CdlBuilder` connects the effect system to the [`CdlEffectWitness`] by fixing the
/// error and warning-log types via [`Effect3`]. It also provides the two surface
/// constructors used by callers: [`CdlBuilder::pure`] to lift a value into the effect
/// context, and [`CdlBuilder::build`] to seed a fresh pipeline in the [`NoData`] state.
pub struct CdlBuilder;

// Effect3: fix the Error and Warning types for the system.
impl Effect3 for CdlBuilder {
    type Fixed1 = CdlError;
    type Fixed2 = CdlWarningLog;
    type HktWitness = CdlEffectWitness<Self::Fixed1, Self::Fixed2>;
}

impl CdlBuilder {
    /// Lifts a value into the `CdlEffect` context (Pure).
    pub fn pure<T>(value: T) -> CdlEffect<T> {
        CdlEffectWitness::<CdlError, CdlWarningLog>::pure(value)
    }

    /// Seeds a fresh pipeline in the `NoData` state with the default configuration.
    pub fn build() -> CdlEffect<CDL<NoData>> {
        Self::pure(CDL {
            state: NoData,
            config: CdlConfig::default(),
        })
    }
}

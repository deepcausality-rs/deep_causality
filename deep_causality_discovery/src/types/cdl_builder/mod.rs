/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::cdl::{BrcdConfigured, SurdConfigured};
use crate::types::cdl_effect::CdlEffectWitness;
use crate::{BrcdLoaderConfig, CDL, CdlEffect, CdlError, CdlWarningLog, SurdLoaderConfig};
use deep_causality_haft::{Effect3, Pure};

/// Entry point for constructing and running a CDL pipeline.
///
/// `CdlBuilder` connects the effect system to the [`CdlEffectWitness`] by fixing the
/// error and warning-log types via [`Effect3`]. It seeds a sub-pipeline from a run config
/// built by [`crate::CdlConfigBuilder`]: [`CdlBuilder::build_surd`] for the SURD
/// sub-pipeline, [`CdlBuilder::build_brcd`] for the BRCD sub-pipeline.
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

    /// Seeds the SURD sub-pipeline from a SURD run config. The whole pipeline runs at
    /// the config's precision `T`, so no turbofish is needed downstream.
    pub fn build_surd<T: Clone>(config: &SurdLoaderConfig<T>) -> CdlEffect<CDL<SurdConfigured<T>>> {
        Self::pure(CDL {
            state: SurdConfigured {
                config: config.clone(),
            },
        })
    }

    /// Seeds the BRCD sub-pipeline from a BRCD run config. The whole pipeline runs at
    /// the config's precision `T`, so no turbofish is needed downstream.
    pub fn build_brcd<T: Clone>(config: &BrcdLoaderConfig<T>) -> CdlEffect<CDL<BrcdConfigured<T>>> {
        Self::pure(CDL {
            state: BrcdConfigured {
                config: config.clone(),
            },
        })
    }
}

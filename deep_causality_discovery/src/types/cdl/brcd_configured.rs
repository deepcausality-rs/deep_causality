/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::cdl::{BrcdConfigured, BrcdLoaded};
use crate::types::data_loader::brcd::BrcdDataLoader;
use crate::{CDL, CdlBuilder, CdlEffect, CdlError, Precision};
use deep_causality_num::ToPrimitive;

// BRCD entry state: load the bundle named by the carried run config.
impl<T: Precision + ToPrimitive> CDL<BrcdConfigured<T>> {
    /// Loads the two datasets and optional CPDAG named by the run config into a
    /// `BrcdInput` bundle, inside the pipeline. A loading failure surfaces as a
    /// `CdlError`.
    pub fn brcd_load_input(self) -> CdlEffect<CDL<BrcdLoaded<T>>> {
        let dataset_path = format!(
            "normal: {} | anomalous: {}",
            self.state.config.normal_path(),
            self.state.config.anomalous_path()
        );
        match BrcdDataLoader::load(&self.state.config) {
            Ok(input) => CdlBuilder::pure(CDL {
                state: BrcdLoaded {
                    input,
                    dataset_path,
                },
            }),
            Err(e) => CdlEffect {
                inner: Err(CdlError::from(e)),
                warnings: Default::default(),
            },
        }
    }
}

// Fluent stage method on the effect.
impl<T: Precision + ToPrimitive> CdlEffect<CDL<BrcdConfigured<T>>> {
    /// See [`CDL::<BrcdConfigured<T>>::brcd_load_input`].
    pub fn brcd_load_input(self) -> CdlEffect<CDL<BrcdLoaded<T>>> {
        self.and_then(|cdl| cdl.brcd_load_input())
    }
}

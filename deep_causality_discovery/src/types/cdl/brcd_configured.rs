/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::cdl::{BrcdConfigured, BrcdLoaded};
use crate::types::data_loader::brcd::BrcdDataLoader;
use crate::{CDL, CdlBuilder, CdlEffect, CdlError, Precision};

// BRCD entry state: load the bundle named by the carried run config.
impl<T: Precision> CDL<BrcdConfigured<T>> {
    /// Loads the two datasets and optional CPDAG named by the run config into a
    /// `BrcdInput` bundle, inside the pipeline. A loading failure surfaces as a
    /// `CdlError`.
    pub fn brcd_load_input(self) -> CdlEffect<CDL<BrcdLoaded<T>>> {
        match BrcdDataLoader::load(&self.state.config) {
            Ok(input) => CdlBuilder::pure(CDL {
                state: BrcdLoaded { input },
            }),
            Err(e) => CdlEffect {
                inner: Err(CdlError::from(e)),
                warnings: Default::default(),
            },
        }
    }
}

// Fluent stage method on the effect.
impl<T: Precision> CdlEffect<CDL<BrcdConfigured<T>>> {
    /// See [`CDL::<BrcdConfigured<T>>::brcd_load_input`].
    pub fn brcd_load_input(self) -> CdlEffect<CDL<BrcdLoaded<T>>> {
        self.and_then(|cdl| cdl.brcd_load_input())
    }
}

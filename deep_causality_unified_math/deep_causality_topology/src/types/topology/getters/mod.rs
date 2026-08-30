/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Getter methods for Topology.

use crate::{SimplicialComplex, Topology};
use deep_causality_tensor::CausalTensor;
use std::sync::Arc;

impl<R, G> Topology<R, G> {
    pub fn complex(&self) -> &Arc<SimplicialComplex<R>> {
        &self.complex
    }

    pub fn grade(&self) -> usize {
        self.grade
    }

    pub fn data(&self) -> &CausalTensor<G> {
        &self.data
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }
}

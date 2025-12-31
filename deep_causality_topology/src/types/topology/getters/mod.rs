/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Getter methods for Topology.

use crate::{SimplicialComplex, Topology};
use deep_causality_tensor::CausalTensor;
use std::sync::Arc;

impl<T> Topology<T> {
    pub fn complex(&self) -> &Arc<SimplicialComplex> {
        &self.complex
    }

    pub fn grade(&self) -> usize {
        self.grade
    }

    pub fn data(&self) -> &CausalTensor<T> {
        &self.data
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }
}

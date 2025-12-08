/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::EffectData;
use crate::NumericValue;
use deep_causality_multivector::CausalMultiVector;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{Manifold, PointCloud, SimplicialComplex};
use std::any::Any;
use std::sync::Arc;

// --- Primitive conversions ---

impl From<bool> for EffectData {
    fn from(val: bool) -> Self {
        EffectData::Bool(val)
    }
}

impl From<f64> for EffectData {
    fn from(val: f64) -> Self {
        EffectData::Float(val)
    }
}

impl From<i64> for EffectData {
    fn from(val: i64) -> Self {
        EffectData::Int(val)
    }
}

impl From<String> for EffectData {
    fn from(val: String) -> Self {
        EffectData::String(val)
    }
}

impl From<&str> for EffectData {
    fn from(val: &str) -> Self {
        EffectData::String(val.to_string())
    }
}

impl From<Vec<EffectData>> for EffectData {
    fn from(val: Vec<EffectData>) -> Self {
        EffectData::Vector(val)
    }
}

// --- Algebraic conversions ---

impl From<NumericValue> for EffectData {
    fn from(val: NumericValue) -> Self {
        EffectData::Numerical(val)
    }
}

impl From<CausalMultiVector<f64>> for EffectData {
    fn from(val: CausalMultiVector<f64>) -> Self {
        EffectData::MultiVector(val)
    }
}

impl From<CausalTensor<f64>> for EffectData {
    fn from(val: CausalTensor<f64>) -> Self {
        EffectData::Tensor(val)
    }
}

impl From<PointCloud<f64>> for EffectData {
    fn from(val: PointCloud<f64>) -> Self {
        EffectData::PointCloud(val)
    }
}

impl From<SimplicialComplex> for EffectData {
    fn from(val: SimplicialComplex) -> Self {
        EffectData::SimplicialComplex(val)
    }
}

impl From<Manifold<f64>> for EffectData {
    fn from(val: Manifold<f64>) -> Self {
        EffectData::Manifold(val)
    }
}

// --- Custom conversion helper ---
// Note: We cannot implement generic From<T> for EffectData because of conflicting implementations
// with strict types, but we can provide a helper constructor.

impl EffectData {
    /// Creates a Custom variant from any compatible type.
    pub fn from_custom<T: Any + Send + Sync + 'static>(val: T) -> Self {
        EffectData::Custom(Arc::new(val))
    }
}

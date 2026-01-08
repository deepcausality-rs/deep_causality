/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;

use crate::NumericValue;
use deep_causality_multivector::CausalMultiVector;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{Manifold, PointCloud, SimplicialComplex};

pub mod effect_data_from;

/// `EffectData` represents a unified type for heterogeneous causal graphs.
///
/// It follows the "Atomic + Escape Hatch" pattern:
/// - **Atomic Variants**: Common types like `Bool`, `Float`, `Int`, `Vector`.
/// - **Algebraic Variants**: `MultiVector` and `Tensor` (fixed to `f64` backing for simplicity).
/// - **Escape Hatch**: `Custom` variant for any other complex type, using type erasure via `Arc`.
#[derive(Debug, Clone)]
pub enum EffectData {
    /// Boolean value (e.g., activation status).
    Bool(bool),
    /// Floating-point value (standard numeric data).
    Float(f64),
    /// Integer value (counts, discrete states).
    Int(i64),
    /// A generic NumericalValue (u8..u128, i8..i128, f32, f64).
    Numerical(NumericValue),
    /// String value (labels, identifiers).
    String(String),
    /// A vector of `EffectData`, enabling recursive heterogeneous collections.
    Vector(Vec<EffectData>),
    /// A generic MultiVector (Geometric Algebra), fixed to `f64`.
    MultiVector(CausalMultiVector<f64>),
    /// A generic CausalTensor, fixed to `f64`.
    Tensor(CausalTensor<f64>),
    /// A PointCloud (0-Complex), fixed to `f64`.
    PointCloud(PointCloud<f64, f64>),
    /// A SimplicialComplex (Higher-order structure).
    SimplicialComplex(SimplicialComplex<f64>),
    /// A Manifold (Valid SimplicialComplex), fixed to `f64`.
    Manifold(Manifold<f64, f64>),
    /// An escape hatch for storing any other type that implements `Any + Send + Sync`.
    /// Note: This uses `Arc` to support cheap cloning of the reference.
    Custom(Arc<dyn Any + Send + Sync>),
}

impl EffectData {
    /// Attempts to downcast the `Custom` variant to a concrete reference `T`.
    /// Returns `None` if the variant is not `Custom` or if the type doesn't match.
    pub fn as_custom<T: Any + 'static>(&self) -> Option<&T> {
        if let EffectData::Custom(arc) = self {
            arc.downcast_ref::<T>()
        } else {
            None
        }
    }
}

/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use core::error::Error;
use core::fmt;

/// A metric tensor refused by
/// [`MetricTensor4D::update_metric_tensor`](crate::MetricTensor4D::update_metric_tensor):
/// [`MetricTensorErrorEnum`] names the rule it breaks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MetricTensorError(pub MetricTensorErrorEnum);

impl Error for MetricTensorError {}

/// The rule a refused metric tensor breaks. Indices follow the coordinate order `t, x, y, z`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MetricTensorErrorEnum {
    /// The entry `g[row][col]` is NaN or infinite, so the tensor has no signature.
    NonFinite { row: usize, col: usize },
    /// The entries `g[row][col]` and `g[col][row]` differ.
    Asymmetric { row: usize, col: usize },
    /// The tensor does not have one negative and three positive eigenvalues; it has `positive`
    /// positive, `negative` negative and `zero` zero eigenvalues.
    Signature {
        positive: usize,
        negative: usize,
        zero: usize,
    },
}

impl MetricTensorError {
    pub const fn new(kind: MetricTensorErrorEnum) -> Self {
        Self(kind)
    }

    #[allow(non_snake_case)]
    pub const fn NonFinite(row: usize, col: usize) -> Self {
        Self(MetricTensorErrorEnum::NonFinite { row, col })
    }

    #[allow(non_snake_case)]
    pub const fn Asymmetric(row: usize, col: usize) -> Self {
        Self(MetricTensorErrorEnum::Asymmetric { row, col })
    }

    #[allow(non_snake_case)]
    pub const fn Signature(positive: usize, negative: usize, zero: usize) -> Self {
        Self(MetricTensorErrorEnum::Signature {
            positive,
            negative,
            zero,
        })
    }
}

impl fmt::Display for MetricTensorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            MetricTensorErrorEnum::NonFinite { row, col } => {
                write!(f, "MetricTensorError: g[{row}][{col}] is not finite")
            }
            MetricTensorErrorEnum::Asymmetric { row, col } => write!(
                f,
                "MetricTensorError: the tensor is not symmetric: g[{row}][{col}] differs from \
                 g[{col}][{row}]"
            ),
            MetricTensorErrorEnum::Signature {
                positive,
                negative,
                zero,
            } => write!(
                f,
                "MetricTensorError: the tensor is not of signature (−, +, +, +): {positive} \
                 positive, {negative} negative and {zero} zero eigenvalues"
            ),
        }
    }
}

/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{CausalTensor, CausalTensorError, CausalTensorMathExt};

impl CausalTensorMathExt<f32> for CausalTensor<f32> {
    fn log_nat(&self) -> Result<CausalTensor<f32>, CausalTensorError> {
        if self.is_empty() {
            return Ok(CausalTensor::from_slice(&[], self.shape()));
        }
        let new_data: Vec<f32> = self.as_slice().iter().map(|&val| val.ln()).collect();
        Ok(CausalTensor::from_slice(&new_data, self.shape()))
    }

    fn log2(&self) -> Result<CausalTensor<f32>, CausalTensorError> {
        if self.is_empty() {
            return Ok(CausalTensor::from_slice(&[], self.shape()));
        }

        let new_data: Vec<f32> = self.as_slice().iter().map(|&val| val.log2()).collect();
        Ok(CausalTensor::from_slice(&new_data, self.shape()))
    }

    fn log10(&self) -> Result<CausalTensor<f32>, CausalTensorError> {
        if self.is_empty() {
            return Ok(CausalTensor::from_slice(&[], self.shape()));
        }

        let new_data: Vec<f32> = self.as_slice().iter().map(|&val| val.log10()).collect();
        Ok(CausalTensor::from_slice(&new_data, self.shape()))
    }

    fn surd_log2(&self) -> Result<CausalTensor<f32>, CausalTensorError> {
        if self.is_empty() {
            return Ok(CausalTensor::from_slice(&[], self.shape()));
        }
        let new_data: Vec<f32> = self
            .as_slice()
            .iter()
            .map(|&val| if val == 0.0 { 0.0 } else { val.log2() })
            .collect();
        Ok(CausalTensor::from_slice(&new_data, self.shape()))
    }

    fn safe_div(&self, rhs: &CausalTensor<f32>) -> Result<CausalTensor<f32>, CausalTensorError> {
        self.broadcast_op(rhs, |numerator, denominator| {
            if denominator.abs() < 1e-7 {
                if numerator.abs() < 1e-7 {
                    Ok(0.0)
                } else {
                    Err(CausalTensorError::DivisionByZero)
                }
            } else {
                Ok(numerator / denominator)
            }
        })
    }
}

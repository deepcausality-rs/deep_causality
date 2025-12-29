/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{CausalTensor, CausalTensorError, CausalTensorMathExt};

impl CausalTensorMathExt<f64> for CausalTensor<f64> {
    fn log_nat(&self) -> Result<CausalTensor<f64>, CausalTensorError> {
        if self.is_empty() {
            return Ok(CausalTensor::new(&[], &self.shape().to_vec()));
        }
        let new_data: Vec<f64> = self.as_slice().iter().map(|&val| val.ln()).collect();
        Ok(CausalTensor::new(&new_data, &self.shape().to_vec()))
    }

    fn log2(&self) -> Result<CausalTensor<f64>, CausalTensorError> {
        if self.is_empty() {
            return Ok(CausalTensor::new(&[], &self.shape().to_vec()));
        }

        let new_data: Vec<f64> = self.as_slice().iter().map(|&val| val.log2()).collect();
        Ok(CausalTensor::new(&new_data, &self.shape().to_vec()))
    }

    fn log10(&self) -> Result<CausalTensor<f64>, CausalTensorError> {
        if self.is_empty() {
            return Ok(CausalTensor::new(&[], &self.shape().to_vec()));
        }

        let new_data: Vec<f64> = self.as_slice().iter().map(|&val| val.log10()).collect();
        Ok(CausalTensor::new(&new_data, &self.shape().to_vec()))
    }

    fn surd_log2(&self) -> Result<CausalTensor<f64>, CausalTensorError> {
        if self.is_empty() {
            return Ok(CausalTensor::new(&[], &self.shape().to_vec()));
        }
        let new_data: Vec<f64> = self
            .as_slice()
            .iter()
            .map(|&val| if val == 0.0 { 0.0 } else { val.log2() })
            .collect();
        Ok(CausalTensor::new(&new_data, &self.shape().to_vec()))
    }

    fn safe_div(&self, rhs: &CausalTensor<f64>) -> Result<CausalTensor<f64>, CausalTensorError> {
        self.broadcast_op(rhs, |numerator, denominator| {
            if denominator.abs() < 1e-14 {
                // If denominator is zero, the result is only well-defined if numerator is also zero.
                if numerator.abs() < 1e-14 {
                    Ok(0.0) // The 0/0 case, which means zero information.
                } else {
                    // This is a true error state (e.g. 1/0)
                    Err(CausalTensorError::DivisionByZero)
                }
            } else {
                Ok(numerator / denominator)
            }
        })
    }
}

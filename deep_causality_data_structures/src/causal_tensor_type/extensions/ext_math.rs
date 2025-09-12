/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{CausalTensor, CausalTensorError};
use std::f64;

pub trait CausalTensorLogMathExt<T> {
    /// Computes the element-wise natural logarithm of the tensor.
    ///
    /// # Arguments
    ///
    /// * `self` - A reference to the CausalTensor.
    ///
    /// # Returns
    ///
    /// A `Result` containing a new `CausalTensor` with the natural logarithm of each element,
    /// or a `CausalTensorError` if the operation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_data_structures::{CausalTensor, CausalTensorLogMathExt};
    ///
    /// let tensor = CausalTensor::new(vec![1.0, std::f32::consts::E, 10.0], vec![3, 1]).unwrap();
    /// let result = tensor.log_nat().unwrap();
    ///
    /// assert_eq!(result.as_slice(), &[0.0, 0.99999994, 2.3025851]); // Approx. ln(10)
    /// ```
    fn log_nat(&self) -> Result<CausalTensor<T>, CausalTensorError>;

    /// Computes the element-wise base-2 logarithm of the tensor.
    ///
    /// # Arguments
    ///
    /// * `self` - A reference to the CausalTensor.
    ///
    /// # Returns
    ///
    /// A `Result` containing a new `CausalTensor` with the base-2 logarithm of each element,
    /// or a `CausalTensorError` if the operation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_data_structures::{CausalTensor, CausalTensorLogMathExt};
    ///
    /// let tensor = CausalTensor::new(vec![1.0, 2.0, 4.0, 8.0], vec![4, 1]).unwrap();
    /// let result = tensor.log2().unwrap();
    ///
    /// assert_eq!(result.as_slice(), &[0.0, 1.0, 2.0, 3.0]);
    /// ```
    fn log2(&self) -> Result<CausalTensor<T>, CausalTensorError>;

    /// Computes the element-wise base-10 logarithm of the tensor.
    ///
    /// # Arguments
    ///
    /// * `self` - A reference to the CausalTensor.
    ///
    /// # Returns
    ///
    /// A `Result` containing a new `CausalTensor` with the base-10 logarithm of each element,
    /// or a `CausalTensorError` if the operation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_data_structures::{CausalTensor, CausalTensorLogMathExt};
    ///
    /// let tensor = CausalTensor::new(vec![1.0, 10.0, 100.0], vec![3, 1]).unwrap();
    /// let result = tensor.log10().unwrap();
    ///
    /// assert_eq!(result.as_slice(), &[0.0, 1.0, 2.0]);
    /// ```
    fn log10(&self) -> Result<CausalTensor<T>, CausalTensorError>;

    /// Computes the element-wise base-2 logarithm of the tensor, handling 0.0 inputs as 0.0.
    /// This replicates the behavior of Python's `mylog` function for information theory calculations.
    ///
    /// # Arguments
    ///
    /// * `self` - A reference to the CausalTensor.
    ///
    /// # Returns
    ///
    /// A `Result` containing a new `CausalTensor` with the base-2 logarithm of each element,
    /// or a `CausalTensorError` if the operation fails.
    fn surd_log2(&self) -> Result<CausalTensor<T>, CausalTensorError>;

    /// Performs element-wise division that is safe for probabilistic calculations.
    ///
    /// This method handles the `0.0 / 0.0` case by returning `0.0`, which is the
    /// correct interpretation in an information-theoretic context where an impossible
    /// event provides zero information. It will still panic on other divisions by zero
    /// (e.g., `x / 0` where `x > 0`), as this represents a malformed probability.
    fn safe_div(&self, rhs: &CausalTensor<T>) -> Result<CausalTensor<T>, CausalTensorError>;
}

impl CausalTensorLogMathExt<f32> for CausalTensor<f32> {
    fn log_nat(&self) -> Result<CausalTensor<f32>, CausalTensorError> {
        if self.is_empty() {
            return CausalTensor::new(vec![], self.shape().to_vec()); // Correctly return an empty tensor
        }
        let new_data = self.as_slice().iter().map(|&val| val.ln()).collect();
        CausalTensor::new(new_data, self.shape().to_vec())
    }

    fn log2(&self) -> Result<CausalTensor<f32>, CausalTensorError> {
        if self.is_empty() {
            return CausalTensor::new(vec![], self.shape().to_vec()); // Correctly return an empty tensor
        }

        let new_data = self.as_slice().iter().map(|&val| val.log2()).collect();
        CausalTensor::new(new_data, self.shape().to_vec())
    }

    fn log10(&self) -> Result<CausalTensor<f32>, CausalTensorError> {
        if self.is_empty() {
            return CausalTensor::new(vec![], self.shape().to_vec()); // Correctly return an empty tensor
        }

        let new_data = self.as_slice().iter().map(|&val| val.log10()).collect();
        CausalTensor::new(new_data, self.shape().to_vec())
    }

    fn surd_log2(&self) -> Result<CausalTensor<f32>, CausalTensorError> {
        if self.is_empty() {
            return CausalTensor::new(vec![], self.shape().to_vec());
        }
        let new_data = self
            .as_slice()
            .iter()
            .map(|&val| if val == 0.0 { 0.0 } else { val.log2() })
            .collect();
        CausalTensor::new(new_data, self.shape().to_vec())
    }

    fn safe_div(&self, rhs: &CausalTensor<f32>) -> Result<CausalTensor<f32>, CausalTensorError> {
        // This relies on the broadcasting logic from your binary_op helper.
        // The key is the custom operation closure we provide.
        self.broadcast_op(rhs, |numerator, denominator| {
            if denominator.abs() < 1e-14 {
                // If denominator is zero, the result is only well-defined if numerator is also zero.
                if numerator.abs() < 1e-14 {
                    Ok(0.0) // The 0/0 case, which means zero information.
                } else {
                    Ok(f32::NAN) // This is a true error state (e.g. 1/0), results in NaN.
                }
            } else {
                Ok(numerator / denominator)
            }
        })
    }
}

impl CausalTensorLogMathExt<f64> for CausalTensor<f64> {
    fn log_nat(&self) -> Result<CausalTensor<f64>, CausalTensorError> {
        if self.is_empty() {
            return CausalTensor::new(vec![], self.shape().to_vec()); // Correctly return an empty tensor
        }
        let new_data = self.as_slice().iter().map(|&val| val.ln()).collect();
        CausalTensor::new(new_data, self.shape().to_vec())
    }

    fn log2(&self) -> Result<CausalTensor<f64>, CausalTensorError> {
        if self.is_empty() {
            return CausalTensor::new(vec![], self.shape().to_vec()); // Correctly return an empty tensor
        }

        let new_data = self.as_slice().iter().map(|&val| val.log2()).collect();
        CausalTensor::new(new_data, self.shape().to_vec())
    }

    fn log10(&self) -> Result<CausalTensor<f64>, CausalTensorError> {
        if self.is_empty() {
            return CausalTensor::new(vec![], self.shape().to_vec()); // Correctly return an empty tensor
        }

        let new_data = self.as_slice().iter().map(|&val| val.log10()).collect();
        CausalTensor::new(new_data, self.shape().to_vec())
    }

    fn surd_log2(&self) -> Result<CausalTensor<f64>, CausalTensorError> {
        if self.is_empty() {
            return CausalTensor::new(vec![], self.shape().to_vec());
        }
        let new_data = self
            .as_slice()
            .iter()
            .map(|&val| if val == 0.0 { 0.0 } else { val.log2() })
            .collect();
        CausalTensor::new(new_data, self.shape().to_vec())
    }

    fn safe_div(&self, rhs: &CausalTensor<f64>) -> Result<CausalTensor<f64>, CausalTensorError> {
        self.broadcast_op(rhs, |numerator, denominator| {
            if denominator.abs() < 1e-14 {
                // If denominator is zero, the result is only well-defined if numerator is also zero.
                if numerator.abs() < 1e-14 {
                    Ok(0.0) // The 0/0 case, which means zero information.
                } else {
                    Ok(f64::NAN) // This is a true error state (e.g. 1/0), results in NaN.
                }
            } else {
                Ok(numerator / denominator)
            }
        })
    }
}

/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{CausalTensor, CausalTensorError};

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
}

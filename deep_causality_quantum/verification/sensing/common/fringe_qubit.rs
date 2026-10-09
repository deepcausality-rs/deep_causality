/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The interferometer read-out as a qubit, for the mechanism path: a qubit on the equator at
//! mid-fringe, a phase channel, the fringe contrast as a dephasing channel, and the output port
//! whose population a positive phase raises, so the read-out is `½(1 + C sin φ)`.

use deep_causality_num_complex::Complex;
use deep_causality_quantum::{
    Axis, Channel, Hypothesis, Observable, QuantumError, QuantumPlant, QubitOperator,
};
use deep_causality_tensor::CausalTensor;

fn real(value: f64) -> Complex<f64> {
    Complex::new(value, 0.0)
}

/// A mechanism candidate named `name` whose own channel is the fringe contrast `contrast`.
///
/// # Errors
///
/// [`QuantumError::CalculationError`] for a contrast outside `[0, 1]`, NaN included, and the
/// channel's refusal of the Kraus operators.
pub fn mechanism(name: &str, contrast: f64) -> Result<Hypothesis<f64>, QuantumError> {
    if !(0.0..=1.0).contains(&contrast) {
        return Err(QuantumError::CalculationError(format!(
            "a fringe contrast lies in [0, 1], got {contrast}"
        )));
    }
    let zero = real(0.0);
    let keep = real((0.5 * (1.0 + contrast)).sqrt());
    let flip = real((0.5 * (1.0 - contrast)).sqrt());
    Ok(Hypothesis::mechanism(
        name,
        Channel::from_kraus(&[
            CausalTensor::from_slice(&[keep, zero, zero, keep], &[2, 2]),
            CausalTensor::from_slice(&[flip, zero, zero, -flip], &[2, 2]),
        ])?,
    ))
}

/// The channel of a phase `phase`, in rad.
///
/// # Errors
///
/// The rotation's refusal of a non-finite angle.
pub fn phase_channel(phase: f64) -> Result<Channel<f64>, QuantumError> {
    Channel::unitary(&QubitOperator::rotation(Axis::Z, phase)?)
}

/// The atoms after the first beam splitter at mid-fringe, `(|0⟩ + i|1⟩)/√2`.
///
/// # Errors
///
/// The plant's refusal of the state.
pub fn plant() -> Result<QuantumPlant<f64>, QuantumError> {
    let a = 0.5f64.sqrt();
    QuantumPlant::from_ket(&CausalTensor::from_slice(
        &[real(a), Complex::new(0.0, a)],
        &[2],
    ))
}

/// The output port, `(|0⟩ − |1⟩)/√2`.
///
/// # Errors
///
/// The observable's refusal of the state.
pub fn output_port() -> Result<Observable<f64, 2>, QuantumError> {
    let a = 0.5f64.sqrt();
    Observable::from_ket(
        "output port",
        &CausalTensor::from_slice(&[real(a), real(-a)], &[2]),
    )
}

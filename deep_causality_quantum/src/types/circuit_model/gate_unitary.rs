/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::QuantumError;
use crate::types::qpu::circuit::GateOp;
use alloc::vec;
use alloc::vec::Vec;
use core::f64::consts::FRAC_PI_4;
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;
use deep_causality_num_complex::Complex;
use deep_causality_tensor::CausalTensor;

/// The widest gate a unitary is formed for: a `2^12 × 2^12` matrix is the numeric semantics'
/// default entry cap, and a wider `Cmz` would allocate before any cap could refuse it.
pub const MAX_GATE_QUBITS: usize = 12;

/// The unitary matrix of one gate over its qubits in ascending order, the first qubit most
/// significant, which is the row-major leg order `embed_on_legs` and the numeric semantics use.
///
/// Returns the ascending qubit list beside the `2^k × 2^k` matrix. The single-qubit gates are the
/// textbook matrices; `Cnot` flips the target where the control is set, whichever of the two is
/// the smaller index; the diagonal family multiplies the all-ones basis state by its phase and
/// leaves every other amplitude alone, which is the same rule the state-vector sampler applies.
///
/// # Errors
///
/// [`QuantumError::CalculationError`] if the scalar cannot represent `1/√2` or `π/4`;
/// [`QuantumError::DimensionMismatch`] if a gate names a qubit twice or acts on more than
/// [`MAX_GATE_QUBITS`] qubits, before any matrix is formed.
pub fn gate_unitary<R>(op: &GateOp) -> Result<(Vec<usize>, CausalTensor<Complex<R>>), QuantumError>
where
    R: RealField + FromPrimitive,
{
    let zero = Complex::new(R::zero(), R::zero());
    let one = Complex::new(R::one(), R::zero());
    let i_unit = Complex::new(R::zero(), R::one());
    let minus_one = Complex::new(-R::one(), R::zero());
    let s = R::from_f64(core::f64::consts::FRAC_1_SQRT_2)
        .ok_or_else(|| QuantumError::CalculationError("the scalar cannot represent 1/√2".into()))?;
    let pi4 = R::from_f64(FRAC_PI_4)
        .ok_or_else(|| QuantumError::CalculationError("the scalar cannot represent π/4".into()))?;
    let t_phase = Complex::new(pi4.cos(), pi4.sin());
    let t_dagger = Complex::new(pi4.cos(), -pi4.sin());

    let mut qubits = op.qubits();
    let before = qubits.len();
    qubits.sort_unstable();
    qubits.dedup();
    if qubits.len() != before {
        return Err(QuantumError::DimensionMismatch(alloc::format!(
            "gate {op:?} names a qubit more than once"
        )));
    }
    if qubits.len() > MAX_GATE_QUBITS {
        return Err(QuantumError::DimensionMismatch(alloc::format!(
            "gate {op:?} acts on {} qubits, above the limit of {MAX_GATE_QUBITS}; its matrix would \
             have 2^{} entries",
            qubits.len(),
            2 * qubits.len()
        )));
    }

    let single = |m: [Complex<R>; 4]| CausalTensor::from_slice(&m, &[2, 2]);
    let diagonal = |k: usize, phase: Complex<R>| {
        let d = 1usize << k;
        let mut data = vec![zero; d * d];
        for idx in 0..d {
            data[idx * d + idx] = if idx == d - 1 { phase } else { one };
        }
        CausalTensor::from_slice(&data, &[d, d])
    };

    let matrix = match op {
        GateOp::H(_) => single([
            Complex::new(s, R::zero()),
            Complex::new(s, R::zero()),
            Complex::new(s, R::zero()),
            Complex::new(-s, R::zero()),
        ]),
        GateOp::X(_) => single([zero, one, one, zero]),
        GateOp::Y(_) => single([zero, -i_unit, i_unit, zero]),
        GateOp::Z(_) => single([one, zero, zero, minus_one]),
        GateOp::S(_) => single([one, zero, zero, i_unit]),
        GateOp::Sdg(_) => single([one, zero, zero, -i_unit]),
        GateOp::T(_) => single([one, zero, zero, t_phase]),
        GateOp::Tdg(_) => single([one, zero, zero, t_dagger]),
        GateOp::Cnot { control, .. } => {
            // Position 0 is the most significant bit of the two-qubit index, and the qubits are
            // sorted, so the control's bit is read off its position in the sorted list.
            let control_bit = if qubits[0] == *control { 2 } else { 1 };
            let target_bit = 3 - control_bit;
            let mut data = vec![zero; 16];
            for input in 0..4usize {
                let output = if input & control_bit != 0 {
                    input ^ target_bit
                } else {
                    input
                };
                data[output * 4 + input] = one;
            }
            CausalTensor::from_slice(&data, &[4, 4])
        }
        GateOp::Cz { .. } => diagonal(2, minus_one),
        GateOp::Csdg { .. } => diagonal(2, -i_unit),
        GateOp::Ccz { .. } => diagonal(3, minus_one),
        GateOp::Cmz { .. } => diagonal(qubits.len(), minus_one),
    };
    Ok((qubits, matrix))
}

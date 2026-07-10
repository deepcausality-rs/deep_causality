/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::QuantumError;
use crate::QuantumOps;
use crate::kernels::gates_haruna;
use deep_causality_algebra::DivisionAlgebra;
use deep_causality_algebra::RealField;
use deep_causality_haft::Functor;
use deep_causality_multivector::MultiVector;
use deep_causality_multivector::{CausalMultiVector, CausalMultiVectorWitness, HilbertState};
use deep_causality_num::FromPrimitive;
use deep_causality_num_complex::Complex;

pub type Operator<R> = HilbertState<R>;
pub type Gate<R> = HilbertState<R>;

/// Calculates the Born probability: $P = |\langle \text{basis} | \text{state} \rangle|^2$.
pub fn born_probability_kernel<R>(
    state: &HilbertState<R>,
    basis: &HilbertState<R>,
) -> Result<R, QuantumError>
where
    R: RealField + FromPrimitive + core::iter::Sum,
{
    if state.mv().metric() != basis.mv().metric() {
        return Err(QuantumError::MetricMismatch(format!(
            "Metric mismatch in Born Probability: {:?} vs {:?}",
            state.mv().metric(),
            basis.mv().metric()
        )));
    }

    let amplitude = state.mv().bracket(basis.mv());
    let p = amplitude.norm_sqr();

    if !p.is_finite() {
        return Err(QuantumError::NonFiniteValue(
            "Born probability is not finite".into(),
        ));
    }

    let lower = R::from_f64(-1e-9)
        .ok_or_else(|| QuantumError::CalculationError("R::from_f64(-1e-9)".into()))?;
    let upper = R::from_f64(1.000001)
        .ok_or_else(|| QuantumError::CalculationError("R::from_f64(1.000001)".into()))?;
    if p < lower || p > upper {
        return Err(QuantumError::NormalizationError(
            "Born probability out of bounds".into(),
        ));
    }

    Ok(p.clamp(R::zero(), R::one()))
}

/// Calculates the expectation value: $\langle A \rangle = \langle \psi | A | \psi \rangle$.
pub fn expectation_value_kernel<R>(
    state: &HilbertState<R>,
    operator: &Operator<R>,
) -> Result<R, QuantumError>
where
    R: RealField + core::iter::Sum,
{
    if state.mv().metric() != operator.mv().metric() {
        return Err(QuantumError::MetricMismatch(format!(
            "Metric mismatch in Expectation Value: {:?} vs {:?}",
            state.mv().metric(),
            operator.mv().metric()
        )));
    }

    let val = state.mv().expectation_value(operator.mv());
    Ok(val.re)
}

/// Applies a quantum gate to a state: $|\psi'\rangle = U |\psi\rangle$.
pub fn apply_gate_kernel<R>(
    state: &HilbertState<R>,
    gate: &Gate<R>,
) -> Result<HilbertState<R>, QuantumError>
where
    R: RealField,
{
    if state.mv().metric() != gate.mv().metric() {
        return Err(QuantumError::MetricMismatch(format!(
            "Metric mismatch in Apply Gate: {:?} vs {:?}",
            state.mv().metric(),
            gate.mv().metric()
        )));
    }

    let new_inner = gate.mv().geometric_product(state.mv());

    if new_inner
        .data()
        .iter()
        .any(|c| !c.re.is_finite() || !c.im.is_finite())
    {
        return Err(QuantumError::NonFiniteValue(
            "Non-finite component in state after gate application".into(),
        ));
    }

    Ok(HilbertState::<R>::from_multivector(new_inner))
}

/// Calculates commutator $[A, B] = AB - BA$.
pub fn commutator_kernel<R>(
    a: &Operator<R>,
    b: &Operator<R>,
) -> Result<HilbertState<R>, QuantumError>
where
    R: RealField,
{
    let a_mv = a.mv();
    let b_mv = b.mv();

    if a_mv.metric() != b_mv.metric() {
        return Err(QuantumError::MetricMismatch(format!(
            "Metric mismatch in Commutator: {:?} vs {:?}",
            a_mv.metric(),
            b_mv.metric()
        )));
    }

    let ab = a_mv.geometric_product(b_mv);
    let ba = b_mv.geometric_product(a_mv);

    let commutator = ab - ba;

    Ok(HilbertState::<R>::from_multivector(commutator))
}

/// Calculates Quantum Fidelity: $F = |\langle \psi_{\text{ideal}} | \psi_{\text{actual}} \rangle|^2$.
pub fn fidelity_kernel<R>(
    ideal: &HilbertState<R>,
    actual: &HilbertState<R>,
) -> Result<R, QuantumError>
where
    R: RealField + FromPrimitive + core::iter::Sum,
{
    born_probability_kernel(actual, ideal)
}

/// Implements Haruna's Logical S-Gate.
pub fn haruna_s_gate_kernel<R>(field: &CausalMultiVector<R>) -> Result<Operator<R>, QuantumError>
where
    R: RealField + FromPrimitive,
{
    let field_complex =
        CausalMultiVectorWitness::fmap(field.clone(), |x| Complex::new(x, R::zero()));
    let result = gates_haruna::logical_s(&field_complex);
    Ok(HilbertState::<R>::from_multivector(result))
}

/// Implements Haruna's Logical Z-Gate.
pub fn haruna_z_gate_kernel<R>(field: &CausalMultiVector<R>) -> Result<Operator<R>, QuantumError>
where
    R: RealField + FromPrimitive,
{
    let field_complex =
        CausalMultiVectorWitness::fmap(field.clone(), |x| Complex::new(x, R::zero()));
    let result = gates_haruna::logical_z(&field_complex);
    Ok(HilbertState::<R>::from_multivector(result))
}

/// Implements Haruna's Logical X-Gate.
pub fn haruna_x_gate_kernel<R>(field: &CausalMultiVector<R>) -> Result<Operator<R>, QuantumError>
where
    R: RealField + FromPrimitive,
{
    let field_complex =
        CausalMultiVectorWitness::fmap(field.clone(), |x| Complex::new(x, R::zero()));
    let result = gates_haruna::logical_x(&field_complex);
    Ok(HilbertState::<R>::from_multivector(result))
}

/// Implements Haruna's Logical Hadamard Gate.
pub fn haruna_hadamard_gate_kernel<R>(
    field_a: &CausalMultiVector<R>,
    field_b: &CausalMultiVector<R>,
) -> Result<Operator<R>, QuantumError>
where
    R: RealField + FromPrimitive,
{
    let a_complex = CausalMultiVectorWitness::fmap(field_a.clone(), |x| Complex::new(x, R::zero()));
    let b_complex = CausalMultiVectorWitness::fmap(field_b.clone(), |x| Complex::new(x, R::zero()));

    if a_complex.metric() != b_complex.metric() {
        return Err(QuantumError::MetricMismatch(format!(
            "Metric mismatch in Haruna Hadamard: {:?} vs {:?}",
            a_complex.metric(),
            b_complex.metric()
        )));
    }

    let result = gates_haruna::logical_hadamard(&a_complex, &b_complex);
    Ok(HilbertState::<R>::from_multivector(result))
}

/// Implements Haruna's Logical CZ Gate.
pub fn haruna_cz_gate_kernel<R>(
    field_a1: &CausalMultiVector<R>,
    field_a2: &CausalMultiVector<R>,
) -> Result<Operator<R>, QuantumError>
where
    R: RealField + FromPrimitive,
{
    let a1_complex =
        CausalMultiVectorWitness::fmap(field_a1.clone(), |x| Complex::new(x, R::zero()));
    let a2_complex =
        CausalMultiVectorWitness::fmap(field_a2.clone(), |x| Complex::new(x, R::zero()));

    if a1_complex.metric() != a2_complex.metric() {
        return Err(QuantumError::MetricMismatch(format!(
            "Metric mismatch in Haruna CZ: {:?} vs {:?}",
            a1_complex.metric(),
            a2_complex.metric()
        )));
    }

    let result = gates_haruna::logical_cz(&a1_complex, &a2_complex);
    Ok(HilbertState::<R>::from_multivector(result))
}

/// Implements Haruna's Logical T-Gate.
pub fn haruna_t_gate_kernel<R>(field: &CausalMultiVector<R>) -> Result<Operator<R>, QuantumError>
where
    R: RealField + FromPrimitive,
{
    let field_complex =
        CausalMultiVectorWitness::fmap(field.clone(), |x| Complex::new(x, R::zero()));
    let result = gates_haruna::logical_t(&field_complex);
    Ok(HilbertState::<R>::from_multivector(result))
}

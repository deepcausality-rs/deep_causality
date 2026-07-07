/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::PhysicsError;
use crate::QuantumOps;
use crate::kernels::quantum::gates_haruna;
use core::fmt::Debug;
use deep_causality_algebra::DivisionAlgebra;
use deep_causality_algebra::RealField;
use deep_causality_haft::Functor;
use deep_causality_multivector::MultiVector;
use deep_causality_multivector::{CausalMultiVector, CausalMultiVectorWitness, HilbertState};
use deep_causality_num::FromPrimitive;
use deep_causality_num_complex::Complex;
use deep_causality_par::MaybeParallel;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::SimplicialManifold;

pub type Operator<R> = HilbertState<R>;
pub type Gate<R> = HilbertState<R>;

/// Calculates the Klein-Gordon operator action: $(\Delta + m^2)\psi$.
pub fn klein_gordon_kernel<R>(
    psi_manifold: &SimplicialManifold<R, R>,
    mass: R,
) -> Result<CausalTensor<R>, PhysicsError>
where
    R: RealField + MaybeParallel + FromPrimitive + Default + PartialEq + Debug,
{
    let laplacian = psi_manifold.laplacian(0);

    if !mass.is_finite() {
        return Err(PhysicsError::NumericalInstability(
            "Mass is not finite in Klein-Gordon".into(),
        ));
    }
    if laplacian.as_slice().iter().any(|v: &R| !v.is_finite()) {
        return Err(PhysicsError::NumericalInstability(
            "Laplacian contains non-finite entries".into(),
        ));
    }

    let m2 = mass * mass;
    if !m2.is_finite() {
        return Err(PhysicsError::NumericalInstability(
            "m^2 overflowed in Klein-Gordon".into(),
        ));
    }
    let psi_data = psi_manifold.data();

    let vertex_count = laplacian.len();
    if psi_data.len() < vertex_count {
        return Err(PhysicsError::DimensionMismatch(
            "psi_data is smaller than laplacian data".to_string(),
        ));
    }
    let psi_vertex_data = &psi_data.as_slice()[..vertex_count];
    if psi_vertex_data.iter().any(|v: &R| !v.is_finite()) {
        return Err(PhysicsError::NumericalInstability(
            "psi data contains non-finite entries".into(),
        ));
    }
    let psi_vertex_tensor =
        CausalTensor::new(psi_vertex_data.to_vec(), laplacian.shape().to_vec())?;
    let m2_psi = psi_vertex_tensor * m2;
    if m2_psi.as_slice().iter().any(|v| !v.is_finite()) {
        return Err(PhysicsError::NumericalInstability(
            "m^2 * psi produced non-finite entries".into(),
        ));
    }

    let result = laplacian + m2_psi;
    if result.as_slice().iter().any(|v: &R| !v.is_finite()) {
        return Err(PhysicsError::NumericalInstability(
            "Klein-Gordon result contains non-finite entries".into(),
        ));
    }

    Ok(result)
}

/// Calculates the Born probability: $P = |\langle \text{basis} | \text{state} \rangle|^2$.
pub fn born_probability_kernel<R>(
    state: &HilbertState<R>,
    basis: &HilbertState<R>,
) -> Result<R, PhysicsError>
where
    R: RealField + MaybeParallel + FromPrimitive + core::iter::Sum,
{
    if state.mv().metric() != basis.mv().metric() {
        return Err(PhysicsError::DimensionMismatch(format!(
            "Metric mismatch in Born Probability: {:?} vs {:?}",
            state.mv().metric(),
            basis.mv().metric()
        )));
    }

    let amplitude = state.mv().bracket(basis.mv());
    let p = amplitude.norm_sqr();

    if !p.is_finite() {
        return Err(PhysicsError::NumericalInstability(
            "Born probability is not finite".into(),
        ));
    }

    let lower = R::from_f64(-1e-9)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(-1e-9)".into()))?;
    let upper = R::from_f64(1.000001)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(1.000001)".into()))?;
    if p < lower || p > upper {
        return Err(PhysicsError::NormalizationError(
            "Born probability out of bounds".into(),
        ));
    }

    Ok(p.clamp(R::zero(), R::one()))
}

/// Calculates the expectation value: $\langle A \rangle = \langle \psi | A | \psi \rangle$.
pub fn expectation_value_kernel<R>(
    state: &HilbertState<R>,
    operator: &Operator<R>,
) -> Result<R, PhysicsError>
where
    R: RealField + MaybeParallel + core::iter::Sum,
{
    if state.mv().metric() != operator.mv().metric() {
        return Err(PhysicsError::DimensionMismatch(format!(
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
) -> Result<HilbertState<R>, PhysicsError>
where
    R: RealField + MaybeParallel,
{
    if state.mv().metric() != gate.mv().metric() {
        return Err(PhysicsError::DimensionMismatch(format!(
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
        return Err(PhysicsError::NumericalInstability(
            "Non-finite component in state after gate application".into(),
        ));
    }

    Ok(HilbertState::<R>::from_multivector(new_inner))
}

/// Calculates commutator $[A, B] = AB - BA$.
pub fn commutator_kernel<R>(
    a: &Operator<R>,
    b: &Operator<R>,
) -> Result<HilbertState<R>, PhysicsError>
where
    R: RealField + MaybeParallel,
{
    let a_mv = a.mv();
    let b_mv = b.mv();

    if a_mv.metric() != b_mv.metric() {
        return Err(PhysicsError::DimensionMismatch(format!(
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
) -> Result<R, PhysicsError>
where
    R: RealField + MaybeParallel + FromPrimitive + core::iter::Sum,
{
    born_probability_kernel(actual, ideal)
}

/// Implements Haruna's Logical S-Gate.
pub fn haruna_s_gate_kernel<R>(field: &CausalMultiVector<R>) -> Result<Operator<R>, PhysicsError>
where
    R: RealField + MaybeParallel + FromPrimitive,
{
    let field_complex =
        CausalMultiVectorWitness::fmap(field.clone(), |x| Complex::new(x, R::zero()));
    let result = gates_haruna::logical_s(&field_complex);
    Ok(HilbertState::<R>::from_multivector(result))
}

/// Implements Haruna's Logical Z-Gate.
pub fn haruna_z_gate_kernel<R>(field: &CausalMultiVector<R>) -> Result<Operator<R>, PhysicsError>
where
    R: RealField + MaybeParallel + FromPrimitive,
{
    let field_complex =
        CausalMultiVectorWitness::fmap(field.clone(), |x| Complex::new(x, R::zero()));
    let result = gates_haruna::logical_z(&field_complex);
    Ok(HilbertState::<R>::from_multivector(result))
}

/// Implements Haruna's Logical X-Gate.
pub fn haruna_x_gate_kernel<R>(field: &CausalMultiVector<R>) -> Result<Operator<R>, PhysicsError>
where
    R: RealField + MaybeParallel + FromPrimitive,
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
) -> Result<Operator<R>, PhysicsError>
where
    R: RealField + MaybeParallel + FromPrimitive,
{
    let a_complex = CausalMultiVectorWitness::fmap(field_a.clone(), |x| Complex::new(x, R::zero()));
    let b_complex = CausalMultiVectorWitness::fmap(field_b.clone(), |x| Complex::new(x, R::zero()));

    if a_complex.metric() != b_complex.metric() {
        return Err(PhysicsError::DimensionMismatch(format!(
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
) -> Result<Operator<R>, PhysicsError>
where
    R: RealField + MaybeParallel + FromPrimitive,
{
    let a1_complex =
        CausalMultiVectorWitness::fmap(field_a1.clone(), |x| Complex::new(x, R::zero()));
    let a2_complex =
        CausalMultiVectorWitness::fmap(field_a2.clone(), |x| Complex::new(x, R::zero()));

    if a1_complex.metric() != a2_complex.metric() {
        return Err(PhysicsError::DimensionMismatch(format!(
            "Metric mismatch in Haruna CZ: {:?} vs {:?}",
            a1_complex.metric(),
            a2_complex.metric()
        )));
    }

    let result = gates_haruna::logical_cz(&a1_complex, &a2_complex);
    Ok(HilbertState::<R>::from_multivector(result))
}

/// Implements Haruna's Logical T-Gate.
pub fn haruna_t_gate_kernel<R>(field: &CausalMultiVector<R>) -> Result<Operator<R>, PhysicsError>
where
    R: RealField + MaybeParallel + FromPrimitive,
{
    let field_complex =
        CausalMultiVectorWitness::fmap(field.clone(), |x| Complex::new(x, R::zero()));
    let result = gates_haruna::logical_t(&field_complex);
    Ok(HilbertState::<R>::from_multivector(result))
}

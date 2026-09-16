/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # The quantum geometric tensor, and the transport it forces
//!
//! A band is usually described by its energies. That leaves out everything about the *states*: how
//! much the wavefunction itself turns as the momentum moves across the Brillouin zone. The
//! **quantum geometric tensor** is what carries that,
//!
//! ```text
//! Q_ij = Σ_{m ≠ n}  ⟨n|v_i|m⟩ ⟨m|v_j|n⟩ / (E_n − E_m)²
//! ```
//!
//! and it is one complex number per pair of axes. Its two parts are two different pieces of
//! physics:
//!
//! ```text
//! g_ij = Re(Q_ij)        the quantum metric      symmetric      how far apart two states are
//! Ω_ij = −2 Im(Q_ij)     the Berry curvature     antisymmetric  a magnetic field in momentum space
//! ```
//!
//! The symmetry is forced, not imposed: a real symmetric part and an imaginary antisymmetric part
//! is what a Hermitian tensor decomposes into, so a run that produced a non-zero `Ω_xx` would be
//! reporting an arithmetic error rather than a discovery.
//!
//! # Why it matters in a flat band
//!
//! Conventional transport comes from band curvature, which is the inverse effective mass. A flat
//! band has none, so conventional transport vanishes and the band should be an insulator. Magic
//! angle twisted bilayer graphene is not an insulator. The missing term is geometric:
//!
//! ```text
//! D = (D_conv + g̃_xx · E_gap) · a²
//! ```
//!
//! With `D_conv = 0` the whole weight comes from the quantum metric, which is the geometric lower
//! bound on conductivity. This run computes `g̃_xx` from the model and hands it to the transport
//! kernel, so the two halves are one chain rather than two separate demonstrations.
//!
//! # What the run does
//!
//! ```text
//! fmap       axis pair → the QGT component for that pair, which may fail
//! sequence   a tensor of fallible components → one fallible tensor of components
//! fmap       Q → its real part, and Q → its imaginary part
//! fold       the diagonal of g → the trace, the geometric weight transport sees
//! ```
//!
//! `sequence` is what makes the failure honest. `fmap` leaves a tensor whose every cell might have
//! failed, and turning that inside out into one result is a single call rather than four checks —
//! and a component that carried no value stops the run instead of quietly vanishing from the
//! output.

mod model;
mod utils_print;

use deep_causality_haft::ResultWitness;
use deep_causality_haft::{Foldable, Functor, Traversable};
use deep_causality_num::Float106;
use deep_causality_physics::{
    Energy, Length, PhysicsError, QuantumMetric, effective_band_drude_weight,
};
use deep_causality_tensor::{CausalTensor, CausalTensorWitness};
use model::{
    AXIS_PAIRS, AXIS_X, E_FLAT_MEV, E_REMOTE_MEV, FLAT_BAND_CURVATURE, NUM_BANDS, ZERO,
    curvature_of, energy_gap_mev, lattice_constant_nm, metric_of, qgt_component, reduced_metric,
    two_band_model,
};
use utils_print::{print_bands, print_header, print_symmetry, print_tensor, print_transport};

/// The working scalar. Switch it to `f32`, `f64` or `deep_causality_num::BFloat16`; the tensor,
/// its decomposition and the transport weight all recompute at that precision.
///
/// It sits at [`Float106`] by default on purpose. A hard-coded `f64` anywhere in the program is
/// invisible while the alias *is* `f64`, and shows up here as a compile error the moment the two
/// types differ.
pub type FloatType = Float106;

fn main() -> Result<(), PhysicsError> {
    print_header();

    let model = two_band_model()?;
    print_bands(&model);

    // fmap: each axis pair names one component. The kernel may refuse any of them, so what comes
    // back is a tensor of results rather than a tensor of numbers.
    let pairs = CausalTensor::new(AXIS_PAIRS.to_vec(), vec![NUM_BANDS, NUM_BANDS])?;
    let attempted = CausalTensorWitness::fmap(pairs, |(i, j)| qgt_component(&model, i, j));

    // sequence: turn the tensor of results inside out. One `?` then covers all four components,
    // and the [2, 2] shape survives the traversal.
    let qgt: CausalTensor<_> =
        CausalTensorWitness::sequence::<_, ResultWitness<PhysicsError>>(attempted)?;

    // The decomposition. Two pointwise maps over the same tensor, because the metric and the
    // curvature are two readings of one object rather than two computations.
    let metric = CausalTensorWitness::fmap(qgt.clone(), metric_of);
    let curvature = CausalTensorWitness::fmap(qgt.clone(), curvature_of);

    print_tensor(&qgt, &metric, &curvature);
    print_symmetry(&metric, &curvature);

    // fold: the trace of the metric, which is the scalar the geometric bound is stated with.
    let trace = CausalTensorWitness::fold(diagonal(&metric)?, ZERO, |sum, g| sum + g);

    // The chain the tensor exists for. `g_xx` is the number computed above, made dimensionless by
    // the cell area, and handed to the transport kernel.
    let g_xx = metric.as_slice()[AXIS_X * NUM_BANDS + AXIS_X];
    let reduced = reduced_metric(g_xx);

    let flat = Energy::new(E_FLAT_MEV)?;
    let remote = Energy::new(E_REMOTE_MEV)?;
    let cell = Length::new(lattice_constant_nm())?;

    // With the geometry, and without it. The difference is the whole point: a flat band has no
    // conventional weight, so everything below is what the quantum metric contributes.
    let geometric = effective_band_drude_weight(
        flat,
        remote,
        FLAT_BAND_CURVATURE,
        QuantumMetric::new(reduced)?,
        cell,
    );
    let conventional = effective_band_drude_weight(
        flat,
        remote,
        FLAT_BAND_CURVATURE,
        QuantumMetric::new(ZERO)?,
        cell,
    );

    print_transport(
        trace,
        g_xx,
        reduced,
        energy_gap_mev(),
        weight(&conventional, "conventional")?,
        weight(&geometric, "geometric")?,
    );

    Ok(())
}

/// The diagonal of a square tensor, as a rank-1 tensor.
fn diagonal(tensor: &CausalTensor<FloatType>) -> Result<CausalTensor<FloatType>, PhysicsError> {
    let values: Vec<FloatType> = (0..NUM_BANDS)
        .map(|axis| tensor.as_slice()[axis * NUM_BANDS + axis])
        .collect();

    Ok(CausalTensor::new(values, vec![NUM_BANDS])?)
}

/// The number a transport effect carries, or the reason it carries none.
fn weight(
    effect: &deep_causality_core::PropagatingEffect<
        deep_causality_physics::BandDrudeWeight<FloatType>,
    >,
    which: &str,
) -> Result<FloatType, PhysicsError> {
    effect
        .value_cloned()
        .map(|weight| weight.value())
        .ok_or_else(|| {
            PhysicsError::NumericalInstability(format!("{which} Drude weight carried no value"))
        })
}

/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Type-state tests for `SolenoidalField`: both construction paths produce
//! discretely divergence-free fields at every precision backend; every
//! rejection branch is covered. The unconstructibility guarantees (no public
//! constructor, no `Add`/`Mul`) are enforced by `compile_fail` doctests on the
//! type itself in `src/units/fluid_dynamics/solenoidal_field/mod.rs`.

use deep_causality_num::{Float106, FromPrimitive, RealField};
use deep_causality_physics::{SolenoidalField, VelocityOneForm};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    ChainComplex, CubicalReggeGeometry, HodgeDecomposition, LatticeComplex, Manifold,
};

fn unit_manifold<R>(n: usize) -> Manifold<LatticeComplex<2, R>, R>
where
    R: RealField + deep_causality_topology::MaybeParallel + FromPrimitive + Default + PartialEq + core::fmt::Debug + core::fmt::Display,
{
    let lattice: LatticeComplex<2, R> = LatticeComplex::square_torus(n);
    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![R::zero(); total], vec![total]).unwrap();
    let metric: CubicalReggeGeometry<2, R> = CubicalReggeGeometry::unit();
    Manifold::from_cubical_with_metric(lattice, data, metric, 0)
}

fn random_cochain<R: RealField + FromPrimitive>(len: usize, seed: u64) -> Vec<R> {
    let mut state = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    (0..len)
        .map(|_| {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let unit = (state >> 11) as f64 / (1u64 << 53) as f64;
            R::from_f64(2.0 * unit - 1.0).expect("[-1,1] lifts")
        })
        .collect()
}

/// Discrete divergence of an edge cochain: place at grade 1, apply δ.
fn divergence<R>(manifold: &Manifold<LatticeComplex<2, R>, R>, one_form: &[R]) -> Vec<R>
where
    R: RealField + deep_causality_topology::MaybeParallel + FromPrimitive + Default + PartialEq + core::fmt::Debug + core::fmt::Display,
{
    let lattice = LatticeComplex::<2, R>::square_torus(
        // shape is square by fixture construction
        manifold.complex().shape()[0],
    );
    let total: usize = (0..=2).map(|g| lattice.num_cells(g)).sum();
    let n0 = lattice.num_cells(0);
    let mut data = vec![R::zero(); total];
    data[n0..n0 + one_form.len()].copy_from_slice(one_form);
    let tensor = CausalTensor::new(data, vec![total]).unwrap();
    let metric: CubicalReggeGeometry<2, R> = CubicalReggeGeometry::unit();
    let m = Manifold::from_cubical_with_metric(lattice, tensor, metric, 0);
    m.codifferential(1).as_slice().to_vec()
}

fn sup_norm<R: RealField>(v: &[R]) -> R {
    v.iter()
        .map(|x| x.abs())
        .fold(R::zero(), |m, x| if x > m { x } else { m })
}

/// Both construction paths produce divergence-free fields, per precision.
fn assert_both_paths_divergence_free<R>(rel_tol: R)
where
    R: RealField + deep_causality_topology::MaybeParallel + FromPrimitive + Default + PartialEq + core::fmt::Debug + core::fmt::Display,
{
    let manifold = unit_manifold::<R>(5);
    let n1 = manifold.complex().num_cells(1);
    let raw = CausalTensor::new(random_cochain::<R>(n1, 67), vec![n1]).unwrap();
    let scale = sup_norm(raw.as_slice());

    // Path 1: per-step Leray projection.
    let velocity = VelocityOneForm::new(raw.clone(), &manifold).unwrap();
    let (leray_field, potential) =
        SolenoidalField::from_leray_projection(&velocity, &manifold).unwrap();
    assert_eq!(potential.len(), manifold.complex().num_cells(0));
    let div = divergence(&manifold, leray_field.as_one_form().as_slice());
    assert!(
        sup_norm(&div) < rel_tol * scale,
        "Leray path divergence {} (scale {scale})",
        sup_norm(&div)
    );

    // Path 2: per-snapshot Hodge projection.
    let decomposition = manifold.hodge_decompose(&raw, 1).unwrap();
    let hodge_field = SolenoidalField::from_hodge_projection(&decomposition).unwrap();
    let div = divergence(&manifold, hodge_field.as_one_form().as_slice());
    assert!(
        sup_norm(&div) < rel_tol * scale,
        "Hodge path divergence {} (scale {scale})",
        sup_norm(&div)
    );

    // The two paths agree on the divergence-free field itself.
    let mut max_gap = R::zero();
    for (a, b) in leray_field
        .as_one_form()
        .as_slice()
        .iter()
        .zip(hodge_field.as_one_form().as_slice().iter())
    {
        let d = (*a - *b).abs();
        if d > max_gap {
            max_gap = d;
        }
    }
    assert!(
        max_gap < rel_tol * scale,
        "construction paths disagree by {max_gap}"
    );
}

#[test]
fn both_paths_divergence_free_f64() {
    assert_both_paths_divergence_free::<f64>(1e-7);
}

#[test]
fn both_paths_divergence_free_f32() {
    assert_both_paths_divergence_free::<f32>(1e-3_f32);
}

#[test]
fn both_paths_divergence_free_float106() {
    assert_both_paths_divergence_free::<Float106>(Float106::from_f64(1e-7));
}

// ---------------------------------------------------------------------------
// Rejection branches
// ---------------------------------------------------------------------------

#[test]
fn from_leray_projection_propagates_projection_failure() {
    // Velocity validated against a 3×3 lattice, projected against a 4×4 one:
    // the inner leray_project dimension mismatch surfaces as a typed
    // PhysicsError::TopologyError.
    let m3 = unit_manifold::<f64>(3);
    let m4 = unit_manifold::<f64>(4);
    let n1_3 = m3.complex().num_cells(1);
    let velocity =
        VelocityOneForm::new(CausalTensor::new(vec![1.0; n1_3], vec![n1_3]).unwrap(), &m3).unwrap();

    let err = SolenoidalField::from_leray_projection(&velocity, &m4).unwrap_err();
    let msg = format!("{err}");
    assert!(
        msg.contains("Topology Error") && msg.contains("Leray projection failed"),
        "got: {msg}"
    );
}

#[test]
fn from_hodge_projection_rejects_wrong_grade() {
    let manifold = unit_manifold::<f64>(3);
    let n0 = manifold.complex().num_cells(0);
    let scalar_field = CausalTensor::new(random_cochain::<f64>(n0, 71), vec![n0]).unwrap();
    let decomposition = manifold.hodge_decompose(&scalar_field, 0).unwrap();

    let err = SolenoidalField::from_hodge_projection(&decomposition).unwrap_err();
    assert!(format!("{err}").contains("grade-1 decomposition"));
}

#[test]
fn from_hodge_projection_rejects_component_length_mismatch() {
    // Adversarial decomposition with disagreeing component lengths.
    let exact = CausalTensor::new(vec![0.0_f64; 4], vec![4]).unwrap();
    let co_exact = CausalTensor::new(vec![0.0_f64; 4], vec![4]).unwrap();
    let harmonic = CausalTensor::new(vec![0.0_f64; 3], vec![3]).unwrap();
    let decomposition = HodgeDecomposition::new(exact, co_exact, harmonic, 1);

    let err = SolenoidalField::from_hodge_projection(&decomposition).unwrap_err();
    assert!(format!("{err}").contains("component length mismatch"));
}

#[test]
fn from_hodge_projection_rejects_non_finite_components() {
    let exact = CausalTensor::new(vec![0.0_f64; 4], vec![4]).unwrap();
    let co_exact = CausalTensor::new(vec![0.0, f64::NAN, 0.0, 0.0], vec![4]).unwrap();
    let harmonic = CausalTensor::new(vec![0.0_f64; 4], vec![4]).unwrap();
    let decomposition = HodgeDecomposition::new(exact, co_exact, harmonic, 1);

    let err = SolenoidalField::from_hodge_projection(&decomposition).unwrap_err();
    assert!(format!("{err}").contains("non-finite coefficient at index 1"));
}

// ---------------------------------------------------------------------------
// Surface
// ---------------------------------------------------------------------------

#[test]
fn read_only_accessors_and_derives() {
    let manifold = unit_manifold::<f64>(3);
    let n1 = manifold.complex().num_cells(1);
    let raw = CausalTensor::new(random_cochain::<f64>(n1, 73), vec![n1]).unwrap();
    let velocity = VelocityOneForm::new(raw, &manifold).unwrap();
    let (field, _potential) = SolenoidalField::from_leray_projection(&velocity, &manifold).unwrap();

    assert_eq!(field.len(), n1);
    assert!(!field.is_empty());
    assert_eq!(field.as_one_form().len(), n1);

    let clone = field.clone();
    assert_eq!(clone, field);
    assert!(format!("{field:?}").contains("SolenoidalField"));
}

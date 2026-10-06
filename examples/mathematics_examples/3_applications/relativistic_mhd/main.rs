/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # GRMHD: a tensor solver coupled to a multivector solver
//!
//! Modelling a black-hole accretion flow or a neutron-star magnetosphere couples general
//! relativity, which supplies the spacetime curvature, to magnetohydrodynamics, which supplies
//! the plasma dynamics. The two halves speak different mathematics: curvature lives in a tensor,
//! and the plasma forces live in a Clifford algebra.
//!
//! This example runs both in one program and lets the first decide the setting of the second:
//!
//! ```text
//! 1. GR solver     CausalTensor        the Einstein tensor G_uv from the metric g_uv
//! 2. coupling      a value decision    curvature intensity selects the Clifford metric
//! 3. MHD solver    CausalMultiVector   the Lorentz force density F = J · B
//! ```
//!
//! The GR step scales the metric by the scalar curvature through `Pure` and `Applicative` on
//! `CausalTensorWitness`: the scaling law is lifted into the tensor and applied at every
//! component. The coupling step reads the local curvature back out and hands the MHD solver
//! `Minkowski(4)` above the threshold and `Euclidean(3)` below it, so the algebra the plasma
//! runs in follows the regime the plasma is in.
//!
//! The world both solvers read lives in a `Context`, one `Data` node per quantity in normalised
//! units: the metric's diagonal, the scalar curvature, the threshold the coupling compares
//! against, and the plasma's current density and magnetic field.

use deep_causality_context::{
    Context, ContextIndexError, Contextoid, ContextoidType, ContextuableGraph, Data, Datable,
    NoSpace, NoSpaceTime, NoTime,
};
use deep_causality_haft::{Applicative, Pure};
use deep_causality_multivector::{CausalMultiVector, CausalMultiVectorError, Metric, MultiVector};
use deep_causality_num::{const_scalar_from_int, lift, lower};
use deep_causality_tensor::{CausalTensor, CausalTensorWitness};

/// The metric tensor is 4×4.
const METRIC_DIM: usize = 4;

/// The spacetime and the plasma in it, one `Data` node per quantity. The units are normalised and
/// the geometry is the metric tensor built from the nodes, so the spatial, temporal and spacetime
/// slots are empty.
type GrmhdContext = Context<Data<FloatType>, NoSpace<FloatType>, NoTime, NoSpaceTime<FloatType>>;

/// Node indices of the world's quantities.
///
/// The diagonal of a Schwarzschild-like metric: time dilation, radial stretching, and two angular
/// components left flat.
const G_00: usize = 0;
const G_11: usize = 1;
const G_22: usize = 2;
const G_33: usize = 3;
/// The scalar curvature `R` driven by the central mass, and the intensity above which the plasma
/// runs in the relativistic algebra.
const SCALAR_CURVATURE: usize = 4;
const RELATIVISTIC_THRESHOLD: usize = 5;
/// Plasma current density flowing toroidally, and the poloidal confinement field.
const CURRENT_DENSITY: usize = 6;
const MAGNETIC_FIELD: usize = 7;

/// Blade indices: each axis owns one bit, and a plane owns the bits of the axes spanning it.
const E_X: usize = 1 << 1;
const E_Y: usize = 1 << 2;
const E_XY: usize = E_X | E_Y;

/// The working scalar. The metric, the curvature and the plasma force all carry it.
pub type FloatType = f64;

/// Small numbers, declared once at the working type rather than lifted at each use.
const ZERO: FloatType = const_scalar_from_int!(FloatType, 0);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_header();
    let world = accretion_flow()?;

    // Step 1: the GR solver. The metric tensor goes in, the Einstein tensor comes out.
    print_step_one();
    let g_uv = spacetime_metric(&world)?;
    let g_tensor = einstein_tensor(&g_uv, read(&world, SCALAR_CURVATURE)?);

    // The time component g_00 carries the gravitational time dilation, which stands in here for
    // the local curvature intensity. A full coupling maps the whole tensor to a Clifford metric.
    let time_dilation = g_tensor.data()[0].abs();
    print_curvature(time_dilation);

    // Step 2: the coupling. The curvature intensity is a value, and it selects the algebra the
    // next solver works in.
    print_step_two();
    let (metric_sig, label) = if time_dilation > read(&world, RELATIVISTIC_THRESHOLD)? {
        (Metric::Minkowski(4), "Relativistic (Minkowski 4D)")
    } else {
        (Metric::Euclidean(3), "Classical (Euclidean 3D)")
    };
    print_metric_choice(label);

    // Step 3: the MHD solver, running in the metric Step 2 chose.
    print_step_three();
    let current = read(&world, CURRENT_DENSITY)?;
    let field = read(&world, MAGNETIC_FIELD)?;
    let force = lorentz_force(current, field, metric_sig)?;
    print_force(force);

    // Step 4: the feedback the next control cycle acts on.
    print_analysis(force < ZERO);
    print_footer();

    Ok(())
}

// --- The world: the spacetime and the plasma in it ---

/// The accretion flow's world, in normalised units, one `Data` node per quantity.
fn accretion_flow() -> Result<GrmhdContext, ContextIndexError> {
    // In node-index order.
    let quantities: [f64; 8] = [
        -0.9, // G_00
        1.1,  // G_11
        1.0,  // G_22
        1.0,  // G_33
        0.1,  // SCALAR_CURVATURE
        0.05, // RELATIVISTIC_THRESHOLD
        10.0, // CURRENT_DENSITY
        2.0,  // MAGNETIC_FIELD
    ];

    let mut context = Context::with_capacity(1, "accretion flow", quantities.len());
    for (id, &value) in (1..).zip(quantities.iter()) {
        context.add_node(Contextoid::new(
            id,
            ContextoidType::Datoid(Data::new(id, lift::<FloatType>(value))),
        ))?;
    }
    Ok(context)
}

/// Read one quantity out of the world.
fn read(context: &GrmhdContext, index: usize) -> Result<FloatType, ContextIndexError> {
    context
        .get_node(index)
        .and_then(|node| node.vertex_type().dataoid())
        .map(Datable::get_data)
        .ok_or_else(|| ContextIndexError::new(format!("no quantity at node {index}")))
}

// --- General relativity: the tensor engine ---

/// The metric tensor of a simplified Schwarzschild-like spacetime, diagonal in these units, built
/// from the diagonal the world holds.
fn spacetime_metric(
    world: &GrmhdContext,
) -> Result<CausalTensor<FloatType>, Box<dyn std::error::Error>> {
    let mut data = vec![ZERO; METRIC_DIM * METRIC_DIM];
    for (i, &node) in [G_00, G_11, G_22, G_33].iter().enumerate() {
        data[i * METRIC_DIM + i] = read(world, node)?;
    }
    Ok(CausalTensor::new(data, vec![METRIC_DIM, METRIC_DIM])?)
}

/// The Einstein tensor `G_uv ~ R · g_uv`, the simplified left-hand side of the field equations.
///
/// `Pure` lifts the scaling law into `CausalTensorWitness` and `Applicative` applies it at every
/// component, so the law is written once and the tensor shape carries it.
fn einstein_tensor(
    g_uv: &CausalTensor<FloatType>,
    curvature: FloatType,
) -> CausalTensor<FloatType> {
    let scale = move |x: FloatType| x * curvature;
    CausalTensorWitness::apply(CausalTensorWitness::pure(scale), g_uv.clone())
}

// --- Magnetohydrodynamics: the multivector engine ---

/// The Lorentz force density `F = J · B`, read in the poloidal direction.
///
/// The current flows along `x` and the confinement field spans the `x ∧ y` plane, so the inner
/// product of the two lands on `y`.
fn lorentz_force(
    current_density: FloatType,
    magnetic_field: FloatType,
    metric: Metric,
) -> Result<FloatType, CausalMultiVectorError> {
    let current = multivector(&[(E_X, current_density)], metric)?;
    let field = multivector(&[(E_XY, magnetic_field)], metric)?;

    let force = current.inner_product(&field);
    blade(&force, E_Y)
}

/// A multivector holding the given coefficients at the given blade indices, zero elsewhere.
fn multivector(
    components: &[(usize, FloatType)],
    metric: Metric,
) -> Result<CausalMultiVector<FloatType>, CausalMultiVectorError> {
    let mut data = vec![ZERO; 1 << metric.dimension()];
    for &(index, value) in components {
        data[index] = value;
    }
    CausalMultiVector::new(data, metric)
}

/// One coefficient of a multivector, read by blade index.
fn blade(
    field: &CausalMultiVector<FloatType>,
    index: usize,
) -> Result<FloatType, CausalMultiVectorError> {
    field
        .get(index)
        .copied()
        .ok_or_else(|| CausalMultiVectorError::data_length_mismatch(index + 1, field.data().len()))
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_header() {
    println!("============================================================");
    println!("   GRMHD: General Relativistic Magnetohydrodynamics");
    println!("============================================================");
    println!("Precision: {}\n", core::any::type_name::<FloatType>());
}

fn print_step_one() {
    println!("\n[Step 1] GR Solver: Calculating Spacetime Curvature...");
}

/// The display boundary: `f64` appears here and nowhere else.
fn print_curvature(time_dilation: FloatType) {
    println!(
        "   -> Local Curvature Intensity: {:.4}",
        lower(time_dilation)
    );
}

fn print_step_two() {
    println!("\n[Step 2] Causal Coupling: Configuring MHD Solver...");
}

fn print_metric_choice(label: &str) {
    println!("   -> Selected Metric: {label}");
}

fn print_step_three() {
    println!("\n[Step 3] MHD Solver: Calculating Plasma Confinement...");
}

fn print_force(force: FloatType) {
    println!("   -> Lorentz Force Density: {:.4}", lower(force));
}

fn print_analysis(reversed: bool) {
    println!("\n[Step 4] Analysis:");
    if reversed {
        println!("   STATUS: Relativistic Reversal Detected!");
        println!("   Action: Adjusting containment field to compensate for frame dragging.");
    } else {
        println!("   STATUS: Standard Confinement.");
    }
}

fn print_footer() {
    println!("\n============================================================");
    println!("CONCLUSION:");
    println!("We successfully coupled a Tensor-based GR solver with a");
    println!("MultiVector-based MHD solver in a single executable.");
    println!("Data flowed from Spacetime Geometry -> Coupling -> Plasma Physics.");
    println!("============================================================");
}

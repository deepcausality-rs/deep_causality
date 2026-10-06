/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_context::{
    Context, ContextIndexError, Contextoid, ContextoidId, ContextoidType, ContextuableGraph, Data,
    NoSpace, NoSpaceTime, NoTime,
};
use deep_causality_multivector::{CausalMultiVector, CausalMultiVectorError, Metric, MultiVector};
use deep_causality_num::{const_scalar_from_int, lift, lower};

// -----------------------------------------------------------------------------------------
// ENGINEERING VALUE:
// Plasma Physics and Fusion research often involve switching between Classical (Euclidean)
// and Relativistic (Minkowski) regimes. Errors in metric signatures can lead to
// catastrophic simulation failures (e.g., calculating force in the wrong direction).
//
// This example demonstrates "Metric Agnosticism":
// The same code `F = J . B` correctly calculates the Lorentz Force in BOTH regimes.
// The `CausalMultiVector` type handles the underlying metric algebra (spacetime signature),
// ensuring physical correctness and safety at the type level.
//
// Each regime is a reactor scenario held in a `Context`: the signature of the algebra, the
// axes the current and the field lie along in it, and the two magnitudes. The calculation
// reads all of them from the scenario it is handed.
// -----------------------------------------------------------------------------------------

/// The working scalar. Current, field and the force they produce all carry it.
pub type FloatType = f64;

/// Small numbers, declared once at the working type rather than lifted at each use.
const ZERO: FloatType = const_scalar_from_int!(FloatType, 0);

/// One quantity of a reactor scenario: the signature of the algebra its plasma is modelled in, a
/// basis axis of that algebra, or a magnitude. A `Context` holds one payload type, so the three
/// kinds share this enum.
#[derive(Debug, Clone, PartialEq)]
enum ReactorQuantity {
    Signature(Metric),
    Axis(usize),
    Magnitude(FloatType),
}

/// `Data` asks its payload for a default, and `Metric` has none, so the default is the zero
/// magnitude. `reactor` sets every node, so no node holds the default.
impl Default for ReactorQuantity {
    fn default() -> Self {
        Self::Magnitude(ZERO)
    }
}

/// A reactor scenario, one `Data` node per quantity. Its geometry is the algebra's basis, so the
/// spatial, temporal and spacetime slots are empty.
type ReactorContext =
    Context<Data<ReactorQuantity>, NoSpace<FloatType>, NoTime, NoSpaceTime<FloatType>>;

/// Contextoid id: the signature of the algebra the plasma is modelled in.
const SIGNATURE: ContextoidId = 1;
/// Contextoid id: the basis axis the current flows along, toroidally.
const TOROIDAL_AXIS: ContextoidId = 2;
/// Contextoid id: the basis axis that spans the field plane with the toroidal one, poloidally.
const POLOIDAL_AXIS: ContextoidId = 3;
/// Contextoid id: the plasma current J, in MA.
const CURRENT: ContextoidId = 4;
/// Contextoid id: the confining magnetic field B, in T.
const FIELD: ContextoidId = 5;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_header();

    // Case A: a stationary reactor, modelled in classical Euclidean geometry.
    print_scenario("A", "Stationary Plasma Fusion (Classical Euclidean Metric)");
    let stationary = reactor(
        "stationary",
        Metric::Euclidean(3),
        0, // current flowing toroidally (x-axis)
        1, // magnetic field applied poloidally (y-axis plane)
    )?;
    calculate_confinement_force(&stationary)?;

    print_separator();

    // Case B: the same calculation in relativistic Minkowski spacetime. The axis indices
    // shift by one because the time dimension comes first.
    print_scenario("B", "Mobile Relativistic Plasma Fusion  (Minkowski Metric)");
    let relativistic = reactor("relativistic", Metric::Minkowski(4), 1, 2)?;
    calculate_confinement_force(&relativistic)?;

    print_explenation();
    Ok(())
}

/// A reactor scenario: the algebra its plasma is modelled in, the axes the current and the field
/// lie along in that algebra, and the plasma current and confining field both scenarios share.
fn reactor(
    label: &str,
    metric: Metric,
    toroidal_axis: usize,
    poloidal_axis: usize,
) -> Result<ReactorContext, ContextIndexError> {
    let facts = [
        (SIGNATURE, ReactorQuantity::Signature(metric)),
        (TOROIDAL_AXIS, ReactorQuantity::Axis(toroidal_axis)),
        (POLOIDAL_AXIS, ReactorQuantity::Axis(poloidal_axis)),
        // Plasma current J: a strong current around the torus, on the order of 10 MA.
        (CURRENT, ReactorQuantity::Magnitude(lift(10.0))),
        // Confining magnetic field B, perpendicular to the current, in Tesla.
        (FIELD, ReactorQuantity::Magnitude(lift(2.0))),
    ];

    let mut context = Context::with_capacity(1, label, facts.len());
    for (id, quantity) in facts {
        context.add_node(Contextoid::new(
            id,
            ContextoidType::Datoid(Data::new(id, quantity)),
        ))?;
    }
    Ok(context)
}

/// Read one quantity out of a reactor scenario.
fn read(context: &ReactorContext, id: ContextoidId) -> Result<ReactorQuantity, ContextIndexError> {
    context.get_data_by_id(id).ok_or_else(|| {
        ContextIndexError::new(format!("no reactor quantity with contextoid id {id}"))
    })
}

/// Calculates the Lorentz Force Density in a Fusion Reactor.
///
/// In Plasma Physics, the force density **F** acting on the fluid is the interaction
/// between the Current Density **J** and the Magnetic Field **B**.
///
/// $$ F = J \cdot B $$
/// (In Geometric Algebra, the contraction of a Vector current and Bivector field).
///
/// The signature, the axes, `J` and `B` are read from the scenario.
fn calculate_confinement_force(reactor: &ReactorContext) -> Result<(), Box<dyn std::error::Error>> {
    let (
        ReactorQuantity::Signature(metric),
        ReactorQuantity::Axis(toroidal_axis),
        ReactorQuantity::Axis(poloidal_axis),
        ReactorQuantity::Magnitude(j_val),
        ReactorQuantity::Magnitude(b_val),
    ) = (
        read(reactor, SIGNATURE)?,
        read(reactor, TOROIDAL_AXIS)?,
        read(reactor, POLOIDAL_AXIS)?,
        read(reactor, CURRENT)?,
        read(reactor, FIELD)?,
    )
    else {
        return Err(ContextIndexError::new(format!(
            "scenario {} holds a quantity of the wrong kind",
            reactor.name()
        ))
        .into());
    };
    if let Some(axis) = [toroidal_axis, poloidal_axis]
        .into_iter()
        .find(|&axis| axis >= metric.dimension())
    {
        return Err(ContextIndexError::new(format!(
            "scenario {} names axis {axis}, outside its {}-dimensional algebra",
            reactor.name(),
            metric.dimension()
        ))
        .into());
    }

    // 1. Setup Reactor Geometry
    let idx_current = 1 << toroidal_axis;
    let idx_field_plane = (1 << toroidal_axis) | (1 << poloidal_axis);
    let idx_force_direction = 1 << poloidal_axis;

    print_geometry(toroidal_axis, poloidal_axis);

    // 2. Plasma current J, along the toroidal axis.
    let j_vec = blade(idx_current, j_val, metric)?;

    // 3. Confining magnetic field B, in the plane of the two axes.
    let b_field = blade(idx_field_plane, b_val, metric)?;
    print_inputs(j_val, b_val);

    // 4. The physics, in one line that holds for any geometry: F = J . B.
    let force = j_vec.inner_product(&b_field);
    let force_val = force.get(idx_force_direction).copied().ok_or_else(|| {
        CausalMultiVectorError::data_length_mismatch(idx_force_direction + 1, force.data().len())
    })?;

    // 5. The sign is the reactor safety check. Euclidean (+1) gives the standard
    //    cross-product direction; Minkowski (-1) reverses it through the spacetime signature.
    print_force(force_val, poloidal_axis);
    Ok(())
}

/// A multivector carrying `value` on a single blade of `metric`'s algebra.
fn blade(
    index: usize,
    value: FloatType,
    metric: Metric,
) -> Result<CausalMultiVector<FloatType>, CausalMultiVectorError> {
    let mut data = vec![ZERO; 1 << metric.dimension()];
    let found = data.len();
    let slot = data
        .get_mut(index)
        .ok_or_else(|| CausalMultiVectorError::data_length_mismatch(index + 1, found))?;
    *slot = value;
    CausalMultiVector::new(data, metric)
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_header() {
    println!("--- PLASMA FUSION SIMULATION: Reactor Confinement Check ---");
    println!("Context: A Stationary Tokamak Reactor (e.g., ITER).");
    println!(
        "Goal: Calculate the Lorentz Force vector to ensure Plasma is confined away from the walls."
    );
    println!(
        "Problem: Relativistic effects in high-energy plasma can alter geometric interactions."
    );
    println!("Solution: Use Geometric Algebra to automatically handle the spacetime signature.\n");
}

fn print_scenario(label: &str, description: &str) {
    println!(">> SCENARIO {label}: {description}");
}

fn print_separator() {
    println!("\n------------------------------------------------------------\n");
}

fn print_geometry(toroidal_axis: usize, poloidal_axis: usize) {
    println!(
        "  [Geometry] Current Axis: e_{toroidal_axis}, Field Plane: e_{toroidal_axis}e_{poloidal_axis}"
    );
}

/// The display boundary: `f64` appears here and nowhere else.
fn print_inputs(j: FloatType, b: FloatType) {
    println!("  [Input] Plasma Current J: {:.1}", lower(j));
    println!("  [Input] Magnetic Field B: {:.1}", lower(b));
}

fn print_force(force: FloatType, poloidal_axis: usize) {
    println!(
        "  [Output] Lorentz Force F: {:.2} e_{}",
        lower(force),
        poloidal_axis
    );
    if lower(force) > 0.0 {
        println!("  => STATUS: Classical behavior. Force pushes +Y.");
    } else {
        println!("  => STATUS: Relativistic signature detected. Force pushes -Y.");
        println!(
            "     (NOTE: In a simulation, this sign flip must be accounted for to prevent wall collision!)"
        );
    }
}

fn print_explenation() {
    println!("\n============================================================");
    println!("WHAT THIS MEANS FOR COMPUTATIONAL PHYSICS:");
    println!("1. Metric Agnosticism: The exact same code 'force = J . B' calculated");
    println!("   the correct geometric result for both Classical and Relativistic systems.");
    println!("   Standard codes require manual 'if/else' logic to handle relativistic sign flips.");
    println!();
    println!("2. Safety: In Plasma Fusion, mixing up coordinate systems or metric signatures");
    println!("   causes 'Magnetic Monopole' errors or incorrect force directions.");
    println!("   Here, the Algebra enforces the laws of physics at the Type Level.");
    println!("============================================================");
    //
    // Furthermore, this architecture supports General Relativistic metrics, paving the
    // way for modeling Magnetohydrodynamics in curved spacetime i.e. for plasma fusion
    // based Space Propulsion systems i.e. Direct Fusion Drives.
}

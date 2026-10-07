/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # The electromagnetic field as one bivector
//!
//! Geometric algebra carries the electric and magnetic fields in a single object: the
//! electromagnetic bivector `F`, which follows from the 4-vector potential `A` by one geometric
//! product with the spacetime gradient, `F = ∇A`.
//!
//! This example takes a linearly polarised plane wave travelling along `z`, whose potential is
//! `A_x(t, z) = cos(ω(t − z))`, samples it at one spacetime point, and reads three quantities
//! out of `F`:
//!
//! ```text
//! grade 0        ∇·A    the Lorenz gauge condition
//! e_t ∧ e_x      E_x    the electric field
//! e_z ∧ e_x      B_y    the magnetic field
//! ```
//!
//! The partials `∂A_x/∂t` and `∂A_x/∂z` come from `DifferentiableField`. The potential is written
//! once as a scalar-generic function, and `gradient` evaluates it over `Dual`, so the `ε` channel
//! carries the exact derivative at the sample point.
//!
//! The wave's angular frequency and the sample event live in a `Context`, one `Data` node each, in
//! natural units with `c = 1`. The potential is built from the frequency, and the field is sampled
//! at the event.
//!
//! **Application: phased-array antenna design.** Beamforming holds phase across thousands of
//! radiating elements. Carrying the potential `A` propagates four scalars through the mesh, and
//! the observable field `F` follows from them by one product, so gauge invariance and phase
//! consistency are properties of the representation and hold at every sample point.

use deep_causality_algebra::Real;
use deep_causality_calculus::{DifferentiableField, DifferentiateFieldExt, Scalar};
use deep_causality_context::{
    Context, ContextIndexError, Contextoid, ContextoidId, ContextoidType, ContextuableGraph, Data,
    NoSpace, NoSpaceTime, NoTime,
};
use deep_causality_multivector::{CausalMultiVector, CausalMultiVectorError, Metric};
use deep_causality_num::{const_scalar_from_float, const_scalar_from_int, lift, lower};

/// The plane wave and the event it is sampled at, one `Data` node each. The nodes hold `f64`
/// because the scalar-generic field below evaluates at a scalar the caller names, which a node of
/// the working type cannot reach; every use lifts its value into that scalar. The event is in
/// `c = 1` units and the `(+, -, -, -)` signature of `Cl(1,3)`, so it is a pair of `Data` nodes
/// and the spatial, temporal and spacetime slots are empty.
type WaveContext = Context<Data<f64>, NoSpace<FloatType>, NoTime, NoSpaceTime<FloatType>>;

/// Contextoid id: the time `t` of the event the field is sampled at, in `c = 1` units.
const SAMPLE_T: ContextoidId = 1;
/// Contextoid id: the position `z` of the event the field is sampled at, in `c = 1` units; `x` and
/// `y` are at the origin.
const SAMPLE_Z: ContextoidId = 2;
/// Contextoid id: the angular frequency `ω` of the wave, in `c = 1` units.
const OMEGA: ContextoidId = 3;

/// `Cl(1,3)` holds `2^4` coefficients.
const COEFFICIENTS: usize = 16;
/// Blade indices in `Cl(1,3)`: each axis owns one bit, and a product owns the bits of its factors.
const SCALAR: usize = 0;
const E_T: usize = 1;
const E_X: usize = 2;
const E_Z: usize = 4;
const E_TX: usize = E_T | E_X;
const E_ZX: usize = E_Z | E_X;

/// The tolerance a gauge residual and a field-strength difference are read as zero within.
const TOLERANCE: FloatType = const_scalar_from_float!(FloatType, 1e-9);

/// The working scalar. The potential, its partials and every blade of `F` carry it.
pub type FloatType = f64;

/// Small numbers, declared once at the working type rather than lifted at each use.
const ZERO: FloatType = const_scalar_from_int!(FloatType, 0);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_header();
    let world = plane_wave()?;

    // Spacetime with the Minkowski signature: time +, space − − −.
    let metric = Metric::Minkowski(4);

    let t: FloatType = lift(read(&world, SAMPLE_T)?);
    let z: FloatType = lift(read(&world, SAMPLE_Z)?);

    // The 4-vector potential of a linearly polarised wave, purely spatial along x in this gauge.
    let wave = PlaneWavePotential {
        omega: read(&world, OMEGA)?,
    };
    let a_x = wave.run(&[t, z]);
    let potential_a = multivector(&[(E_X, a_x)], metric)?;
    print_potential(a_x);

    // The gradient ∇ = e^μ ∂_μ, with its components supplied by the tangent functor: the
    // potential is evaluated over `Dual`, and the ε channel returns ∂A_x/∂t and ∂A_x/∂z exactly.
    let [da_dt, da_dz] = wave.gradient(&[t, z]);
    let gradient = multivector(&[(E_T, da_dt), (E_Z, da_dz)], metric)?;

    // F = ∇A. The geometric product splits into the inner part, which carries the divergence,
    // and the outer part, which carries the field strength.
    let field_f = gradient.geometric_product(&potential_a);

    // The grade-0 part is ∇·A. This gauge sets A_x as a function of t and z alone, so the
    // divergence reduces to ∂_x A_x and reads zero.
    let divergence = blade(&field_f, SCALAR)?;
    print_gauge(divergence, Real::abs(divergence) < TOLERANCE);

    // The bivector parts are the observable fields: a time-space blade for E, a space-space
    // blade for B.
    let e_field = blade(&field_f, E_TX)?;
    let b_field = blade(&field_f, E_ZX)?;
    print_fields(e_field, b_field);

    // A plane wave in natural units carries equal field strengths, which is the statement that
    // it propagates at c.
    let equal_strength = Real::abs(Real::abs(e_field) - Real::abs(b_field)) < TOLERANCE;
    print_footer(equal_strength);

    Ok(())
}

/// The sampled wave, one `Data` node per quantity.
fn plane_wave() -> Result<WaveContext, ContextIndexError> {
    let facts = [(SAMPLE_T, 1.0), (SAMPLE_Z, 0.5), (OMEGA, 1.0)];

    let mut context = Context::with_capacity(1, "plane wave", facts.len());
    for (id, value) in facts {
        context.add_node(Contextoid::new(
            id,
            ContextoidType::Datoid(Data::new(id, value)),
        ))?;
    }
    Ok(context)
}

/// Read one quantity out of the wave's context.
fn read(context: &WaveContext, id: ContextoidId) -> Result<f64, ContextIndexError> {
    context
        .get_data_by_id(id)
        .ok_or_else(|| ContextIndexError::new(format!("no wave quantity with contextoid id {id}")))
}

/// A multivector holding the given coefficients at the given blade indices, zero elsewhere. An
/// index outside the algebra is an error.
fn multivector(
    components: &[(usize, FloatType)],
    metric: Metric,
) -> Result<CausalMultiVector<FloatType>, CausalMultiVectorError> {
    let mut data = vec![ZERO; COEFFICIENTS];
    let found = data.len();
    for &(index, value) in components {
        let slot = data.get_mut(index).ok_or_else(|| {
            CausalMultiVectorError::data_length_mismatch(index.saturating_add(1), found)
        })?;
        *slot = value;
    }
    CausalMultiVector::new(data, metric)
}

/// One coefficient of a multivector, read by blade index.
fn blade(
    field: &CausalMultiVector<FloatType>,
    index: usize,
) -> Result<FloatType, CausalMultiVectorError> {
    field.get(index).copied().ok_or_else(|| {
        CausalMultiVectorError::data_length_mismatch(index.saturating_add(1), field.data().len())
    })
}

/// The plane-wave potential component `A_x(t, z) = cos(ω(t − z))`, written once as a
/// scalar-generic field. The same definition serves the value at `FloatType` and the partials at
/// `Dual<FloatType>`, which is what `gradient` calls it with.
///
/// The struct holds the angular frequency read from the context. `run` works at a scalar the
/// *caller* names, which a value of the working type cannot reach, so the frequency is held at
/// `f64` and lifted per scalar.
struct PlaneWavePotential {
    omega: f64,
}

impl DifferentiableField<2> for PlaneWavePotential {
    fn run<S: Scalar>(&self, tz: &[S; 2]) -> S {
        let omega = lift::<S>(self.omega);
        (omega * (tz[0] - tz[1])).cos()
    }
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_header() {
    println!("--- MAXWELL'S UNIFICATION: The Geometric Gradient ---");
    println!("Goal: Derive E and B fields from a single Vector Potential A.");
    println!("Check: Verify the Lorenz Gauge condition (Divergence = 0).\n");
    println!("Precision: {}\n", core::any::type_name::<FloatType>());
}

/// The display boundary: `f64` appears here and nowhere else.
fn print_potential(a_x: FloatType) {
    println!("Vector Potential A: {:.4} e_x", lower(a_x));
}

fn print_gauge(divergence: FloatType, satisfied: bool) {
    if satisfied {
        println!(">> SUCCESS: Lorenz Gauge Satisfied (Divergence ~ 0).");
    } else {
        println!(">> WARNING: Gauge Broken. Div = {:.4}", lower(divergence));
    }
}

fn print_fields(e_field: FloatType, b_field: FloatType) {
    println!("\n>> Extracted Physical Fields:");
    println!(
        "   Electric Field E (Time-Space Bivector): {:.4}",
        lower(e_field)
    );
    println!(
        "   Magnetic Field B (Space-Space Bivector): {:.4}",
        lower(b_field)
    );
}

fn print_footer(equal_strength: bool) {
    if equal_strength {
        println!(">> PHYSICS VERIFIED: |E| = |B|. Wave propagating at c.");
    }
}

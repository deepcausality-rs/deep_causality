/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Laser Resonator Stability
//!
//! A Gaussian beam makes one **full** round trip of a linear cavity: two flat mirrors with a
//! thermal lens between them. Each optical element is a named stage and the six compose into one
//! `CausalFlow` pipeline that threads the complex beam parameter `q`.
//!
//! Whether a resonator is stable is a property of its round-trip matrix, not of the beam at any
//! one point. For a round trip `[[A, B], [C, D]]` the cavity is stable exactly when
//!
//! ```text
//! m = (A + D) / 2      and      -1 <= m <= 1
//! ```
//!
//! so the run builds that matrix by multiplying the element matrices in traversal order, reports
//! `m`, and then checks the consequence that makes `m` meaningful: a stable cavity has a
//! self-reproducing mode, so the `q` that comes back must be the `q` that set out.
//!
//! For the cavity below the round trip comes out as exactly `-I`, giving `m = -1`. That is the
//! *boundary* of the stability range, which is worth seeing: the beam does reproduce itself, but
//! the cavity has no margin, and any drift in the thermal lens pushes it out.
//!
//! ## APIs Demonstrated
//! - `CausalFlow::value` and `.bind` once per optical element
//! - `gaussian_q_propagation` and `beam_spot_size`
//! - `EinSumOp::mat_mul` to accumulate the round-trip matrix

use deep_causality_algebra::{DivisionAlgebra, Real};
use deep_causality_core::{CausalEffect, CausalFlow, PropagatingEffect, PropagatingProcess};
use deep_causality_num::{const_scalar_from_float, const_scalar_from_int, lower};
use deep_causality_num_complex::Complex;
use deep_causality_physics::{
    AbcdMatrix, ComplexBeamParameter, IndexOfRefraction, PhysicsError, Wavelength, beam_spot_size,
    gaussian_q_propagation, lens_maker,
};
use deep_causality_tensor::{CausalTensor, EinSumOp, Tensor};

/// Small whole numbers, declared once at the working type.
const ZERO: FloatType = const_scalar_from_int!(FloatType, 0);
const ONE: FloatType = const_scalar_from_int!(FloatType, 1);
const TWO: FloatType = const_scalar_from_int!(FloatType, 2);

/// Cavity geometry, in metres.
const DRIFT_L1: FloatType = const_scalar_from_float!(FloatType, 0.5);
const DRIFT_L2: FloatType = const_scalar_from_float!(FloatType, 0.5);
/// Radius of curvature of the biconvex thermal lens, in metres.
const LENS_RADIUS: FloatType = const_scalar_from_float!(FloatType, 0.5);
/// Refractive index of the lens material.
const LENS_INDEX: FloatType = const_scalar_from_float!(FloatType, 1.5);
/// Nd:YAG wavelength, 1064 nm.
const WAVELENGTH_M: FloatType = const_scalar_from_float!(FloatType, 1064e-9);
/// Input beam waist, 1 mm.
const WAIST_M: FloatType = const_scalar_from_float!(FloatType, 1e-3);
/// Metres to millimetres and to nanometres, for the display boundary.
const MM_PER_M: FloatType = const_scalar_from_int!(FloatType, 1000);
const NM_PER_M: FloatType = const_scalar_from_int!(FloatType, 1_000_000_000);

/// A cavity counts as stable when `|m| <= 1`; this is the slack allowed on that comparison.
const STABILITY_MARGIN: FloatType = const_scalar_from_float!(FloatType, 1e-12);
/// How closely the round trip must reproduce the input `q` to call the mode self-consistent.
const EIGENMODE_TOLERANCE: FloatType = const_scalar_from_float!(FloatType, 1e-12);

/// `f64` is the right precision here: the round trip is six 2x2 products and one Moebius map, so
/// the eigenmode residual sits at machine epsilon either way. `Float106` tightens the residual
/// and leaves every reported spot size unchanged.
pub type FloatType = f64;

fn main() -> Result<(), PhysicsError> {
    print_header();

    let wavelength = Wavelength::<FloatType>::new(WAVELENGTH_M)?;
    let focal_length = thermal_lens_focal_length()?;

    // At a waist the wavefront is flat, so q = i z_R with z_R = pi w0^2 / lambda.
    let rayleigh = FloatType::pi() * WAIST_M * WAIST_M / wavelength.value();
    let q_initial = ComplexBeamParameter::new(Complex::new(ZERO, rayleigh))?;
    print_input(WAIST_M, rayleigh, focal_length);

    // One full round trip: out through the lens to the far mirror, and back again. Flat mirrors
    // contribute the identity, so they are the reflections between the two passes.
    let cavity = round_trip(focal_length)?;

    let process = cavity
        .iter()
        .fold(CausalFlow::value(q_initial), |flow, element| {
            let matrix = element.matrix.clone();
            flow.bind(move |value, _s, _c| propagate(value, &matrix))
        })
        .into_process();

    let q_final = match process.value() {
        Some(q) => *q,
        None => {
            print_failure(process.error().map(|e| format!("{e:?}")));
            return Ok(());
        }
    };

    print_trace(&trace(q_initial, &cavity, wavelength)?);
    print_stability(&stability(&cavity, q_initial, q_final)?);

    Ok(())
}

/// Propagates `q` through one element, or short-circuits when the upstream carried no beam.
fn propagate(
    value: CausalEffect<ComplexBeamParameter<FloatType>>,
    matrix: &AbcdMatrix<FloatType>,
) -> PropagatingProcess<ComplexBeamParameter<FloatType>, (), ()> {
    let q = match value.into_value() {
        Some(q) => q,
        None => return fail("the flow carried no beam parameter"),
    };
    match gaussian_q_propagation(q, matrix).value_cloned() {
        // A physical beam keeps Im(q) > 0. Once that fails the beam has diffracted away and the
        // flow enters the error channel rather than reporting a spot size for a lost beam.
        Some(next) if next.value().im > ZERO => PropagatingEffect::pure(next),
        Some(_) => fail("beam diverged: Im(q) <= 0"),
        None => fail("q propagation produced no value"),
    }
}

fn fail<T: Default + Clone + core::fmt::Debug>(
    reason: impl Into<String>,
) -> PropagatingProcess<T, (), ()> {
    PropagatingEffect::from_error(deep_causality_core::CausalityError::new(
        deep_causality_core::CausalityErrorEnum::Custom(reason.into()),
    ))
}

/// One optical element of the cavity.
#[derive(Clone)]
struct Element {
    label: &'static str,
    matrix: AbcdMatrix<FloatType>,
}

/// The six elements a ray meets on one round trip, in traversal order.
fn round_trip(focal_length: FloatType) -> Result<Vec<Element>, PhysicsError> {
    Ok(vec![
        Element {
            label: "drift L1",
            matrix: drift(DRIFT_L1)?,
        },
        Element {
            label: "thermal lens",
            matrix: thin_lens(focal_length)?,
        },
        Element {
            label: "drift L2",
            matrix: drift(DRIFT_L2)?,
        },
        // The far mirror is flat, so it reflects without focusing: the identity matrix.
        Element {
            label: "drift L2 (return)",
            matrix: drift(DRIFT_L2)?,
        },
        Element {
            label: "thermal lens (return)",
            matrix: thin_lens(focal_length)?,
        },
        Element {
            label: "drift L1 (return)",
            matrix: drift(DRIFT_L1)?,
        },
    ])
}

/// Free-space propagation, `[[1, L], [0, 1]]`.
fn drift(length: FloatType) -> Result<AbcdMatrix<FloatType>, PhysicsError> {
    matrix(ONE, length, ZERO, ONE)
}

/// A thin lens, `[[1, 0], [-1/f, 1]]`.
fn thin_lens(focal_length: FloatType) -> Result<AbcdMatrix<FloatType>, PhysicsError> {
    matrix(ONE, ZERO, -ONE / focal_length, ONE)
}

fn matrix(
    a: FloatType,
    b: FloatType,
    c: FloatType,
    d: FloatType,
) -> Result<AbcdMatrix<FloatType>, PhysicsError> {
    CausalTensor::new(vec![a, b, c, d], vec![2, 2])
        .map(AbcdMatrix::new)
        .map_err(|e| PhysicsError::DimensionMismatch(format!("2x2 ABCD matrix: {e:?}")))
}

/// Focal length of the biconvex thermal lens, from the lens-maker equation.
fn thermal_lens_focal_length() -> Result<FloatType, PhysicsError> {
    let index = IndexOfRefraction::<FloatType>::new(LENS_INDEX)?;
    let power = lens_maker(index, LENS_RADIUS, -LENS_RADIUS)
        .value_cloned()
        .ok_or_else(|| PhysicsError::NumericalInstability("lens_maker".into()))?;
    Ok(ONE / power.value())
}

/// The beam at the exit of each element, for reporting.
struct Sample {
    label: &'static str,
    spot_size: FloatType,
    curvature: Option<FloatType>,
}

fn trace(
    q_initial: ComplexBeamParameter<FloatType>,
    cavity: &[Element],
    wavelength: Wavelength<FloatType>,
) -> Result<Vec<Sample>, PhysicsError> {
    let mut q = q_initial;
    let mut samples = Vec::with_capacity(cavity.len() + 1);
    samples.push(sample("input", q, wavelength)?);
    for element in cavity {
        q = gaussian_q_propagation(q, &element.matrix)
            .value_cloned()
            .ok_or_else(|| PhysicsError::NumericalInstability("q propagation".into()))?;
        samples.push(sample(element.label, q, wavelength)?);
    }
    Ok(samples)
}

fn sample(
    label: &'static str,
    q: ComplexBeamParameter<FloatType>,
    wavelength: Wavelength<FloatType>,
) -> Result<Sample, PhysicsError> {
    let spot_size = beam_spot_size(q, wavelength)
        .value()
        .ok_or_else(|| PhysicsError::NumericalInstability("beam_spot_size".into()))?
        .value();

    // R = |q|^2 / Re(q). At a waist Re(q) is zero and the wavefront is plane, which has no
    // finite radius; report that as absent rather than dividing by zero and printing `inf`.
    let re = q.value().re;
    let curvature = if Real::abs(re) > EIGENMODE_TOLERANCE {
        Some(q.value().norm_sqr() / re)
    } else {
        None
    };

    Ok(Sample {
        label,
        spot_size,
        curvature,
    })
}

/// What decides whether the cavity is a resonator at all.
struct Stability {
    a: FloatType,
    d: FloatType,
    m: FloatType,
    eigenmode_residual: FloatType,
    stable: bool,
    marginal: bool,
    self_reproducing: bool,
}

fn stability(
    cavity: &[Element],
    q_initial: ComplexBeamParameter<FloatType>,
    q_final: ComplexBeamParameter<FloatType>,
) -> Result<Stability, PhysicsError> {
    // ABCD matrices compose against the direction of travel, so the element met first sits
    // rightmost in the product.
    let mut round = identity()?;
    for element in cavity {
        round = multiply(element.matrix.inner(), &round)?;
    }
    let r = round.as_slice();
    let (a, d) = (r[0], r[3]);
    let m = (a + d) / TWO;
    let eigenmode_residual = (q_final.value() - q_initial.value()).norm_sqr();

    Ok(Stability {
        a,
        d,
        m,
        eigenmode_residual,
        stable: Real::abs(m) <= ONE + STABILITY_MARGIN,
        marginal: Real::abs(Real::abs(m) - ONE) <= STABILITY_MARGIN,
        self_reproducing: eigenmode_residual < EIGENMODE_TOLERANCE,
    })
}

fn identity() -> Result<CausalTensor<FloatType>, PhysicsError> {
    CausalTensor::new(vec![ONE, ZERO, ZERO, ONE], vec![2, 2])
        .map_err(|e| PhysicsError::DimensionMismatch(format!("2x2 identity: {e:?}")))
}

fn multiply(
    lhs: &CausalTensor<FloatType>,
    rhs: &CausalTensor<FloatType>,
) -> Result<CausalTensor<FloatType>, PhysicsError> {
    CausalTensor::ein_sum(&EinSumOp::mat_mul(lhs.clone(), rhs.clone()))
        .map_err(|e| PhysicsError::DimensionMismatch(format!("ABCD product: {e:?}")))
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_header() {
    println!("=== Laser Resonator Stability ===");
    println!("Precision: {}\n", core::any::type_name::<FloatType>());
}

/// The display boundary: `f64` appears here and nowhere else.
fn print_input(waist: FloatType, rayleigh: FloatType, focal_length: FloatType) {
    println!(
        "Cavity:     flat mirror | {:.1} m | thermal lens | {:.1} m | flat mirror",
        lower(DRIFT_L1),
        lower(DRIFT_L2)
    );
    println!("Wavelength: {:.1} nm", lower(WAVELENGTH_M * NM_PER_M));
    println!(
        "Lens:       f = {:.3} m (biconvex, n = {:.1}, R = {:.1} m)",
        lower(focal_length),
        lower(LENS_INDEX),
        lower(LENS_RADIUS)
    );
    println!(
        "Input beam: w0 = {:.2} mm at a waist, z_R = {:.3} m\n",
        lower(waist * MM_PER_M),
        lower(rayleigh)
    );
}

fn print_trace(samples: &[Sample]) {
    println!("--- One full round trip ---");
    println!("  {:<22} {:>12} {:>14}", "at", "w (mm)", "R (m)");
    for s in samples {
        match s.curvature {
            Some(r) => println!(
                "  {:<22} {:>12.4} {:>14.4}",
                s.label,
                lower(s.spot_size * MM_PER_M),
                lower(r)
            ),
            None => println!(
                "  {:<22} {:>12.4} {:>14}",
                s.label,
                lower(s.spot_size * MM_PER_M),
                "plane"
            ),
        }
    }
}

fn print_stability(s: &Stability) {
    println!("\n--- Stability of the round trip ---");
    println!(
        "  round-trip A, D          = {:.6}, {:.6}",
        lower(s.a),
        lower(s.d)
    );
    println!("  m = (A + D) / 2          = {:.6}", lower(s.m));
    println!("  stable when -1 <= m <= 1 = {}", s.stable);
    if s.marginal {
        println!("  |m| = 1 exactly: the cavity sits on the stability boundary, so it has a");
        println!("  self-reproducing mode but no margin against lens drift.");
    }
    println!(
        "\n  |q_out - q_in|^2         = {:.2e}   (a stable cavity reproduces its own mode)",
        lower(s.eigenmode_residual)
    );
    println!(
        "  => {}",
        if s.self_reproducing {
            "the round trip returns the beam it started with"
        } else {
            "the beam did NOT reproduce"
        }
    );
}

fn print_failure(error: Option<String>) {
    match error {
        Some(e) => println!("\nThe beam was lost: {e}"),
        None => println!("\nThe cavity produced no final beam."),
    }
}

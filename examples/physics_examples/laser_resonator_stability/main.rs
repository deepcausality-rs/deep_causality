/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Laser Resonator Stability
//!
//! A Gaussian beam makes one **full** round trip of a linear cavity: two flat mirrors with a
//! thermal lens between them. Each optical element is a named stage and the six compose into one
//! `CausalFlow` pipeline that threads the complex beam parameter `q`. The cavity, its two drift
//! lengths, the lens's radius and index, the laser wavelength and the input waist, is the
//! pipeline's context: the input `q` is built from it, and each element builds its ABCD matrix
//! from it.
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
//! - `CausalFlow::value`, `.context` and `.bind` once per optical element
//! - A `Context` of `Data` contextoids as the cavity the elements are built from
//! - `gaussian_q_propagation` and `beam_spot_size`
//! - `EinSumOp::mat_mul` to accumulate the round-trip matrix

use deep_causality_algebra::{DivisionAlgebra, Real};
use deep_causality_context::{
    Context, ContextIndexError, Contextoid, ContextoidType, ContextuableGraph, Data, Datable,
    NoSpace, NoSpaceTime, NoTime,
};
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

/// Metres to millimetres and to nanometres, for the display boundary.
const MM_PER_M: FloatType = const_scalar_from_int!(FloatType, 1000);
const NM_PER_M: FloatType = const_scalar_from_int!(FloatType, 1_000_000_000);

/// A cavity counts as stable when `|m| <= 1`; this is the slack allowed on that comparison.
const STABILITY_MARGIN: FloatType = const_scalar_from_float!(FloatType, 1e-12);
/// How closely the round trip must reproduce the input `q` to call the mode self-consistent.
const EIGENMODE_TOLERANCE: FloatType = const_scalar_from_float!(FloatType, 1e-12);

/// The cavity and its laser line as a context: one `Data` contextoid per world fact. The cavity is
/// described by lengths along the beam, not by positions, so the spatial, temporal and spacetime
/// slots are empty.
type CavityContext = Context<Data<FloatType>, NoSpace<FloatType>, NoTime, NoSpaceTime<FloatType>>;

/// Node indices of the cavity's world facts; `linear_cavity` adds them in this order.
/// Drift from the first flat mirror to the thermal lens, in metres.
const DRIFT_L1: usize = 0;
/// Drift from the thermal lens to the far flat mirror, in metres.
const DRIFT_L2: usize = 1;
/// Radius of curvature of the biconvex thermal lens, in metres.
const LENS_RADIUS: usize = 2;
/// Refractive index of the lens material.
const LENS_INDEX: usize = 3;
/// Laser wavelength, in metres.
const WAVELENGTH: usize = 4;
/// Waist of the input beam, in metres.
const WAIST: usize = 5;
/// How many world facts the cavity holds.
const CAVITY_FACTS: usize = 6;

/// `f64` is the right precision here: the round trip is six 2x2 products and one Moebius map, so
/// the eigenmode residual sits at machine epsilon either way. `Float106` tightens the residual
/// and leaves every reported spot size unchanged.
pub type FloatType = f64;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_header();

    let cavity = linear_cavity()?;
    let wavelength = Wavelength::<FloatType>::new(read(&cavity, WAVELENGTH)?)?;
    let focal_length = thermal_lens_focal_length(&cavity)?;
    let waist = read(&cavity, WAIST)?;

    // At a waist the wavefront is flat, so q = i z_R with z_R = pi w0^2 / lambda.
    let rayleigh = FloatType::pi() * waist * waist / wavelength.value();
    let q_initial = ComplexBeamParameter::new(Complex::new(ZERO, rayleigh))?;
    print_input(&cavity, waist, rayleigh, focal_length)?;

    // One full round trip: out through the lens to the far mirror, and back again. Flat mirrors
    // contribute the identity, so they are the reflections between the two passes. The cavity
    // rides in the context channel, and each element builds its matrix from it.
    let process = ROUND_TRIP
        .iter()
        .fold(
            CausalFlow::value(q_initial).context(cavity.clone()),
            |flow, &element| flow.bind(move |value, _s, cavity| propagate(value, cavity, element)),
        )
        .into_process();

    let q_final = match process.value() {
        Some(q) => *q,
        None => {
            print_failure(process.error().map(|e| format!("{e:?}")));
            return Err("the resonator lost the beam".into());
        }
    };

    print_trace(&trace(q_initial, &cavity, wavelength)?);
    print_stability(&stability(&cavity, q_initial, q_final)?);

    Ok(())
}

/// Propagates `q` through one element, built from the cavity the context carries, or
/// short-circuits when the upstream carried no beam.
fn propagate(
    value: CausalEffect<ComplexBeamParameter<FloatType>>,
    cavity: Option<CavityContext>,
    element: Element,
) -> PropagatingProcess<ComplexBeamParameter<FloatType>, (), CavityContext> {
    let q = match value.into_value() {
        Some(q) => q,
        None => return fail("the flow carried no beam parameter"),
    };
    let cavity = match cavity {
        Some(c) => c,
        None => return fail("the flow carried no cavity"),
    };
    let matrix = match element.matrix(&cavity) {
        Ok(m) => m,
        Err(e) => return fail(format!("{e:?}")),
    };
    match gaussian_q_propagation(q, &matrix).value_cloned() {
        // A physical beam keeps Im(q) > 0. Once that fails the beam has diffracted away and the
        // flow enters the error channel rather than reporting a spot size for a lost beam.
        Some(next) if next.value().im > ZERO => {
            PropagatingProcess::with_state(PropagatingEffect::pure(next), (), Some(cavity))
        }
        Some(_) => fail("beam diverged: Im(q) <= 0"),
        None => fail("q propagation produced no value"),
    }
}

fn fail<T: Default + Clone + core::fmt::Debug, C: Clone + core::fmt::Debug>(
    reason: impl Into<String>,
) -> PropagatingProcess<T, (), C> {
    PropagatingProcess::from_error(deep_causality_core::CausalityError::new(
        deep_causality_core::CausalityErrorEnum::Custom(reason.into()),
    ))
}

/// Builds the cavity: each world fact as a `Data` contextoid at its node index.
fn linear_cavity() -> Result<CavityContext, ContextIndexError> {
    let mut facts = [ZERO; CAVITY_FACTS];
    facts[DRIFT_L1] = const_scalar_from_float!(FloatType, 0.5);
    facts[DRIFT_L2] = const_scalar_from_float!(FloatType, 0.5);
    facts[LENS_RADIUS] = const_scalar_from_float!(FloatType, 0.5);
    facts[LENS_INDEX] = const_scalar_from_float!(FloatType, 1.5);
    // Nd:YAG, 1064 nm.
    facts[WAVELENGTH] = const_scalar_from_float!(FloatType, 1064e-9);
    // 1 mm.
    facts[WAIST] = const_scalar_from_float!(FloatType, 1e-3);

    let mut cavity = Context::with_capacity(1, "linear cavity", CAVITY_FACTS);
    for (id, value) in (1..).zip(facts) {
        cavity.add_node(Contextoid::new(
            id,
            ContextoidType::Datoid(Data::new(id, value)),
        ))?;
    }
    Ok(cavity)
}

/// Reads one world fact out of the cavity, or the error naming the node that holds none.
fn read(cavity: &CavityContext, index: usize) -> Result<FloatType, PhysicsError> {
    cavity
        .get_node(index)
        .and_then(|node| node.vertex_type().dataoid())
        .map(|data| data.get_data())
        .ok_or_else(|| {
            PhysicsError::CalculationError(format!("the cavity holds no Datoid at node {index}"))
        })
}

/// One optical element of the cavity: its label and the optic a ray meets there.
#[derive(Clone, Copy)]
struct Element {
    label: &'static str,
    optic: Optic,
}

/// What an element does to the beam. A drift names the cavity node that holds its length.
#[derive(Clone, Copy)]
enum Optic {
    Drift(usize),
    ThermalLens,
}

impl Element {
    /// The element's ABCD matrix, built from the cavity the context holds.
    fn matrix(&self, cavity: &CavityContext) -> Result<AbcdMatrix<FloatType>, PhysicsError> {
        match self.optic {
            Optic::Drift(length) => drift(read(cavity, length)?),
            Optic::ThermalLens => thin_lens(thermal_lens_focal_length(cavity)?),
        }
    }
}

/// The six elements a ray meets on one round trip, in traversal order.
const ROUND_TRIP: [Element; 6] = [
    Element {
        label: "drift L1",
        optic: Optic::Drift(DRIFT_L1),
    },
    Element {
        label: "thermal lens",
        optic: Optic::ThermalLens,
    },
    Element {
        label: "drift L2",
        optic: Optic::Drift(DRIFT_L2),
    },
    // The far mirror is flat, so it reflects without focusing: the identity matrix.
    Element {
        label: "drift L2 (return)",
        optic: Optic::Drift(DRIFT_L2),
    },
    Element {
        label: "thermal lens (return)",
        optic: Optic::ThermalLens,
    },
    Element {
        label: "drift L1 (return)",
        optic: Optic::Drift(DRIFT_L1),
    },
];

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

/// Focal length of the biconvex thermal lens the cavity holds, from the lens-maker equation.
fn thermal_lens_focal_length(cavity: &CavityContext) -> Result<FloatType, PhysicsError> {
    let index = IndexOfRefraction::<FloatType>::new(read(cavity, LENS_INDEX)?)?;
    let radius = read(cavity, LENS_RADIUS)?;
    let power = lens_maker(index, radius, -radius)
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
    cavity: &CavityContext,
    wavelength: Wavelength<FloatType>,
) -> Result<Vec<Sample>, PhysicsError> {
    let mut q = q_initial;
    let mut samples = Vec::with_capacity(ROUND_TRIP.len() + 1);
    samples.push(sample("input", q, wavelength)?);
    for element in &ROUND_TRIP {
        q = gaussian_q_propagation(q, &element.matrix(cavity)?)
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
    cavity: &CavityContext,
    q_initial: ComplexBeamParameter<FloatType>,
    q_final: ComplexBeamParameter<FloatType>,
) -> Result<Stability, PhysicsError> {
    // ABCD matrices compose against the direction of travel, so the element met first sits
    // rightmost in the product.
    let mut round = identity()?;
    for element in &ROUND_TRIP {
        round = multiply(element.matrix(cavity)?.inner(), &round)?;
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
fn print_input(
    cavity: &CavityContext,
    waist: FloatType,
    rayleigh: FloatType,
    focal_length: FloatType,
) -> Result<(), PhysicsError> {
    println!(
        "Cavity:     flat mirror | {:.1} m | thermal lens | {:.1} m | flat mirror",
        lower(read(cavity, DRIFT_L1)?),
        lower(read(cavity, DRIFT_L2)?)
    );
    println!(
        "Wavelength: {:.1} nm",
        lower(read(cavity, WAVELENGTH)? * NM_PER_M)
    );
    println!(
        "Lens:       f = {:.3} m (biconvex, n = {:.1}, R = {:.1} m)",
        lower(focal_length),
        lower(read(cavity, LENS_INDEX)?),
        lower(read(cavity, LENS_RADIUS)?)
    );
    println!(
        "Input beam: w0 = {:.2} mm at a waist, z_R = {:.3} m\n",
        lower(waist * MM_PER_M),
        lower(rayleigh)
    );
    Ok(())
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

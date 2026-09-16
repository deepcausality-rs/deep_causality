/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Model layer for the aneurysm study: the vessel geometry, the blood and flow constants, the
//! haemodynamic closures, and the degeneration model.
//!
//! Every quantity is typed [`FloatType`], so switching the alias in `main` re-runs the geometry,
//! the stress profile and the accumulation at another precision.

use crate::FloatType;
use deep_causality_algebra::Real;
use deep_causality_linear::CsrMatrix;
use deep_causality_num::{const_scalar_from_float, const_scalar_from_int, lift_count};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{Manifold, Simplex, SimplicialComplex, SimplicialManifold, Skeleton};

/// The small whole numbers the model is written with.
pub const ZERO: FloatType = const_scalar_from_int!(FloatType, 0);
pub const ONE: FloatType = const_scalar_from_int!(FloatType, 1);
pub const TWO: FloatType = const_scalar_from_int!(FloatType, 2);
pub const FOUR: FloatType = const_scalar_from_int!(FloatType, 4);
pub const HALF: FloatType = const_scalar_from_float!(FloatType, 0.5);

// =============================================================================
// Vessel geometry
// =============================================================================

/// Segment length, in metres. Forty millimetres of a cerebral artery.
pub const SEGMENT_LENGTH_M: FloatType = const_scalar_from_float!(FloatType, 0.040);

/// The healthy lumen radius, in metres. Two millimetres.
pub const HEALTHY_RADIUS_M: FloatType = const_scalar_from_float!(FloatType, 0.002);

/// The aneurysm dome radius at its widest, in metres. Five millimetres.
pub const DOME_RADIUS_M: FloatType = const_scalar_from_float!(FloatType, 0.005);

/// Where the dome sits on the centreline, as node indices. The bulge spans `[NECK_IN, NECK_OUT]`
/// and peaks halfway between them.
pub const NECK_IN: usize = 14;
pub const NECK_OUT: usize = 26;

// =============================================================================
// Blood and flow
// =============================================================================

/// Dynamic viscosity of whole blood, in Pa·s.
pub const BLOOD_VISCOSITY: FloatType = const_scalar_from_float!(FloatType, 0.0035);

/// Volumetric flow rate through the segment, in m³/s. Three millilitres per second, in the range
/// a middle cerebral artery carries.
pub const FLOW_RATE_M3S: FloatType = const_scalar_from_float!(FloatType, 3.0e-6);

// =============================================================================
// Degeneration model
// =============================================================================

/// The wall shear stress below which the endothelium stops maintaining the wall, in Pa.
pub const LOW_SHEAR_THRESHOLD_PA: FloatType = const_scalar_from_float!(FloatType, 0.4);

/// Degeneration accrued per cardiac cycle at a total loss of shear, dimensionless. This constant
/// sets the pace of the run; a clinical model would calibrate it against longitudinal imaging.
pub const DEGENERATION_PER_CYCLE: FloatType = const_scalar_from_float!(FloatType, 1.0e-7);

/// Cardiac cycles per reported epoch. A million cycles is about nine days at 72 beats a minute.
pub const CYCLES_PER_EPOCH: u64 = 1_000_000;

/// How many epochs the run reports.
pub const EPOCHS: usize = 12;

/// The degeneration index at which the wall is flagged as rupture-prone, dimensionless.
pub const RUPTURE_THRESHOLD: FloatType = const_scalar_from_float!(FloatType, 0.75);

// =============================================================================
// Haemodynamics
// =============================================================================

/// The Poiseuille wall-shear closure for a tube: `τ = 4μQ / (πR³)`, in Pa.
///
/// Fully developed laminar flow through a circular lumen carries a parabolic velocity profile, and
/// the wall shear is the viscosity times that profile's slope at the wall. Widening the lumen
/// therefore drops the shear as the cube of the radius, which is why the dome starves.
pub fn wall_shear_stress(radius_m: FloatType) -> FloatType {
    FOUR * BLOOD_VISCOSITY * FLOW_RATE_M3S / (FloatType::pi() * radius_m * radius_m * radius_m)
}

/// The lumen radius at a centreline node, in metres. A raised cosine carries the wall smoothly
/// from the healthy calibre out to the dome and back, so the gradient stencil reads a continuous
/// slope across the neck.
pub fn radius_at(node: usize) -> FloatType {
    if node <= NECK_IN || node >= NECK_OUT {
        return HEALTHY_RADIUS_M;
    }
    let span = lift_count::<FloatType>((NECK_OUT - NECK_IN) as u64);
    let offset = lift_count::<FloatType>((node - NECK_IN) as u64);
    let bulge = DOME_RADIUS_M - HEALTHY_RADIUS_M;

    // A raised cosine over the neck-to-neck span: zero at both necks, one at the apex.
    HEALTHY_RADIUS_M + bulge * HALF * (ONE - Real::cos(TWO * FloatType::pi() * offset / span))
}

/// The degeneration index after each epoch.
///
/// Below the threshold the wall degrades at a rate set by how far the shear has fallen. At or
/// above it the endothelium holds its maintenance programme and the index stays put.
pub fn degeneration_history(dome_shear: FloatType) -> Vec<FloatType> {
    let deficit = if dome_shear < LOW_SHEAR_THRESHOLD_PA {
        (LOW_SHEAR_THRESHOLD_PA - dome_shear) / LOW_SHEAR_THRESHOLD_PA
    } else {
        ZERO
    };

    let per_epoch = DEGENERATION_PER_CYCLE * lift_count::<FloatType>(CYCLES_PER_EPOCH) * deficit;

    (1..=EPOCHS)
        .map(|epoch| per_epoch * lift_count::<FloatType>(epoch as u64))
        .collect()
}

/// The vessel centreline as a line manifold: `nodes` vertices joined by `nodes - 1` edges,
/// carrying the lumen radius on the vertices.
///
/// The edge slots pad the payload out to the manifold's cell count, and each holds the healthy
/// radius. That keeps every slot a valid geometry, so a pointwise `fmap` over the payload stays
/// total. The analysis reads the first `nodes` entries, which are the vertices.
pub fn build_vessel_manifold(
    nodes: usize,
) -> Result<SimplicialManifold<FloatType, FloatType>, Box<dyn std::error::Error>> {
    let vertices = (0..nodes).map(|i| Simplex::new(vec![i])).collect();
    let edges = (0..nodes - 1)
        .map(|i| Simplex::new(vec![i, i + 1]))
        .collect();

    // Boundary operator d1: each edge (i, i+1) leaves vertex i and enters vertex i+1.
    let mut triplets = Vec::with_capacity(2 * (nodes - 1));
    for e in 0..nodes - 1 {
        triplets.push((e, e, -1i8));
        triplets.push((e + 1, e, 1));
    }
    let d1 = CsrMatrix::from_triplets(nodes, nodes - 1, &triplets)?;

    let complex = SimplicialComplex::new(
        vec![Skeleton::new(0, vertices), Skeleton::new(1, edges)],
        vec![d1],
        vec![],
        vec![],
    );

    let mut data: Vec<FloatType> = (0..nodes).map(radius_at).collect();
    data.extend(std::iter::repeat_n(HEALTHY_RADIUS_M, nodes - 1));

    let payload = CausalTensor::new(data, vec![2 * nodes - 1])?;
    Ok(Manifold::new(complex, payload, 0)?)
}

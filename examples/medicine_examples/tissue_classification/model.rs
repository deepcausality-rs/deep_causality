/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Model layer for the tissue classifier: the sample geometries, the Vietoris-Rips complex, and
//! the two topological readings taken from it.
//!
//! A point cloud of voxel centres becomes a simplicial complex by joining every pair of points
//! closer than a fixed radius. The complex then answers two questions. Its Euler characteristic
//! says whether the sample encloses a void, and a local density map says where that void sits.

use crate::FloatType;
use deep_causality_algebra::Real;
use deep_causality_haft::{CoMonad, Foldable};
use deep_causality_num::{const_scalar_from_float, const_scalar_from_int, lift_count};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{BaseTopology, PointCloud, PointCloudWitness, TopologyError};

/// Voxel centres carry three coordinates each.
pub const DIMENSIONS: usize = 3;

/// The Vietoris-Rips radius, in the sample's own length units. Two voxels closer than this are
/// joined by an edge.
pub const RIPS_RADIUS: FloatType = const_scalar_from_float!(FloatType, 0.62);

/// The radius the local-density count uses. It matches the Rips radius, so a point's density is
/// the number of neighbours it is joined to in the complex.
pub const DENSITY_RADIUS: FloatType = RIPS_RADIUS;

/// The small numbers the geometry is written with.
const ZERO: FloatType = const_scalar_from_int!(FloatType, 0);
const ONE: FloatType = const_scalar_from_int!(FloatType, 1);
const TWO: FloatType = const_scalar_from_int!(FloatType, 2);
/// The two concentric rings of the solid sample: radius, then how many voxels sit on it.
const INNER_RING: (FloatType, usize) = (const_scalar_from_float!(FloatType, 0.45), 7);
const OUTER_RING: (FloatType, usize) = (const_scalar_from_float!(FloatType, 0.9), 16);

/// Voxels per sample.
pub const SAMPLE_POINTS: usize = 24;

/// A tissue sample: its voxel centres, ready to triangulate.
pub type Sample = PointCloud<FloatType, FloatType>;

/// Healthy tissue: a solid disc of voxels with a filled centre.
///
/// The points sit on concentric rings, which keeps the spacing even enough that the Rips complex
/// closes every triangle across the middle.
pub fn solid_tissue() -> Result<Sample, TopologyError> {
    let mut coords = Vec::with_capacity(SAMPLE_POINTS * DIMENSIONS);

    // A centre point, then two rings around it.
    coords.extend_from_slice(&[ZERO, ZERO, ZERO]);
    for (radius, count) in [INNER_RING, OUTER_RING] {
        for k in 0..count {
            let angle = turn_fraction(k, count);
            coords.push(radius * Real::cos(angle));
            coords.push(radius * Real::sin(angle));
            coords.push(ZERO);
        }
    }
    build_sample(coords)
}

/// Pathological tissue: a ring of voxels around an empty centre, which is the signature of a
/// necrotic core. The cells at the rim are alive and the middle has died out.
pub fn necrotic_tissue() -> Result<Sample, TopologyError> {
    let mut coords = Vec::with_capacity(SAMPLE_POINTS * DIMENSIONS);

    for k in 0..SAMPLE_POINTS {
        let angle = turn_fraction(k, SAMPLE_POINTS);
        coords.push(ONE * Real::cos(angle));
        coords.push(ONE * Real::sin(angle));
        coords.push(ZERO);
    }
    build_sample(coords)
}

/// What one sample's Vietoris-Rips complex reports: its cell counts by grade, and the Euler
/// characteristic those counts give.
#[derive(Debug, Clone, Copy)]
pub struct TopologyReading {
    pub vertices: usize,
    pub edges: usize,
    pub triangles: usize,
    pub euler_characteristic: isize,
}

/// Triangulates the sample and reads its topology.
///
/// `χ = V − E + F − …` is a topological invariant: it counts a shape's connected pieces against
/// the holes in them, and it holds under any deformation that leaves the connectivity alone. A
/// filled, connected sample reads 1. Enclosing a void drops it to 0 or below, which is the reading
/// a necrotic core produces.
///
/// `BaseTopology::euler_characteristic` supplies it. That trait sits under every topological
/// structure in the workspace, so a point cloud, a graph and a complex all answer the same
/// question the same way. It also answers for a complex whose vertex links fail the manifold
/// conditions, and the Vietoris-Rips complex of a sampled surface is exactly such a complex.
pub fn read_topology(sample: &Sample) -> Result<TopologyReading, TopologyError> {
    let complex = sample.triangulate(RIPS_RADIUS)?;
    let at = |grade: usize| complex.num_elements_at_grade(grade).unwrap_or(0);

    Ok(TopologyReading {
        vertices: at(0),
        edges: at(1),
        triangles: at(2),
        euler_characteristic: complex.euler_characteristic(),
    })
}

/// The local density at every voxel: how many other voxels lie within [`DENSITY_RADIUS`].
///
/// `extend` focuses the cloud on one voxel at a time and hands the closure that focused view, so
/// the closure reads its own coordinates and the whole cloud together. The Euler characteristic
/// says a void exists; this map says which voxels sit next to it.
pub fn local_density(sample: &Sample) -> Sample {
    PointCloudWitness::extend(sample, |view| {
        let here = view.cursor();
        let points = view.points().as_slice();
        let count = (0..view.len())
            .filter(|&other| other != here && distance(points, here, other) <= DENSITY_RADIUS)
            .count();
        lift_count::<FloatType>(count as u64)
    })
}

/// The lowest and highest local density in the sample, from one `fold` over the density map.
///
/// The spread is what separates the two geometries. A solid mass has an interior and a rim, so its
/// voxels range from densely surrounded to sparsely surrounded. A ring is rim everywhere, so every
/// voxel sees the same number of neighbours and the range collapses to a point.
pub fn density_range(density: Sample) -> (FloatType, FloatType) {
    let start = density.metadata().as_slice()[0];
    PointCloudWitness::fold(density, (start, start), |(low, high), count| {
        (
            if count < low { count } else { low },
            if count > high { count } else { high },
        )
    })
}

/// The Euclidean distance between two voxels of a flattened `[n, 3]` coordinate table.
fn distance(points: &[FloatType], a: usize, b: usize) -> FloatType {
    let (base_a, base_b) = (a * DIMENSIONS, b * DIMENSIONS);
    let squared = (0..DIMENSIONS).fold(ZERO, |sum, axis| {
        let gap = points[base_a + axis] - points[base_b + axis];
        sum + gap * gap
    });
    Real::sqrt(squared)
}

/// The fraction of a full turn that point `k` of `count` sits at, in radians.
fn turn_fraction(k: usize, count: usize) -> FloatType {
    let two_pi = TWO * FloatType::pi();
    two_pi * lift_count::<FloatType>(k as u64) / lift_count::<FloatType>(count as u64)
}

/// A point cloud from a flattened coordinate table, with unit metadata on every voxel.
fn build_sample(coords: Vec<FloatType>) -> Result<Sample, TopologyError> {
    let points = coords.len() / DIMENSIONS;
    let positions = CausalTensor::new(coords, vec![points, DIMENSIONS])?;
    let metadata = CausalTensor::new(vec![ONE; points], vec![points])?;
    PointCloud::new(positions, metadata, 0)
}

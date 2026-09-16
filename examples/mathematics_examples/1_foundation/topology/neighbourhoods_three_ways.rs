/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # One smoothing law, three notions of neighbour
//!
//! `CoMonad::extend` hands a closure a view of the whole structure focused on one element. What
//! the closure can reach from that focus is the structure's own idea of a neighbourhood, and
//! three topology carriers answer that differently:
//!
//! ```text
//! HypergraphWitness   a hyperedge holds any number of nodes at once, so neighbours are
//!                     everyone sharing a group
//! MixedGraphWitness   directed and undirected edges coexist, so neighbours come from
//!                     parents, children and undirected links
//! PointCloudWitness   points carry coordinates, so neighbours are whatever falls inside
//!                     a radius
//! ```
//!
//! Six air-quality sensors report a reading. The same smoothing law blends each reading with the
//! mean of its neighbours, and only the neighbourhood changes between the three runs: sensors
//! grouped by the building they sit on, sensors linked by the prevailing wind, and sensors within
//! 40 metres of each other.
//!
//! Each witness also carries `Functor` and `Foldable`, so the readings map to an index and reduce
//! to a network mean through the same two calls on all three carriers.

use deep_causality_algebra::Real;
use deep_causality_haft::{CoMonad, Foldable, Functor};
use deep_causality_linear::CsrMatrix;
use deep_causality_num::{const_scalar_from_float, const_scalar_from_int, lift, lift_count, lower};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    Hypergraph, HypergraphWitness, MixedGraph, MixedGraphWitness, PointCloud, PointCloudWitness,
};

/// Six sensors and their raw readings, in µg/m³ of particulate matter.
const READINGS: [f64; 6] = [12.0, 48.0, 31.0, 9.0, 55.0, 22.0];
const N_SENSORS: usize = READINGS.len();

/// Sensor positions in metres, as `[x, y]` pairs.
const POSITIONS: [[f64; 2]; N_SENSORS] = [
    [0.0, 0.0],
    [20.0, 10.0],
    [35.0, 5.0],
    [80.0, 0.0],
    [95.0, 15.0],
    [120.0, 5.0],
];
const SPATIAL_DIM: usize = 2;

/// Buildings, as the hyperedges of the hypergraph. Sensor 2 sits on two of them.
const BUILDINGS: [&[usize]; 3] = [&[0, 1, 2], &[2, 3], &[3, 4, 5]];

/// The prevailing wind, as directed edges, plus one undirected link across a shared courtyard.
const WIND_ARCS: [(usize, usize); 4] = [(0, 1), (1, 2), (3, 4), (4, 5)];
const COURTYARD_LINKS: [(usize, usize); 1] = [(2, 3)];

/// Two sensors within this distance, in metres, are spatial neighbours.
const NEIGHBOUR_RADIUS: FloatType = const_scalar_from_int!(FloatType, 40);

/// How much weight the neighbourhood mean carries against a sensor's own reading.
const SMOOTHING_ALPHA: FloatType = const_scalar_from_float!(FloatType, 0.5);

/// The reading that maps to an index of 100, in µg/m³.
const INDEX_REFERENCE: FloatType = const_scalar_from_int!(FloatType, 50);

/// The working scalar. Readings, positions and every smoothed value carry it.
pub type FloatType = f64;

/// Small numbers, declared once at the working type rather than lifted at each use.
const ZERO: FloatType = const_scalar_from_int!(FloatType, 0);
const ONE: FloatType = const_scalar_from_int!(FloatType, 1);
const HUNDRED: FloatType = const_scalar_from_int!(FloatType, 100);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let readings: Vec<FloatType> = READINGS.iter().map(|&r| lift::<FloatType>(r)).collect();
    print_header(&readings);

    // ---------------------------------------------------------------------
    // 1. Hypergraph: a hyperedge is a building, and it holds every sensor on it at once.
    // ---------------------------------------------------------------------
    let hypergraph = build_hypergraph(&readings)?;
    let by_building = HypergraphWitness::extend(&hypergraph, |w| {
        let neighbours = readings_at(w.data(), &hypergraph_neighbours(w, w.cursor()));
        smoothed(HypergraphWitness::extract(w), &neighbours)
    });

    // ---------------------------------------------------------------------
    // 2. Mixed graph: the wind runs one way, the courtyard link runs both.
    // ---------------------------------------------------------------------
    let mixed = build_mixed_graph(&readings)?;
    let by_wind = MixedGraphWitness::extend(&mixed, |w| {
        let neighbours = readings_at(w.data(), &mixed_graph_neighbours(w, w.cursor()));
        smoothed(MixedGraphWitness::extract(w), &neighbours)
    });

    // ---------------------------------------------------------------------
    // 3. Point cloud: a neighbourhood is a ball of the given radius.
    // ---------------------------------------------------------------------
    let cloud = build_point_cloud(&readings)?;
    let by_distance = PointCloudWitness::<FloatType>::extend(&cloud, |w| {
        let neighbours = readings_at(w.metadata(), &point_cloud_neighbours(w, w.cursor()));
        smoothed(PointCloudWitness::<FloatType>::extract(w), &neighbours)
    });

    print_smoothing(
        &readings,
        by_building.data().as_slice(),
        by_wind.data().as_slice(),
        by_distance.metadata().as_slice(),
    );

    // ---------------------------------------------------------------------
    // 4. Functor and Foldable read the same on every carrier.
    // ---------------------------------------------------------------------
    // `fmap` carries each reading to an index against the reference concentration, and `fold`
    // reduces the carrier to the network mean. Neither call names a neighbourhood.
    let reference = INDEX_REFERENCE;
    let to_index = move |v: FloatType| v * HUNDRED / reference;

    let indexed = HypergraphWitness::fmap(by_building.clone(), to_index);
    print_index(indexed.data().as_slice());

    print_means(
        network_mean(HypergraphWitness::fold(by_building, ZERO, |acc, v| acc + v)),
        network_mean(MixedGraphWitness::fold(by_wind, ZERO, |acc, v| acc + v)),
        network_mean(PointCloudWitness::<FloatType>::fold(
            by_distance,
            ZERO,
            |acc, v| acc + v,
        )),
    );

    print_footer();
    Ok(())
}

/// The smoothing law, written once. A reading is blended with the mean of whatever its carrier
/// calls its neighbours, and a sensor standing alone keeps the value it reported.
fn smoothed(reading: FloatType, neighbours: &[FloatType]) -> FloatType {
    match neighbours.len() {
        0 => reading,
        n => {
            let sum = neighbours.iter().fold(ZERO, |acc, &v| acc + v);
            let mean = sum / lift_count::<FloatType>(n as u64);
            let alpha = SMOOTHING_ALPHA;

            (ONE - alpha) * reading + alpha * mean
        }
    }
}

/// The readings held at the given indices.
fn readings_at(data: &CausalTensor<FloatType>, indices: &[usize]) -> Vec<FloatType> {
    let values = data.as_slice();
    indices.iter().map(|&i| values[i]).collect()
}

/// The mean of the whole network, from the sum a `fold` produced.
fn network_mean(sum: FloatType) -> FloatType {
    sum / lift_count::<FloatType>(N_SENSORS as u64)
}

// -----------------------------------------------------------------------------------------
// The three carriers, and what each calls a neighbour
// -----------------------------------------------------------------------------------------

/// Sensors grouped by building. The incidence matrix has one row per sensor and one column per
/// building, and it holds `i8` because membership is combinatorial.
fn build_hypergraph(
    readings: &[FloatType],
) -> Result<Hypergraph<FloatType>, Box<dyn std::error::Error>> {
    let mut triplets = Vec::new();
    for (building, members) in BUILDINGS.iter().enumerate() {
        for &sensor in members.iter() {
            triplets.push((sensor, building, 1i8));
        }
    }
    let incidence = CsrMatrix::from_triplets(N_SENSORS, BUILDINGS.len(), &triplets)?;
    let data = CausalTensor::new(readings.to_vec(), vec![N_SENSORS])?;

    Ok(Hypergraph::new(incidence, data, 0)?)
}

/// Every other sensor sharing a building with this one.
fn hypergraph_neighbours(h: &Hypergraph<FloatType>, node: usize) -> Vec<usize> {
    let incidence = h.incidence();
    let mut found = Vec::new();

    for building in 0..h.num_hyperedges() {
        if incidence.get_value_at(node, building) == 0 {
            continue;
        }
        for other in 0..h.num_nodes() {
            if other != node && incidence.get_value_at(other, building) != 0 {
                found.push(other);
            }
        }
    }
    found.sort_unstable();
    found.dedup();

    found
}

/// Sensors linked by the prevailing wind, with one undirected courtyard link.
fn build_mixed_graph(
    readings: &[FloatType],
) -> Result<MixedGraph<FloatType>, Box<dyn std::error::Error>> {
    let data = CausalTensor::new(readings.to_vec(), vec![N_SENSORS])?;
    let mut graph = MixedGraph::new(N_SENSORS, data, 0)?;

    for &(from, to) in WIND_ARCS.iter() {
        graph.add_arc(from, to)?;
    }
    for &(u, v) in COURTYARD_LINKS.iter() {
        graph.add_undirected(u, v)?;
    }

    Ok(graph)
}

/// Everything one edge away, whichever way the edge points.
fn mixed_graph_neighbours(g: &MixedGraph<FloatType>, vertex: usize) -> Vec<usize> {
    let mut found = g.parents(vertex);
    found.extend(g.children(vertex));
    found.extend(g.undirected_neighbors(vertex));
    found.sort_unstable();
    found.dedup();

    found
}

/// Sensors at their measured positions, carrying the readings as metadata.
fn build_point_cloud(
    readings: &[FloatType],
) -> Result<PointCloud<FloatType, FloatType>, Box<dyn std::error::Error>> {
    let coords: Vec<FloatType> = POSITIONS
        .iter()
        .flat_map(|p| p.iter().map(|&c| lift::<FloatType>(c)))
        .collect();
    let points = CausalTensor::new(coords, vec![N_SENSORS, SPATIAL_DIM])?;
    let metadata = CausalTensor::new(readings.to_vec(), vec![N_SENSORS])?;

    Ok(PointCloud::new(points, metadata, 0)?)
}

/// Every sensor inside the radius.
fn point_cloud_neighbours(cloud: &PointCloud<FloatType, FloatType>, point: usize) -> Vec<usize> {
    let coords = cloud.points().as_slice();
    let radius = NEIGHBOUR_RADIUS;
    let at = |i: usize, d: usize| coords[i * SPATIAL_DIM + d];

    (0..cloud.len())
        .filter(|&other| {
            if other == point {
                return false;
            }
            let squared = (0..SPATIAL_DIM).fold(ZERO, |acc, d| {
                let delta = at(point, d) - at(other, d);
                acc + delta * delta
            });

            Real::sqrt(squared) <= radius
        })
        .collect()
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

/// The display boundary: `f64` appears here and nowhere else.
fn print_header(readings: &[FloatType]) {
    println!("=== One smoothing law, three notions of neighbour ===\n");
    println!(
        "  {N_SENSORS} air-quality sensors, blended at alpha = {SMOOTHING_ALPHA} with the mean"
    );
    println!("  of whatever each carrier calls a neighbour.\n");
    println!("  raw readings (ug/m3): {:?}\n", shown(readings));
}

fn print_smoothing(
    raw: &[FloatType],
    building: &[FloatType],
    wind: &[FloatType],
    distance: &[FloatType],
) {
    println!("--- CoMonad::extend over three carriers ---");
    println!("  sensor      raw   by building     by wind   by distance");
    for i in 0..N_SENSORS {
        println!(
            "  s{i}       {:6.2}      {:8.2}    {:8.2}      {:8.2}",
            lower(raw[i]),
            lower(building[i]),
            lower(wind[i]),
            lower(distance[i])
        );
    }
    println!();
    println!("  Hypergraph   a hyperedge holds a whole building, so s2 reaches s0, s1 and s3");
    println!("  MixedGraph   the wind points one way and the courtyard link points both");
    println!("  PointCloud   a ball of {NEIGHBOUR_RADIUS} m, so the two clusters stay apart");
}

fn print_index(indexed: &[FloatType]) {
    println!("\n--- Functor::fmap, the same call on every carrier ---");
    println!(
        "  index against {INDEX_REFERENCE} ug/m3: {:?}",
        shown_rounded(indexed)
    );
}

fn print_means(building: FloatType, wind: FloatType, distance: FloatType) {
    println!("\n--- Foldable::fold, the same call on every carrier ---");
    println!("  network mean by building:  {:6.2} ug/m3", lower(building));
    println!("  network mean by wind:      {:6.2} ug/m3", lower(wind));
    println!("  network mean by distance:  {:6.2} ug/m3", lower(distance));
}

fn print_footer() {
    println!("\n--- The point ---");
    println!("  `extend` supplies the focus and the whole structure around it. Each carrier");
    println!("  answers `what can I reach from here` in its own terms, and the smoothing law");
    println!("  above is written once and reads the answer whichever carrier asked it.");
}

fn shown(values: &[FloatType]) -> Vec<f64> {
    values.iter().map(|&v| lower(v)).collect()
}

fn shown_rounded(values: &[FloatType]) -> Vec<f64> {
    values.iter().map(|&v| lower(v).round()).collect()
}

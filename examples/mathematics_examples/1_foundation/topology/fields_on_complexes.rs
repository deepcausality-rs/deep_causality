/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # One field, three complexes
//!
//! A field is a value attached to every cell of a discrete structure. Three witnesses carry that
//! idea over three different structures:
//!
//! ```text
//! CellComplexWitness<C>        CellField<C, T>          Functor, Foldable
//! LatticeComplexWitness<D, R>  LatticeField<D, R, T>    Functor, Foldable
//! TopologyWitness<R>           Topology<R, G>           Functor, Foldable, CoMonad
//! ```
//!
//! `Functor` and `Foldable` are the floor all three stand on, so a calibration law and an
//! aggregation are each written once and run on every carrier. `TopologyWitness` adds `CoMonad`,
//! which is what a neighbourhood operation needs, and section 4 uses it.
//!
//! The setting is a rooftop solar survey. The same irradiance ramp is sampled on three layouts —
//! hex-packed modules on a honeycomb, a square grid, and an unstructured triangulated plot — and
//! each carrier holds one reading per site.
//!
//! Every value in each carrier is independent of its complex: `fmap` reaches the readings and
//! carries the structure across by sharing the `Arc`, so the honeycomb's incidence, the lattice's
//! shape and the complex's boundary operators all survive a change of units.

use deep_causality_algebra::Real;
use deep_causality_haft::{CoMonad, Foldable, Functor};
use deep_causality_linear::CsrMatrix;
use deep_causality_num::{lift, lift_count, lower};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    CellComplexWitness, CellField, HoneycombLattice, LatticeComplex, LatticeComplexWitness,
    LatticeField, Simplex, SimplicialComplex, Skeleton, Topology, TopologyWitness,
};
use std::sync::Arc;

/// The honeycomb layout, in rows and columns of hexagons.
const HONEYCOMB_SIZE: [usize; 2] = [2, 2];
/// The square grid layout, in sites per side.
const GRID_SHAPE: [usize; 2] = [3, 3];
const GRID_DIM: usize = 2;

/// The triangulated plot: five sites and the six edges joining them.
const PLOT_SITES: usize = 5;
const PLOT_EDGES: [[usize; 2]; 6] = [[0, 1], [0, 2], [1, 2], [1, 3], [2, 3], [3, 4]];
/// Readings live on the sites, which are the grade-0 cells.
const SITE_GRADE: usize = 0;

/// The irradiance ramp across the roof, in W/m²: the shaded edge, and the span to the sunlit one.
const IRRADIANCE_BASE: f64 = 620.0;
const IRRADIANCE_SPAN: f64 = 310.0;

/// One site on the triangulated plot sits under a flue, which cuts its reading to this fraction.
const SHADED_SITE: usize = 2;
const SHADE_FACTOR: f64 = 0.45;

/// Calibration: module area in m², peak-equivalent hours in a day, and watts per kilowatt.
const MODULE_AREA: f64 = 1.7;
const PEAK_HOURS: f64 = 4.6;
const WATTS_PER_KW: f64 = 1000.0;

/// A reading this far below its neighbourhood mean is reported as shaded.
const SHADE_THRESHOLD: f64 = 0.8;

/// The working scalar. Readings, calibrated yields and every total carry it.
pub type FloatType = f64;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_header();

    // ---------------------------------------------------------------------
    // 1. The same ramp, sampled on three layouts.
    // ---------------------------------------------------------------------
    let honeycomb =
        Arc::new(HoneycombLattice::new(HONEYCOMB_SIZE, [false, false]).as_cell_complex());
    let hex_sites = honeycomb.cells_vec(SITE_GRADE).len();
    let hex_field = CellField::new(honeycomb, ramp(hex_sites));

    let lattice = Arc::new(LatticeComplex::<GRID_DIM, FloatType>::new(
        GRID_SHAPE,
        [false, false],
    ));
    let grid_sites = GRID_SHAPE.iter().product();
    let grid_field = LatticeField::new(lattice, ramp(grid_sites));

    let complex = Arc::new(build_plot()?);
    let plot_field = Topology::new(
        complex.clone(),
        SITE_GRADE,
        CausalTensor::new(shaded_ramp(PLOT_SITES), vec![PLOT_SITES])?,
        0,
    )?;

    print_layouts(
        hex_sites,
        hex_field.values(),
        grid_sites,
        grid_field.values(),
        plot_field.data().as_slice(),
    );

    // ---------------------------------------------------------------------
    // 2. Functor: one calibration law on three carriers.
    // ---------------------------------------------------------------------
    // W/m² to kWh per module per day. The law is a closure; the three calls differ only in the
    // witness in front of them, and each carries its own structure across untouched.
    let calibrate = calibration();
    let hex_yield = CellComplexWitness::fmap(hex_field, calibrate);
    let grid_yield = LatticeComplexWitness::<GRID_DIM, FloatType>::fmap(grid_field, calibrate);
    let plot_yield = TopologyWitness::<FloatType>::fmap(plot_field.clone(), calibrate);

    print_calibration(
        hex_yield.values(),
        grid_yield.values(),
        plot_yield.data().as_slice(),
    );

    // The structure survived the change of units.
    assert_eq!(plot_yield.grade(), SITE_GRADE);
    assert_eq!(grid_yield.len(), grid_sites);

    // ---------------------------------------------------------------------
    // 3. Foldable: one aggregation on three carriers.
    // ---------------------------------------------------------------------
    let zero = lift::<FloatType>(0.0);
    let add = |acc: FloatType, v: FloatType| acc + v;

    let hex_total = CellComplexWitness::fold(hex_yield, zero, add);
    let grid_total = LatticeComplexWitness::<GRID_DIM, FloatType>::fold(grid_yield, zero, add);
    let plot_total = TopologyWitness::<FloatType>::fold(plot_yield, zero, add);

    print_totals(
        (hex_sites, hex_total),
        (grid_sites, grid_total),
        (PLOT_SITES, plot_total),
    );

    // ---------------------------------------------------------------------
    // 4. CoMonad: the one operation that needs a neighbourhood.
    // ---------------------------------------------------------------------
    // `TopologyWitness` carries `extend`, so a closure gets the whole field focused on one site
    // and can reach the sites joined to it through the complex's own boundary operator. A reading
    // far under its neighbourhood mean is a shaded module.
    let flagged = TopologyWitness::<FloatType>::extend(&plot_field, |w| {
        let readings = w.data().as_slice();
        let neighbours = vertex_neighbours(w.complex(), w.cursor());
        let mine = TopologyWitness::<FloatType>::extract(w);

        match neighbours.len() {
            0 => lift::<FloatType>(0.0),
            n => {
                let sum = neighbours.iter().fold(zero, |acc, &i| acc + readings[i]);
                let mean = sum / lift_count::<FloatType>(n as u64);
                mine / mean
            }
        }
    });

    let ratios = flagged.data().as_slice();
    print_shading(plot_field.data().as_slice(), ratios);

    // The site under the flue is the one the neighbourhood test picks out.
    let threshold = lift::<FloatType>(SHADE_THRESHOLD);
    let shaded: Vec<usize> = (0..PLOT_SITES).filter(|&i| ratios[i] < threshold).collect();
    assert_eq!(shaded, vec![SHADED_SITE]);

    print_footer();
    Ok(())
}

/// The irradiance ramp across the roof, sampled at `count` evenly spaced sites.
fn ramp(count: usize) -> Vec<FloatType> {
    let base = lift::<FloatType>(IRRADIANCE_BASE);
    let span = lift::<FloatType>(IRRADIANCE_SPAN);
    let last = lift_count::<FloatType>((count.max(2) - 1) as u64);

    (0..count)
        .map(|i| base + span * lift_count::<FloatType>(i as u64) / last)
        .collect()
}

/// The same ramp with one site under a flue.
fn shaded_ramp(count: usize) -> Vec<FloatType> {
    let mut values = ramp(count);
    values[SHADED_SITE] *= lift::<FloatType>(SHADE_FACTOR);

    values
}

/// W/m² to kWh per module per day, the law all three carriers are mapped with.
fn calibration() -> impl Fn(FloatType) -> FloatType + Copy {
    let area = lift::<FloatType>(MODULE_AREA);
    let hours = lift::<FloatType>(PEAK_HOURS);
    let per_kw = lift::<FloatType>(WATTS_PER_KW);

    move |irradiance| irradiance * area * hours / per_kw
}

/// The triangulated plot: five sites joined by six edges.
fn build_plot() -> Result<SimplicialComplex<FloatType>, Box<dyn std::error::Error>> {
    let sites: Vec<Simplex> = (0..PLOT_SITES).map(|i| Simplex::new(vec![i])).collect();
    let edges: Vec<Simplex> = PLOT_EDGES
        .iter()
        .map(|&[a, b]| Simplex::new(vec![a, b]))
        .collect();

    let mut triplets: Vec<(usize, usize, i8)> = Vec::with_capacity(2 * PLOT_EDGES.len());
    for (edge, &[from, to]) in PLOT_EDGES.iter().enumerate() {
        triplets.push((from, edge, -1));
        triplets.push((to, edge, 1));
    }
    let d1 = CsrMatrix::from_triplets(PLOT_SITES, PLOT_EDGES.len(), &triplets)?;

    Ok(SimplicialComplex::new(
        vec![Skeleton::new(0, sites), Skeleton::new(1, edges)],
        vec![d1],
        vec![],
        vec![],
    ))
}

/// The sites joined to this one, read out of the complex's own boundary operator `∂₁`.
fn vertex_neighbours(complex: &SimplicialComplex<FloatType>, site: usize) -> Vec<usize> {
    let Ok(d1) = complex.boundary_operator(1) else {
        return Vec::new();
    };
    let (sites, edges) = d1.shape();
    let mut found = Vec::new();

    for edge in 0..edges {
        if d1.get_value_at(site, edge) == 0 {
            continue;
        }
        for other in 0..sites {
            if other != site && d1.get_value_at(other, edge) != 0 {
                found.push(other);
            }
        }
    }
    found.sort_unstable();
    found.dedup();

    found
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_header() {
    println!("=== One field, three complexes ===\n");
    println!("  A rooftop irradiance ramp, sampled on a honeycomb, a square grid and an");
    println!("  unstructured plot. Functor and Foldable serve all three; CoMonad serves one.\n");
}

/// The display boundary: `f64` appears here and nowhere else.
fn print_layouts(
    hex_sites: usize,
    hex: &[FloatType],
    grid_sites: usize,
    grid: &[FloatType],
    plot: &[FloatType],
) {
    println!("--- 1. The three carriers, in W/m2 ---");
    println!(
        "  CellField<HoneycombCell>   {hex_sites:2} sites   {:?}",
        rounded(hex)
    );
    println!(
        "  LatticeField<2, f64>       {grid_sites:2} sites   {:?}",
        rounded(grid)
    );
    println!(
        "  Topology<f64, f64>         {PLOT_SITES:2} sites   {:?}   <- site {SHADED_SITE} sits under a flue",
        rounded(plot)
    );
}

fn print_calibration(hex: &[FloatType], grid: &[FloatType], plot: &[FloatType]) {
    println!("\n--- 2. Functor::fmap, one calibration law, in kWh per module per day ---");
    println!("  CellComplexWitness      {:?}", to_two_places(hex));
    println!("  LatticeComplexWitness   {:?}", to_two_places(grid));
    println!("  TopologyWitness         {:?}", to_two_places(plot));
}

fn print_totals(hex: (usize, FloatType), grid: (usize, FloatType), plot: (usize, FloatType)) {
    println!("\n--- 3. Foldable::fold, one aggregation ---");
    println!("  carrier                 sites    total kWh/day    per site");
    for (name, (sites, total)) in [
        ("CellComplexWitness   ", hex),
        ("LatticeComplexWitness", grid),
        ("TopologyWitness      ", plot),
    ] {
        println!(
            "  {name}    {sites:3}        {:9.2}      {:6.2}",
            lower(total),
            lower(total / lift_count::<FloatType>(sites as u64))
        );
    }
    println!("\n  The totals differ because the layouts hold different numbers of sites.");
    println!("  Per site is the comparable figure, and the shaded plot sits below the rest.");
}

fn print_shading(readings: &[FloatType], ratios: &[FloatType]) {
    println!("\n--- 4. CoMonad::extend, the operation that needs a neighbourhood ---");
    println!("  site   reading   neighbours   ratio to their mean");
    for site in 0..PLOT_SITES {
        let neighbours = PLOT_EDGES.iter().filter(|e| e.contains(&site)).count();
        let mark = if lower(ratios[site]) < SHADE_THRESHOLD {
            "  <- shaded"
        } else {
            ""
        };
        println!(
            "  s{site}      {:6.1}          {neighbours}                 {:5.3}{mark}",
            lower(readings[site]),
            lower(ratios[site])
        );
    }
}

fn print_footer() {
    println!("\n--- The point ---");
    println!("  The calibration and the aggregation are each written once and run on all");
    println!("  three carriers, because Functor and Foldable is what all three implement.");
    println!("  The neighbourhood test runs on the one carrier that implements CoMonad, and");
    println!("  it reaches the neighbours through the complex's own boundary operator.");
}

fn rounded(values: &[FloatType]) -> Vec<f64> {
    values.iter().map(|&v| Real::round(lower(v))).collect()
}

fn to_two_places(values: &[FloatType]) -> Vec<f64> {
    values
        .iter()
        .map(|&v| (lower(v) * 100.0).round() / 100.0)
        .collect()
}

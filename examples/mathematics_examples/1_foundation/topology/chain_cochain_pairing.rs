/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Chains, cochains, and the pairing between them
//!
//! A chain is *where* you go: a weighted sum of cells, here the road segments a delivery van
//! drives, signed by the direction it drives them. A cochain is *what you measure* along the way:
//! one number per cell, here the height a segment climbs.
//!
//! Evaluating one against the other is the pairing `⟨ω, c⟩ = Σ ω(cell)·c(cell)`, the discrete line
//! integral. It is the operation the whole discrete exterior calculus is built around.
//!
//! ```text
//! ChainWitness<R>    Type<G> = Chain<R, G>    Functor, Foldable   over the weights
//! CochainWitness     Type<R> = Cochain<R>     Functor, Foldable   over the values
//! ```
//!
//! Both witnesses map the payload and carry the structure across: `fmap` on a chain leaves the
//! complex and the grade alone, and `fmap` on a cochain leaves the degree alone. The pairing is
//! linear in each argument, so a `fmap` that scales either side scales the result, which is what
//! sections 3 and 4 measure.
//!
//! The route is a closed loop. A cochain that is the coboundary of a height function pairs to
//! exactly zero on it, because climbing back to where you started nets no height. A cochain that
//! comes from nowhere in particular pairs to its circulation instead, and section 2 shows both.

use deep_causality_haft::{Foldable, Functor};
use deep_causality_linear::CsrMatrix;
use deep_causality_num::{const_scalar_from_float, const_scalar_from_int, lift, lower};
use deep_causality_topology::{
    Chain, ChainWitness, Cochain, CochainWitness, Simplex, SimplicialComplex, Skeleton,
};
use std::sync::Arc;

/// Four depots, at these heights in metres.
const DEPOT_HEIGHTS: [f64; 4] = [100.0, 140.0, 115.0, 175.0];
const N_DEPOTS: usize = DEPOT_HEIGHTS.len();

/// Five road segments, each running from the lower-numbered depot to the higher.
const SEGMENTS: [[usize; 2]; 5] = [[0, 1], [0, 2], [1, 2], [1, 3], [2, 3]];
const N_SEGMENTS: usize = SEGMENTS.len();

/// The route `0 → 1 → 3 → 2 → 0`, as a signed traversal count per segment. A negative weight is a
/// segment driven against the direction it is stored in.
const ROUTE: [f64; N_SEGMENTS] = [1.0, -1.0, 0.0, 1.0, -1.0];

/// A wind-assist form, one value per segment. Nothing generates it, so it carries circulation.
const WIND_ASSIST: [f64; N_SEGMENTS] = [5.0, 2.0, 3.0, 1.0, 6.0];

/// Chains and cochains live at grade and degree 1: they are indexed by segments.
const GRADE: usize = 1;

/// How many times over the route is repeated in section 3.
const REPEATS: FloatType = const_scalar_from_int!(FloatType, 2);
/// The factor the wind-assist form is scaled by in section 4.
const FORM_SCALE: FloatType = const_scalar_from_float!(FloatType, 1.5);

/// The working scalar. Heights, weights and every pairing carry it.
pub type FloatType = f64;

/// Small numbers, declared once at the working type rather than lifted at each use.
const ZERO: FloatType = const_scalar_from_int!(FloatType, 0);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_header();

    let complex = Arc::new(build_road_network()?);

    // ---------------------------------------------------------------------
    // 1. The two objects.
    // ---------------------------------------------------------------------
    let route = build_chain(complex.clone(), &ROUTE)?;
    let climb = coboundary_of_heights();
    let wind = Cochain::new(
        WIND_ASSIST.iter().map(|&v| lift::<FloatType>(v)).collect(),
        GRADE,
    );
    print_objects(&route, &climb, &wind);

    // ---------------------------------------------------------------------
    // 2. The pairing, on an exact cochain and on a general one.
    // ---------------------------------------------------------------------
    // `climb` is the coboundary of the depot heights: its value on a segment is the height
    // difference across it. Pairing that against a closed loop returns zero, because the route
    // ends where it started and the heights cancel term by term.
    let net_climb = pair(&climb, &route);
    let circulation = pair(&wind, &route);
    print_pairing(net_climb, circulation);

    assert_eq!(net_climb, ZERO);

    // ---------------------------------------------------------------------
    // 3. Functor on the chain: drive the route twice.
    // ---------------------------------------------------------------------
    // `ChainWitness::fmap` reaches the weights and leaves the complex and the grade in place, so
    // the result is still a 1-chain over the same road network.
    let repeats = REPEATS;
    let twice = ChainWitness::<FloatType>::fmap(route.clone(), move |w| w * repeats);
    let twice_circulation = pair(&wind, &twice);
    print_chain_scaling(twice.grade(), circulation, twice_circulation);

    assert_eq!(twice_circulation, circulation * repeats);

    // ---------------------------------------------------------------------
    // 4. Functor on the cochain: a stronger wind.
    // ---------------------------------------------------------------------
    // `CochainWitness::fmap` reaches the values and carries the degree across. The degree is held,
    // never recounted from the value count, which is what keeps a degree-1 form a degree-1 form.
    let scale = FORM_SCALE;
    let stronger = CochainWitness::fmap(wind.clone(), move |v| v * scale);
    let scaled_circulation = pair(&stronger, &route);
    print_cochain_scaling(stronger.degree(), circulation, scaled_circulation);

    assert_eq!(scaled_circulation, circulation * scale);

    // ---------------------------------------------------------------------
    // 5. Foldable on both sides.
    // ---------------------------------------------------------------------
    // A fold drops the structure and keeps the payload, so each side reduces to one number: the
    // net signed traversal count, and the total assist across every segment.
    let traversals = ChainWitness::<FloatType>::fold(route, ZERO, |acc, w| acc + w);
    let total_assist = CochainWitness::fold(wind, ZERO, |acc, v| acc + v);
    print_folds(traversals, total_assist);

    print_footer();
    Ok(())
}

/// The pairing `⟨ω, c⟩ = Σ ω(cell)·c(cell)`, the discrete line integral of a form along a chain.
fn pair(form: &Cochain<FloatType>, chain: &Chain<FloatType, FloatType>) -> FloatType {
    let weights = chain.weights();

    form.values()
        .iter()
        .enumerate()
        .fold(ZERO, |acc, (cell, &value)| {
            acc + value * weights.get_value_at(0, cell)
        })
}

/// The coboundary of the depot heights: one height difference per segment.
///
/// This is `δh`, the discrete gradient. A cochain built this way is exact, which is what makes its
/// pairing with any closed loop vanish.
fn coboundary_of_heights() -> Cochain<FloatType> {
    let values = SEGMENTS
        .iter()
        .map(|&[from, to]| lift::<FloatType>(DEPOT_HEIGHTS[to] - DEPOT_HEIGHTS[from]))
        .collect();

    Cochain::new(values, GRADE)
}

/// The road network as a simplicial complex: depots as vertices, segments as edges.
fn build_road_network() -> Result<SimplicialComplex<FloatType>, Box<dyn std::error::Error>> {
    let depots: Vec<Simplex> = (0..N_DEPOTS).map(|i| Simplex::new(vec![i])).collect();
    let segments: Vec<Simplex> = SEGMENTS
        .iter()
        .map(|&[a, b]| Simplex::new(vec![a, b]))
        .collect();

    // ∂₁ carries incidence signs: a segment leaves one depot and arrives at the other.
    let mut triplets: Vec<(usize, usize, i8)> = Vec::with_capacity(2 * N_SEGMENTS);
    for (cell, &[from, to]) in SEGMENTS.iter().enumerate() {
        triplets.push((from, cell, -1));
        triplets.push((to, cell, 1));
    }
    let d1 = CsrMatrix::from_triplets(N_DEPOTS, N_SEGMENTS, &triplets)?;

    Ok(SimplicialComplex::new(
        vec![Skeleton::new(0, depots), Skeleton::new(1, segments)],
        vec![d1],
        vec![],
        vec![],
    ))
}

/// A 1-chain over the road network, holding one signed weight per segment.
fn build_chain(
    complex: Arc<SimplicialComplex<FloatType>>,
    weights: &[f64],
) -> Result<Chain<FloatType, FloatType>, Box<dyn std::error::Error>> {
    let triplets: Vec<(usize, usize, FloatType)> = weights
        .iter()
        .enumerate()
        .map(|(cell, &w)| (0, cell, lift::<FloatType>(w)))
        .collect();
    let sparse = CsrMatrix::from_triplets(1, N_SEGMENTS, &triplets)?;

    Ok(Chain::new(complex, GRADE, sparse))
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_header() {
    println!("=== Chains, cochains, and the pairing between them ===\n");
    println!("  A chain says where the van drives. A cochain says what a segment measures.");
    println!("  The pairing evaluates one against the other.\n");
}

/// The display boundary: `f64` appears here and nowhere else.
fn print_objects(
    route: &Chain<FloatType, FloatType>,
    climb: &Cochain<FloatType>,
    wind: &Cochain<FloatType>,
) {
    println!("--- 1. The road network and the two objects ---");
    println!("  depots      {N_DEPOTS}, at heights {DEPOT_HEIGHTS:?} m");
    println!("  segments    {SEGMENTS:?}");
    println!();
    println!(
        "  route       grade {}, weights {:?}   <- the 1-chain, signed by direction",
        route.grade(),
        ROUTE
    );
    println!(
        "  climb       degree {}, values  {:?}   <- the coboundary of the heights",
        climb.degree(),
        shown(climb.values())
    );
    println!(
        "  wind        degree {}, values  {:?}   <- a form with no potential behind it",
        wind.degree(),
        shown(wind.values())
    );
}

fn print_pairing(net_climb: FloatType, circulation: FloatType) {
    println!("\n--- 2. The pairing <w, c> ---");
    println!(
        "  <climb, route>   {:7.2} m     the route is a closed loop and climb is exact,",
        lower(net_climb)
    );
    println!("                                 so every height cancels and the sum is zero");
    println!(
        "  <wind,  route>   {:7.2}       a form with no potential keeps a circulation",
        lower(circulation)
    );
}

fn print_chain_scaling(grade: usize, once: FloatType, twice: FloatType) {
    println!("\n--- 3. Functor on the chain: drive it {REPEATS} times ---");
    println!("  ChainWitness::fmap reaches the weights and carries the complex across.");
    println!("  grade after fmap   {grade}");
    println!(
        "  <wind, route>      {:7.2}  ->  <wind, {REPEATS}*route>  {:7.2}",
        lower(once),
        lower(twice)
    );
}

fn print_cochain_scaling(degree: usize, plain: FloatType, scaled: FloatType) {
    println!("\n--- 4. Functor on the cochain: a wind {FORM_SCALE} times stronger ---");
    println!("  CochainWitness::fmap reaches the values and carries the degree across.");
    println!("  degree after fmap  {degree}");
    println!(
        "  <wind, route>      {:7.2}  ->  <{FORM_SCALE}*wind, route>  {:7.2}",
        lower(plain),
        lower(scaled)
    );
}

fn print_folds(traversals: FloatType, total_assist: FloatType) {
    println!("\n--- 5. Foldable on both sides ---");
    println!(
        "  fold over the chain     {:7.2}   net signed traversals, the loop balancing out",
        lower(traversals)
    );
    println!(
        "  fold over the cochain   {:7.2}   the assist summed over every segment",
        lower(total_assist)
    );
}

fn print_footer() {
    println!("\n--- The point ---");
    println!("  The pairing is linear in each argument, so a `fmap` on either side moves");
    println!("  straight through it. That is what lets a unit change, a rescaling or a");
    println!("  repeated route be applied to the object it belongs to and read off the");
    println!("  result, with no second implementation of the integral.");
}

fn shown(values: &[FloatType]) -> Vec<f64> {
    values.iter().map(|&v| lower(v)).collect()
}

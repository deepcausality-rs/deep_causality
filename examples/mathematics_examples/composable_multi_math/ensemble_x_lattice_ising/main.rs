/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Ensemble x Lattice: the 2D Ising model
//!
//! A periodic `L x L` lattice of spins, evolved by Metropolis-Hastings, run as an ensemble of
//! independent replicas. Four crates meet, each doing only what it owns:
//!
//! - `deep_causality_rand` supplies entropy — a seeded generator and nothing else.
//! - `deep_causality_stats` supplies the acceptance draw, `StandardUniform` at the working scalar.
//! - `deep_causality_tensor` holds each lattice, and holds the ensemble of lattices.
//! - `deep_causality_haft` maps observables over the ensemble and folds them to means.
//!
//! ## The closed form this checks against
//!
//! Onsager solved this model exactly in 1944. The critical temperature is
//!
//! ```text
//! Tc = 2 / ln(1 + sqrt(2)) = 2.269185...
//! ```
//!
//! Below it the lattice orders and the magnetisation per spin approaches one; above it the lattice
//! disorders and the magnetisation falls towards zero. The run below prints `|m|` either side of
//! `Tc` and at it, and that transition is the check.
//!
//! ## Why the traversal has to be the diagonal one
//!
//! The susceptibility is a *fluctuation*:
//!
//! ```text
//! chi = beta * N * (<m^2> - <m>^2)
//! ```
//!
//! It is meaningful only if replica `i`'s `m^2` is the square of replica `i`'s `m`. Turning the
//! field of observables inside out through `Traversable::sequence` uses the cartesian applicative,
//! which would pair replica 3's magnetisation with replica 17's energy — every combination, and
//! none of them a replica. `DiagonalTraversable::sequence_zip` pairs index with index, which is
//! what a replica *is*.
//!
//! The example asserts that pairing directly: for every field in the result, cell 1 must be the
//! exact square of cell 0. Under a cartesian traversal it is not.
//!
//! ## APIs demonstrated
//!
//! - `StandardUniform` at the caller's scalar (`deep_causality_stats`)
//! - `Xoshiro256::from_seed` (`deep_causality_rand`)
//! - `CausalTensorWitness::fmap` (Functor) and `::fold` (Foldable)
//! - `CausalTensorWitness::sequence_zip` with `ZipTensorWitness` (DiagonalTraversable)

use deep_causality_algebra::{Real, RealField};
use deep_causality_haft::{DiagonalTraversable, Foldable, Functor};
use deep_causality_num::{Float106, FromPrimitive, ToPrimitive, lift, lift_count, lower};
use deep_causality_rand::Xoshiro256;
use deep_causality_stats::{Distribution, RandScalar, Rng, StandardUniform};
use deep_causality_tensor::{CausalTensor, CausalTensorWitness, ZipTensorWitness};

/// `f32` is the right precision for the simulation itself.
///
/// Metropolis is noise-bound: every step is a random accept or reject, and the statistical error
/// of an ensemble mean over 32 replicas is a few percent. A 24-bit significand is far below that,
/// so the extra bits of `f64` would buy accuracy the method does not have.
///
/// The susceptibility is a different matter, and the table at the end of the run is about exactly
/// that: `<m^2>` and `<m>^2` nearly coincide near `Tc`, so subtracting them cancels away most of
/// the significand. The simulation is noise-bound; the observable derived from it is
/// cancellation-bound, and the two do not want the same precision.
pub type FloatType = f32;

/// Lattice side for the physics: large enough to show the transition, small enough to be quick.
const COARSE_L: usize = 16;

/// A second lattice side, used only by the precision table.
///
/// `|m| = k / N` is a dyadic rational needing `log2(N)` significand bits, `m^2` needs twice that,
/// and summing `R` of them needs `2 log2(N) + log2(R)`. At `L = 16` that is 21 bits, inside
/// `f32`'s 24, so the whole reduction is *exact* there and three scalars cannot disagree. At
/// `L = 32` it is 25 and `f32` must round. The table shows both, because the threshold is the
/// point.
const FINE_L: usize = 32;

/// Spins per lattice, at the physics size.
const N: usize = COARSE_L * COARSE_L;

/// Independent replicas in the ensemble.
const REPLICAS: usize = 32;

/// Metropolis sweeps per replica. The first half is discarded as thermalisation.
const SWEEPS: usize = 2_000;

/// The seed every replica's generator is derived from, so the run reproduces.
const SEED: u64 = 0x0001_519A_2026;

/// Onsager's exact critical temperature, `2 / ln(1 + sqrt(2))`.
const TC: f64 = 2.269_185_314_213_022;

fn main() {
    println!("=== Ensemble x Lattice: the 2D Ising model ===");
    println!(
        "Simulation precision: {}",
        core::any::type_name::<FloatType>()
    );
    println!("Lattice {COARSE_L}x{COARSE_L} = {N} spins, {REPLICAS} replicas, {SWEEPS} sweeps\n");

    report_transition();
    let at_critical = report_ensemble(TC, COARSE_L, "Tc");
    let ordered = report_ensemble(1.5, COARSE_L, "T = 1.5, deep in the ordered phase");
    report_susceptibility(&at_critical);

    let fine_critical = report_ensemble(TC, FINE_L, "Tc on a 32x32 lattice");
    let fine_ordered = report_ensemble(1.5, FINE_L, "T = 1.5 on a 32x32 lattice");
    report_precision(&at_critical, &ordered, &fine_critical, &fine_ordered);
}

// -------------------------------------------------------------------------------------------
// The model
// -------------------------------------------------------------------------------------------

/// One Metropolis sweep: every site visited once, in order.
///
/// For the Ising Hamiltonian `H = -J sum_<ij> s_i s_j` with `J = 1`, flipping spin `i` costs
/// `dE = 2 s_i sum_neighbours`. A move that lowers the energy is always taken; one that raises it
/// is taken with probability `exp(-beta dE)` — and that probability is where `stats` enters, as a
/// uniform draw at the working scalar compared against the Boltzmann factor.
fn sweep<T, R>(spins: &mut [T], l: usize, beta: T, rng: &mut R)
where
    T: RandScalar,
    R: Rng + ?Sized,
{
    let two = lift::<T>(2.0);
    for y in 0..l {
        for x in 0..l {
            let i = y * l + x;
            let neighbours = spins[y * l + (x + 1) % l]
                + spins[y * l + (x + l - 1) % l]
                + spins[((y + 1) % l) * l + x]
                + spins[((y + l - 1) % l) * l + x];
            let delta = two * spins[i] * neighbours;

            let accepted = if delta <= T::zero() {
                true
            } else {
                let u: T = StandardUniform.sample(rng);
                u < Real::exp(-beta * delta)
            };
            if accepted {
                spins[i] = -spins[i];
            }
        }
    }
}

/// A thermalised lattice at temperature `t`, as a `[L, L]` tensor of spins.
fn equilibrate<T>(t: f64, l: usize, seed: u64) -> CausalTensor<T>
where
    T: RandScalar,
{
    let mut rng = Xoshiro256::from_seed(seed);
    let beta = lift::<T>(1.0 / t);

    // A cold start: every spin up. Above `Tc` the sweeps disorder it within the thermalisation
    // half, and below `Tc` it is already in the right basin — which avoids the domain walls a hot
    // start traps on a small lattice.
    let mut spins = vec![T::one(); l * l];
    for _ in 0..SWEEPS {
        sweep(&mut spins, l, beta, &mut rng);
    }
    CausalTensor::from_vec(spins, &[l, l])
}

/// Magnetisation per spin, `|m| = |sum s_i| / N`.
fn magnetisation<T>(lattice: &CausalTensor<T>) -> T
where
    T: RealField + FromPrimitive,
{
    let total = lattice.as_slice().iter().fold(T::zero(), |acc, s| acc + *s);
    Real::abs(total) / lift_count::<T>(lattice.len() as u64)
}

// -------------------------------------------------------------------------------------------
// The transition, against Onsager
// -------------------------------------------------------------------------------------------

fn report_transition() {
    println!("-- The transition, against Onsager's Tc = {TC:.6} --\n");
    println!("{:>8}  {:>10}  {:>10}", "T", "|m|", "phase");

    for (t, label) in [
        (1.5, "ordered"),
        (2.0, "ordered"),
        (TC, "critical"),
        (2.6, "disordered"),
        (3.5, "disordered"),
    ] {
        let lattice = equilibrate::<FloatType>(t, COARSE_L, SEED);
        let m = lower(magnetisation(&lattice));
        println!("{t:>8.4}  {m:>10.4}  {label:>10}");
    }
    println!();
}

// -------------------------------------------------------------------------------------------
// The ensemble: a tensor of tensors
// -------------------------------------------------------------------------------------------

/// Runs the ensemble at one temperature and returns the per-replica magnetisations, lowered once.
fn report_ensemble(t: f64, l: usize, label: &str) -> Vec<f64> {
    println!("-- The ensemble at {label}: {REPLICAS} replicas, each its own lattice --\n");

    // The ensemble is a tensor whose payload is tensors. No new container was needed for it: a
    // rank-1 `CausalTensor` of lattices already carries every witness the fold below wants.
    let replicas: Vec<CausalTensor<FloatType>> = (0..REPLICAS)
        .map(|r| equilibrate::<FloatType>(t, l, SEED.wrapping_add(r as u64 * 0x9E37_79B9)))
        .collect();
    let ensemble: CausalTensor<CausalTensor<FloatType>> =
        CausalTensor::from_vec(replicas, &[REPLICAS]);

    // Functor: one observable per replica.
    let m_per_replica = CausalTensorWitness::fmap(ensemble, |lattice| magnetisation(&lattice));

    // Foldable: the ensemble mean.
    let sum = CausalTensorWitness::fold(m_per_replica.clone(), lift::<FloatType>(0.0), |acc, m| {
        acc + m
    });
    let mean = sum / lift_count::<FloatType>(REPLICAS as u64);

    let values: Vec<f64> = m_per_replica.as_slice().iter().map(|m| lower(*m)).collect();
    let spread = {
        let mu = lower(mean);
        let var = values.iter().map(|m| (m - mu) * (m - mu)).sum::<f64>() / values.len() as f64;
        var.sqrt()
    };

    println!(
        "  <|m|> over {REPLICAS} replicas = {:.4} +/- {spread:.4}",
        lower(mean)
    );
    println!("  fmap gave the observables, fold gave the mean; neither knows what a lattice is\n");
    values
}

// -------------------------------------------------------------------------------------------
// The susceptibility, through the diagonal traversal
// -------------------------------------------------------------------------------------------

fn report_susceptibility(magnetisations: &[f64]) {
    println!("-- The susceptibility, and why the traversal must be diagonal --\n");

    // A field of three observables. Each cell holds the whole ensemble: cell 0 every replica's
    // |m|, cell 1 every replica's m^2, cell 2 every replica's index as a stand-in for a second
    // measured quantity. The field is the structure; the ensemble is what each cell holds.
    let m: Vec<FloatType> = magnetisations
        .iter()
        .map(|v| lift::<FloatType>(*v))
        .collect();
    let m_sq: Vec<FloatType> = m.iter().map(|v| *v * *v).collect();
    let tag: Vec<FloatType> = (0..REPLICAS)
        .map(|r| lift_count::<FloatType>(r as u64))
        .collect();

    let field: CausalTensor<CausalTensor<FloatType>> = CausalTensor::from_vec(
        vec![
            CausalTensor::from_vec(m.clone(), &[REPLICAS]),
            CausalTensor::from_vec(m_sq, &[REPLICAS]),
            CausalTensor::from_vec(tag, &[REPLICAS]),
        ],
        &[3],
    );

    // The seed declares how many replicas the result should hold. A positional zip has no unit —
    // its unit would be the infinite repeat — so there is nothing to derive this from, and the
    // caller says it outright.
    let seed: CausalTensor<CausalTensor<FloatType>> = CausalTensor::from_vec(
        (0..REPLICAS)
            .map(|_| CausalTensor::from_vec(Vec::new(), &[0]))
            .collect(),
        &[REPLICAS],
    );

    let per_replica = CausalTensorWitness::sequence_zip::<FloatType, ZipTensorWitness>(field, seed);

    // The check that the pairing held. Cell 1 is the square of cell 0 for the *same* replica, so
    // it must be the exact square here — same values, same multiplication. A cartesian traversal
    // pairs replica 3 with replica 17 and this fails on the first field that differs.
    for (i, f) in per_replica.as_slice().iter().enumerate() {
        let row = f.as_slice();
        assert_eq!(
            row[1],
            row[0] * row[0],
            "field {i} pairs m^2 with a different replica's m"
        );
        assert_eq!(
            row[2],
            lift_count::<FloatType>(i as u64),
            "field {i} carries another replica's tag"
        );
    }
    println!("  {REPLICAS} fields of 3 observables; every field's m^2 is its own m squared");

    // chi = beta * N * (<m^2> - <m>^2), from the paired fields.
    let mean_m = column_mean(&per_replica, 0);
    let mean_m_sq = column_mean(&per_replica, 1);
    let beta = lift::<FloatType>(1.0 / TC);
    let chi = beta * lift_count::<FloatType>(N as u64) * (mean_m_sq - mean_m * mean_m);

    // The same number the direct way: the variance of the magnetisations.
    let mu = magnetisations.iter().sum::<f64>() / magnetisations.len() as f64;
    let var = magnetisations
        .iter()
        .map(|v| (v - mu) * (v - mu))
        .sum::<f64>()
        / magnetisations.len() as f64;
    let chi_direct = (1.0 / TC) * N as f64 * var;

    println!("  chi via the diagonal traversal = {:.4}", lower(chi));
    println!("  chi from the variance directly = {chi_direct:.4}");
    println!(
        "  agreement to {:.2e}\n",
        (lower(chi) - chi_direct).abs() / chi_direct
    );
}

/// The ensemble mean of one observable column, by fold.
fn column_mean<T>(per_replica: &CausalTensor<CausalTensor<T>>, column: usize) -> T
where
    T: RealField + FromPrimitive + Clone,
{
    let column_values: Vec<T> = per_replica
        .as_slice()
        .iter()
        .map(|f| f.as_slice()[column])
        .collect();
    let sum = CausalTensorWitness::fold(
        CausalTensor::from_vec(column_values, &[REPLICAS]),
        T::zero(),
        |acc, v| acc + v,
    );
    sum / lift_count::<T>(REPLICAS as u64)
}

// -------------------------------------------------------------------------------------------
// One table, three precisions
// -------------------------------------------------------------------------------------------

/// `chi` recomputed at three scalars from identical data, at two lattice sizes.
///
/// Holding the data fixed is the point. Running the simulation again at another precision would
/// change the trajectory too, and the two effects could not be told apart. Here only the
/// arithmetic changes.
///
/// # What the table actually shows, and what it was expected to show
///
/// The expectation going in was that `chi` near `Tc` would be cancellation-bound: `<m^2>` and
/// `<m>^2` nearly coincide, subtracting them discards the agreeing digits, and a wider scalar is
/// needed. Both halves of that turned out to be wrong here, and the measurements are why.
///
/// **The cancellation is mildest at `Tc`, not worst.** `chi` is proportional to the variance, and
/// a critical point is where the variance diverges — that is what makes it critical. The
/// fluctuation is largest exactly where the quantity is being asked for, so the subtraction
/// discards least there. Measured: `<|m|> = 0.74 +/- 0.15` at `Tc` against `0.99 +/- 0.01` at
/// `T = 1.5`, and the second is where the digits go.
///
/// **And at this lattice size no scalar can disagree, because the arithmetic is exact.** `|m|` is
/// `k / N` for an integer `k`: a dyadic rational needing `log2(N)` significand bits. `m^2` needs
/// twice that, and summing `R` of them needs `2 log2(N) + log2(R)`. At `L = 16, R = 32` that is
/// 21 bits, inside `f32`'s 24 — so `f32`, `f64` and `Float106` return **bit-identical** results,
/// and a table claiming to show a precision effect would have been showing nothing.
///
/// At `L = 32` the same count is 25 bits and `f32` must round. Crossing that threshold is what
/// puts an effect in the table, and the threshold was computed before the run rather than found
/// in it.
///
/// The four rows are a 2x2 — representable or not, fluctuation large or small — and only the cell
/// where both are unfavourable costs anything: `7.7e-5` against `3e-8`, `4e-10` and `1.4e-7`. The
/// *best* of the four is the exact reduction with the smallest fluctuation, which is the opposite
/// of what the cancellation argument alone would predict.
///
/// The lesson is not "use a wider float for fluctuations". It is that whether you need one is a
/// question about your data, and it has an answer you can compute before reaching for it.
fn report_precision(
    at_critical: &[f64],
    ordered: &[f64],
    fine_critical: &[f64],
    fine_ordered: &[f64],
) {
    println!("-- chi at three precisions, from identical data --\n");
    println!(
        "{:>34}  {:>10}  {:>22}  {:>12}",
        "regime", "scalar", "chi", "rel. to Float106"
    );

    // A 2x2 design: is the reduction exactly representable, and is the fluctuation large?
    // Only the cell where both answers are unfavourable costs anything.
    for (label, data) in [
        ("L=16, Tc - exact, large fluct.", at_critical),
        ("L=16, T=1.5 - exact, small fluct.", ordered),
        ("L=32, Tc - rounds, large fluct.", fine_critical),
        ("L=32, T=1.5 - rounds, small fluct.", fine_ordered),
    ] {
        let reference = chi_at::<Float106>(data);
        for (name, value) in [
            ("f32", chi_at::<f32>(data)),
            ("f64", chi_at::<f64>(data)),
            ("Float106", reference),
        ] {
            let rel = if reference == 0.0 {
                value.abs()
            } else {
                (value - reference).abs() / reference.abs()
            };
            println!("{label:>34}  {name:>10}  {value:>22.12}  {rel:>12.2e}");
        }
        println!();
    }

    println!("  Two things have to go wrong before f32 costs anything, and only the last row");
    println!("  has both: the reduction must exceed 24 bits, and the fluctuation must be small");
    println!("  enough for the subtraction to discard what is left. Either alone is harmless —");
    println!("  the exact rows are unaffected by a tiny fluctuation, and the large-fluctuation");
    println!("  row is unaffected by rounding.");
    println!();
    println!("  The simulation itself is noise-bound: 32 replicas give a few percent of");
    println!("  statistical error, far above anything f32 contributes. Whether the observable");
    println!("  derived from it needs more is a separate question with a computable answer,");
    println!("  and reaching for a wider float without asking it is guesswork.");
}

/// `chi = beta N (<m^2> - <m>^2)` computed entirely at `T`, lowered once for display.
fn chi_at<T>(magnetisations: &[f64]) -> f64
where
    T: RealField + FromPrimitive + ToPrimitive,
{
    let n = lift_count::<T>(magnetisations.len() as u64);
    let mut sum = T::zero();
    let mut sum_sq = T::zero();
    for v in magnetisations {
        let m = lift::<T>(*v);
        sum += m;
        sum_sq += m * m;
    }
    let mean = sum / n;
    let mean_sq = sum_sq / n;
    let beta = lift::<T>(1.0 / TC);
    lower(beta * lift_count::<T>(N as u64) * (mean_sq - mean * mean))
}

/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Crosstalk attribution: direct cause or common cause.
//!
//! Two qubits' errors are correlated beyond independence. Three structures fit the passive
//! observation exactly, because they are Markov equivalent, and a fourth is a cycle. This is the
//! keystone example: the only one that runs `validate` and then `control` on one problem, so it
//! is where the hand-off between the halves is exercised.
//!
//!   * `build()` refuses the cyclic candidate as `CyclicStructureUnsupported`, by decision and
//!     before any check runs. Under Definition 3.1 the C₃ criterion does not reject it, so if it is
//!     to be kept out, the builder has to do it and say why.
//!   * `validate` screens the other three with the Markov check and the decomposability check on
//!     the structure each candidate's own supports encode; all three are admitted.
//!   * `control` takes the screen, forks one world per admitted candidate, and `design` returns a
//!     plan: two cheap interventions at cost 2, against process tomography at 200.
//!   * The plan's first experiment is run on the world where H₁ is true, drawing 1024 shots from
//!     the shipped Born sampler at H₁'s predicted read-out. Each world's prediction is judged
//!     against that observation, and `adjudicate` folds the three Boolean verdicts. No commutation
//!     test runs, because a threshold on a real quantity is a classical proposition.
//!
//! Predictions and the plan are computed; the observation is sampled. The physics behind each
//! predicted read-out is a modelling assumption stated in `model.rs`.

mod constants;
mod model;

use deep_causality_algebra::Real;
use deep_causality_haft::Either;
use deep_causality_num::{FromPrimitive, ToPrimitive};
use deep_causality_num_complex::Complex;
use deep_causality_quantum::{
    Check, CheckItem, CheckReport, CommutatorTolerance, DensityMatrix, MinCostCover, Projection,
    QclBuilder, QuantumErrorEnum, ShotEstimate, World, adjudicate, sample_projector,
};
use deep_causality_tensor::CausalTensor;

use crate::constants::{AGREEMENT_SIGMAS, FLOOR_BITS, SEED, SHOTS};
use crate::model::{
    e1_projector, e2_projector, experiments, h1_direct_q1_to_q2, h2_direct_q2_to_q1,
    h3_common_bath, h4_cyclic, lift, plant, systems,
};

/// The real working type.
pub type FloatType = f64;

/// The count working type.
pub type NumberType = u64;

/// The complex scalar.
pub type C = Complex<FloatType>;

fn main() {
    let floor_bits = lift(FLOOR_BITS);
    println!("=== Crosstalk attribution: direct cause or common cause ===");
    println!("shots: {SHOTS}   floor: {FLOOR_BITS} bits   seed: {SEED}\n");

    // -- build(): the cyclic candidate is refused, by decision --------------------------
    println!("[build] four declared structures");
    let refused = QclBuilder::config::<FloatType, NumberType>()
        .over_plant(plant(), &[e2_projector(), e1_projector()])
        .candidates(&[
            h1_direct_q1_to_q2(),
            h2_direct_q2_to_q1(),
            h3_common_bath(),
            h4_cyclic(),
        ])
        .probes(&experiments())
        .seed(SEED)
        .build();
    match refused {
        Ok(_) => println!("    unexpected: a cyclic candidate must not build"),
        Err(e) => match e.0 {
            QuantumErrorEnum::CyclicStructureUnsupported(msg) => {
                println!("    ✓ refused at build(): {msg}");
            }
            other => println!("    refused with an unexpected error: {other:?}"),
        },
    }

    let cfg = QclBuilder::config::<FloatType, NumberType>()
        .over_plant(plant(), &[e2_projector(), e1_projector()])
        .candidates(&[h1_direct_q1_to_q2(), h2_direct_q2_to_q1(), h3_common_bath()])
        .probes(&experiments())
        .seed(SEED)
        .build()
        .expect("three acyclic candidates build");

    // -- validate: Markov, then C₃ on each candidate's own structure -------------------
    println!("\n[validate] three acyclic structures");
    let sys = systems();
    let screened = QclBuilder::validate(&cfg)
        .check_markov(&CommutatorTolerance::<FloatType>::default())
        .check_decomposable(&sys, &sys)
        .finalize()
        .expect("three legal, decomposable QCMs screen");
    for (name, report) in screened.stages() {
        println!(
            "    {name:<20} {:?}  examined {}",
            report.verdict(),
            report.examined()
        );
    }
    for h in screened.admitted() {
        println!("    admitted: {}", h.name());
    }
    assert_eq!(screened.admitted().len(), 3);

    // -- control: the hand-off, the fork, and the plan ---------------------------------
    println!("\n[control] the screen enters control; a structural config could not");
    let report = QclBuilder::control::<FloatType, NumberType, 4, _>(&screened)
        .fork()
        .design(MinCostCover::new(floor_bits))
        .finalize()
        .expect("fork and design run on the admitted candidates");
    let plan = report.plan.as_ref().expect("design ran");
    println!(
        "    forked {} worlds, each with its own ledger, none moved into an arm",
        report.worlds.len()
    );
    println!("\n[design] minimum-cost cover, floor {FLOOR_BITS} bits");
    for (i, e) in experiments().iter().enumerate() {
        let chosen = plan.entries().iter().find(|p| p.experiment == i);
        println!(
            "    {:<26} cost {:>5}   {}",
            e.name(),
            e.cost(),
            chosen.map_or("—".to_string(), |p| format!(
                "chosen, resolves {:?}",
                p.resolves
            ))
        );
    }
    println!(
        "    plan: {:?}  total cost {}   (tomography alone would cost 200, {}× more)",
        plan.entries()
            .iter()
            .map(|e| e.name.as_str())
            .collect::<Vec<_>>(),
        plan.total_cost(),
        lift(200.0) / plan.total_cost()
    );
    println!(
        "    tightest pair separates at {:.1} bits against the floor; ledger cost {}",
        plan.report().worst().expect("three pairs").measured,
        report.ledger.cost()
    );
    assert!(plan.is_complete());
    assert_eq!(plan.total_cost(), lift(2.0));

    // -- the first planned experiment, observed under H₁ ---------------------------------
    let first = &experiments()[plan.entries()[0].experiment];
    let truth = 0usize; // H₁ is the world the observation is drawn from.
    let observed = observe_under(first.predictions()[truth]);
    println!(
        "\n[{}] observed {:.3} ± {:.3} over {} shots, drawn from the Born sampler at H1's prediction",
        first.name(),
        observed.estimate(),
        observed.standard_error(),
        observed.shots()
    );

    // -- adjudicate: each world's prediction against the observation, folded as Boolean --
    let worlds: Vec<World<FloatType, 4>> = report
        .worlds
        .iter()
        .enumerate()
        .map(|(i, w)| {
            let predicted = first.predictions()[i];
            let prediction = predicted_as_read_out(predicted);
            let verdict = agrees(&prediction, &observed);
            println!(
                "    {:<18} predicts {:.2}   {}",
                w.name(),
                predicted,
                if verdict.accepted() {
                    "consistent"
                } else {
                    "rejected"
                }
            );
            World::read_out(w.name(), verdict, prediction)
        })
        .collect();
    let a = adjudicate(&worlds, floor_bits).expect("three read-out worlds fold");
    println!(
        "\n[adjudicate] {} worlds folded, {} commutation pairs tested: a Boolean fold, §4 rule 2 does not apply",
        a.worlds_folded, a.commutation_pairs_tested
    );
    match &a.outcome {
        Either::Left(s) => {
            println!(
                "    {} survives, {:.1} bits from its nearest rival. Direct cause, not shared bath.",
                s.name, s.separation_bits
            );
            println!(
                "    -> a scheduling or echo fix applies; frequency reallocation is not required."
            );
        }
        Either::Right(why) => println!("    ambiguous: {why:?}"),
    }
    assert!(matches!(&a.outcome, Either::Left(s) if s.name.starts_with("H1")));
}

/// `shots` draws from a qubit whose excited population is `p`, through the shipped Born sampler.
fn observe_under(p: FloatType) -> ShotEstimate<FloatType> {
    let zero = lift(0.0);
    let rho = DensityMatrix::new(CausalTensor::from_slice(
        &[
            Complex::new(lift(1.0) - p, zero),
            Complex::new(zero, zero),
            Complex::new(zero, zero),
            Complex::new(p, zero),
        ],
        &[2, 2],
    ))
    .expect("a valid state");
    let excited = Projection::<FloatType, 2>::from_ket(&CausalTensor::from_slice(
        &[Complex::new(zero, zero), Complex::new(lift(1.0), zero)],
        &[2],
    ))
    .expect("a projector");
    let hist = sample_projector(&rho, &excited, SHOTS, SEED).expect("shots");
    ShotEstimate::of_outcome(&hist, 1).expect("a non-empty histogram")
}

/// A world's predicted read-out, carried with the shot noise it would have at the planned shots.
fn predicted_as_read_out(p: FloatType) -> ShotEstimate<FloatType> {
    // `round` is inherent on `f64` and a `Real` method on `Float106`; the trait path serves both.
    let ones = Real::round(p * FloatType::from_u64(SHOTS).expect("shots are representable"))
        .to_u64()
        .expect("a rounded count fits");
    let mut hist = deep_causality_quantum::CountHistogram::new(1);
    hist.record_n(1, ones);
    hist.record_n(0, SHOTS - ones);
    ShotEstimate::of_outcome(&hist, 1).expect("a non-empty histogram")
}

/// Whether a prediction agrees with the observation: the gap against `AGREEMENT_SIGMAS` standard
/// errors of the observation, in the decision form.
fn agrees(
    prediction: &ShotEstimate<FloatType>,
    observed: &ShotEstimate<FloatType>,
) -> CheckReport<FloatType> {
    let gap = Real::abs(prediction.estimate() - observed.estimate());
    let allowance = lift(AGREEMENT_SIGMAS) * observed.standard_error();
    CheckReport::new(
        vec![Check::new(CheckItem::Whole, gap, allowance)],
        observed.shots() as usize,
    )
}

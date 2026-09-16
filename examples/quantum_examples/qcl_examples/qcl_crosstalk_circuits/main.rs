/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Crosstalk attribution over circuit-derived candidates.
//!
//! The v1 example declares its four structures as factorizations. This one writes `H₁`, `H₂` and
//! the cyclic `H₄` as `CircuitModel` values and lets the wiring carry the structure: `build()` on
//! `.over_circuit` refuses the cycle by name, `validate` screens each acyclic circuit's dilation by
//! Markov and C₃, and the dilation's normalised factors become the candidate the plant pipeline
//! forks. `H₃` stays the v1 factorization, for the reason `model.rs` states. The plant, the
//! observables, the probes, the plan and the adjudication are the v1 example's, so the decision is
//! what is reproduced: three admitted, the cycle refused, `{do(Q1), do(Q2)}` at cost 2 against
//! tomography at 200, and `H1 Q1->Q2` the survivor.

mod constants;
mod model;

use deep_causality_algebra::Real;
use deep_causality_haft::Either;
use deep_causality_num::{Float106, lift_count, lower, to_count};
use deep_causality_num_complex::Complex;
use deep_causality_quantum::{
    Check, CheckItem, CheckReport, CircuitModel, CommutatorTolerance, DensityMatrix, Hypothesis,
    MinCostCover, Projection, QclBuilder, QuantumErrorEnum, ScreenOrigin, ShotEstimate, World,
    adjudicate, sample_projector,
};
use deep_causality_tensor::CausalTensor;

use crate::constants::{
    AGREEMENT_SIGMAS, CANDIDATE_COUNT, COST_TOMOGRAPHY, EXPECTED_PLAN_COST, FLOOR_BITS, ONE, SEED,
    SHOTS, ZERO,
};
use crate::model::{
    e1_projector, e2_projector, experiments, h1_circuit, h2_circuit, h3_common_bath,
    h4_cyclic_circuit, plant, systems,
};

/// The real working type. Switch it to `f32`, `f64`, or `Float106` to define the precision level.
pub type FloatType = Float106;

/// The count working type.
pub type NumberType = u64;

/// The complex scalar.
pub type C = Complex<FloatType>;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let floor_bits = FLOOR_BITS;
    println!("=== Crosstalk attribution over circuits: direct cause or common cause ===");
    println!("precision: {}", core::any::type_name::<FloatType>());
    println!(
        "shots: {SHOTS}   floor: {} bits   seed: {SEED}\n",
        lower(FLOOR_BITS)
    );

    // -- build(): the cyclic circuit is refused, by its wiring ---------------------------
    println!("[build] the cyclic circuit H4 Q1->Q2->B->Q1");
    match QclBuilder::config::<FloatType, NumberType>()
        .over_circuit(h4_cyclic_circuit()?)
        .build()
    {
        Ok(_) => println!("    UNEXPECTED: a cyclic grouping must not build"),
        Err(e) => match e.0 {
            QuantumErrorEnum::CyclicStructureUnsupported(msg) => {
                println!("    ✓ refused at build(): {msg}")
            }
            other => panic!("refused with an unexpected error: {other:?}"),
        },
    }

    // -- validate: each acyclic circuit's dilation, Markov then C₃ ------------------------
    println!("\n[validate] two circuits, each screened through its dilation");
    let h1 = screened_candidate("H1 Q1->Q2", h1_circuit()?)?;
    let h2 = screened_candidate("H2 Q2->Q1", h2_circuit()?)?;
    let h3 = h3_common_bath()?;
    println!(
        "    {:<12} kept as the v1 factorization (see model.rs)",
        h3.name()
    );

    // -- the plant pipeline over the circuit-derived candidates ------------------------
    let cfg = QclBuilder::config::<FloatType, NumberType>()
        .over_plant(plant()?, &[e2_projector()?, e1_projector()?])
        .candidates(&[h1, h2, h3])
        .probes(&experiments()?)
        .seed(SEED)
        .build()?;
    println!("\n[validate] the three candidates on the plant subject");
    let sys = systems();
    let screened = QclBuilder::validate(&cfg)
        .check_markov(&CommutatorTolerance::<FloatType>::default())
        .check_decomposable(&sys, &sys)
        .finalize()?;
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
    let all_admitted = screened.admitted().len() == CANDIDATE_COUNT;
    println!(
        "    all {CANDIDATE_COUNT} circuit-derived candidates admitted: {}",
        yes_no(all_admitted)
    );

    // -- control: fork and plan --------------------------------------------------------
    println!("\n[control] the screen enters control");
    let report = QclBuilder::control::<FloatType, NumberType, 4, _>(&screened)
        .fork()
        .design(MinCostCover::new(floor_bits))
        .finalize()?;
    let plan = report
        .plan
        .as_ref()
        .ok_or("design should have produced a plan")?;
    println!(
        "\n[design] minimum-cost cover, floor {} bits",
        lower(FLOOR_BITS)
    );
    for (i, e) in experiments()?.iter().enumerate() {
        let chosen = plan.entries().iter().find(|p| p.experiment == i);
        println!(
            "    {:<26} cost {:>5}   {}",
            e.name(),
            lower(e.cost()),
            chosen.map_or("—".to_string(), |p| format!(
                "chosen, resolves {:?}",
                p.resolves
            ))
        );
    }
    let names: Vec<&str> = plan.entries().iter().map(|e| e.name.as_str()).collect();
    println!(
        "    plan: {names:?}  total cost {}   (tomography alone would cost 200, {}× more)",
        lower(plan.total_cost()),
        lower(COST_TOMOGRAPHY / plan.total_cost())
    );

    let plan_is_complete = plan.is_complete();
    let plan_costs_two = plan.total_cost() == EXPECTED_PLAN_COST;
    let plan_picked_the_interventions =
        names.contains(&"E1 do(Q1=|1>) P(e2)") && names.contains(&"E2 do(Q2=|1>) P(e1)");

    println!(
        "    covers every pair: {}   costs {}: {}   picked both interventions: {}",
        yes_no(plan_is_complete),
        lower(EXPECTED_PLAN_COST),
        yes_no(plan_costs_two),
        yes_no(plan_picked_the_interventions)
    );

    // -- the first planned experiment, observed under H₁, and the adjudication ------------
    let probes = experiments()?;
    let first = &probes[plan.entries()[0].experiment];
    let truth = report
        .worlds
        .iter()
        .position(|w| w.name().starts_with("H1"))
        .ok_or("H1 should be among the forked worlds")?;
    let observed = observe_under(first.predictions()[truth])?;
    println!(
        "\n[{}] observed {:.3} ± {:.3} over {} shots, drawn from the Born sampler at H1's prediction",
        first.name(),
        lower(observed.estimate()),
        lower(observed.standard_error()),
        observed.shots()
    );
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
                lower(predicted),
                if verdict.accepted() {
                    "consistent"
                } else {
                    "rejected"
                }
            );
            World::read_out(w.name(), verdict, prediction)
        })
        .collect();
    let a = adjudicate(&worlds, floor_bits)?;
    match &a.outcome {
        Either::Left(s) => println!(
            "\n[adjudicate] {} survives, {:.1} bits from its nearest rival. Direct cause, not shared bath.",
            s.name,
            lower(s.separation_bits)
        ),
        Either::Right(why) => println!("\n[adjudicate] ambiguous: {why:?}"),
    }
    let named_the_truth = matches!(&a.outcome, Either::Left(s) if s.name.starts_with("H1"));
    println!(
        "    the adjudication named the structure the observation came from: {}",
        yes_no(named_the_truth)
    );

    println!("\n=== the v1 decision, reproduced over circuit-derived candidates ===");
    Ok(())
}

/// A check's verdict, as a word.
fn yes_no(ok: bool) -> &'static str {
    if ok { "yes" } else { "NO" }
}

/// One circuit through `.over_circuit`, screened by Markov and C₃ on its dilation, and the
/// dilation's normalised factors as the named structural candidate.
fn screened_candidate(
    name: &str,
    circuit: CircuitModel<FloatType>,
) -> Result<Hypothesis<FloatType>, Box<dyn std::error::Error>> {
    let cfg = QclBuilder::config::<FloatType, NumberType>()
        .over_circuit(circuit)
        .build()?;
    let nodes: Vec<usize> = (0..cfg.subject().model().nodes().len()).collect();
    let screened = QclBuilder::validate(&cfg)
        .check_markov(&CommutatorTolerance::<FloatType>::default())
        .check_decomposable(&nodes, &nodes)
        .finalize()?;

    if screened.origin() != ScreenOrigin::Circuit {
        return Err("a circuit subject should screen with a circuit origin".into());
    }

    let dilation = cfg.subject().model().dilation()?;
    let dag = cfg.subject().model().induced_dag();
    println!(
        "    {name:<12} {} nodes, edges {:?}; stages: {}",
        dag.num_vertices(),
        (0..dag.num_vertices())
            .flat_map(|v| dag.children(v).into_iter().map(move |c| (v, c)))
            .collect::<Vec<_>>(),
        screened
            .stages()
            .iter()
            .map(|(s, r)| format!("{s} {:?}", r.verdict()))
            .collect::<Vec<_>>()
            .join(", ")
    );
    Ok(dilation.hypothesis(name)?)
}

/// `shots` draws from a qubit whose excited population is `p`, through the shipped Born sampler.
fn observe_under(p: FloatType) -> Result<ShotEstimate<FloatType>, Box<dyn std::error::Error>> {
    let zero = ZERO;
    let one = ONE;
    let rho = DensityMatrix::new(CausalTensor::from_slice(
        &[
            Complex::new(one - p, zero),
            Complex::new(zero, zero),
            Complex::new(zero, zero),
            Complex::new(p, zero),
        ],
        &[2, 2],
    ))?;
    let excited = Projection::<FloatType, 2>::from_ket(&CausalTensor::from_slice(
        &[Complex::new(zero, zero), Complex::new(one, zero)],
        &[2],
    ))?;
    let hist = sample_projector(&rho, &excited, SHOTS, SEED)?;

    Ok(ShotEstimate::of_outcome(&hist, 1)?)
}

/// A world's predicted read-out, carried with the shot noise it would have at the planned shots.
fn predicted_as_read_out(p: FloatType) -> ShotEstimate<FloatType> {
    let ones = to_count(p * lift_count::<FloatType>(SHOTS)).expect("a rounded count fits");
    let mut hist = deep_causality_quantum::CountHistogram::new(1).expect("a one-bit histogram");
    hist.record_n(1, ones).expect("a count that fits");
    hist.record_n(0, SHOTS - ones).expect("a count that fits");
    ShotEstimate::of_outcome(&hist, 1).expect("a non-empty histogram")
}

/// Whether a prediction agrees with the observation: the gap against `AGREEMENT_SIGMAS` standard
/// errors of the observation, in the decision form.
fn agrees(
    prediction: &ShotEstimate<FloatType>,
    observed: &ShotEstimate<FloatType>,
) -> CheckReport<FloatType> {
    let gap = Real::abs(prediction.estimate() - observed.estimate());
    let allowance = AGREEMENT_SIGMAS * observed.standard_error();
    CheckReport::new(
        vec![Check::new(CheckItem::Whole, gap, allowance)],
        observed.shots() as usize,
    )
}

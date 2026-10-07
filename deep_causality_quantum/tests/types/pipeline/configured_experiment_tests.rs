/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! A configured experiment's predictions, computed by a response model.
//!
//! A mechanism's prediction is the Born read-out of the plant evolved by the candidate's channel
//! and then by the configuration's; a structural candidate's is its model under the response's
//! interventions, evaluated against the response's instrument. The crosstalk values were derived
//! by hand from the diagonal factors (see `crosstalk_predictions_match_the_hand_derivation`).

use deep_causality_num_complex::Complex;
use deep_causality_quantum::utils_tests::{
    CROSSTALK_Q1, CROSSTALK_Q2, CrosstalkModel, CrosstalkSetting, crosstalk_candidates,
};
use deep_causality_quantum::{
    Channel, ConfiguredExperiment, Hypothesis, Observable, QuantumError, QuantumErrorEnum,
    QuantumPlant, QubitOperator, Response, ResponseModel,
};
use deep_causality_tensor::CausalTensor;

type C = Complex<f64>;

fn ket(a: f64, b: f64) -> CausalTensor<C> {
    CausalTensor::from_slice(&[Complex::new(a, 0.0), Complex::new(b, 0.0)], &[2])
}

/// A qubit whose excited population is `p`.
fn plant(p: f64) -> QuantumPlant<f64> {
    QuantumPlant::from_ket(&ket((1.0 - p).sqrt(), p.sqrt())).unwrap()
}

fn excited() -> Observable<f64, 2> {
    Observable::from_ket("excited", &ket(0.0, 1.0)).unwrap()
}

fn mechanism(name: &str, u: QubitOperator<f64>) -> Hypothesis<f64> {
    Hypothesis::mechanism(name, Channel::unitary(&u).unwrap())
}

/// A qubit setting: leave it, flip it, or undo the `flip` candidate's flip.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Setting {
    Passive,
    Flipped,
    Echo,
}

struct QubitModel;

impl ResponseModel<f64, Setting> for QubitModel {
    fn respond(
        &self,
        candidate: &Hypothesis<f64>,
        setting: &Setting,
    ) -> Result<Response<f64>, QuantumError> {
        let u = match setting {
            Setting::Passive => QubitOperator::identity(),
            Setting::Flipped => QubitOperator::pauli_x(),
            Setting::Echo if candidate.name() == "flip" => QubitOperator::pauli_x(),
            Setting::Echo => QubitOperator::identity(),
        };
        Ok(Response::Channel(Channel::unitary(&u)?))
    }
}

fn experiment<Cf>(configuration: Cf, observable: usize) -> ConfiguredExperiment<f64, Cf> {
    ConfiguredExperiment::new("e", 1.0, 256, configuration, observable).unwrap()
}

#[test]
fn test_the_constructor_keeps_its_fields_and_refuses_what_experiment_refuses() {
    let e = ConfiguredExperiment::new("E1 hold", 2.5, 512, Setting::Flipped, 3).unwrap();
    assert_eq!(
        (
            e.name(),
            e.cost(),
            e.shots(),
            *e.configuration(),
            e.observable()
        ),
        ("E1 hold", 2.5, 512, Setting::Flipped, 3)
    );
    for cost in [-1.0, f64::NAN] {
        assert!(matches!(
            ConfiguredExperiment::new("e", cost, 1, Setting::Passive, 0)
                .unwrap_err()
                .0,
            QuantumErrorEnum::NonFiniteValue(_)
        ));
    }
    assert!(matches!(
        ConfiguredExperiment::new("e", 0.0, 0, Setting::Passive, 0)
            .unwrap_err()
            .0,
        QuantumErrorEnum::NormalizationError(_)
    ));
}

#[test]
fn test_a_mechanism_predicts_through_its_channel_then_the_configurations() {
    let p = plant(0.2);
    let observables = [excited()];
    let candidates = [
        mechanism("keep", QubitOperator::identity()),
        mechanism("flip", QubitOperator::pauli_x()),
    ];
    // Under Echo the model undoes `flip` only, so both read the plant's own 0.2.
    let expected = [
        (Setting::Passive, [0.2, 0.8]),
        (Setting::Flipped, [0.8, 0.2]),
        (Setting::Echo, [0.2, 0.2]),
    ];
    for (setting, values) in expected {
        let e = experiment(setting, 0);
        for (h, want) in candidates.iter().zip(values) {
            let predicted = e.predict(&QubitModel, h, &p, &observables).unwrap();
            let Response::Channel(configured) = QubitModel.respond(h, &setting).unwrap() else {
                panic!("a channel");
            };
            let by_hand = excited()
                .read_out(
                    &p.evolve(h.channel().unwrap())
                        .unwrap()
                        .evolve(&configured)
                        .unwrap(),
                )
                .unwrap();
            assert_eq!(predicted, by_hand, "{} under {setting:?}", h.name());
            assert!(
                (predicted - want).abs() < 1e-12,
                "{} under {setting:?}",
                h.name()
            );
        }
    }
}

#[test]
fn test_crosstalk_predictions_equal_evaluate_on_the_intervened_factors() {
    let observables: [Observable<f64, 4>; 0] = [];
    let plant = QuantumPlant::<f64>::default();
    let settings = [
        CrosstalkSetting::Passive { read: CROSSTALK_Q2 },
        CrosstalkSetting::Hold {
            node: CROSSTALK_Q1,
            read: CROSSTALK_Q2,
        },
        CrosstalkSetting::Hold {
            node: CROSSTALK_Q2,
            read: CROSSTALK_Q1,
        },
    ];
    for h in crosstalk_candidates().unwrap() {
        for setting in settings {
            let predicted = experiment(setting, 0)
                .predict(&CrosstalkModel, &h, &plant, &observables)
                .unwrap();
            let Response::Intervention {
                factors,
                instrument,
            } = CrosstalkModel.respond(&h, &setting).unwrap()
            else {
                panic!("an intervention");
            };
            let mut intervened = h.clone();
            for (node, factor) in factors.clone() {
                intervened = intervened.intervene_mechanism(node, factor).unwrap();
            }
            assert_eq!(predicted, intervened.evaluate(&instrument).unwrap());
            if let [(node, factor)] = factors.as_slice() {
                assert_eq!(
                    predicted,
                    h.predict(*node, factor.clone(), &instrument).unwrap()
                );
            }
        }
    }
}

#[test]
fn test_crosstalk_predictions_match_the_hand_derivation() {
    // Holding Q1 excited and reading Q2: H₁ passes Q1's state through Q2's factor (0.05), H₂'s
    // Q2 keeps its own excited population (0.1), and H₃'s bath average gives 0.9·0.05 + 0.1·0.05.
    // Holding Q2 and reading Q1 is the mirror image.
    let plant = QuantumPlant::<f64>::default();
    let hold = |node, read| experiment(CrosstalkSetting::Hold { node, read }, 0);
    let want = [
        (hold(CROSSTALK_Q1, CROSSTALK_Q2), [0.05, 0.1, 0.05]),
        (hold(CROSSTALK_Q2, CROSSTALK_Q1), [0.1, 0.05, 0.05]),
    ];
    for (e, values) in want {
        for (h, v) in crosstalk_candidates().unwrap().iter().zip(values) {
            let p = e.predict::<_, 4>(&CrosstalkModel, h, &plant, &[]).unwrap();
            assert!((p - v).abs() < 1e-12, "{} predicts {p}, want {v}", h.name());
        }
    }
}

#[test]
fn test_a_response_of_the_wrong_kind_is_refused_by_name() {
    let p = plant(0.2);
    let observables = [excited()];
    // A mechanism answered with interventions.
    let flip = mechanism("flip", QubitOperator::pauli_x());
    let e = experiment(CrosstalkSetting::Passive { read: CROSSTALK_Q2 }, 0);
    struct Interventions;
    impl ResponseModel<f64, CrosstalkSetting> for Interventions {
        fn respond(
            &self,
            _: &Hypothesis<f64>,
            _: &CrosstalkSetting,
        ) -> Result<Response<f64>, QuantumError> {
            Ok(Response::Intervention {
                factors: vec![],
                instrument: CausalTensor::from_slice(&[Complex::new(1.0, 0.0)], &[1, 1]),
            })
        }
    }
    match e
        .predict(&Interventions, &flip, &p, &observables)
        .unwrap_err()
        .0
    {
        QuantumErrorEnum::CalculationError(msg) => {
            assert!(msg.contains("mechanism candidate 'flip'"), "{msg}")
        }
        other => panic!("expected CalculationError, got {other:?}"),
    }
    // A structural candidate answered with a channel.
    let [h1, ..] = crosstalk_candidates().unwrap();
    match experiment(Setting::Passive, 0)
        .predict(&QubitModel, &h1, &p, &observables)
        .unwrap_err()
        .0
    {
        QuantumErrorEnum::CalculationError(msg) => {
            assert!(msg.contains("structural candidate 'H1 Q1->Q2'"), "{msg}")
        }
        other => panic!("expected CalculationError, got {other:?}"),
    }
}

#[test]
fn test_an_observable_the_plant_does_not_expose_and_a_model_refusal_propagate() {
    let p = plant(0.2);
    let keep = mechanism("keep", QubitOperator::identity());
    match experiment(Setting::Passive, 1)
        .predict(&QubitModel, &keep, &p, &[excited()])
        .unwrap_err()
        .0
    {
        QuantumErrorEnum::DimensionMismatch(msg) => assert!(msg.contains("observable 1"), "{msg}"),
        other => panic!("expected DimensionMismatch, got {other:?}"),
    }
    // The crosstalk model refuses a mechanism.
    let e = experiment(CrosstalkSetting::Passive { read: CROSSTALK_Q2 }, 0);
    assert!(e.predict(&CrosstalkModel, &keep, &p, &[excited()]).is_err());
}

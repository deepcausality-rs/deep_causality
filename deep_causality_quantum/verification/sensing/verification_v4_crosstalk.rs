/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! V4: crosstalk, attributed among error models from the regenerated simulations of Sarovar et al.
//!
//! Sarovar et al., *Detecting crosstalk errors in quantum information processors*, Quantum 4, 321
//! (2020), arXiv:1908.09855. The paper runs random single-qubit sequences in parallel on every
//! region, tests the settings and results for conditional independence with the PC algorithm, and
//! publishes the crosstalk edges it finds for four simulated error models (§7): a one-way
//! depolarisation, {S0–R1}; a coherent ZZ rotation, {R0–R1, R0–S1, S0–R1}; detection crosstalk,
//! {R0–R1}; and a six-qubit ladder whose bottom row depolarises the top, {R0–S5, R1–S4, R2–S3}.
//!
//! `papers/regenerated/regenerate_sarovar2020.py` regenerates the four with the published
//! parameters and design, adds a crosstalk-free control and a shared bath that flips both
//! read-outs together, and runs pyGSTi's PC detection on each. The verification simulates every candidate
//! model with the crate's channels and attributes each circuit's outcome frequencies among them
//! through QCL. A candidate implies the pairs whose variables its predictions couple directly: two
//! results, when their joint probability differs from the product of their marginals; a setting and
//! another region's result, when changing the setting alone moves the result's probability both
//! given the settings and given the setting's own region's result as well. The second condition
//! leaves out a setting that reaches the result only through its own region's result, as under
//! detection crosstalk, and one whose effect averages out over its own region's result, as under a
//! shared bath; the PC skeleton leaves both out. QCL establishes the pairs every holding candidate
//! implies.
//!
//! The verification checks that the survivor is the generating model and establishes the published
//! edges, that the control establishes none, that QCL separates detection crosstalk from a shared
//! bath though both imply the same pair, that the ZZ data without the ZZ candidate leave no
//! survivor, and that QCL establishes the published edges on the draws where PC does not.
//!
//! Every comparison is judged at five standard errors: with up to 1,800 comparisons a scenario,
//! the chance that the generating model fails one by chance stays below 10⁻³.

#[path = "common/attribution.rs"]
mod attribution;
#[path = "common/data.rs"]
mod data;
#[path = "common/fringe_qubit.rs"]
mod fringe_qubit;
#[path = "common/report.rs"]
mod report;

use attribution::{Attribution, Measurement, attribute, describe};
use data::{load_csv, manifest_dir};
use deep_causality_haft::Either;
use deep_causality_num_complex::Complex;
use deep_causality_quantum::{Ambiguity, Channel};
use deep_causality_tensor::{CausalTensor, Tensor};
use report::Report;
use std::collections::BTreeSet;
use std::error::Error;

/// The agreement, in standard errors, and the floor, in bits.
const SIGMAS: f64 = 5.0;
const FLOOR_BITS: f64 = 5.0;
/// §7: the local depolarisation after every gate, the ladder's after every idle, the crosstalk
/// depolarisation, the ZZ angle, the detection flip; and the shared bath's flip.
const P_LOCAL: f64 = 0.01;
const P_IDLE_LADDER: f64 = 5e-3;
const P_CROSS: f64 = 0.01;
const EPSILON: f64 = 0.02;
const P_M: f64 = 0.01;
const P_BATH: f64 = 0.01;
/// The change in a probability below which a prediction does not depend on a variable, and the
/// circuits of a design that test it.
const RESOLUTION: f64 = 1e-9;
const SAMPLED: usize = 20;

type Matrix = CausalTensor<Complex<f64>>;
type Edges = BTreeSet<String>;
/// A scenario, PC's edges on its written draw, the published edges, and in how many of ten further
/// draws PC finds them.
type PcResult = (String, Edges, Edges, u32);

/// A two-qubit candidate.
#[derive(Clone, Copy, PartialEq)]
enum Pair {
    Free,
    OneWay,
    Zz,
    Detection,
    Bath,
}

impl Pair {
    const ALL: [Pair; 5] = [
        Pair::Free,
        Pair::OneWay,
        Pair::Zz,
        Pair::Detection,
        Pair::Bath,
    ];

    fn name(self) -> &'static str {
        match self {
            Pair::Free => "crosstalk-free",
            Pair::OneWay => "one-way depolarisation",
            Pair::Zz => "coherent ZZ",
            Pair::Detection => "detection crosstalk",
            Pair::Bath => "shared bath",
        }
    }
}

/// A ladder candidate: its name and the pairs whose source's gates depolarise the target.
struct Ladder {
    name: &'static str,
    pairs: Vec<(usize, usize)>,
}

/// A scenario's data: each region's bag, and per circuit the settings and the outcome counts.
struct Scenario {
    bags: Vec<Vec<Vec<u8>>>,
    circuits: Vec<(Vec<usize>, Vec<f64>)>,
}

impl Scenario {
    fn sequences(&self, settings: &[usize]) -> Vec<&[u8]> {
        settings
            .iter()
            .enumerate()
            .map(|(r, &s)| self.bags[r][s].as_slice())
            .collect()
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("V4 Sarovar et al. 2020: crosstalk, attributed among error models");
    let mut report = Report::default();
    let pair_models: Vec<Vec<Vec<Channel<f64>>>> = Pair::ALL
        .iter()
        .map(|&p| layers(p))
        .collect::<Result<_, _>>()?;
    let mut matched = Vec::new();

    for (scenario, generating, published) in [
        ("one_way", Pair::OneWay, vec!["S0-R1"]),
        ("zz", Pair::Zz, vec!["R0-R1", "S0-R1", "S1-R0"]),
        ("detection", Pair::Detection, vec!["R0-R1"]),
    ] {
        let data = load(scenario, 2)?;
        let (result, established) = attribute_pairs(&data, &pair_models, &Pair::ALL)?;
        let published: Edges = published.into_iter().map(String::from).collect();
        let ok = matches!(&result.outcome, Either::Left(s) if s.name == generating.name())
            && established == published;
        matched.push((scenario, established == published));
        report.check(
            &format!(
                "{scenario}: the survivor is the generating model and its pairs are published"
            ),
            ok,
            format!(
                "QCL: {}; establishes {established:?}",
                describe(&result.outcome)
            ),
        );
    }

    let ladder = load("ladder", 6)?;
    let candidates = ladder_candidates();
    let (result, established) = attribute_ladder(&ladder, &candidates)?;
    let published: Edges = ["S3-R2", "S4-R1", "S5-R0"]
        .into_iter()
        .map(String::from)
        .collect();
    matched.push(("ladder", established == published));
    report.check(
        "ladder: the survivor is the generating model and its pairs are published",
        matches!(&result.outcome, Either::Left(s) if s.name == candidates[1].name)
            && established == published,
        format!(
            "QCL: {}; establishes {established:?}",
            describe(&result.outcome)
        ),
    );

    let free = load("free", 2)?;
    let (result, established) = attribute_pairs(&free, &pair_models, &Pair::ALL)?;
    report.check(
        "the crosstalk-free control establishes no pair",
        established.is_empty() && result.holding.iter().any(|h| h == Pair::Free.name()),
        format!(
            "QCL: {}; the models below the control's resolution hold with it",
            describe(&result.outcome)
        ),
    );

    let bath = load("bath", 2)?;
    let (on_bath, bath_pairs) = attribute_pairs(&bath, &pair_models, &Pair::ALL)?;
    let detection = load("detection", 2)?;
    let (on_detection, detection_pairs) = attribute_pairs(&detection, &pair_models, &Pair::ALL)?;
    report.check(
        "QCL separates detection crosstalk from a shared bath, which imply the same pair",
        matches!(&on_bath.outcome, Either::Left(s) if s.name == Pair::Bath.name())
            && !on_detection.holding.iter().any(|h| h == Pair::Bath.name())
            && bath_pairs == detection_pairs,
        format!(
            "on the bath's data, {}; on the detection data the bath holds: {}; both establish \
             {bath_pairs:?}",
            describe(&on_bath.outcome),
            on_detection.holding.iter().any(|h| h == Pair::Bath.name())
        ),
    );

    let zz = load("zz", 2)?;
    let without: Vec<Pair> = Pair::ALL.into_iter().filter(|&p| p != Pair::Zz).collect();
    let models: Vec<Vec<Vec<Channel<f64>>>> = without
        .iter()
        .map(|&p| layers(p))
        .collect::<Result<_, _>>()?;
    let (result, _) = attribute_pairs(&zz, &models, &without)?;
    report.check(
        "without the ZZ candidate, the ZZ data leave no survivor",
        matches!(result.outcome, Either::Right(Ambiguity::NoSurvivor { .. })),
        format!("QCL: {}", describe(&result.outcome)),
    );

    let pc = load_pc()?;
    let missed: Vec<&str> = pc
        .iter()
        .filter(|(_, edges, published, _)| edges != published)
        .map(|(s, ..)| s.as_str())
        .collect();
    report.check(
        "on the draws where PC misses the published edges, QCL establishes them",
        missed
            .iter()
            .all(|s| matched.iter().any(|(m, ok)| m == s && *ok)),
        format!(
            "PC on these draws misses {missed:?}; over ten further draws it finds the published \
             set in {}",
            pc.iter()
                .map(|(s, _, _, k)| format!("{s} {k}"))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    );

    report.finish("V4").map_err(Into::into)
}

/// The attribution of a two-qubit scenario among `candidates`, simulated by `models`, and the pairs
/// every holding candidate implies.
fn attribute_pairs(
    data: &Scenario,
    models: &[Vec<Vec<Channel<f64>>>],
    candidates: &[Pair],
) -> Result<(Attribution, Edges), Box<dyn Error>> {
    let predict = |k: usize, settings: &[usize]| -> Result<[f64; 4], Box<dyn Error>> {
        two_qubit(candidates[k], &models[k], &data.sequences(settings))
    };
    let mut measurements = Vec::new();
    for (i, (settings, counts)) in data.circuits.iter().enumerate() {
        let predictions = (0..candidates.len())
            .map(|k| predict(k, settings))
            .collect::<Result<Vec<_>, _>>()?;
        let total: f64 = counts.iter().sum();
        for outcome in 1..4 {
            measurements.push(frequency(
                format!("circuit {i}, outcome {outcome:02b}"),
                counts[outcome],
                total,
                predictions.iter().map(|p| p[outcome]).collect(),
            ));
        }
    }
    let names: Vec<&str> = candidates.iter().map(|c| c.name()).collect();
    let result = attribute(&names, &measurements, SIGMAS, FLOOR_BITS)?;
    let mut implied = Vec::new();
    for (k, c) in candidates.iter().enumerate() {
        if result.holding.iter().any(|h| h == c.name()) {
            implied.push(implied_pairs(data, 2, |settings| {
                Ok(predict(k, settings)?.to_vec())
            })?);
        }
    }
    Ok((result, intersection(implied)))
}

/// The attribution of the ladder among `candidates`, on each qubit's marginal, and the pairs every
/// holding candidate implies.
fn attribute_ladder(
    data: &Scenario,
    candidates: &[Ladder],
) -> Result<(Attribution, Edges), Box<dyn Error>> {
    let channels = ladder_channels()?;
    let predict = |k: usize, settings: &[usize]| -> Result<Vec<f64>, Box<dyn Error>> {
        ladder_marginals(&candidates[k], &channels, &data.sequences(settings))
    };
    let mut measurements = Vec::new();
    for (i, (settings, counts)) in data.circuits.iter().enumerate() {
        let predictions = (0..candidates.len())
            .map(|k| predict(k, settings))
            .collect::<Result<Vec<_>, _>>()?;
        let total: f64 = counts.iter().sum();
        for q in 0..6 {
            let ones: f64 = counts
                .iter()
                .enumerate()
                .filter(|(o, _)| (o >> (5 - q)) & 1 == 1)
                .map(|(_, n)| n)
                .sum();
            measurements.push(frequency(
                format!("circuit {i}, qubit {q}"),
                ones,
                total,
                predictions.iter().map(|p| p[q]).collect(),
            ));
        }
    }
    let names: Vec<&str> = candidates.iter().map(|c| c.name).collect();
    let result = attribute(&names, &measurements, SIGMAS, FLOOR_BITS)?;
    let mut implied = Vec::new();
    for (k, c) in candidates.iter().enumerate() {
        if result.holding.iter().any(|h| h == c.name) {
            implied.push(implied_pairs(data, 6, |settings| {
                let marginals = predict(k, settings)?;
                Ok((0..64)
                    .map(|o| {
                        (0..6)
                            .map(|q| {
                                if bit(o, q, 6) {
                                    marginals[q]
                                } else {
                                    1.0 - marginals[q]
                                }
                            })
                            .product()
                    })
                    .collect())
            })?);
        }
    }
    Ok((result, intersection(implied)))
}

/// A frequency of `count` in `total` shots, with its binomial standard error, at least one shot's.
fn frequency(name: String, count: f64, total: f64, predictions: Vec<f64>) -> Measurement {
    let f = count / total;
    Measurement {
        name,
        value: f,
        standard_error: (f * (1.0 - f) / total).sqrt().max(1.0 / total),
        predictions,
    }
}

/// The pairs a candidate's predictions couple directly over the first [`SAMPLED`] circuits of a
/// design. `predict` returns the joint distribution of the results, region 0 the most significant
/// bit.
fn implied_pairs<F>(data: &Scenario, regions: usize, predict: F) -> Result<Edges, Box<dyn Error>>
where
    F: Fn(&[usize]) -> Result<Vec<f64>, Box<dyn Error>>,
{
    let mut edges = Edges::new();
    let mut marginal = BTreeSet::new();
    let mut conditional = BTreeSet::new();
    for (settings, _) in data.circuits.iter().take(SAMPLED) {
        let base = predict(settings)?;
        for i in 0..regions {
            for j in (i + 1)..regions {
                if (both(&base, i, j, regions) - one(&base, i, regions) * one(&base, j, regions))
                    .abs()
                    > RESOLUTION
                {
                    edges.insert(format!("R{i}-R{j}"));
                }
            }
        }
        for source in 0..regions {
            for alternative in 0..data.bags[source].len() {
                let mut changed = settings.clone();
                changed[source] = alternative;
                let moved = predict(&changed)?;
                for target in (0..regions).filter(|&t| t != source) {
                    if (one(&moved, target, regions) - one(&base, target, regions)).abs()
                        > RESOLUTION
                    {
                        marginal.insert((source, target));
                    }
                    let given = |joint: &[f64], r: bool| {
                        let p_source = if r {
                            one(joint, source, regions)
                        } else {
                            1.0 - one(joint, source, regions)
                        };
                        let p_both = if r {
                            both(joint, source, target, regions)
                        } else {
                            one(joint, target, regions) - both(joint, source, target, regions)
                        };
                        (p_source > 1e-6).then(|| p_both / p_source)
                    };
                    for r in [false, true] {
                        if let (Some(a), Some(b)) = (given(&base, r), given(&moved, r))
                            && (a - b).abs() > RESOLUTION
                        {
                            conditional.insert((source, target));
                        }
                    }
                }
            }
        }
    }
    edges.extend(
        marginal
            .intersection(&conditional)
            .map(|(s, t)| format!("S{s}-R{t}")),
    );
    Ok(edges)
}

/// Whether outcome `o` reads 1 on region `q` of `regions`.
fn bit(o: usize, q: usize, regions: usize) -> bool {
    (o >> (regions - 1 - q)) & 1 == 1
}

/// The probability that region `q` reads 1.
fn one(joint: &[f64], q: usize, regions: usize) -> f64 {
    joint
        .iter()
        .enumerate()
        .filter(|(o, _)| bit(*o, q, regions))
        .map(|(_, p)| p)
        .sum()
}

/// The probability that regions `i` and `j` both read 1.
fn both(joint: &[f64], i: usize, j: usize, regions: usize) -> f64 {
    joint
        .iter()
        .enumerate()
        .filter(|(o, _)| bit(*o, i, regions) && bit(*o, j, regions))
        .map(|(_, p)| p)
        .sum()
}

fn intersection(sets: Vec<Edges>) -> Edges {
    sets.into_iter()
        .reduce(|a, b| a.intersection(&b).cloned().collect())
        .unwrap_or_default()
}

/// The outcome distribution over 00, 01, 10 and 11, qubit 0 the more significant bit.
fn two_qubit(
    pair: Pair,
    layers: &[Vec<Channel<f64>>],
    sequences: &[&[u8]],
) -> Result<[f64; 4], Box<dyn Error>> {
    let zero = Complex::new(0.0, 0.0);
    let mut entries = vec![zero; 16];
    entries[0] = Complex::new(1.0, 0.0);
    let mut rho = CausalTensor::from_slice(&entries, &[4, 4]);
    for (g0, g1) in sequences[0].iter().zip(sequences[1]) {
        rho = layers[index(*g0)][index(*g1)].apply(&rho)?;
    }
    let d = rho.as_slice();
    let p = [d[0].re, d[5].re, d[10].re, d[15].re];
    Ok(match pair {
        Pair::Detection => [
            p[0],
            p[1],
            (1.0 - P_M) * p[2] + P_M * p[3],
            (1.0 - P_M) * p[3] + P_M * p[2],
        ],
        Pair::Bath => [0, 1, 2, 3].map(|o| (1.0 - P_BATH) * p[o] + P_BATH * p[3 - o]),
        _ => p,
    })
}

/// Each qubit's probability of reading 1 under a ladder candidate: the models keep the state a
/// product, so each qubit evolves alone.
fn ladder_marginals(
    candidate: &Ladder,
    channels: &[[Channel<f64>; 2]; 3],
    sequences: &[&[u8]],
) -> Result<Vec<f64>, Box<dyn Error>> {
    (0..6)
        .map(|q| {
            let mut rho = CausalTensor::from_slice(
                &[
                    Complex::new(1.0, 0.0),
                    Complex::new(0.0, 0.0),
                    Complex::new(0.0, 0.0),
                    Complex::new(0.0, 0.0),
                ],
                &[2, 2],
            );
            for t in 0..sequences[q].len() {
                let crossed = candidate
                    .pairs
                    .iter()
                    .any(|&(s, target)| target == q && sequences[s][t] != b'I');
                rho = channels[index(sequences[q][t])][usize::from(crossed)].apply(&rho)?;
            }
            Ok(rho.as_slice()[3].re)
        })
        .collect()
}

fn ladder_candidates() -> Vec<Ladder> {
    let down = vec![(3, 2), (4, 1), (5, 0)];
    let up: Vec<(usize, usize)> = down.iter().map(|&(s, t)| (t, s)).collect();
    let rows = vec![
        (0, 1),
        (1, 0),
        (1, 2),
        (2, 1),
        (5, 4),
        (4, 5),
        (4, 3),
        (3, 4),
    ];
    vec![
        Ladder {
            name: "crosstalk-free",
            pairs: vec![],
        },
        Ladder {
            name: "vertical, bottom to top",
            pairs: down.clone(),
        },
        Ladder {
            name: "vertical, top to bottom",
            pairs: up.clone(),
        },
        Ladder {
            name: "vertical, both ways",
            pairs: down.into_iter().chain(up).collect(),
        },
        Ladder {
            name: "horizontal neighbours",
            pairs: rows,
        },
    ]
}

/// The ladder's single-qubit layer channels, by gate and by whether crosstalk depolarises the
/// qubit: the gate, its local depolarisation, then the crosstalk's.
fn ladder_channels() -> Result<[[Channel<f64>; 2]; 3], Box<dyn Error>> {
    let channel = |g: u8, crossed: bool| -> Result<Channel<f64>, Box<dyn Error>> {
        let local = if g == b'I' { P_IDLE_LADDER } else { P_LOCAL };
        let mut kraus = depolarising(local)
            .iter()
            .map(|k| k.matmul(&gate(g)))
            .collect::<Result<Vec<_>, _>>()?;
        if crossed {
            kraus = then(&kraus, &depolarising(P_CROSS))?;
        }
        Ok(Channel::from_kraus(&kraus)?)
    };
    Ok([
        [channel(b'X', false)?, channel(b'X', true)?],
        [channel(b'Y', false)?, channel(b'Y', true)?],
        [channel(b'I', false)?, channel(b'I', true)?],
    ])
}

/// A two-qubit candidate's layer channels, indexed by the gates on qubits 0 and 1: the layer's
/// unitary, each qubit's local depolarisation, and the candidate's crosstalk.
fn layers(pair: Pair) -> Result<Vec<Vec<Channel<f64>>>, Box<dyn Error>> {
    let local = depolarising(P_LOCAL);
    let both: Vec<Matrix> = local
        .iter()
        .flat_map(|a| local.iter().map(move |b| a.kronecker(b)))
        .collect::<Result<_, _>>()?;
    let id = identity();
    let cross: Vec<Matrix> = depolarising(P_CROSS)
        .iter()
        .map(|c| id.kronecker(c))
        .collect::<Result<_, _>>()?;
    b"XYI"
        .iter()
        .map(|&g0| {
            b"XYI"
                .iter()
                .map(|&g1| {
                    let u = if pair == Pair::Zz && g0 == b'X' {
                        id.kronecker(&gate(g1))?.matmul(&zz_rotation()?)?
                    } else {
                        gate(g0).kronecker(&gate(g1))?
                    };
                    let mut kraus = both
                        .iter()
                        .map(|k| k.matmul(&u))
                        .collect::<Result<Vec<_>, _>>()?;
                    if pair == Pair::OneWay && g0 == b'X' {
                        kraus = then(&kraus, &cross)?;
                    }
                    Ok(Channel::from_kraus(&kraus)?)
                })
                .collect()
        })
        .collect()
}

/// The Kraus family of `first` followed by `second`.
fn then(first: &[Matrix], second: &[Matrix]) -> Result<Vec<Matrix>, Box<dyn Error>> {
    Ok(second
        .iter()
        .flat_map(|b| first.iter().map(move |a| b.matmul(a)))
        .collect::<Result<_, _>>()?)
}

/// Eq. (14): exp(−i/2 [π/2 X⊗I + ε/2 Z⊗Z]). X⊗I and Z⊗Z anticommute and square to the identity,
/// so the exponential is cos θ I − i sin θ (a X⊗I + b Z⊗Z)/θ with a = π/4, b = ε/4, θ² = a² + b².
fn zz_rotation() -> Result<Matrix, Box<dyn Error>> {
    let (a, b) = (std::f64::consts::FRAC_PI_4, EPSILON / 4.0);
    let theta = a.hypot(b);
    let [i, x, _, z] = paulis();
    let xi = x.kronecker(&i)?;
    let zz = z.kronecker(&z)?;
    let ii = i.kronecker(&i)?;
    let s = theta.sin() / theta;
    Ok(&(&ii * Complex::new(theta.cos(), 0.0))
        + &(&(&(&xi * Complex::new(a, 0.0)) + &(&zz * Complex::new(b, 0.0)))
            * Complex::new(0.0, -s)))
}

/// The Kraus family of (1 − p) ρ + p I/2.
fn depolarising(p: f64) -> [Matrix; 4] {
    let [i, x, y, z] = paulis();
    let (keep, flip) = ((1.0 - 0.75 * p).sqrt(), (0.25 * p).sqrt());
    [
        &i * Complex::new(keep, 0.0),
        &x * Complex::new(flip, 0.0),
        &y * Complex::new(flip, 0.0),
        &z * Complex::new(flip, 0.0),
    ]
}

/// X_π/2, Y_π/2 and the idle.
fn gate(g: u8) -> Matrix {
    let [i, x, y, _] = paulis();
    let h = std::f64::consts::FRAC_1_SQRT_2;
    match g {
        b'X' => &(&i * Complex::new(h, 0.0)) + &(&x * Complex::new(0.0, -h)),
        b'Y' => &(&i * Complex::new(h, 0.0)) + &(&y * Complex::new(0.0, -h)),
        _ => i,
    }
}

fn identity() -> Matrix {
    paulis()[0].clone()
}

fn paulis() -> [Matrix; 4] {
    let c = |re: f64, im: f64| Complex::new(re, im);
    let m = |e: [Complex<f64>; 4]| CausalTensor::from_slice(&e, &[2, 2]);
    [
        m([c(1.0, 0.0), c(0.0, 0.0), c(0.0, 0.0), c(1.0, 0.0)]),
        m([c(0.0, 0.0), c(1.0, 0.0), c(1.0, 0.0), c(0.0, 0.0)]),
        m([c(0.0, 0.0), c(0.0, -1.0), c(0.0, 1.0), c(0.0, 0.0)]),
        m([c(1.0, 0.0), c(0.0, 0.0), c(0.0, 0.0), c(-1.0, 0.0)]),
    ]
}

fn index(g: u8) -> usize {
    match g {
        b'X' => 0,
        b'Y' => 1,
        _ => 2,
    }
}

/// A scenario's bags and circuits from `papers/regenerated`.
fn load(name: &str, regions: usize) -> Result<Scenario, Box<dyn Error>> {
    let dir = manifest_dir().join("papers/regenerated");
    let mut bags = vec![Vec::new(); regions];
    for line in std::fs::read_to_string(dir.join(format!("sarovar2020_{name}_bags.csv")))?
        .lines()
        .skip(1)
    {
        let cells: Vec<&str> = line.split(',').collect();
        bags[cells[0].parse::<usize>()?].push(cells[2].as_bytes().to_vec());
    }
    let circuits = load_csv(&dir.join(format!("sarovar2020_{name}.csv")))?
        .into_iter()
        .map(|row| {
            (
                row[..regions].iter().map(|&s| s as usize).collect(),
                row[regions..].to_vec(),
            )
        })
        .collect();
    Ok(Scenario { bags, circuits })
}

/// pyGSTi's PC result per scenario: the edges on the written draw, with S before R, the published
/// set, and in how many of ten further draws PC finds it.
fn load_pc() -> Result<Vec<PcResult>, Box<dyn Error>> {
    let path = manifest_dir().join("papers/regenerated/sarovar2020_edges.csv");
    let edges = |cell: &str| -> Edges {
        cell.split_whitespace()
            .map(|e| {
                let (a, b) = e.split_once('-').unwrap_or((e, ""));
                if a.starts_with('R') && b.starts_with('S') {
                    format!("{b}-{a}")
                } else {
                    e.to_string()
                }
            })
            .collect()
    };
    std::fs::read_to_string(path)?
        .lines()
        .skip(1)
        .map(|line| {
            let cells: Vec<&str> = line.split(',').collect();
            Ok((
                cells[0].to_string(),
                edges(cells[1]),
                edges(cells[2]),
                cells[3].parse()?,
            ))
        })
        .collect()
}

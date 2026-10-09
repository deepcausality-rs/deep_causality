# Raw information: the sensing page

Step 0 of `docs/writing_guides/TurnRawMaterialintoWriting.pdf`. This file holds facts and their sources
and no prose for the reader. Every fact carries the letter of its source. `04_sensing_page_draft.md`
turns these facts into the page.

Read on 2026-10-08: every file of the two sensing examples, `qcl_gravimeter_systematics` and
`qcl_gradiometer_crosstalk`; `deep_causality_quantum/src/types/{pipeline,design,instrument,decision}`;
the eight verifications and their README; the first pages and the quoted passages of the
papers named below. Both examples and all eight verifications were run in release mode; each example was
run twice and printed identical output.

## Sources

| Letter | File or run |
|---|---|
| K | `examples/quantum_examples/qcl_examples/qcl_gravimeter_systematics/{main,model,model_config,constants}.rs`, `README.md` |
| K-run | `cargo run --release -p quantum_examples --example qcl_gravimeter_systematics`, 2026-10-08 |
| L | `examples/quantum_examples/qcl_examples/qcl_gradiometer_crosstalk/{main,model,model_config,constants}.rs`, `README.md` |
| L-run | `cargo run --release -p quantum_examples --example qcl_gradiometer_crosstalk`, 2026-10-08 |
| M | `deep_causality_quantum/verification/sensing/*.rs`, `README.md` |
| M-run | `for v in v1_wavefront v2_clipping v3_transverse_motion v4_crosstalk v5_light_shift v6_time_model v7_gradiometer_sweep v10_zeeman_field_map; do cargo run --release -p deep_causality_quantum --features qcm --example verification_$v; done`, 2026-10-08 |
| P | `deep_causality_quantum/src/types/{pipeline/campaign.rs,design/experiment_design.rs,qpu/shot_estimate.rs,instrument/interferometer_model.rs}` |
| N1 | Everitt et al., arXiv:2608.25563 (vessel trial) |
| N2 | Lellouch and Holynski, arXiv:2504.11119 (gradiometer model at sea) |
| N3 | Karcher et al., arXiv:1804.04909 |
| N4 | Pagot et al., arXiv:2410.07720 |
| N5 | Louchet-Chauvet et al., New J. Phys. 13 065025 (2011) |
| Q | derived by hand, named where used |

## Why a sensing lab names causes, from N1 to N5

- An atom gravimeter measures gravity through the phase of free-falling atoms, Φ = k·g T². (N3)
- "The dominant limit in the accuracy of cold atom gravimeters is due to the wavefront distortions of the lasers beamsplitters." (N3)
- Coriolis and the two-photon light shift "are the next largest contributors to the inaccuracy budget". (N4)
- In one published budget the wavefront row is 0.0 ± 4.0 µGal of a total uncertainty of 5.2 µGal. (N5, Table 1)
- The same paper's mean tidal correction over twelve days is −18.8 µGal. (N5, Table 1)
- A strapdown quantum gravimeter on a 29 m vessel corrected the ship's inertial solution over a 45 nmi (83 km) route in about 6 h, with GNSS excluded from the whole chain. The aided position ended 2.2 nmi from the truth, the unaided inertial solution about 14 nmi. (N1; the paper prints the endpoint as both 4.0 and 4.1 km and the duration once as a 5 h window, so the page quotes nautical miles)
- The quantum sensor is hybridized with a classical accelerometer for atom-referenced bias stabilization. (N1)
- A model of an atom gradiometer coupled to a map-matching filter traces its errors at sea to the platform's rotation rates and rotational accelerations, and derives a tilt requirement below 3.3°. (N2)

## What the library does, from P

- `design` picks the minimum-cost set of experiments that separates every pair of candidates by at least a floor in bits, and lists pairs no experiment reaches. (P)
- Separation in bits is the Bhattacharyya distance of the two predicted read-outs over n draws, n·(−log₂(√(pq) + √((1−p)(1−q)))); it adds up over draws. (P `shot_estimate.rs`)
- Under equal priors, the chance of confusing two candidates is at most ½·2^(−bits), so a 5-bit floor keeps it at or below 1/64. (Q: the Bhattacharyya bound, T. Kailath, IEEE Trans. Commun. Technol. 15, 52, 1967; the crate does not state it)
- Priced in time, an experiment costs its setup time plus the draws it needs times the time per draw; `design` sizes each experiment's draws to the fewest that reach the floor, within the instrument's white-noise range. (P `experiment_design.rs`, `interferometer_model.rs`)
- A campaign runs the cheapest planned experiment that can separate a candidate still holding, compares every candidate's prediction with the observation, and stops at the first candidate that holds and separates by the floor, or when no candidate holds. (P `campaign.rs`)
- When the context moves a planned prediction by more than a set drift, the campaign plans again. (P `campaign.rs`)

## The gravimeter run, from K and K-run

- The instrument is Louchet-Chauvet et al.'s gravimeter: T = 70 ms, 22.1 µGal/√Hz in one configuration, white noise to 5000 s. (K, K-run)
- After the tide correction the gravimeter reads −5 µGal against a trusted reference. (K)
- Seven candidates: H1 Coriolis, H2 quadratic Zeeman, H3 tilt, H4 mirror vibration, H5 wavefront, H6 two-photon light shift, H7 detection clipping. (K)
- Each candidate is calibrated to produce the −5 µGal offset in the passive configuration; the baseline prints "every candidate can produce the passive reading". (K, K-run)
- What each configuration does, as `model.rs` writes it: a 180° turn flips Coriolis and clipping; reversing the wave vector flips quadratic Zeeman; the accelerometer correction removes mirror vibration; a 1.5 mrad tilt moves the tilt candidate; halving the coil current moves Zeeman and a third of the light shift; extrapolating to 0 K removes the wavefront; halving the Rabi frequency halves the light shift; displacing the cloud 1 mm moves clipping by 14.2 µGal. (K `model.rs`)
- Setup times in s: E1 30, E2 600, E3 0, E4 900, E5 60, E6 20,000, E7 60, E8 300. All are placeholders except E6, derived from Karcher et al.'s ultracold operation. (K `constants.rs`)
- The plan: E1, E2, E3, E4, E7, E8 for 4,956 s; no planned experiment moves the wavefront, so the plan identifies it by elimination. (K-run)
- Campaigns, one per cause: H1 1,815 s (E1, E8, E3, E2), H2 166 s (E1), H3 2,716 s (five experiments), H4 1,079 s (three), H5 4,953 s (six), H6 4,953 s (six), H7 534 s (E1, E8). Each names the cause the observations came from. (K-run)
- Each experiment's context holds the Earth tide at the time it runs; re-plans per campaign: 3, 0, 4, 2, 5, 5, 1. The tide is a placeholder, 80 µGal semidiurnal. (K, K-run)
- A +5 µGal offset from the light shift: the baseline refuses tilt, which predicts 76.43 µGal with the tide against a reading of 82.54 µGal, before any plan. (K-run)
- Contexts that record no tide: every candidate predicts −5.00 µGal against a reading of 72.41 µGal, and the baseline refuses all seven. (K-run)
- Clipping as the cause and left off the list: the plan drops the cloud displacement, and the campaign names Coriolis after E1, E3, E2 and 1,447 s. (K-run)
- Every observation is drawn from the named cause. (K `main.rs`)

## The gradiometer run, from L and L-run

- The instrument is Sorrentino et al.'s gradiometer: two clouds 30 cm apart, T = 160 ms, contrast 0.45. (L)
- Four candidates: H1 light leaks from cloud A into B's detection signal, H2 the reverse, H3 the shared laser phase alone, H4 platform rotation. Each fits the passive read-out exactly. (L, L-run)
- E1 brightens A and darkens B: H1 predicts 0.0788, the others 0.0200; it takes 340 draws and 60 s. (L-run)
- The plan is E1, E2 and a rotation step for 721.8 s. (L-run)
- The campaign spends 60 s for a leak from A to B, 120 s from B to A, 721.9 s for the shared phase and for rotation. (L-run)
- A 6 % leak biases the gradient by −0.044 rad, −355.7 E; rotation at Earth's horizontal rate by +6.9 × 10⁻⁴ rad, +5.5 E. (L-run)
- A leak both ways: the full plan reports it outside the model; the campaign stops at H1 after E1. (L-run)
- The leak, the dark background and the setup times are placeholders. (L `constants.rs`)

## The verification, from M and M-run

- Eight verifications, 53 checks, all pass. (M-run)
- Karcher 2018: the temperature scan attributes to the wavefront; Table I's extrapolations reproduce within one standard error. The printed bound on atom interactions is one standard error; at three, interactions stay possible. Two of the paper's figures disagree with each other. (M)
- Farah 2014: Coriolis and the wavefront cannot explain +46 µGal; clipping can. The text uses two conversion factors and prints kHz/mm for Hz/mm. (M)
- Louchet-Chauvet 2011: only the 180° turn separates Coriolis from the wavefront; a printed uncertainty total and one equation disagree with their inputs. (M)
- Sarovar 2020: QCL names the generating crosstalk model in all four published scenarios; the paper's own detection method, rerun on the same data, misses the published answer in three, and finds it in 7 to 9 of ten further datasets. (M)
- Gauguet 2008: the shift is the two-photon light shift, 32 against 33 mrad; repeat measurements spread ±16 % against a stated ±10 %. (M)
- Ménoret 2018: a white-noise model predicts the 10-minute and 1-hour scatter; the 1-day scatter, 9.4 against 2.55 nm/s², lies beyond white noise. (M)
- Sorrentino 2014: three slopes attribute as the paper says; the text's list of dominant drifts leaves out the third largest. (M)
- Hu 2017: the digitised field map gives 2.010 µGal against the printed 2.04; one coefficient's unit is printed wrong. (M)

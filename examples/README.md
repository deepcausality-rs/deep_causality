# DeepCausality Examples Overview

This directory contains examples demonstrating various features and applications of the DeepCausality library. Each example showcases how to model and reason about causal relationships using the Effect Propagation Process (EPP) and PropagatingEffect monads.

## Example Categories

| Category | Description                                                         |
|----------|---------------------------------------------------------------------|
| [Starter Example](#starter-example) | Basic introduction to DeepCausality                                 |
| [Classical Causality](#classical-causality-examples) | Traditional causal inference methods (CATE, DBN, Granger, RCM, SCM) |
| [Causal Discovery](#causal-discovery-examples) | SURD decomposition, mRMR feature selection, and the CDL pipeline    |
| [Causal Uncertain](#causal-uncertain-examples) | Uncertain<T> / MaybeUncertain<T> as monadic propagation chains      |
| [Causal Counterfactual](#causal-counterfactual-examples) | Pearl-style one-shot `do`-operator interventions (estimand = difference between worlds) |
| [Causal Correction](#causal-correction-examples) | Closed-loop corrective `alternate_value` (monitor a trajectory; snap it back inside the safe envelope) |
| [CSM Examples](#csm-examples) | Causal State Machine patterns                                       |
| [Core Examples](#core-examples) | PropagatingEffect and PropagatingProcess fundamentals               |
| [Avionics Examples](#avionics-examples) | High-assurance GNC and Safety Critical Systems                      |
| [Chronometric Examples](#chronometric-examples) | Chronometric geodesy from satellite clock data                      |
| [Mathematics Examples](#mathematics-examples) | Multi-mathematics composition (HKT, Causal Monad)                   |
| [Physics Examples](#physics-examples) | Multi-physics simulations with Geometric Algebra                    |
| [Quantum Examples](#quantum-examples) | Quantum computing, quantum geometry, topological matter, electroweak loops, quantum gravity |
| [Medicine Examples](#medicine-examples) | Biomedical and life sciences applications                           |
| [Material Examples](#material-examples) | Material Science and Metamaterials                                  |
| [Tokio Example](#tokio-example) | Async integration with tokio runtime                                |

---

## Starter Example

**Location:** `examples/starter_example`

Basic introduction to DeepCausality. **Start here if you are new.**

| Example | Focus | Command |
|---------|-------|---------|
| Starter | CausaloidGraph basics | `cargo run -p starter_example --example starter_example` |

---  

## Classical Causality Examples

**Location:** `examples/classical_causality_examples`

Traditional causal inference methods implemented using the DeepCausality framework. Each method is implemented **twice**: once with `Causaloid` + Contextual Alternation, once with `PropagatingProcess` + the `Alternatable` family. 

| Example | Method | `via_causaloid` command | `via_monad` command |
|---------|--------|-------------------------|---------------------|
| CATE | Conditional Average Treatment Effect | `cargo run -p classical_causality_examples --example cate_via_causaloid` | `cargo run -p classical_causality_examples --example cate_via_monad` |
| DBN | Dynamic Bayesian Network | `cargo run -p classical_causality_examples --example dbn_via_causaloid` | `cargo run -p classical_causality_examples --example dbn_via_monad` |
| Granger | Granger Causality Test | `cargo run -p classical_causality_examples --example granger_via_causaloid` | `cargo run -p classical_causality_examples --example granger_via_monad` |
| RCM | Rubin Causal Model | `cargo run -p classical_causality_examples --example rcm_via_causaloid` | `cargo run -p classical_causality_examples --example rcm_via_monad` |
| SCM | Pearl's Ladder of Causation | `cargo run -p classical_causality_examples --example scm_via_causaloid` | `cargo run -p classical_causality_examples --example scm_via_monad` |

See [classical_causality_examples/README.md](classical_causality_examples/README.md) for the full side-by-side comparison.

---

## Causal Discovery Examples

**Location:** `examples/causal_discovery_examples`

Runnable examples for the `deep_causality_algorithms` and
`deep_causality_discovery` crates: information-theoretic causal decomposition
(SURD), feature selection (mRMR), and the full Causal Discovery Language (CDL)
pipeline.

| Example      | Method                                                | Command |
|--------------|-------------------------------------------------------|---------|
| SURD         | SURD-states causal decomposition                      | `cargo run -p causal_discovery_examples --example example_surd` |
| mRMR         | Minimum-Redundancy Maximum-Relevance feature selection | `cargo run -p causal_discovery_examples --example example_mrmr` |
| mRMR (CDL)   | mRMR with missing-value cleaning                      | `cargo run -p causal_discovery_examples --example example_mrmr_cdl` |
| CDL (SURD)   | Load -> clean -> mRMR -> SURD -> analyze              | `cargo run -p causal_discovery_examples --example example_surd_discovery` |
| CDL (BRCD)   | Root-cause ranking with a supplied CPDAG (real Sock Shop data) | `cargo run -p causal_discovery_examples --example example_brcd_discovery` |
| CDL (BRCD/BOSS) | Root-cause ranking, CPDAG learned via BOSS         | `cargo run -p causal_discovery_examples --example example_brcd_boss_discovery` |
| ML × Causal RCA | Candle anomaly detector gates the BRCD root-cause explainer via a `PropagatingProcess` (ML detects, causality explains) | `cargo run -p causal_discovery_examples --example example_ml_rca` |

See [causal_discovery_examples/README.md](causal_discovery_examples/README.md) for detailed documentation.

---

## Causal Uncertain Examples

**Location:** `examples/causal_uncertain_examples`

Runnable examples for the `deep_causality_uncertain` crate, restructured so
each example is a daisy-chained monadic pipeline. The `Uncertain<f64>` and
`MaybeUncertain<f64>` API does the numerical work; the surrounding monad
supplies the chain's plumbing and short-circuit on failure.

| Example           | Monad                                          | Topic                                                                                          | Command                                                              |
|-------------------|------------------------------------------------|------------------------------------------------------------------------------------------------|----------------------------------------------------------------------|
| GPS Navigation    | PropagatingEffect (stateless)                  | Position noise → distance → travel time → route choice → fuel                                  | `cargo run -p causal_uncertain_examples --example gps_navigation`    |
| Sensor Processing | PropagatingProcess<_, FleetState, FleetConfig> | Six-stage fleet pipeline: triage → validate → fuse → anomaly → fallback → reliability verdict  | `cargo run -p causal_uncertain_examples --example sensor_processing` |
| Clinical Trial    | PropagatingEffect over MaybeUncertain<f64>     | Five-stage aspirin trial: cohort → presence → lift → aggregate → verdict                       | `cargo run -p causal_uncertain_examples --example clinical_trial`    |

See [causal_uncertain_examples/README.md](causal_uncertain_examples/README.md) for detailed documentation.

---

## Causal Counterfactual Examples

**Location:** `examples/causal_counterfactual_examples`

Five worked examples in the Judea Pearl tradition: a one-shot value
substitution evaluates the chain on the factual world and one or more
counterfactual worlds; the difference between worlds is the estimand.

| Example | Monad | Topic | Command |
|---|---|---|---|
| counterfactual_treatment_effect   | PropagatingEffect             | CATE as `do(T=1) − do(T=0)` on a single chain                                       | `cargo run -p causal_counterfactual_examples --example counterfactual_treatment_effect` |
| counterfactual_envelope_fault     | PropagatingProcess (stateful) | Mid-chain stall-airspeed injection; same aircraft, same configuration, new value    | `cargo run -p causal_counterfactual_examples --example counterfactual_envelope_fault` |
| counterfactual_treatment_options  | PropagatingEffect             | Two intervention sites on one chain: beta-blocker vs surgical clip                  | `cargo run -p causal_counterfactual_examples --example counterfactual_treatment_options` |
| counterfactual_cascade_failure    | PropagatingProcess (stateful) | Network N-k contingency analysis as a chain of composing interventions              | `cargo run -p causal_counterfactual_examples --example counterfactual_cascade_failure` |
| counterfactual_resection_intervention | PropagatingEffect         | Epilepsy surgery screening as `do(connectome = resected_at_R)` for each region      | `cargo run -p causal_counterfactual_examples --example counterfactual_resection_intervention` |

See [causal_counterfactual_examples/README.md](causal_counterfactual_examples/README.md) for detailed documentation.

---

## Causal Correction Examples

**Location:** `examples/causal_correction_examples`

Five worked examples in the control-theory tradition: monitor a trajectory
tick by tick; when the value drifts outside the safe envelope, `alternate_value`
snaps it back and the chain continues from the corrected state. The first
four run the same chain twice: open loop (no monitor, catastrophic failure)
and closed loop (monitor + `alternate_value`, failure averted). The fifth,
`corrective_ddos_detector`, runs closed loop only. Its point is the stateful
sliding-window detector that drives the intervention.

| Example | Monad | Topic | Command |
|---|---|---|---|
| corrective_lane_keeping           | PropagatingProcess (stateful) | Vehicle drifts under crosswind; P-controller fires every time offset crosses 0.30 m | `cargo run -p causal_correction_examples --example corrective_lane_keeping` |
| corrective_glucose_pump           | PropagatingProcess (stateful) | Glucose climbs across three meals; corrective bolus fires above 180 mg/dL          | `cargo run -p causal_correction_examples --example corrective_glucose_pump` |
| corrective_decompression_stops    | PropagatingProcess (stateful) | Diver ascends from 30 m; decompression stop inserted when N2 supersaturation rises | `cargo run -p causal_correction_examples --example corrective_decompression_stops` |
| corrective_network_failover       | PropagatingProcess (stateful) | Primary switch fails; monitor detects zero delivery; standby switch takes over     | `cargo run -p causal_correction_examples --example corrective_network_failover` |
| corrective_ddos_detector          | PropagatingProcess (stateful) | Volumetric DDoS; sliding-window z-score detector in `State`; 5 consecutive 3σ seconds engage the NIC throttle | `cargo run -p causal_correction_examples --example corrective_ddos_detector` |

See [causal_correction_examples/README.md](causal_correction_examples/README.md) for detailed documentation.

---

## CSM Examples

**Location:** `examples/csm_examples`

Causal State Machine patterns for stateful causal reasoning.

| Example | Pattern | Command |
|---------|---------|---------|
| CSM Basic | Simple monitoring system | `cargo run -p csm_examples --example csm_example` |
| CSM Context | Shared mutable state via `Arc<RwLock>` | `cargo run -p csm_examples --example csm_context_example` |
| CSM Effect Ethos | Deontic reasoning integration | `cargo run -p csm_examples --example csm_effect_ethos_example` |

See [csm_examples/README.md](csm_examples/README.md) for detailed documentation.

---

## Core Examples

**Location:** `examples/core_examples`

Fundamental examples demonstrating the monadic API.

| Example | Focus | Command |
|---------|-------|---------|
| PropagatingEffect | Basic monadic composition | `cargo run -p core_examples --example propagating_effect_example` |
| PropagatingEffect Counterfactual | Counterfactual reasoning | `cargo run -p core_examples --example propagating_effect_counterfactual_example` |
| PropagatingProcess | Stateful composition | `cargo run -p core_examples --example propagating_process_example` |
| PropagatingProcess Counterfactual | Stateful counterfactuals | `cargo run -p core_examples --example propagating_process_counterfactual` |
---

## Avionics Examples

**Location:** `examples/avionics_examples`

High-assurance examples for Aerospace, Defense, and Safety Critical systems.

| Example | Domain | Description | Command |
|---------|--------|-------------|---------|
| magnav | Navigation | Magnetic Navigation using Causal Particle Filters (Bayesian estimation) | `cargo run -p avionics_examples --example magnav` |
| geometric_tcas | Collision Avoidance | NextGen TCAS using Geometric Algebra collision detection and `AlternatableValue` safety interlocks | `cargo run -p avionics_examples --example geometric_tcas` |
| hypersonic_2t | Defense/Tracking | Tracking Hypersonic Glide Vehicles (HGV) using Dual-Time (2T) Physics in 6D phase space | `cargo run -p avionics_examples --example hypersonic_2t` |
| flight_envelope_monitor | Health Monitoring | Three-stage stateful pipeline (sensor collection → bind chain → envelope hypergraph) demonstrating uniform composition through `PropagatingProcess<_, FlightState, AircraftConfig>` | `cargo run -p avionics_examples --example flight_envelope_monitor` |
| ins_gnss_blackout | Navigation / Timing | INS clock holdover through a GPS-denial blackout (jamming / urban canyon / tunnel) on **real Galileo** data: a grmhd-style regime detector + the `alternate_value` corrective loop (withheld through the dark) + a carried relativistic clock kernel, in one auditable `CausalFlow` — the navigation/timing core of any GPS-denied flight | `cargo run -p avionics_examples --example ins_gnss_blackout` |

### CFD Examples

**Location:** `examples/avionics_examples/cfd`

Computational fluid dynamics driving real engineering decisions, each ending in a self-verifying
gate set that exits nonzero on regression. Run these with `--release`.

| Example | Domain | Description | Command |
|---------|--------|-------------|---------|
| plasma_blackout_corridor | Multiphysics / GNC | One continuous Mach-25 descent through plasma blackout: a tensor-train compressed compressible carrier with a shock-fitted Rankine–Hugoniot inflow strip, evolved-state Park-2T ionization gated against the RAM-C II flight anchor, flow-resolved GNSS denial driving a 17-state ESKF, O(1) counterfactual bank branches, and a cybernetic envelope gate whose clamped command actually steers the 3-DOF lift | `cargo run --release -p avionics_examples --example plasma_blackout_corridor` |
| plasma_blackout_weather | Digital Twin / Dispersion | The table factory for the corridor: six weather conditions as counterfactual worlds alternated from one validated baseline, flown concurrently, reduced to a dispersion table tracking navigation precision against weather | `cargo run --release -p avionics_examples --example plasma_blackout_weather` |
| plasma_blackout_retropulsion | Multiphysics / GNC | Closes the family loop: consumes the weather table **in flight**, commits an ignition inside the Jarvinen–Adams band, and forks the marched plume-coupled state mid-burn — a *state* fork a parameter sweep cannot express — then coasts to the ignition altitude and lands at 2.0 m/s | `cargo run --release -p avionics_examples --example plasma_blackout_retropulsion` |
| flight_envelope_placard | Certification / Placards | A Mach-altitude matrix in, one placard table out: US-1976 freestream, dynamic pressure, exact Rankine–Hugoniot stagnation temperature, Sutton–Graves heating, every point gated and any out-of-envelope point named | `cargo run --release -p avionics_examples --example flight_envelope_placard` |
| nozzle_operating_map | Propulsion | A back-pressure sweep over a converging–diverging duct: shock position and thrust across the operating regimes, gated against gas-dynamics closed forms | `cargo run --release -p avionics_examples --example nozzle_operating_map` |
| viv_resonance_margin | Structures / Aeroelasticity | Vortex-induced-vibration margin as a computed study: one validated cylinder-wake case marched per airspeed, shedding frequency extracted from each wake probe, margin to the structural mode tabled | `cargo run --release -p avionics_examples --example viv_resonance_margin` |
| turbulence_flow | Turbulence / Chaos | Forecast horizon of a chaotic convective flow (Lorenz / Rayleigh–Bénard truncation); the same `Rk4` march at f32/f64/Float106 shows precision setting how far ahead a turbulent flow can be trusted | `cargo run -p avionics_examples --example turbulence_flow` |

The three plasma-blackout examples are one story, not three variants — see
[cfd/plasma_blackout/README.md](avionics_examples/cfd/plasma_blackout/README.md) for the family
overview.

See [avionics_examples/README.md](avionics_examples/README.md) for detailed documentation.

> The CFD/MMS verification examples (Taylor–Green MMS, Re-1600 DEC solver, lid-driven cavity, graded-MMS, cylinder) have moved into the `deep_causality_cfd` crate as self-verifying examples under `deep_causality_cfd/verification/`; run them with `cargo run -p deep_causality_cfd --example <name>_verification`.

---

## Chronometric Examples

**Location:** `examples/chronometric_examples`

Chronometric geodesy demonstrations using the J2-corrected weak-field 1PN kernel from `deep_causality_physics::chronometric`. Recovers gravitational parameters from satellite clock time-dilation data, with `CausalMonad` bind-chain composition end-to-end.

| Example | Domain | Description | Command |
|---------|--------|-------------|---------|
| gm_recovery | Geodesy | Recovers Earth's geocentric gravitational constant ($GM_\oplus$) and derived planetary mass ($M_\oplus = GM_\oplus / G$) from one full GPS week of Galileo broadcast clock and SP3 orbit data (satellite E14). Validates against published JGM-3 / IERS 2010 references at ~0.2 % relative error | `cargo run -p chronometric_examples --example gm_recovery` |

See [chronometric_examples/README.md](chronometric_examples/README.md) for detailed documentation.

---

## Mathematics Examples

**Location:** `examples/mathematics_examples`

Consolidated examples for all four major DeepCausality mathematics crates
(`deep_causality_multivector`, `deep_causality_sparse`, `deep_causality_tensor`,
`deep_causality_topology`), plus the cross-crate composition examples that show
how they fit together through HKT (`Functor`, `Monad`, `CoMonad`) and the causal
effect monad.

### Subfolders

| Subfolder | Crate | Description |
|-----------|-------|-------------|
| [algebra](mathematics_examples/algebra/README.md) | `deep_causality_multivector` | Clifford and geometric-algebra examples (basic, PGA3D, Dixon, Maxwell, GRMHD, plus the `algebraic_scanner` study of complex structure) |
| [sparse](mathematics_examples/sparse/README.md) | `deep_causality_sparse` | CSR-format sparse matrix ops and the HKT functor view |
| [tensor](mathematics_examples/tensor/README.md) | `deep_causality_tensor` | `CausalTensor` construction, `EinSumOp`, Einstein-field index gymnastics, HKT (Functor, Applicative) |
| [topology](mathematics_examples/topology/README.md) | `deep_causality_topology` | Graphs, simplicial and cubical complexes, manifolds, differential forms, lattice gauge fields |
| [composable_multi_math](mathematics_examples/composable_multi_math/README.md) | cross-crate | HKT and causal-monad composition across two or three of the above crates |
| [isomorphism](mathematics_examples/isomorphism/README.md) | cross-crate | `iso` bridges from `deep_causality_num::iso` / `deep_causality_haft::iso` (tensor <-> sparse, multifield <-> tuple carrier) |

### Highlights

| Example | Crate | Description | Command |
|---------|-------|-------------|---------|
| algebraic_scanner | multivector | Scans Clifford algebras `Cl(p, q, r)` for complex structure (`I² = -1`) | `cargo run -p mathematics_examples --example algebraic_scanner_examples` |
| maxwell_multivector | multivector | Unifies electric and magnetic fields into a single electromagnetic-field bivector | `cargo run -p mathematics_examples --example maxwell_multivector_examples` |
| pga3d_multivector | multivector | Projective Geometric Algebra (PGA) for rigid-body motions in graphics and robotics | `cargo run -p mathematics_examples --example pga3d_multivector_examples` |
| basic_csr_ops | sparse | Constructing a `CsrMatrix` from triplets; row/column iteration | `cargo run -p mathematics_examples --example basic_csr_ops_examples` |
| ein_sum_causal_tensor | tensor | Einstein-summation contractions via `EinSumOp` | `cargo run -p mathematics_examples --example ein_sum_causal_tensor_examples` |
| einstein_field_causal_tensor | tensor | Index raising and lowering with the metric; Ricci-style contractions | `cargo run -p mathematics_examples --example einstein_field_causal_tensor_examples` |
| manifold_analysis | topology | Constructing a `Manifold<SimplicialComplex<R>, F>`; Euler characteristic; orientation | `cargo run -p mathematics_examples --example manifold_analysis_examples` |
| cubical_heat_diffusion | topology | Explicit-Euler heat diffusion on a cubical manifold with a Moore-neighborhood stencil | `cargo run -p mathematics_examples --example cubical_heat_diffusion_examples` |
| lattice_gauge_simulation | topology | SU(3) lattice gauge theory: Metropolis thermalization, plaquette, Wilson loop, Polyakov loop, APE smearing, Wilson flow | `cargo run -p mathematics_examples --example lattice_gauge_simulation_examples` |
| tensor_x_topology_laplacian | composition | Discrete Laplacian on a 1D simplicial manifold via `ManifoldWitness::extend` (CoMonad) | `cargo run -p mathematics_examples --example tensor_x_topology_laplacian_examples` |
| triple_hkt_stress_field | composition | 3D linear-elastic stress on a tetrahedral mesh: strain, Hooke, normal, Cauchy traction, material rotor, von Mises in one `extend` call | `cargo run -p mathematics_examples --example triple_hkt_stress_field_examples` |
| effect_diffusion_on_manifold | composition | Heat equation: spatial Laplacian via `extend`, time stepping via `bind`, stability short-circuit on CFL violation | `cargo run -p mathematics_examples --example effect_diffusion_on_manifold_examples` |
| capstone_spinor_minkowski | composition (capstone) | Parallel transport of a unit timelike spinor along a discretized Minkowski worldline in `Cl(3,1)`. Final drift versus closed-form `(cosh θ, sinh θ)` is ~1.7e-31 at `Float106`, fifteen orders of magnitude tighter than f64 | `cargo run -p mathematics_examples --example capstone_spinor_minkowski_examples` |
| tensor_sparse_memory_budget | isomorphism | Dense `CausalTensor` <-> `CsrMatrix` via the `tensor-iso` feature: sparsify, run a sparse-only op, materialise back to dense | `cargo run -p mathematics_examples --example tensor_sparse_memory_budget` |
| multifield_data_pipeline | isomorphism | `CausalMultiField<T>` <-> `(CausalTensor<T>, Metric, dx, shape)` iso lets external code build/extract/transform a multifield without touching `pub(crate)` internals | `cargo run -p mathematics_examples --example multifield_data_pipeline` |

See [mathematics_examples/README.md](mathematics_examples/README.md) for the full
table of all 35 registered examples and the precision-abstraction decision tree
(`f32` vs `f64` vs `Float106`).

---

## Physics Examples

**Location:** `examples/physics_examples`

Multi-physics simulations using Geometric Algebra, Tensor operations, and Topology.

| Example | Domain | Command |
|---------|--------|---------|
| Bernoulli Flow Network | Fluid Dynamics | `cargo run -p physics_examples --example bernoulli_flow_network` |
| Carnot Cycle Engine | Thermodynamics | `cargo run -p physics_examples --example carnot_cycle_engine` |
| Laser Resonator Stability | Optics | `cargo run -p physics_examples --example laser_resonator_stability` |
| Maxwell's Unification | Electromagnetism | `cargo run -p physics_examples --example maxwell_example` |
| GRMHD | Relativity | `cargo run -p physics_examples --example grmhd_example` |
| Geometric Tilt | Robotics/IMU | `cargo run -p physics_examples --example geometric_tilt` |
| Algebraic Scanner | Abstract Algebra | `cargo run -p physics_examples --example algebraic_scanner` |
| Multi-Physics Pipeline | Particle Physics | `cargo run -p physics_examples --example multi_physics_pipeline` |
| Gravitational Wave | Relativity | `cargo run -p physics_examples --example gravitational_wave` |
| Event Horizon Probe | Relativity | `cargo run -p physics_examples --example event_horizon_probe` |
| Gauge EM | Electromagnetism | `cargo run -p physics_examples --example gauge_em` |
| Gauge GR | General Relativity | `cargo run -p physics_examples --example gauge_gr` |
| Gauge Weak Force | Weak Force | `cargo run -p physics_examples --example gauge_weak_force` |
| Gauge Lattice U(1) 2D | Lattice Gauge | `cargo run -p physics_examples --example gauge_lattice_u1_2d` |

See [physics_examples/README.md](physics_examples/README.md) for detailed documentation.

---

## Quantum Examples

**Location:** `examples/quantum_examples`

Examples whose subject matter is directly quantum, consolidated here from
the physics, material, and mathematics example crates: quantum computing,
quantum geometry of electronic bands, topological quantum matter,
electroweak loop corrections, the Hopf/Bloch-sphere structure of a qubit
state, and a quantum-gravity matrix model.

| Example | Field | Command |
|---------|-------|---------|
| Quantum Counterfactual | Quantum Computing | `cargo run -p quantum_examples --example quantum_counterfactual` |
| Quantum Geometric Tensor | Condensed Matter | `cargo run -p quantum_examples --example quantum_geometric_tensor` |
| Gauge Electroweak | Quantum Field Theory | `cargo run -p quantum_examples --example gauge_electroweak` |
| Topological Insulator | Quantum Materials | `cargo run -p quantum_examples --example topological_insulator` |
| Hopf Fibration Multivector | Quantum State Geometry | `cargo run -p quantum_examples --example hopf_fibration_multivector` |
| IKKT Matrix Model | Quantum Gravity | `cargo run -p quantum_examples --example ikkt_matrix_model` |

See [quantum_examples/README.md](quantum_examples/README.md) for detailed documentation.

---

## Medicine Examples

**Location:** `examples/medicine_examples`

Biomedical and life sciences applications using causal monads.

| Example | Domain | Command |
|---------|--------|---------|
| Protein Folding | Biophysics | `cargo run -p medicine_examples --example protein_folding` |
| MRI Tissue Classification | Medical Imaging | `cargo run -p medicine_examples --example tissue_classification` |
| Aneurysm Risk (Hemodynamics) | Cardiovascular | `cargo run -p medicine_examples --example aneurysm_risk` |
| Diving Decompression | Hyperbaric Medicine | `cargo run -p medicine_examples --example diving_decompression` |
| Epilepsy Virtual Resection | Neurology | `cargo run -p medicine_examples --example epilepsy` |
| Tumor Treatment (TTFields) | Oncology | `cargo run -p medicine_examples --example tumor_treatment` |

See [medicine_examples/README.md](medicine_examples/README.md) for detailed documentation.

---

## Material Examples

**Location:** `examples/material_examples`

Material Science and Metamaterial simulations using topology, multivectors, and causal interventions.

| Example | Domain | Command |
|---------|--------|---------|
| Hyperlens | Metamaterials | `cargo run -p material_examples --example hyperlens_example` |
| Structural Health Monitor | Smart Materials | `cargo run -p material_examples --example structural_health_monitor_example` |

See [material_examples/README.md](material_examples/README.md) for detailed documentation.

---

## Tokio Example

**Location:** `examples/tokio_example`

Asynchronous integration with the tokio runtime.

| Example | Focus | Command |
|---------|-------|---------|
| Tokio | Async causal inference | `cargo run -p tokio_example --example tokio_example` |

---

## License

All examples are licensed under the [MIT license](LICENSE).
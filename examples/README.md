# DeepCausality Examples Overview

This directory contains examples demonstrating various features and applications of the DeepCausality library. Each example showcases how to model and reason about causal relationships using the Effect Propagation Process (EPP) and PropagatingEffect monads.

## Example Categories

| Category | Description                                                         |
|----------|---------------------------------------------------------------------|
| [Starter Example](#starter-example) | Basic introduction to DeepCausality                                 |
| [Classical Causality](#classical-causality-examples) | Traditional causal inference methods (CATE, DBN, Granger, RCM, SCM) |
| [CSM Examples](#csm-examples) | Causal State Machine patterns                                       |
| [Core Examples](#core-examples) | PropagatingEffect and PropagatingProcess fundamentals               |
| [Avionics Examples](#avionics-examples) | High-assurance GNC and Safety Critical Systems                      |
| [Chronometric Examples](#chronometric-examples) | Chronometric geodesy from satellite clock data                      |
| [Mathematics Examples](#mathematics-examples) | Multi-mathematics composition (HKT, Causal Monad)                   |
| [Physics Examples](#physics-examples) | Multi-physics simulations with Geometric Algebra                    |
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

Traditional causal inference methods implemented using the DeepCausality framework.

| Example | Method | Command |
|---------|--------|---------|
| CATE | Conditional Average Treatment Effect | `cargo run -p classical_causality_examples --example cate_example` |
| DBN | Dynamic Bayesian Network | `cargo run -p classical_causality_examples --example dbn_example` |
| Granger | Granger Causality Test | `cargo run -p classical_causality_examples --example granger_example` |
| RCM | Rubin Causal Model | `cargo run -p classical_causality_examples --example rcm_example` |
| SCM | Pearl's Ladder of Causation | `cargo run -p classical_causality_examples --example scm_example` |

See [classical_causality_examples/README.md](classical_causality_examples/README.md) for detailed documentation.

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
| Control Flow Builder | Builder patterns | `cargo run -p core_examples --example control_flow_builder` |
| Control Flow Strict ZST | Zero-sized type control | `cargo run -p core_examples --example control_flow_strict_zst` |

---

## Avionics Examples

**Location:** `examples/avionics_examples`

High-assurance examples for Aerospace, Defense, and Safety Critical systems.

| Example | Domain | Description | Command |
|---------|--------|-------------|---------|
| [magnav](magnav/README.md) | Navigation | Magnetic Navigation using Causal Particle Filters (Bayesian estimation) | `cargo run -p avionics_examples --example magnav` |
| [geometric_tcas](geometric_tcas/README.md) | Collision Avoidance | NextGen TCAS using Geometric Algebra collision detection and `Intervenable` safety interlocks | `cargo run -p avionics_examples --example geometric_tcas` |
| [hypersonic_2t](hypersonic_2t/README.md) | Defense/Tracking | Tracking Hypersonic Glide Vehicles (HGV) using Dual-Time (2T) Physics in 6D phase space | `cargo run -p avionics_examples --example hypersonic_2t` |
| [flight_envelope_monitor](flight_envelope_monitor/README.md) | Health Monitoring | Three-stage stateful pipeline (sensor collection → bind chain → envelope hypergraph) demonstrating uniform composition through `PropagatingProcess<_, FlightState, AircraftConfig>` | `cargo run -p avionics_examples --example flight_envelope_monitor` |

See [avionics_examples/README.md](avionics_examples/README.md) for detailed documentation.

---

## Chronometric Examples

**Location:** `examples/chronometric_examples`

Chronometric geodesy demonstrations using the J2-corrected weak-field 1PN kernel from `deep_causality_physics::chronometric`. Recovers gravitational parameters from satellite clock time-dilation data, with `CausalMonad` bind-chain composition end-to-end.

| Example | Domain | Description | Command |
|---------|--------|-------------|---------|
| [gm_recovery](chronometric_examples/README.md) | Geodesy | Recovers Earth's geocentric gravitational constant ($GM_\oplus$) and derived planetary mass ($M_\oplus = GM_\oplus / G$) from one full GPS week of Galileo broadcast clock and SP3 orbit data (satellite E14). Validates against published JGM-3 / IERS 2010 references at ~0.2 % relative error | `cargo run -p chronometric_examples --example gm_recovery` |

See [chronometric_examples/README.md](chronometric_examples/README.md) for detailed documentation.

---

## Mathematics Examples

**Location:** `examples/mathematics_examples`

Multi-mathematics composition through Higher-Kinded Types (`Functor`, `Monad`, `CoMonad`) and the `CausalEffectPropagationProcess` monad. Each example demonstrates how tensor, geometric algebra (multivector), topology (simplicial manifolds), and the causal effect system compose through one uniform API. 

### Standalone

| Example | Domain | Command |
|---------|--------|---------|
| [algebraic_scanner](mathematics_examples/algebraic_scanner/README.md) | Abstract Algebra | `cargo run -p mathematics_examples --example algebraic_scanner` |

### HKT-Only Composition (Functor, Monad, CoMonad)

These examples compose two or three crates through witness traits, with no effect machinery.

| Example | Composes              | Description | Command |
|---------|-----------------------|-------------|---------|
| [tensor_x_algebra_rotation_field](mathematics_examples/tensor_x_algebra_rotation_field/README.md) | tensor x multivector  | Rotates a grid of vectors by a single Clifford rotor via `Functor::fmap` on a tensor of multivectors | `cargo run -p mathematics_examples --example tensor_x_algebra_rotation_field` |
| [tensor_x_topology_laplacian](mathematics_examples/tensor_x_topology_laplacian/README.md) | tensor x topology     | Discrete Laplacian on a 1D simplicial manifold via `ManifoldWitness::extend` (CoMonad) | `cargo run -p mathematics_examples --example tensor_x_topology_laplacian` |
| [triple_hkt_stress_field](mathematics_examples/triple_hkt_stress_field/README.md) | tensor x multivector x topology | 3D linear-elastic stress analysis blueprint on a tetrahedral mesh; six-step pipeline (strain, Hooke, normal, Cauchy traction, material rotor, von Mises) in one `extend` call. Documented placeholders show where to plug in a real material model, mesh source, or failure criterion | `cargo run -p mathematics_examples --example triple_hkt_stress_field` |

### Causal Monad Composition (CausalEffectPropagationProcess)

These examples wrap operations in the causal monad. Each step is a `bind` with logging and short-circuit error propagation.

| Example | Composes | Description | Command |
|---------|----------|-------------|---------|
| [effect_kalman_predict_correct](mathematics_examples/effect_kalman_predict_correct/README.md) | tensor + multivector + core | Predict / correct / verify skeleton (the structural shape of a Kalman filter); tensor matrix-multiply for predict, Clifford rotor for correct, NaN gate for verify. README enumerates the eight pieces a production filter adds | `cargo run -p mathematics_examples --example effect_kalman_predict_correct` |
| [effect_diffusion_on_manifold](mathematics_examples/effect_diffusion_on_manifold/README.md) | topology + tensor + core | Heat equation on a 1D manifold: spatial Laplacian via `extend` (CoMonad), time stepping via `bind` (Monad), with stability short-circuit on CFL violation | `cargo run -p mathematics_examples --example effect_diffusion_on_manifold` |
| [effect_tensor_algebra_roundtrip](mathematics_examples/effect_tensor_algebra_roundtrip/README.md) | tensor + multivector + core | Lift a 3-vector into `Cl(3,0)`, rotate, lower back, verify norm preservation by tensor dot product. Carried value type changes between `bind` calls; the monad threads them | `cargo run -p mathematics_examples --example effect_tensor_algebra_roundtrip` |

### Capstone

| Example | Composes | Description | Command |
|---------|----------|-------------|---------|
| [capstone_spinor_minkowski](mathematics_examples/capstone_spinor_minkowski/README.md) | all three + core | Parallel transport of a unit timelike spinor along a discretized Minkowski worldline in `Cl(3,1)`. Topology supplies the path, tensor stores per-edge rapidities, multivector builds the boost rotors, the causal monad orders the steps. Final drift versus the closed-form `cosh(theta), sinh(theta)` is ~1.7e-31 at `Float106`, fifteen orders of magnitude tighter than f64 | `cargo run -p mathematics_examples --example capstone_spinor_minkowski` |

See [mathematics_examples/README.md](mathematics_examples/README.md) for detailed documentation, including the precision-abstraction decision tree (`f32` vs `f64` vs `Float106`).

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
| Quantum Counterfactual | Quantum | `cargo run -p physics_examples --example quantum_counterfactual` |
| Quantum Geometric Tensor | Condensed Matter | `cargo run -p physics_examples --example quantum_geometric_tensor` |
| IKKT Matrix Model | Quantum Gravity | `cargo run -p physics_examples --example ikkt_matrix_model` |
| Gravitational Wave | Relativity | `cargo run -p physics_examples --example gravitational_wave` |
| Event Horizon Probe | Relativity | `cargo run -p physics_examples --example event_horizon_probe` |
| Gauge EM | Electromagnetism | `cargo run -p physics_examples --example gauge_em` |
| Gauge GR | General Relativity | `cargo run -p physics_examples --example gauge_gr` |
| Gauge Electroweak | Electroweak | `cargo run -p physics_examples --example gauge_electroweak` |
| Gauge Weak Force | Weak Force | `cargo run -p physics_examples --example gauge_weak_force` |
| Gauge Lattice U(1) 2D | Lattice Gauge | `cargo run -p physics_examples --example gauge_lattice_u1_2d` |

See [physics_examples/README.md](physics_examples/README.md) for detailed documentation.

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
| Topological Insulator | Quantum Materials | `cargo run -p material_examples --example topological_insulator_example` |
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
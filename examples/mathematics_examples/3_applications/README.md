# Applications: one use case per example

Every example here takes a problem someone actually has and solves it end to end, in the least
code that shows the whole shape. `1_foundation/` teaches the vocabulary and `2_composition/`
teaches the mechanisms; these put both to work on a use case and print an answer.

Each example is a folder with a `main.rs`. Run any of them from the repository root:

```bash
cargo run -p mathematics_examples --example <example_name>
```

| Folder | Domain | What it does | Command |
|---|---|---|---|
| [differentiate_under_integral](differentiate_under_integral/) | sensitivity analysis | One quadrature sweep over `Dual` returns a definite integral together with its derivative in a parameter, which is the Leibniz rule obtained from the tangent functor | `cargo run -p mathematics_examples --example differentiate_under_integral_examples` |
| [electromagnetic_field](electromagnetic_field/) | antenna design | Derives the electromagnetic bivector `F = ∇A` from the 4-vector potential by one geometric product, then reads the Lorenz gauge, `E` and `B` out of its blades | `cargo run -p mathematics_examples --example electromagnetic_field_examples` |
| [imu_tilt_estimation](imu_tilt_estimation/) | robotics, attitude | A geometric tilt estimator with an adaptive gravity observer: a gyro rotor prediction, a Kalman update on body-frame gravity with motion detection and adaptive noise, and a blended correction rotor | `cargo run -p mathematics_examples --example imu_tilt_estimation_examples` |
| [lattice_gauge_thermalization](lattice_gauge_thermalization/) | lattice QCD | An SU(3) gauge field on a `4⁴` lattice: hot start, Metropolis thermalization, plaquette, Wilson loop, Polyakov loop, APE smearing and Wilson gradient flow | `cargo run -p mathematics_examples --example lattice_gauge_thermalization_examples` |
| [relativistic_mhd](relativistic_mhd/) | plasma astrophysics | A tensor GR solver hands its local curvature to a multivector MHD solver, and that value selects the Clifford metric the plasma forces are computed in | `cargo run -p mathematics_examples --example relativistic_mhd_examples` |
| [relativistic_spinor_transport](relativistic_spinor_transport/) | special relativity | Parallel transport of a unit timelike spinor along a discretized Minkowski worldline in `Cl(3,1)`, with a stability invariant checked at every step. Drift against the closed form is ~1.7e-31 at `Float106` | `cargo run -p mathematics_examples --example relativistic_spinor_transport_examples` |
| [standard_model_symmetry](standard_model_symmetry/) | particle physics | Basis vectors, anticommutation and complex scalar multiplication in the Dixon algebra `Cl_C(6)`, the setting Standard Model constructions from octonions use | `cargo run -p mathematics_examples --example standard_model_symmetry_examples` |
| [structural_stress_on_mesh](structural_stress_on_mesh/) | structural FEA | A six-step linear-elastic pipeline over a tetrahedral mesh — strain, Hooke, normal, Cauchy traction, material rotor, von Mises — inside one `extend` call | `cargo run -p mathematics_examples --example structural_stress_on_mesh_examples` |

## The house rules

Every example here follows the same four:

1. **Precision is a parameter.** A `FloatType` alias sits directly above `main`, and every
   quantity in the program carries it. Changing that one line reruns the whole example at
   another scalar.
2. **Values cross the precision boundary through `deep_causality_num::lift`.** Configuration
   literals are written as `f64`, the widest form a source file holds, and lifted once.
3. **Printing lives in helper functions below `main`.** They hold the only `lower` calls, so
   `f64` appears at the display boundary and nowhere else.
4. **Errors travel through `?` and return from `main`.** Where a fixed-signature closure leaves
   no error channel, `expect` states the invariant that makes the call total.

The file order is the same everywhere: imports, constants, the `FloatType` alias, `main`, the
functions `main` calls, custom structs, then the printing helpers.

## What each example composes

| Example | Crates it crosses |
|---|---|
| differentiate_under_integral | calculus × num_dual × algebra |
| electromagnetic_field | multivector × calculus |
| imu_tilt_estimation | multivector × tensor × core |
| lattice_gauge_thermalization | topology × num_complex × rand |
| relativistic_mhd | tensor × multivector × haft |
| relativistic_spinor_transport | topology × tensor × multivector × linear × core |
| standard_model_symmetry | multivector × num_complex |
| structural_stress_on_mesh | topology × tensor × multivector × linear |

Two examples carry a longer write-up of their own:
[relativistic_spinor_transport](relativistic_spinor_transport/README.md) covers the algebra of
boost rotors, and [structural_stress_on_mesh](structural_stress_on_mesh/README.md) covers the
path from the blueprint bodies to a production FEA inner loop.

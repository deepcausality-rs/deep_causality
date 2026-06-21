Consolidate all CFD relevant code currently scatterered across te physics crate and the example 
folder into a dedicated CFD crate called deep_causality_cfd.

Plan new crate: deep_causality_cfd

Important: Set the deep_causality_cfd crate to publish=false in its Cargo.toml

Create from the line of no external dependencies, which means for file operations and adjacent areas, such STD format
you as can pull in external dependencies.

the proposed code structure in the new crate:

benches/ (benchmarks for solvers)
docs/

- prompts (carbon copy of all Ai agent prompts inputs)
- openspecs (carbon copy of the open specs implemented)
  examples/(the actual code examples)
  src/
- errors (the usual enum embeeded in struct error pattern)
- extensions (type extensions, if any)
- traits (if any)
- types (all relevent types)
- solvers (the actual CFD solvers)
- theories (the NS physics theory)
  tests/ - mirror the src tree
  validation/ - the "example" folder used to validate the CFD solves

The central requirments are:

- precision as a parameter so that any solver / theory can run natively at any supported float precisom level. That
  means zero downcasting to f64 or other shortcuts in the solver.
- Composable solvers. Consider the existing HKT mechanis for first class compostion,
  See the existing crates for the HKT & algebra foundation
- deep_causality_haft
- deep_causality_num

for benches, you may have to migrate existing benchmarks from the physics crate into the new benches/ folder in the new
crate.

for the validation, migrate the folowing code examples from the
avionics examples in examples/avionics_examples into the validation/ folder  
and registrer them as examples

[[example]]
name = "cfd_taylor_green"
path = "cfd_taylor_green/main.rs"

[[example]]
name = "dec_taylor_green_re1600"
path = "dec_taylor_green_re1600/main.rs"

[[example]]
name = "dec_lid_cavity_re1000"
path = "dec_lid_cavity_re1000/main.rs"

[[example]]
name = "dec_graded_mms"
path = "dec_graded_mms/main.rs"

[[example]]
name = "dec_cylinder_wake"
path = "dec_cylinder_wake/main.rs"

[[example]]
name = "dec_cylinder_validation"
path = "dec_cylinder_validation/main.rs"

For the theories and solvers, migrate all fluid dynamics theories from the pjhyisc crate

deep_causality_physics/src/theories/fluid_dynamics

the distinction betwen a theory and a solver is as following

A theory refers to a certain navier strokes regime that is used in multiple
solvers. A solver, however, uses a theory and or multiple physics kernels,
but only solves a designated use case.

The solvers may need some proper design work.
the solvers needs to be generic of a real field to fully implement precision as a parameter end to end.
That means you may have to refactor some of the avionics example codes into a standalone solver.
Make sure that the solver configuration is encapsulated into a dedicated and solver specific configuration struct.

Design a configuration builder that for each solver builds a correct configuration in a typical build up pattern.
Take inspiration from the existing configuration builder in the Discovery DSL.
deep_causality_discovery/src


The goal is, eventually, to increase expressiveness and reduce code lines by
building a highly powerful CFD Domain Specific Language (DSL) similar
to the existing CausalFlow or Discovery DSL. 

CausalFLow DSL:
deep_causality_core/src/types/causal_flow

CausalDisovery DSL:
deep_causality_discovery/src

Lets name the new CFL DSL "Flow"

From there, the idea od the new Flow DSL is to enable

* Composoable solvers
* Composoable inflow and outflow zones
* Multi-physics, similar to the CausalFlow. See examples/physics_examples/multi_physics_pipeline
* Counterfactuals e.g interve to change material, mesh, temprature etc. See examples/causal_counterfactual_examples
* Control flow e.g. loop and either, similar to the corrective exzmples. See examples/causal_correction_examples
* The actual examples of the deep_causality_cfd are written in the new Flow DSL

The new Flow DSL integrartes, or wraps, with the existing CausalFLow DSL to process complex physics in a
dediated workflow e.g. during pre-processing or between steps and the CausalDisovery DSL e.g. when to isoalate
contributimg factors via SURD.

The new Flow DSL needs to be easy to extend to add new solvers.
Again, consider the HKT mechanism to streamline maintance.

Order of next steps

1) Design the new Flow DSL from the ground up
2) Design the all CFD solvers to interface cleanly with the Flow DSL
3) Design the verification with the new Flow DSL
4) Design the code examples with the new Flow DSL

Then, use the newly designed Flow DSL to showcase how common problems
in Fluid Dynamics could be sovled with the new DSL. Consider these as real-examples
to be added to the new crate after the code migration, implementation and Flow DSL has been implemented.

Ensure that each example uses a float type alias "FloatType" for precision as a parameter.

/// Switch this alias to `f32` for low precision, `f64` for standard precision, or `Float106` for high precision.
pub type FloatType = Float106;

Then show me the code examples for review. This may result in some refinement and iteration

Once the design and plan is approved, derive the full specification using the OSPX skills for implementing the
deep_causality_cfd crate, the Flow DSL, the solver migration and veriification, as well as the code examples via the
Flow DSL.

Important: Set the deep_causality_cfd crate to publish=false in its Cargo.toml 

# Decentralised Structural Health Monitoring

A micrometeoroid strikes one plate of a pressurised hull. Whether the section survives depends on
what the plate does in the next fraction of a second, and on how the hull is bonded together.

It is a demonstration rather than a solver. The point is that once the essence of the problem is
written as a causal process over the library's types, precision as a parameter and categorical
composition come for free: the same source runs at four scalars, a redistribution step is one
`extend`, and the counterfactual is the same cascade run from an intervened value.
[What this example is, and where it stops](#what-this-example-is-and-where-it-stops) says what the
model holds fixed and how it grows toward an engineering-grade treatment.

```bash
cargo run -p material_examples --example structural_health_monitor_example
```

## The problem

A plate carrying more than its yield strength does not simply fail. It sheds what it held onto the
plates it is bonded to, which may then exceed their own limit and shed in turn. That is a **failure
cascade**, and whether it stops is a property of the topology, not of any single plate.

The plate cannot ask a ground station what to do. The cascade completes in a fraction of a second
and the round trip is seconds, so the decision is local or it is too late.

## The hull

Six plates, a hexagonal ring, and two cross-braces:

```text
0 - 1 - 2          bonds: 0-1, 1-2, 2-3, 3-4, 4-5, 5-0
|       |          bracing: 0-4, 1-5
5 - 4 - 3
```

The bracing is what makes the cascade reach plates that are not ring neighbours of the strike.

## What the code demonstrates

| Operation | Carries |
|---|---|
| `extend` | one redistribution step over the whole hull |
| `alternate_value_if` | the intervention, as Pearl's do-operator |

**The graph comonad carries the cascade.** The hull is a `Graph` whose payload is the stress on each
plate. One redistribution step is a single `extend`: the closure receives the hull focused on one
plate, reads that plate's stress, asks the graph for its bonds, and returns the plate's next stress.
The topology is never copied into a side structure, so the cascade is computed on the graph that
describes the hull.

The rule is local. A plate over yield sheds everything it holds. Every surviving plate takes an
equal share of each failed neighbour's load, split among that neighbour's surviving bonds. A plate
that shed in an earlier step neither sheds again nor receives.

**The causal monad carries the intervention.** `alternate_value_if` substitutes the value the sensor
reported with the value the intervention forces, and records the substitution. Running the same
cascade from the observed reading and from the intervened one is the counterfactual, computed twice
rather than asserted once.

## Hooke's law

`σ = E·ε`, so `ε = σ/E`. Engaging the active reinforcement raises the modulus, and the same stress
then deforms the plate less:

```text
as manufactured        E =  200 GPa   strain 1.600e-3
reinforcement engaged  E =  400 GPa   strain 8.000e-4
```

## Output

Nominal service load is 120 MPa on every plate. The strike adds 200 MPa to plate 2, taking it to
320 MPa against a 250 MPa yield strength.

```text
Without intervention
  impact    120.0  120.0  320.0  120.0  120.0  120.0   over yield: [2]
  step 1    120.0  280.0    0.0  280.0  120.0  120.0   over yield: [1, 3]
  step 2    260.0    0.0    0.0    0.0  400.0  260.0   over yield: [0, 4, 5]
  step 3      0.0    0.0    0.0    0.0    0.0    0.0   no plate over yield
  settled: 6 of 6 plates breached

With intervention
  impact    120.0  120.0  100.0  120.0  120.0  120.0   no plate over yield
  settled: 0 of 6 plates breached
```

Plate 2 sheds 320 MPa to its two bonds, taking plates 1 and 3 to 280 MPa each. Plate 3 has one
surviving bond left and passes its whole 280 MPa to plate 4, which reaches 400 MPa. Three steps and
the section is gone. Holding plate 2 at 100 MPa leaves every plate under yield and nothing moves.

## Precision is a parameter

```rust
pub type FloatType = Float106;
```

Every constant is declared at that type through `const_scalar_from_int!`, so no conversion runs at
any call site. It sits at `Float106` rather than `f64` on purpose: a hard-coded `f64` is invisible
while the alias *is* `f64`, and a compile error the moment the two differ. All four scalars run.

## What this example covers 

The goal here is to reformulate the essence of a structural problem as a dynamic causal process, and to get
precision as a parameter and categorical composition for free once it is in that form. The essence
of a cascade is that a local rule plus a topology decides a global outcome, so the model keeps that
and holds everything else simple: load splits equally among surviving bonds, yield is instantaneous
and total, and a share falling on an already-breached plate leaves the model. Strain is Hooke's law
at the reported stress, uniaxial and elastic.

A production-grade solver adds the parts an engineer needs: a load path from geometry, stiffness
and boundary conditions, plasticity and a time constant so failure has dynamics, and load that
routes onward through the structure instead of leaving it.

## How to grow the example toward a complete solver

Each step keeps the structure already here.

- **Stiffness-weighted redistribution.** Weight each share by its bond stiffness instead of
  splitting equally. The `extend` closure changes; the graph and the loop stay as they are.
- **Load that routes onward.** Share a failed plate's load among the survivors it can still reach,
  which is a traversal inside the same closure and the reason the bonds live on the graph.
- **Plasticity and time.** Let a yielded plate carry a reduced load and give shedding a time
  constant. The step becomes a time step, which is the shape `extend` already has.
- **Richer per-plate state.** The payload type is a parameter of the graph, so a stress tensor per
  plate is a type change rather than a restructure, and `deep_causality_uncertain` carries a load
  that is a distribution instead of a number.

## Files

| File | Holds |
|---|---|
| `main.rs` | the alias, the intervention, and the cascade loop |
| `model.rs` | the material constants, the hull topology, the redistribution step, Hooke's law |
| `utils_print.rs` | the presentation, and the only `lower` calls |

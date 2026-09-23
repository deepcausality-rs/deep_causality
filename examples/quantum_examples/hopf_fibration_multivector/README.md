# The Hopf Fibration: Why a Qubit's Global Phase Is Unobservable

This example shows and measures the geometry behind a textbook statement: multiplying a quantum
state by `e^{iθ}` changes nothing you can measure.

```bash
cargo run -p quantum_examples --example hopf_fibration_multivector
```

## The problem

A qubit state `|ψ⟩ = α|0⟩ + β|1⟩` with `|α|² + |β|² = 1` is a point on the 3-sphere `S³`: two
complex amplitudes are four real numbers, and the normalisation removes one degree of freedom. An
experiment reads off the Bloch vector, a point on the ordinary 2-sphere `S²`.

The map between them is the **Hopf fibration** `h: S³ → S²`. It is not one-to-one. Every point of
`S²` is the image of a whole circle of points in `S³`, the **fiber**. Moving along the fiber is
exactly multiplying the state by a global phase.

So "global phase carries no physics" is a geometric statement: the fiber is what the projection
forgets.

## The fibration, in geometric algebra

`HopfState` holds the state as a rotor in `Cl(3)`, the even subalgebra whose elements are the unit
quaternions. Both maps are one line of algebra:

| Operation | Algebra | Meaning |
|---|---|---|
| `from_spinor` | `(α, β) ∈ ℂ² → R ∈ S³` | the state as a rotor |
| `project` | `R → R σ₃ R̃` | the Bloch vector, as a sandwich product |
| `fiber_shift` | `(R, θ) → R e^{−θe₁₂/2}` | one step along the fiber |

The projection applies the rotor's rotation to the north pole. No coordinate formula for the Bloch
vector appears anywhere.

## What the code demonstrates

| Operation | Carries |
|---|---|
| `fmap` | one fiber step at a time, across a full circuit |
| `zip_with` | each state paired against its own projection |
| `fold` | the circuit reduced to the furthest each one travelled |

`fmap` walks the fiber. `zip_with` pairs each state with the shadow it casts, so the run measures
the two displacements against each other. `fold` reduces a complete circuit to two numbers, which
extends the claim from one convenient angle to every phase.

## Output

```text
The six cardinal states and their Bloch vectors
  state       x        y        z      lands on
  |0⟩     +0.000   +0.000   +1.000    +z
  |1⟩     +0.000   +0.000   -1.000    -z
  |+⟩     +1.000   +0.000   +0.000    +x
  |−⟩     -1.000   +0.000   +0.000    -x
  |+i⟩    +0.000   +1.000   +0.000    +y
  |−i⟩    +0.000   -1.000   +0.000    -y

Walking the fiber from |+>, one complete circuit in 8 steps
  theta      state moved on S^3   shadow moved on S^2
   1.571 rad         0.585786            0.000000
   3.142 rad         2.000000            0.000000
   4.712 rad         3.414214            0.000000
   6.283 rad         4.000000            0.000000
   7.854 rad         3.414214            0.000000
   9.425 rad         2.000000            0.000000
  10.996 rad         0.585786            0.000000
  12.566 rad         0.000000            0.000000

Over the whole fiber
  the state moved by up to        4.000000
  its shadow moved by up to       0.000000
```

The shadow column is zero at every step. That is the claim.

### The cardinal table is the load-bearing check

The six cardinal states pin all three axes: `|0⟩` and `|1⟩` the poles, `|±⟩` the `x` axis, `|±i⟩`
the `y` axis. A projection that exchanged two axes would still look plausible on any one of them; it
would place `|+⟩` on `+y` while other rows stayed correct. Printing all six exposes that.

### A circuit is `4π`, not `2π`

`fiber_shift(θ)` multiplies by `e^{−θe₁₂/2}`, so the phase it applies is `θ/2` and the rotor returns
to itself only after `θ` has run through twice the circle. That factor of two is the **spinor double
cover**: `SU(2)` wraps `SO(3)` twice.

The table shows it. At `θ = 2π` the state is as far from its start as it can get (squared distance
`4`, the antipode `−R`), while its shadow has not moved. The state returns at `θ = 4π`. A spinor
takes two turns to come back; its shadow takes one.

## Precision is a parameter

```rust
pub type FloatType = Float106;
```

Every constant is declared at that type through `const_scalar_from_int!`, and `1/√2` is computed at
the working precision instead of rounded through a wider one. The alias defaults to `Float106` so
that a hard-coded `f64`, invisible while the alias *is* `f64`, becomes a compile error. All four
scalars run.

## What this example covers

The example restates the essence of the fibration as a composition over the library's types;
precision as a parameter and categorical composition then come for free. The essence: a projection
with a circle's worth of fiber forgets exactly that circle. The model keeps that and holds everything
else simple: one qubit, pure states only, no dynamics and no noise. The displacements are squared
Euclidean distances in the ambient algebra instead of geodesic distances on the spheres; they order
the steps identically and read more directly off the coefficients.

A treatment aimed at quantum control adds the parts an experimentalist needs: mixed states, where
the Bloch vector moves inside the ball instead of on its surface, and relative phase between qubits,
which is observable and outside this example's scope.

## How to grow the example toward a control-engineering tool

Each step keeps the structure already here.

- **Geodesic distance on the spheres.** Replace the squared chord with the arc it subtends. The
  `zip_with` closure changes; the walk and the fold stay as they are.
- **A control pulse instead of a fiber step.** Let the `fmap` closure apply an arbitrary rotor
  instead of one in the `e₁₂` plane, and the same walk reports how much of a pulse's effect is
  physical and how much is phase the projection discards.
- **Mixed states.** Carry a Bloch vector of length below one. The payload type is a parameter of
  the tensor, so this takes a type change, not a restructure; `deep_causality_uncertain` carries an
  amplitude that is a distribution instead of a number.
- **Hopfion field configurations.** The same projection over a grid of points instead of one
  state; `fmap` already has that shape.

## Files

| File | Holds |
|---|---|
| `main.rs` | the alias, the fiber walk, and the three categorical operations |
| `model.rs` | the constants, the cardinal states, the fiber, and the two displacements |
| `utils_print.rs` | the presentation, and the only `lower` calls |

# Relativistic Spinor Transport in Minkowski Cl(3,1)

## Introduction

This example parallel-transports a unit timelike spinor along a discretized worldline in flat Minkowski spacetime, one causal-monad step per segment. The spinor generalizes a 4-vector and describes particles with spin. At each segment it receives a Lorentz boost, the special-relativity analogue of a rotation, which mixes a time coordinate with a space coordinate.

The same inner loop appears wherever something with orientation or spin moves along a path made of segments. GPS satellites need relativistic corrections of about 38 microseconds per day or the position error grows by 10 km within a day. Particle accelerators (LHC, Fermilab) simulate beam dynamics with relativistic transports along every magnet section. Astrodynamics codes for deep-space missions track frame transformations along long worldlines. Quantum-optics simulations propagate qubit states by chained unitary rotors, the same operation in a different signature.

Five mathematical structures take part: topology, tensor, geometric algebra, the monadic effect chain and a high-precision scalar. Each step extracts a rapidity from the manifold's data tensor, rotates a multivector and advances the causal monad. The final spinor differs from the closed-form value by roughly `1.7e-31` at `Float106`, fifteen orders of magnitude tighter than the same calculation at `f64`.

## How to Run

```bash
cargo run -p mathematics_examples --example relativistic_spinor_transport_examples
```

## What It Demonstrates

One uniform API (`extract`, `bind`, `geometric_product`) describes a problem that crosses topology, tensor algebra, geometric algebra and effect tracking, with each crate meeting the next through the witness:

- `deep_causality_topology::Manifold` provides the discretized timelike path. Vertices index "events"; edges index transport segments.
- `deep_causality_tensor::CausalTensor` stores the per-edge rapidities inside the manifold's data tensor. Edges are addressed by repositioning the comonadic cursor and calling `ManifoldWitness::extract`.
- `deep_causality_multivector::CausalMultiVector` carries the spinor and builds the per-edge boost rotor in `Cl(3,1)` with signature `(+,-,-,-)`.
- `deep_causality_core::CausalEffectPropagationProcess` chains one transport step per edge. The per-step log records the spinor state; the error path fires on numerical instability.

## Mathematical Content

In Cl(3,1) the bivector `e0 ^ e1` squares to `+1`, so it generates hyperbolic (boost) rotations:

```
B(theta)  = cosh(theta/2) - sinh(theta/2) * e0^e1
B~(theta) = cosh(theta/2) + sinh(theta/2) * e0^e1
```

The reverse flips the sign of every grade-2 element. The sign on the bivector puts the sandwich on
the standard active convention `(t, x) -> (cosh(theta) t + sinh(theta) x, ...)`, which the code
builds and the composition check below reads back. The transport law on a vector is the sandwich `psi' = B psi B~`. For an initial `psi = e0`, after edges with rapidities `theta_1, ..., theta_N`, the result is

```
psi_final = cosh(sum theta_i) * e0 + sinh(sum theta_i) * e1
```

The example checks this composition law numerically.

## Bit-String Basis Indexing

The 16 basis elements of `Cl(3,1)` are indexed by the bitmask over `(e0, e1, e2, e3)`: `e0` lives at index `0b0001 = 1`, `e1` at `0b0010 = 2`, the bivector `e0 ^ e1` at `0b0011 = 3`, and the pseudoscalar at `0b1111 = 15`.

## Key APIs

- `Metric::Minkowski(4)` for the `(+,-,-,-)` signature
- `ManifoldWitness::extract` after cursor repositioning
- `CausalMultiVector::geometric_product` for the sandwich rotor
- `ProcessWitness::bind` for chained transport with a stability gate

## Adaptation

- Replace constant rapidities with a per-edge function of position to model an accelerated worldline.
- Use a 2D simplicial complex with curvature to study true parallel transport on a non-flat manifold.
- Stack additional bivector generators (`e1 ^ e2`, `e0 ^ e3`) to add rotations and boosts in other planes.
- Lift to `Cl(2 ^ N, 2 ^ N)` Dirac spinors and represent the gamma matrices via the `multifield::gamma` module.

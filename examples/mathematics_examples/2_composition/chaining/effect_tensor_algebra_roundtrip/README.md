# Tensor <-> Algebra Round-Trip Inside the Causal Monad

## Introduction

This example converts a vector into geometric algebra and back inside the causal monad, and checks that the round trip preserves its length. Numerical pipelines convert data between representations: a vector becomes a quaternion, the quaternion rotates something, the result becomes a vector again, and a downstream consumer reads it. Each conversion can hide a bug, so the check is whether the round trip preserved the property it should.

The example takes a vector of length 5 (the 3-4-5 triangle), lifts it into a geometric-algebra representation, rotates it, brings it back to a plain vector, and asks whether the length is still 5. Rotations preserve length, so the answer should be yes; a no means something in the pipeline is wrong.

This is the shape of a property-based test, a regression baseline, and the conservation-law check that catches bugs in graphics code, robotics transforms, physics engines and signal-processing pipelines. The value type changes at every step (vector -> multivector -> multivector -> vector -> scalar), and the monad keeps the bookkeeping out of sight while each stage does its own job.

A 3-vector travels through four `bind` steps: lift into `Cl(3,0)`, rotate, lower back to a tensor, then compute its squared norm by tensor dot product.

## How to Run

```bash
cargo run -p mathematics_examples --example effect_tensor_algebra_roundtrip_examples
```

## What It Demonstrates

`bind` is heterogeneous in the value parameter. The chain has type sequence

```
Process<CausalTensor<f64>>
  -> Process<CausalMultiVector<f64>>
  -> Process<CausalMultiVector<f64>>
  -> Process<CausalTensor<f64>>
  -> Process<f64>
```

Each transition is one `ProcessWitness::bind` call. The state, context and accumulating log pass through unchanged; only the value type changes.

The round trip preserves the squared norm to machine epsilon. That is the test: a defect in the lift, the rotation or the lowering would make the final norm drift visibly.

## Mathematical Content

A pure-vector multivector in `Cl(3,0)` has components `(e1, e2, e3)` and zero elsewhere. A rotor for a 90-degree rotation in the `e1^e2` plane:

```
R = cos(theta/2) - sin(theta/2) * e12
```

The sandwich `R v R~` rotates `v` while preserving `|v|^2 = v . v`. The example verifies this last equality with a tensor dot product after lowering.

## Key APIs

- `ProcessWitness::bind` chained across four value types
- `CausalMultiVector::geometric_product`
- `EinSumOp::dot_prod`

## Adaptation

- Replace the single rotor with a composition of rotors to model a rigid-body chain.
- Swap `Metric::Euclidean(3)` for `Metric::Minkowski(4)` to study Lorentz boosts.
- Add a deliberate corruption between lift and rotate to see the drift grow.

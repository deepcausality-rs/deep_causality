# Tensor <-> Algebra Round-Trip Inside the Causal Monad

## Introduction

Every nontrivial numerical pipeline converts data between representations. A vector becomes a quaternion, the quaternion gets applied to something, the result is converted back to a vector, and a downstream consumer reads the answer. Every conversion is a place where bugs hide. The standard engineering question is: did the round-trip preserve the property it was supposed to preserve.

This example is the minimal version of that pattern. It takes a vector with length 5 (the 3-4-5 triangle), lifts it into a geometric-algebra representation, rotates it, brings it back to a plain vector, and asks: is the length still 5. The answer should be yes, exactly, because rotations preserve length. If the answer is no, something in the pipeline is wrong.

This is the shape of a property-based test, a regression baseline, and the kind of conservation-law check that catches bugs in graphics code, robotics transforms, physics engines, and signal-processing pipelines before they reach production. The fact that the value type changes at every step (vector -> multivector -> multivector -> vector -> scalar) is the point: the monad keeps the bookkeeping invisible while each stage does exactly its own job.

A 3-vector travels through four `bind` steps: lift into `Cl(3,0)`, rotate, lower back to a tensor, then compute its norm by tensor dot product. The carried value type changes at every step; the monad threads them in a single straight line.

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

Each transition is one `ProcessWitness::bind` call. The state, context, and accumulating log pass through unchanged; only the value type morphs.

The round-trip preserves the squared norm to machine epsilon. That fact is the test: if either the lift, the rotation, or the lower introduced a defect, the final norm would drift visibly.

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

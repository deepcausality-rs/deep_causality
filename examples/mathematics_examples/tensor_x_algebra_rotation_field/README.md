# Tensor x Algebra: Rotating a Discrete Vector Field

## Introduction

Imagine a grid where every cell holds an arrow pointing somewhere. You want to rotate every arrow by the same angle, all at once. That is what this example does.

You hit this pattern in computer graphics (rotating every normal vector on a mesh after the camera moves), in robotics (transforming a grid of velocity vectors into a different reference frame), in fluid simulation (rotating the velocity field of every cell during an advection step), and in any image processing job that aligns or warps oriented features. The "Clifford rotor" piece is a numerically robust replacement for the rotation matrices and quaternions you may already know. The result is the same; the bookkeeping is simpler and composes better when you stack multiple rotations.

A `CausalTensor` holds a grid of `CausalMultiVector` values. Each cell is rotated by the same Clifford-algebra rotor using a single `Functor::fmap` call on the outer tensor.

## How to Run

```bash
cargo run -p mathematics_examples --example tensor_x_algebra_rotation_field
```

## What It Demonstrates

Two HKT layers stack cleanly:

1. The outer container is a tensor (rank-2 grid).
2. Each cell is itself a multivector in `Cl(2,0)`.

The map operation lives at the tensor level. The cell-level closure performs the geometric product `R v R~`. No special handling is needed; the same `fmap` that works on `CausalTensor<f64>` works on `CausalTensor<CausalMultiVector<f64>>`.

## Mathematical Content

The unit vector `e1` is rotated by 90 degrees in the `e1^e2` plane. The rotor takes the form

```
R = cos(theta/2) - sin(theta/2) * e12
```

with reverse `R~`. After the operation, every cell points along `e2`.

## Key APIs

- `CausalTensor::from_shape_fn` for grid construction
- `CausalTensorWitness::fmap` for the outer Functor walk
- `CausalMultiVector::new`, `geometric_product` for the per-cell algebra
- `Metric::Euclidean(2)` for the `Cl(2,0)` signature

## Adaptation

- Replace the rotor with a per-cell function of position to model a vector field on a curved background.
- Swap `Metric::Euclidean(2)` for `Metric::Euclidean(3)` to rotate 3D vectors.
- Stack a `bind` after the `fmap` to chain a second algebraic operation.

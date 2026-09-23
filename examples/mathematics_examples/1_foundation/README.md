# Foundation: the vocabulary, one crate at a time

Two folders teach the towers the rest of the workspace builds on; eight show the API surface
of one crate each.

Start here to learn what a bound promises, what a trait provides, or what one crate does.
[2_composition](../2_composition/) uses this vocabulary to cross crate boundaries, and
[3_applications](../3_applications/) builds whole use cases on top.

Run any example from the repository root:

```bash
cargo run -p mathematics_examples --example <example_name>
```

## The two towers

| Folder | Crate | What it teaches |
|---|---|---|
| [algebra](algebra/) | `deep_causality_algebra` | the scalar tower: what `Real`, `Field`, `RealField` and `Normed` each promise, and what a bound decides |
| [haft](haft/) | `deep_causality_haft` | the categorical tower: `Functor`, `Monad`, `CoMonad`, `Traversable` and the rest, one trait at a time |

The two meet in the crates above them: a witness is generic in its element, and the element is
generic in its scalar.

## One crate at a time

Each folder uses a single crate and shows what each operation returns.

| Folder | Crate | What it covers |
|---|---|---|
| [calculus](calculus/) | `deep_causality_calculus` | differentiation by forward-mode AD, quadrature, time integrators |
| [fft](fft/) | `deep_causality_fft` | plan-based FFT, rFFT, and the Hermitian half-spectrum |
| [linear](linear/) | `deep_causality_linear` | sparse CSR and dense matrices and their operations |
| [multivector](multivector/) | `deep_causality_multivector` | geometric algebra: multivectors, fields, the matrix isomorphism |
| [num](num/) | `deep_causality_num`, `deep_causality_num_complex` | the four scalars and the precision boundary; 𝔽₂; the Cayley-Dickson ladder |
| [stats](stats/) | `deep_causality_stats` | moments, correlation, and the shaped distributions |
| [tensor](tensor/) | `deep_causality_tensor` | N-index tensors, broadcasting, Einstein summation |
| [topology](topology/) | `deep_causality_topology` | graphs, complexes, manifolds, boundary operators |

`deep_causality_num` appears in every one of them: the `FloatType` alias at the top of each
file, and the `lift` / `lower` pair at its boundaries.

## House rules

Every example here follows four rules:

1. **Precision is a parameter.** One `FloatType` alias, directly above `main`, threaded through
   every numerical site.
2. **Values cross the precision boundary through `deep_causality_num::lift`.** `lift`,
   `lift_usize`, `lift_count` on the way in; `lower` on the way out.
3. **Printing lives in helper functions below `main`.** `main` reads as the narrative.
4. **Fallible calls propagate with `?`** out of a `main` that returns `Result`.

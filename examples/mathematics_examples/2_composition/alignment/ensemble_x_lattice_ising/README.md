# Ensemble × Lattice: the 2D Ising model

A periodic 16×16 lattice of spins, evolved by Metropolis-Hastings, run as an ensemble of 32
independent replicas. Four crates meet, each doing only what it owns.

```bash
cargo run -p mathematics_examples --example ensemble_x_lattice_ising_examples
```

## What composes with what

| Crate | Role |
|---|---|
| `deep_causality_rand` | entropy — a seeded generator |
| `deep_causality_stats` | the acceptance draw, `StandardUniform` at the working scalar |
| `deep_causality_tensor` | each lattice, and the ensemble of lattices |
| `deep_causality_haft` | `fmap` for observables, `fold` for means, `sequence_zip` for the pairing |

The ensemble is a `CausalTensor<CausalTensor<FloatType>>` — a tensor whose payload is tensors. No
container was written for it. A rank-1 tensor of lattices already carries every witness the folds
need, and a new type would have duplicated that surface under a different name.

## The closed form it checks against

Onsager solved this model exactly in 1944:

```text
Tc = 2 / ln(1 + √2) = 2.269185…
```

Below `Tc` the lattice orders and the magnetisation per spin approaches one; above it the lattice
disorders and the magnetisation falls to zero. That transition is the check, and the run prints it:

| T | \|m\| | phase |
|---|---|---|
| 1.5000 | 0.9922 | ordered |
| 2.0000 | 0.9453 | ordered |
| **2.2692** | **0.8125** | **critical** |
| 2.6000 | 0.0391 | disordered |
| 3.5000 | 0.0547 | disordered |

The magnetisation collapses between `Tc` and 2.6, which is where Onsager says it must.

## Why the traversal has to be the diagonal one

The susceptibility is a *fluctuation*:

```text
chi = beta · N · (⟨m²⟩ − ⟨m⟩²)
```

It means something only if replica *i*'s `m²` is the square of replica *i*'s `m`. Turning the field
of observables inside out through `Traversable::sequence` uses the cartesian applicative, which
pairs replica 3's magnetisation with replica 17's energy — every combination across the replicas,
and each one its own replica.

`DiagonalTraversable::sequence_zip` pairs index with index, which is what a replica *is*. The
example asserts it: for every field in the result, cell 1 must be the
**exact** square of cell 0, and cell 2 must carry its own index. Under a cartesian traversal the
first field that mismatches fails the assertion.

The result agrees with the variance computed directly to `3e-8`:

```text
chi via the diagonal traversal = 2.5193
chi from the variance directly = 2.5193
```

## Precision per part

The simulation runs at `f32`. Metropolis is noise-bound — every step is a random accept or reject,
and 32 replicas give a few percent of statistical error, far above anything a 24-bit significand
contributes.

The susceptibility was expected to be different: `⟨m²⟩` and `⟨m⟩²` nearly coincide, so subtracting
them should discard the agreeing digits. **Measurement contradicted that twice**, and the table is
the four-cell design that separates the two reasons:

| regime | f32, relative to `Float106` |
|---|---|
| L=16, `Tc` — exact reduction, large fluctuation | `3.03e-8` |
| L=16, `T=1.5` — exact reduction, small fluctuation | `4.37e-10` |
| L=32, `Tc` — reduction rounds, large fluctuation | `1.42e-7` |
| **L=32, `T=1.5` — reduction rounds, small fluctuation** | **`7.73e-5`** |

Two corrections fall out of it.

**The cancellation is mildest at `Tc`.** `chi` is proportional to the variance, and a
critical point is where the variance diverges — that is what makes it critical. The fluctuation is
largest exactly where the quantity is being asked for. Measured: `⟨|m|⟩ = 0.74 ± 0.15` at `Tc`
against `0.99 ± 0.01` at `T = 1.5`.

**And at 16×16 no scalar can disagree, because the arithmetic is exact.** `|m| = k/N` is a dyadic
rational needing `log₂ N` significand bits; `m²` needs twice that; summing `R` of them needs
`2 log₂ N + log₂ R`. At `L = 16, R = 32` that is 21 bits, inside `f32`'s 24 — so `f32`, `f64` and
`Float106` return **bit-identical** results. At `L = 32` it is 25 bits and `f32` must round, which
is why the table has a second lattice size in it. The threshold was computed before the run, not
found in it.

Only the cell where both go wrong costs anything, and it costs four orders of magnitude more than
the other three. The *best* of the four is the exact reduction with the smallest fluctuation —
the opposite of what the cancellation argument alone predicts.

The lesson is that whether a wider float helps is a
question about your data with an answer you can compute, and reaching for a wider float without
asking it is guesswork.

## Deferred

The same composition over `LatticeGaugeField` and `try_metropolis_sweep`, which `deep_causality_topology`
already carries — Wilson loops in a U(1) gauge theory. Ising first because its exact answer is
unambiguous; the gauge version is a capstone once this is proven.

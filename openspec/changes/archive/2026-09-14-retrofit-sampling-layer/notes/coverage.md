<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Coverage over the changed files

`cargo llvm-cov --summary-only` over `rand`, `stats`, `haft`, `tensor` and `linear`.

| File | Lines | Regions |
|---|---|---|
| `stats/types/distr/unit_interval/unit_draws.rs` | 100.00% | 100.00% |
| `stats/types/distr/uniform_int/mod.rs` | 100.00% | 100.00% |
| `stats/types/distr/categorical/mod.rs` | 100.00% | 100.00% |
| `stats/types/distr/cauchy/mod.rs` | 100.00% | 100.00% |
| `stats/types/distr/exponential/mod.rs` | 100.00% | 100.00% |
| `stats/types/distr/log_normal/mod.rs` | 100.00% | 100.00% |
| `stats/types/distr/weibull/mod.rs` | 100.00% | 100.00% |
| `stats/types/distr/normal/mod.rs` | 100.00% | 100.00% |
| `stats/types/distr/normal/standard_normal.rs` | 100.00% | 100.00% |
| `stats/traits/random_ext.rs` | 100.00% | 100.00% |
| `tensor/extensions/ext_hkt_zip.rs` | 100.00% | 100.00% |
| `linear/extensions/hkt/dense_vector_witness.rs` | 100.00% | 100.00% |
| `stats/types/distr/bernoulli/mod.rs` | 98.04% | 100.00% |
| `rand/types/qmc/sobol.rs` | 98.59% | 100.00% |
| `tensor/extensions/ext_hkt.rs` | 95.58% | 96.85% |
| `stats/types/distr/poisson/mod.rs` | 94.87% | 97.22% |
| `rand/traits/rng.rs` | 92.06% | 94.92% |

`traits/rand_width.rs` and `traversable/diagonal.rs` carry no executable code — an associated
constant and a trait declaration — so llvm-cov reports no regions for them.

## The misses, named

**`rng.rs` lines 105-107 are `Rng::fill`, and no test can reach them.** The method is
`fn fill<T: Fill + ?Sized>(&mut self, dest: &mut T) { dest.fill(self) }`, and **`Fill` has no
implementors anywhere in the workspace** — searched, zero. Writing the test is what found it: the
obvious call, `rng.fill(&mut buffer[..])`, is `error[E0277]: the trait bound [u8]: Fill is not
satisfied`, because nothing implements the trait for anything.

So this is not a coverage gap. It is dead public surface: a trait with no implementations and a
method on `Rng` that no caller can invoke. It predates this change, which only retyped the
neighbouring `random_bool`, and removing it is a decision for whoever owns that API rather than
something to slip into a close-out. Recorded here so the next coverage pass does not spend the same
hour on it.

**The remaining four lines are monomorphisation artifacts, not untested paths.** Checked one by one
in the annotated report, and each is an `Unexecuted instantiation` line:

```text
Unexecuted instantiation: <Poisson<_>>::new
Unexecuted instantiation: <Bernoulli>::new::<_>
Unexecuted instantiation: <SobolSequence>::new
```

`llvm-cov` counts every monomorphisation separately and lists the uninstantiated placeholder as
unexecuted. `Poisson<f32>`, `Poisson<f64>` and `Poisson<Float106>` are each exercised; `Poisson<_>`
is not a thing that runs. The same holds for the other two, and the region percentages — 97.22% and
100% — are the truer reading of those files.

**`ext_hkt.rs` at 95.58% is mostly not this change's code.** The file holds `CausalTensorWitness`'s
whole witness surface; `sequence_zip` is one impl at the end of it, and it is covered.

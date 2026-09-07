<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# C4 — hand-rolled linear-algebra site inventory

Built by sweeping every root crate, `deep_causality_utils/`, and `examples/` on 2026-09-07, after the
two `linear` defects were fixed (tasks 6.1–6.5b). Every site below was opened and read; nothing here
is inferred from a name.

## The classification, and why it differs from the spec's

The spec drew three classes — `replace`, `replace-with-care`, `keep`. The maintainer's **dual
mandate** replaces that with four, because "the hand-rolled version is right and the crate has
nothing like it" is a reason to **move it into the crate**, not a reason to keep it:

| Class | Meaning |
|---|---|
| **replace** | The crate already has this operation. Call it. |
| **move-in** | This is linear algebra the crate lacks. Move it into `linear`, dispatch from the call site. |
| **replace-with-care** | A change in performance or behaviour. Measure or pin before landing. |
| **keep** | Not linear algebra, or the same arithmetic the crate reaches by another route. |

`keep` is now a small class, and every member of it names a reason that is *not* "the crate has no
such function".

## Correction: the affected crates are six, not nine

The spec says "nine consumer crates". Four crates were swept and hold **no** hand-rolled linear
algebra at all: `deep_causality_ethos`, `deep_causality_core`, `deep_causality_data_structures` and
`deep_causality_discovery` — the last of these matches only on data-loading code and comments that
use the word *matrix*. Two example crates carry a site each; the other fourteen carry none.

The crates with sites are `deep_causality_quantum`, `deep_causality_physics`, `deep_causality_cfd`,
`deep_causality_algorithms`, `deep_causality`, and two under `examples/`.

## Correction: the ideal-MHD matvec is in `physics`, and there are four copies

The spec attributes the silently-skipping CSR matrix-vector product to the ideal-MHD solver without
naming a file, and the risk register discusses it as one site. It is in **`deep_causality_physics`**,
not `cfd`, and it appears **four times in two files**:

| Site | Scalar |
|---|---|
| `kernels/mhd/ideal.rs:204` `apply_csr_real` | `CsrMatrix<R>` |
| `kernels/mhd/ideal.rs:236` `apply_csr_i8` | `CsrMatrix<i8>` × `&[R]` |
| `kernels/mhd/grmhd.rs:119` `apply_csr_real` | `CsrMatrix<R>` |
| `kernels/mhd/grmhd.rs:149` `apply_csr_i8` | `CsrMatrix<i8>` × `&[R]` |

All four carry the same guard, `if col < vector.len()`, which drops a term rather than refusing the
product — so a mismatched vector yields a plausible wrong answer. `CsrMatrix::vec_mult` already
returns `LengthMismatch` for exactly this input, which makes the two `apply_csr_real` copies a
replace. The two `apply_csr_i8` copies have no counterpart in the crate: they multiply a `CsrMatrix<i8>`
of orientation signs by a vector of `R`, lifting the sign at the multiplication site to keep the
coboundary operators in `i8`. That is a mixed-scalar matvec, and it is a move-in.

## A. Replace — the crate already has it

| Site | Operation | Crate function |
|---|---|---|
| quantum `qgates/operator_linalg.rs:64` `frobenius_norm` | `sqrt(Σ re²+im²)` folded over a slice | `vector_norm_l2` — see the note below |
| quantum `qgates/channel.rs:377` | entrywise max-modulus residual (1 of 5) | `Normed::modulus`, behind one helper |
| quantum `qgates/operator_linalg.rs:88` | (2 of 5) | " |
| quantum `verdict/projection.rs:83` | (3 of 5) | " |
| quantum `verdict/projection.rs:181` | (4 of 5) | " |
| quantum `verdict/projection.rs:203` | (5 of 5) | " |
| quantum `qpu/sim.rs:72`, `:77` | complex multiply, squared modulus | `Complex` operators, `modulus_squared` |
| quantum `qcm/hypothesis.rs:464` | complex multiply into a trace accumulator | `Complex` operators |
| quantum `qgates/channel.rs:34`, `:163` | complex multiply, real scaling | `Complex` operators, `scale_by_real` |
| quantum `verdict/born.rs:47` | complex multiply into a trace accumulator | `Complex` operators |
| quantum `density_matrix.rs:100`, `:135`, `:150`, `:233`, `:260` | modulus, squared modulus, conjugate multiply, real scaling | `Normed`, `Complex` operators |
| quantum `verdict/projection.rs:130`, `:143`, `:249` | squared modulus, conjugate multiply | `Normed`, `Complex` operators |
| physics `kernels/mhd/ideal.rs:204`, `grmhd.rs:119` `apply_csr_real` | CSR matvec with a silent column skip | `CsrMatrix::vec_mult` |
| cfd `navigation/reentry_nav.rs:240` `norm` | Euclidean norm of `[R; 3]` | `vector_norm_l2` |
| cfd `navigation/ins_error_state.rs:184` `position_error_norm` | " | `vector_norm_l2` |
| examples `avionics_examples/src/shared/utils.rs:47` `norm3` | " | `vector_norm_l2` |

The three `[R; 3]` norms are written `(x*x + y*y + z*z).sqrt()`, so each carries the overflow the
scaled form was just fixed for. Replacing them is a correctness change, not only a consolidation.

**`frobenius_norm` is a replace only because of 6.5a.** The spec's scenario "The Frobenius norm is
left where it is" reasons that delegating buys nothing, since `modulus_squared` is the same direct
form and only `modulus` is scaled. That held of the crate as it stood. `matrix_norm_frobenius` and
`vector_norm_l2` are both scaled now, and the Frobenius norm is the two-norm of the entries read as
one vector, so the delegation removes an overflow above about `1.34e154` rather than buying nothing.
The spec scenario must be rewritten before this lands. Cost: the bound widens from `R: RealField` to
`R: RealField + FromPrimitive`, because `Complex<T>: FromPrimitive` requires it — the same widening
task 5.11 made to the physics entropy kernel, and breaking for the same reason.

## B. Move into `linear`, then dispatch

Each row names the callers that justify the addition. The identity spec's rule — a function earns
its place at two callers — is met by every row but the last two, which are noted.

| New crate function | Operation | Sites it absorbs |
|---|---|---|
| `dot` | inner product of two slices | algorithms `brcd_gaussian.rs:654`; cfd `navigation/eskf.rs:45`; cfd `types/flow/corridor/mod.rs:61` `dot3`; physics `kernels/astro/ks_propagator.rs:232` `dot4` — **4 callers** |
| `determinant_3x3` | `3×3` determinant, written out | physics `kernels/fluids/kinematics.rs:126`; physics `theories/general_relativity/gr_utils.rs:118` — **2** |
| `trace_of_square` | `tr(A²)` for a `3×3` | physics `kernels/fluids/kinematics.rs:114`; physics `kernels/fluids/coherent_structures.rs:46` — **2** |
| `double_dot` | Frobenius inner product `A : B` | physics `kernels/fluids/governing.rs:232`; physics `kernels/fluids/compressible.rs:273` — **2** |
| `csr_i8_vec_mult` | `CsrMatrix<i8>` × `&[R]`, lifting the sign at the multiply | physics `kernels/mhd/ideal.rs:236`; physics `kernels/mhd/grmhd.rs:149` — **2** |
| `eigen_symmetric_3x3` | closed-form symmetric `3×3` eigenvalues (Smith 1961) | physics `kernels/fluids/coherent_structures.rs` `symmetric_3x3_eigenvalues` — **1 caller, plus the general `eigen_hermitian` path it becomes a fast case of.** The spec classified this `keep` on the ground that the closed form beats a general Hermitian decomposition. It does; under the dual mandate that is an argument for the crate owning both, with the `3×3` case dispatched to the closed form, not for physics keeping a private eigensolver |
| `inverse_3x3`, `inverse_4x4` | cofactor inverse | physics `theories/general_relativity/gr_utils.rs:114`, `:12`, reached through `adm_state.rs:126` — **1 caller each.** Whether these move in or are replaced by the crate's LU `inverse` is the measurement task 6.9 owns; either way the private copies go |
| `mat3_vec` | `3×3` matrix times `[R; 3]` | cfd `navigation/reentry_nav.rs` — **1 caller.** Folds into the general dense path unless the benchmark says otherwise |

## C. Replace with care

| Site | The care needed |
|---|---|
| cfd `navigation/eskf.rs:31` `mat_mul`, `:42` `mat_vec` over `[[R; M]; M]` at `M = 17` | Stack-allocated and const-generic. Moving a 17×17 filter kit onto a heap matrix type may cost more than the duplication does. Benchmark before and after; revert on regression. This is the site the spec names, and the `[R; 17]` shape is confirmed across `navigation/ins_error_state.rs`, `nav_sensors.rs`, `reentry_nav.rs` and `types/flow/corridor/trajectory_nav.rs` |
| physics `kernels/mhd/{ideal,grmhd}.rs` `apply_csr_real` → `vec_mult` | A behaviour change on a shipped solver: a silently dropped term becomes `LengthMismatch`. Needs its own test pinning the new refusal, per the spec's requirement that a behaviour change is stated rather than absorbed |

## D. Keep, with the reason

| Site | Why it stays |
|---|---|
| physics `theories/electromagnetism/gauge_em_ops_impl.rs:309` `dot_product_3d`, `:328` `cross_product_3d` | These take `CausalMultiVector`, not a vector of scalars. They are geometric-algebra operations, and their home — if they move at all — is `deep_causality_multivector`, which sits **above** `linear`. Moving them into `linear` would invert the dependency |
| algorithms `brcd_algo.rs:488` `transpose`, `:498` `transpose_int`; `brcd_boss_bootstrap.rs:270`, `:280` | Not transposes despite the name: each selects a subset of columns and gathers them into rows, and `transpose_int` does it over `usize` with no arithmetic at all. There is no linear algebra to move. The **four copies are two functions duplicated verbatim across two files**, which is a real duplication and is recorded here so it is not lost — but it de-duplicates inside `algorithms`, not into `linear` |
| quantum `qcode/*`, `PackedGf2` users | Already on the crate's exact 𝔽₂ paths |

## E. Verified absent

Searched and not found, recorded so the next reader does not repeat the search:

- **No hand-rolled Gaussian elimination, LU, QR, Cholesky, back-substitution or tridiagonal solve
  anywhere outside `deep_causality_linear`.** The only matches for those names are Gaussian
  *distributions* in `physics/kernels/photonics`, `algorithms/causal_discovery/brcd` and an example.
- No hand-rolled matrix inverse outside `physics/theories/general_relativity`.
- No linear algebra in `ethos`, `core`, `data_structures` or `discovery`.
- No linear algebra in fourteen of the sixteen example crates.

## F. The three reachability pre-passes

Not linear algebra, but the spec's own requirement, so it is inventoried here.

| Site | Direction | Body |
|---|---|---|
| `deep_causality/src/traits/causable_graph/graph/mod.rs:154` `cone` | **backward** (`inbound_edges`), inclusive of the start | the expensive one |
| `deep_causality/src/traits/causable_graph/graph_reasoning/mod.rs:187` | forward (`outbound_edges`) | byte-identical to the next |
| `deep_causality/src/traits/causable_graph/graph_reasoning/stateful.rs:140` | forward (`outbound_edges`) | byte-identical to the previous |

The two forward copies are identical character for character; the backward one is the same traversal
with the edge accessor swapped. One implementation parameterised by direction serves all three.

Two behaviours must survive the collapse, both currently implicit:

- Every copy wraps the edge lookup in `if let Ok(...)`, so a graph that is not frozen yields no edges
  and the traversal simply stops. That is the not-frozen behaviour the spec requires be preserved
  rather than papered over — and it is preserved by *swallowing*, which the collapsed version must
  reproduce deliberately rather than by accident.
- Every copy indexes `reachable[start] = true` directly, so a start vertex outside the graph
  **panics**. The spec requires the collapsed version match the pre-collapse behaviour "rather than a
  newly introduced error"; the pre-collapse behaviour is a panic, and whether to keep it is a decision
  for the maintainer rather than something to settle silently while collapsing.

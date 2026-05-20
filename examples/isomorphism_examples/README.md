# Isomorphism Examples

Showcase examples that demonstrate where the three-tier iso surface (from `deep_causality_num::iso{::witness,}` and `deep_causality_haft::iso`) collapses type-conversion boilerplate.

> **Status**: these examples target the `implement-isomorphism` change. They will compile once that change lands and the concrete iso instances (Quaternion <-> Cl(3,0)-even rotor, CausalTensor rank-2 <-> CsrMatrix, Complex <-> Cl(0,1) multivector) are shipped. Until then, they are reference designs.

Each example is structured as a **BEFORE / AFTER** comparison so the simplification is visible at the point of use. Every example also asserts that the two paths produce byte-identical results, so the iso isn't just shorter — it's provably the same.

## The three examples

### 1. `quaternion_rotor_pipeline`

Rotate a 3D vector through a sequence of Clifford rotors. The BEFORE pattern packs every rotor's coefficients into an 8-element vector by hand (with the basis-index mnemonic in a comment). The AFTER pattern builds each rotor as a `Quaternion`, then lifts it to `CausalMultiVector` via `.into()`.

**Iso used**: `Quaternion <-> CausalMultiVector<F>` (Cl(3,0)-even rotor representation).

**Why it matters**: this is the most common multi-vector idiom in the codebase. Two existing examples (`triple_hkt_stress_field`, `effect_tensor_algebra_roundtrip`) hand-pack rotor coefficients today; the iso removes a documented source of basis-index off-by-one mistakes.

### 2. `tensor_sparse_memory_budget`

A heat-flow adjacency matrix arrives as a dense `CausalTensor`. Sparsify it (`CsrMatrix::from(tensor)`), apply a sparse-only operation (e.g. row-sum extraction), then materialise the result back to dense via `sparse.to_dense()` for downstream pipeline consumption.

**Iso used**: `CausalTensor<F>` (rank-2) <-> `CsrMatrix<F>` (mixed-tier: forward `From`, reverse `Iso<CsrMatrix, CausalTensor>` + inherent `to_dense()` alias).

**Why it matters**: this is the canonical worked example for the mixed-tier pattern. Any future analysis pipeline that wants to move large coefficient matrices through sparse intermediates copies this shape.

### 3. `complex_clifford_equivalence`

Demonstrate that complex multiplication is exactly the Cl(0,1) geometric product. Take two complex numbers, multiply them three ways: directly via `Complex::*`, by lifting both into `CausalMultiVector` (Cl(0,1) metric) and using `geometric_product`, and by chaining through `to_target` / `to_source`. Assert all three results are byte-identical.

**Iso used**: `Complex<F>` <-> `CausalMultiVector<F>` (Cl(0,1) metric) with the full algebraic-marker stack (`FieldIso`, `DivisionAlgebraIso`).

**Why it matters**: this is the foundational "Clifford is a generalisation of complex" worked example. It also exercises the full marker-subtrait chain (Iso -> GroupIso -> RingIso -> FieldIso -> DivisionAlgebraIso) on one type pair, which no other example covers.

## Running

Once the `implement-isomorphism` change lands:

```bash
cargo run --example quaternion_rotor_pipeline -p isomorphism_examples
cargo run --example tensor_sparse_memory_budget -p isomorphism_examples
cargo run --example complex_clifford_equivalence -p isomorphism_examples
```

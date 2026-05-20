# Isomorphism Examples

Runnable examples that demonstrate where the three-tier iso surface (from `deep_causality_num::iso{::witness,}` and `deep_causality_haft::iso`) collapses type-conversion boilerplate.

Each example is structured as a **BEFORE / AFTER** comparison where applicable, so the simplification is visible at the point of use. Every example also asserts that the two paths produce byte-identical results, so the iso isn't just shorter — it's provably the same.

## The three examples

### 1. `tensor_sparse_memory_budget`

A heat-flow adjacency matrix arrives as a dense `CausalTensor`. Sparsify it (`CsrMatrix::try_from(tensor)?`), apply a sparse-only operation (row-sum extraction), then materialise the result back to dense via `sparse.to_dense()` for a downstream pipeline that expects dense tensors.

**Iso used**: `CausalTensor<F>` (rank-2) <-> `CsrMatrix<F>` (mixed-tier: forward `TryFrom`, reverse `Iso<CsrMatrix, CausalTensor>` + inherent `to_dense()` alias). Requires the `tensor-iso` feature on `deep_causality_sparse` (enabled by this crate's `Cargo.toml`).

**Why it matters**: this is the canonical worked example for the mixed-tier orphan-rule pattern. Any future analysis pipeline that wants to move large coefficient matrices through sparse intermediates copies this shape.

### 2. `effect_process_witness_duality`

`PropagatingEffect<T>` and `PropagatingProcess<T, (), ()>` are both type aliases for the same concrete `CausalEffectPropagationProcess` carrier; each ships its own HKT witness with independently-written `Functor` / `Monad` impls. This example shows that both witness paths produce byte-identical outputs, and writes a generic pipeline parameterised over an arbitrary `Functor<W>` that accepts either witness interchangeably.

**Iso used**: none. The example explains *why* no `NaturalIso` is needed (the carrier is one type; the iso bodies would be identity), and demonstrates the dual-witness pattern that replaces it. The consistency property is pinned by direct `assert_eq!` tests under `deep_causality_core/tests/iso/`.

**Why it matters**: practitioners moving across the Markovian / non-Markovian boundary need to know that the unit-state, unit-context case is free (same type). Non-trivial state would be a lossy conversion, deferred to a separate change.

### 3. `multifield_data_pipeline`

`CausalMultiField<T>` keeps its fields `pub(crate)`, which from outside the multivector crate means: no public constructor takes `(tensor, metric, dx, shape)`, no public accessor returns the owned tensor, no generic "map the underlying tensor" helper can be written. The iso provides exactly one typed bridge that opens the door without breaking encapsulation.

This example builds three external pipeline helpers — `load_multifield`, `map_underlying_tensor`, `export_multifield` — that **cannot exist outside the multivector crate without the iso**. It then runs a realistic three-stage pipeline (load tensor data → apply tensor-level transformations → export the result) using only those helpers and `CausalTensor` arithmetic.

**Iso used**: `CausalMultiField<T>` <-> `MultiFieldCarrier<T>` (the tuple `(CausalTensor<T>, Metric, [T; 3], [usize; 3])`) via the `StandardIso` blanket impl on bidirectional `From`.

**Why it matters**: this is the case where the iso unlocks code that is structurally impossible without it. A new external consumer of `CausalMultiField` no longer requires adding accessors or constructors to the multivector crate. The iso is the entire public API surface for "build / extract / transform underlying" workflows.

## Running

```bash
cargo run --example tensor_sparse_memory_budget -p isomorphism_examples
cargo run --example effect_process_witness_duality -p isomorphism_examples
cargo run --example multifield_data_pipeline -p isomorphism_examples
```

---

## Adding New Examples

1. Create directory: `examples/<your_example>/`
2. Add `main.rs` with doc comments (`//!` module docs)
3. Add `README.md` with:
    - How to run
    - Engineering value
    - Key concepts
    - APIs demonstrated
    - Adaptation suggestions
4. Register in `Cargo.toml`:
   ```toml
   [[example]]
   name = "your_example"
   path = "examples/your_example/main.rs"
   ```

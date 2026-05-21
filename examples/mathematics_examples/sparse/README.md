# Sparse Matrix Examples

Examples for the `deep_causality_sparse` crate: CSR-format sparse matrices used as
the storage backend for boundary operators in the simplicial and cubical topology
layer, and for any application that needs structured sparse linear algebra.

Run from the repository root:

```bash
cargo run -p mathematics_examples --example <example_name>
```

| File | Description | Command |
|------|-------------|---------|
| [basic_csr_ops.rs](basic_csr_ops.rs) | Constructing a `CsrMatrix` from triplets; row/column iteration; sparse-dense conversion via the optional `tensor-iso` feature | `cargo run -p mathematics_examples --example basic_csr_ops_examples` |
| [hkt_csr_ops.rs](hkt_csr_ops.rs) | `CsrMatrix` as an HKT functor. `Functor::fmap` over CSR values shows sparse matrices participating in the same composition story as `CausalTensor` and `Manifold` | `cargo run -p mathematics_examples --example hkt_csr_ops_examples` |

# Duality

Two views of one thing, and the bridge between them. An adjunction relates two functors that
face each other; an `iso` relates two carriers holding the same data in different shapes.

| Example | What it shows | Command |
|---|---|---|
| [stokes_adjunction](stokes_adjunction/) | `d ⊣ ∂`: Stokes' theorem as an `Adjunction` between `DifferentialForm` and `Chain`, and why the two extracting operations return `Result` | `cargo run -p mathematics_examples --example stokes_adjunction_examples` |
| [tensor_sparse_memory_budget](tensor_sparse_memory_budget/) | The same pipeline twice, hand-rolled conversions against the iso: 28 lines against 3 | `cargo run -p mathematics_examples --example tensor_sparse_memory_budget` |
| [multifield_data_pipeline](multifield_data_pipeline/) | Load, transform and export a `CausalMultiField` from outside the crate that owns it, with metadata preserved end to end | `cargo run -p mathematics_examples --example multifield_data_pipeline` |

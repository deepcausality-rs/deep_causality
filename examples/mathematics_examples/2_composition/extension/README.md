# Extension

`CoMonad::extend` hands a closure a cursor into the whole structure and asks for one value back,
at every position. The closure may call into any crate it likes, which is the reach: topology
supplies the walk, tensor holds the payload, and the kernel decides what a neighbourhood means.

This is how a graph convolution, a diffusion step, a stencil or a cellular automaton is written
without a loop over positions.

| Example | What it shows | Command |
|---|---|---|
| [graph_convolution](graph_convolution/) | A GNN layer: `GraphWitness::extend` mean-pools each node with its neighbours, then `fmap` applies the activation | `cargo run -p mathematics_examples --example graph_convolution_examples` |
| [sparse_matrix_contextual_sum](sparse_matrix_contextual_sum/) | The same vocabulary on a sparse matrix: `fmap`, `pure`, `apply`, `extend` and `fold` on `CsrMatrixWitness` | `cargo run -p mathematics_examples --example sparse_matrix_contextual_sum_examples` |
| [manifold_laplacian_stencil](manifold_laplacian_stencil/) | The discrete Laplacian as a comonadic extension: geometry is the context, the tensor is the payload | `cargo run -p mathematics_examples --example manifold_laplacian_stencil_examples` |
| [diffusion_space_and_time](diffusion_space_and_time/) | Two layers at once: `extend` is the step in space, `bind` is the step in time, on the same value | `cargo run -p mathematics_examples --example diffusion_space_and_time_examples` |

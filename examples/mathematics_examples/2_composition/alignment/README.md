# Alignment

Two structures sampled at the same positions pair slot `i` with slot `i`. `Semigroupal::zip_with` pairs and combines in one pass, and
`DiagonalTraversable::sequence_zip` lifts that to a whole structure of them.

A container can be read as an applicative in more than one way. `CausalTensorWitness` broadcasts
one function across every slot; `ZipTensorWitness` projects to the same `CausalTensor` and aligns
instead. The witness at the call site picks the reading.

| Example | What it shows | Command |
|---|---|---|
| [zip_and_convolve](zip_and_convolve/) | `Semigroupal`, `Convolutional`, `MonoidalApplicative` and `LaxMonoidal` across four crates: the zip witnesses, `Complex` and `Dual` | `cargo run -p mathematics_examples --example zip_and_convolve_examples` |
| [ensemble_x_lattice_ising](ensemble_x_lattice_ising/) | The 2D Ising model: `DiagonalTraversable::sequence_zip` keeps each replica's observables together, which is what the susceptibility needs | `cargo run -p mathematics_examples --example ensemble_x_lattice_ising_examples` |

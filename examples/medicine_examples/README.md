# Medicine Examples

Six worked problems from biomedicine, each solved end to end with the unified-math stack.

## Quick Start

Run any example from the repository root:

```bash
cargo run -p medicine_examples --example <example_name>
```

| Example | Domain | What it does |
|---|---|---|
| [aneurysm_risk](aneurysm_risk/) | cerebrovascular | Reads the wall shear stress profile along a vessel centreline, finds the starved dome and the peak gradient at its neck, and accumulates wall degeneration over cardiac cycles |
| [diving_decompression](diving_decompression/) | hyperbaric | Plans a dive with the Bühlmann ZH-L16C algorithm over sixteen tissue compartments, schedules the decompression stops, and reads the gas-loading rate from one evaluation over `Dual` |
| [epilepsy](epilepsy/) | neurology | Builds a connectome that seizes, then resects each region in turn to find the one curative target |
| [protein_folding](protein_folding/) | biophysics | Folds a chain through the generalized master equation, where memory kernels carry the path the chain took into its next step |
| [tissue_classification](tissue_classification/) | medical imaging | Separates a solid mass from a necrotic core by the Euler characteristic of a Vietoris-Rips complex |
| [tumor_treatment](tumor_treatment/) | oncology | Aims Tumour Treating Fields at a glioblastoma by ascending the exact gradient of the treatment objective |

## The house rules

Every example here follows the same four:

1. **Precision is a parameter.** A `FloatType` alias sits directly above `main`, and every quantity
   in the program carries it. Changing that one line reruns the whole example at another scalar.
   All six run at `BFloat16`, `f32`, `f64` and `Float106`, and the precision is the only thing that
   moves.
2. **Values cross the precision boundary through `deep_causality_num::lift`.** Configuration
   literals are written as `f64`, the widest form a source file holds, and lifted once.
3. **Printing lives in `utils_print.rs`.** It holds the only `lower` calls, so `f64` appears at the
   display boundary and nowhere else.
4. **Errors travel through `?` and return from `main`.**

Each example is a folder of three files: `main.rs` holds `main`, the alias and the constants `main`
uses; `model.rs` holds the domain model; `utils_print.rs` holds the presentation.

## Which categorical operation each example uses

The mathematics crates compose through `deep_causality_haft`. A crate that owns a container
declares a witness, and the categorical traits are implemented against that witness. What each
example reaches for, and why:

| Operation | Reads | Used by | For |
|---|---|---|---|
| `fmap` | one element | aneurysm_risk | the wall-shear closure, one node at a time |
| `extend` | an element and its neighbourhood | aneurysm_risk, epilepsy, tissue_classification | the stress gradient, the Kuramoto coupling, the local density |
| `fold` | the whole payload | aneurysm_risk, epilepsy, protein_folding, tissue_classification | the dome minimum, the order parameter, the probability mass, the density range |
| `zip_with` | two structures, slot by slot | diving_decompression | a tissue tension against its own half-time |
| `sequence` | a structure of fallible values | protein_folding | one fallible distribution out of many fallible entries |
| `try_step` | a value in sequence | diving_decompression, tumor_treatment | the dive phases, the gradient ascent |
| `DifferentiableField` | a scalar-generic model | diving_decompression, tumor_treatment | the gas-loading rate, the treatment gradient |

The pattern to take away: `fmap` applies a law to one value, `fold` reduces the payload to a
number, and `extend` is what a quantity asks for when it needs to see a neighbourhood. A reader who
learns `extend` on a graph can run it on a manifold or a point cloud, which is what the three
examples above do.

## Crates used

| Crate | Purpose |
|---|---|
| `deep_causality_haft` | the categorical traits: `Functor`, `CoMonad`, `Foldable`, `Semigroupal`, `Traversable` |
| `deep_causality_topology` | manifolds, graphs, point clouds, simplicial complexes |
| `deep_causality_tensor` | payload tensors and the zip witness |
| `deep_causality_calculus` | the differentiable field and arrow traits |
| `deep_causality_physics` | the generalized master equation and the probability type |
| `deep_causality_core` | `CausalFlow` and `PropagatingEffect` |
| `deep_causality_num` | the scalar tower and the lifts |
| `deep_causality_rand` | the seeded generator behind the sampled tumour |

## See Also

- [mathematics_examples](../mathematics_examples/README.md) - the vocabulary these build on
- [physics_examples](../physics_examples/README.md) - pure physics simulations
- [case_study_icu_sepsis](../case_study_icu_sepsis/) - clinical sepsis prediction

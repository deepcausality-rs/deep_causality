# Material Science Examples

Two problems from materials and structures, each solved end to end with the unified-math stack.

## Quick Start

Run either example from the repository root:

```bash
cargo run -p material_examples --example <example_name>
```

| Example | Domain | What it does | Command |
|---|---|---|---|
| [hyperlens](hyperlens/) | metamaterials, optics | Describes vacuum and a hyperbolic metamaterial as two metric signatures, then sweeps object periods to find the resolution limit each imposes | `--example hyperlens_example` |
| [structural_health_monitor](structural_health_monitor/) | structures, safety | Runs a failure cascade across a bonded hull with the graph comonad, and compares it against the same cascade after a recorded intervention | `--example structural_health_monitor_example` |

## Technial properties

1. **Precision is a parameter.** A `FloatType` alias sits directly above `main`, and every quantity
   carries it. Both default to `Float106` rather than `f64`, because a hard-coded `f64` is
   invisible while the alias *is* `f64` and a compile error the moment the two differ. Both run at
   `BFloat16`, `f32`, `f64` and `Float106`.
2. **Constants are declared at the working type** through `const_scalar_from_int!` and
   `const_scalar_from_float!`, so the compiler resolves them against the alias and no conversion
   runs at any call site.
3. **Printing lives in `utils_print.rs`.** It holds the only `lower` calls, so `f64` appears at the
   display boundary and nowhere else.
4. **Errors travel through `?` and return from `main`.**

Each example is a folder of three files: `main.rs` holds `main`, the alias and the categorical
operations; `model.rs` holds the domain model; `utils_print.rs` holds the presentation.

## Which operation each example uses

| Operation | Reads | Used by | For |
|---|---|---|---|
| `fmap` | one element | hyperlens | the dispersion relation, one object period at a time |
| `fold` | the whole payload | hyperlens | the finest period that still propagates |
| `extend` | an element and its neighbourhood | structural_health_monitor | one load-redistribution step across the bonded hull |
| `alternate_value_if` | a value in a flow | structural_health_monitor | the intervention, as Pearl's do-operator |

The pattern to take away: `fmap` applies a law to one value, `fold` reduces a payload to a number,
and `extend` is what a quantity asks for when it needs to see a neighbourhood. A reader who learns
`extend` on a graph can run it on a manifold or a point cloud, which is what the medicine examples
do.

## Crates used

| Crate | Purpose |
|---|---|
| `deep_causality_haft` | the categorical traits: `Functor`, `Foldable`, `CoMonad` |
| `deep_causality_metric` | metric signatures, which the hyperlens reads as material properties |
| `deep_causality_topology` | the hull graph and its witness |
| `deep_causality_tensor` | the payload tensors |
| `deep_causality_core` | `CausalFlow` and the intervention |
| `deep_causality_num` | the scalar tower, the lifts, and the const-scalar macros |
| `deep_causality_algebra` | the `Real` bound |

## See Also

- [medicine_examples](../medicine_examples/README.md) - the same house rules on six biomedical problems
- [mathematics_examples](../mathematics_examples/README.md) - the vocabulary these build on

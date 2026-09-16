# Composition: how a value crosses a crate boundary

Every example here spans more than one crate. The folder an example sits in names the
*mechanism* it uses to cross, because that is the part that transfers: a reader who learns
`extend` on a graph can run it on a manifold, a point cloud or a sparse matrix.

| Folder | Mechanism | The move |
|---|---|---|
| [nesting](nesting/) | `Functor` over a nested element | a witness whose element type is another crate's type |
| [extension](extension/) | `CoMonad::extend` | the closure gets a cursor and calls into any crate it likes |
| [chaining](chaining/) | `Monad::bind`, Kleisli | the value crosses in sequence, one step at a time |
| [alignment](alignment/) | `Semigroupal::zip_with`, `DiagonalTraversable` | two structures paired position by position |
| [operators](operators/) | `Arrow` | the computation is the value, held now and run later |
| [duality](duality/) | `Adjunction`, `iso` | two views of one thing, and the bridge between them |

Run any example from the repository root:

```bash
cargo run -p mathematics_examples --example <example_name>
```

Every example follows the same house rules: precision is a parameter (`FloatType`), values
cross the precision boundary through `deep_causality_num::lift`, printing lives in helper
functions below `main`, and the header echoes the working type so the output says which scalar
produced it. Each example is a folder with a `main.rs`.

`1_foundation/` teaches the vocabulary these use; `3_applications/` puts whole use cases on top.

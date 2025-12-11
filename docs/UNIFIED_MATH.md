# DeepCausality: A Unified Mathematical Framework

DeepCausality pioners **unifying discrete mathematics** through Category Theory.

By treating Tensors, Geometric Algebra (Multivectors), and Topology as **composable monadic structures**, the library allows you to express complex multi-domain systems in a single flow.

---

## 🔗 The Unifying Glue: HKT & Monads

The foundation is the **Higher-Kinded Type (HKT)** system provided by `deep_causality_haft`. Typically, Rust libraries are siloed: you can't easily map a function generic over "any container" that works for both a `Tensor` and a `Graph`.

DeepCausality implements the **Witness Pattern** for its core types:

| Domain | Type | Witness (HKT) | Role |
|--------|------|---------------|------|
| **Mechanics** | `CausalTensor<T>` | `CausalTensorWitness` | The Container (Data) |
| **Algebra** | `CausalMultiVector<T>` | `CausalMultiVectorWitness` | The Transformer (Ops) |
| **Topology** | `Manifold<T>` | `ManifoldWitness` | The Context (Space) |
| **Structure** | `CsrMatrix<T>` | `CsrMatrixWitness` | The Network (Relations) |
| **Causality** | `PropagatingEffect<T>` | `Effect5Witness` | The Flow (Time) |

Because they all implement `Functor` and `Monad` (via `deep_causality_haft`), you can chain them.

---

## 🌀 Concept: The Causal-Geometric Cycle

A typical advanced simulation follows a specific monadic cycle:

1.  **Topology (Context)**: You start with a `Manifold`. Use **Comonad Extraction** (`extract`) or **Extension** (`extend`) to get the local neighborhood data.
2.  **Mechanics (Field)**: The local data is a `CausalTensor`. You perform numerical operations (e.g., Einstein Summation).
3.  **Algebra (Logic)**: You map the tensor into a `CausalMultiVector` to apply Geometric Algebra (rotations, spinors) or Boolean logic.
4.  **Causality (Effect)**: The result is wrapped in a `PropagatingEffect` monad, which handles time stepping, error propagation, and logging.

### Code Example: From Geometry to Effect

```rust
// 1. TOPOLOGY: Start with a Manifold (Space)
let manifold = Manifold::new(complex, data, 0)?;

// 2. COMONAD: Extend over the neighborhood (e.g., for a Laplacian operator)
// "For every point, look at my neighbors and calculate X"
let field_update = ManifoldWitness::extend(&manifold, |local_view| {
    // 3. TENSOR: Perform a localized tensor contraction 
    let local_data: CausalTensor<f64> = local_view.data();
    let contracted = local_data.ein_sum("ij, j -> i", &metric).unwrap();
    
    // 4. ALGEBRA: Convert to Multivector for geometric rotation
    let mv = CausalMultiVector::from_tensor(contracted);
    let rotated = mv.geometric_product(&rotor);
    
    rotated.scalar_part()
});

// 5. CAUSALITY: Wrap the entire state transition in the Causal Monad
let effect = PropagatingEffect::pure(field_update)
    .bind(|state| {
        // Log the transition, check for errors, propagate to t+1
        if state.contains_nan() {
             PropagatingEffect::fail("Numerical Instability")
        } else {
             PropagatingEffect::pure(state)
        }
    });
```

---

## 📐 Adjunctions: The Bridge Between Worlds

The library also implements **Adjunctions** (using `deep_causality_haft::BoundedAdjunction`). An adjunction describes a relationship between two categories where "mapping one way is equivalent to mapping the other way."

In DeepCausality, this formally links **Geometry** (Multivectors) and **Topology** (Manifolds), allowing you to translate problems between domains mathematically rigorously.

---

## 🚀 Why This Matters?

1.  **Safety**: The type system enforces physical correctness. You can't accidentally add a `Tensor` to a `Graph` without a defined morphism.
2.  **Composability**: You can plug a Quantum Mechanical module (Algebra) into a Relativistic Gravity module (Topology) because they speak the same Monadic language.
3.  **Parsimony**: You learn **one API** (`map`, `bind`, `extend`, `extract`). It works for Quantum States, Tensors, and Causal Strings alike.

---

## Next Steps

*   Explore **[HAFT Concepts](HAFT.md)** to understand the HKT traits.
*   See **[Topology](TOPOLOGY.md)** for the geometric foundations.
*   See **[Physics](PHYSICS.md)** for the applied kernels.

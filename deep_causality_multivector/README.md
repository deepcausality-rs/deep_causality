# DeepCausality Multivector

A dynamic, universal Clifford Algebra implementation for Rust, designed for theoretical physics, causal modeling, and geometric algebra applications.

## Features

*   **Dynamic Metric Signature**: Supports arbitrary signatures $Cl(p, q, r)$ at runtime via the `Metric` enum.
    *   Euclidean, Non-Euclidean, Minkowski, PGA, and Custom signatures.
*   **Universal Multivector**: A single type `CausalMultiVector<T>` can represent scalars, vectors, bivectors, and higher-grade blades.
*   **Comprehensive Operations**:
    *   Geometric Product, Outer Product, Inner Product (Left Contraction).
    *   Reversion, Squared Magnitude, Inverse, Dual.
    *   Grade Projection.
*   **Higher-Kinded Types (HKT)**: Implements `Functor`, `Applicative`, and `Monad` (via `deep_causality_haft`) for advanced functional patterns.
    *   `Monad::bind` implements the **Tensor Product** of algebras.

## Pre-configured Algebras

### Complex

Algebras:
*   **$Cl_{\mathbb{C}}(2)$ (Complex Quaternions)**: The minimal complex Clifford algebra, often used for $\mathfrak{spin}(3, 1)$ representations.
*   **$Cl_{\mathbb{C}}(4)$ (Quaternion Operator Algebra)**: Hosts the $\mathfrak{spin}(4) \sim \mathfrak{su}(2)_L \oplus \mathfrak{su}(2)_R$ electroweak symmetries. ($\mathcal{M}_{\mathbb{H}}$)
*   **$Cl_{\mathbb{C}}(6)$ (Octonion Operator Algebra)**: Hosts the $\mathfrak{spin}(6) \sim \mathfrak{su}(4)$ Pati-Salam symmetries, and the colour group $\mathfrak{su}(3)_C$. ($\mathcal{L}_{\mathbb{O}}$)
*   **$Cl_{\mathbb{C}}(8)$ (Dixon Left Multiplication Algebra)**: Hosts $\mathfrak{spin}(8)$ triality. ($\mathcal{L}_{\mathcal{A}}$)
*   **$Cl_{\mathbb{C}}(10)$ (Grand Unified Algebra)**: Hosts the full $\mathfrak{spin}(10)$ gauge symmetry. ($\mathcal{M}_{\mathcal{A}}$)

Type: `ComplexMultiVector`

| Algebra (Contextual Name) | Canonical Signature | Constructor / Alias |
|:--------------------------------------------|:--------------------|:----------------------------------|
| **Complex Quaternions** | $Cl(2, 0)$ | `new_complex_pauli` (Alias for `new_complex_clifford_2`) |
| **Quaternion Operator** | $Cl(0, 4)$ | `new_quaternion_operator` (Alias for `new_complex_clifford_4`) |
| **Octonion Operator** | $Cl(0, 6)$ | `new_octonion_operator` (Alias for `new_complex_clifford_6`) |
| **Dixon Left Mult. Alg.** | $Cl(0, 8)$ | `new_dixon_algebra_left` (Alias for `new_complex_clifford_8`) |
| **Grand Unified Algebra** | $Cl(0, 10)$ | `new_gut_algebra` (Alias for `new_complex_clifford_10`) |


### Real

Algebras:
*   $Cl(N, 0)$: Generic N-dimensional Euclidean algebra.
*   $Cl(0, 1)$: Isomorphic to Complex Numbers $\mathbb{C}$.
*   $Cl(1, 0)$: Isomorphic to Split-Complex (Hyperbolic) Numbers.
*   $Cl(0, 2)$: Isomorphic to Quaternions $\mathbb{H}$.
*   $Cl(2, 0)$: Isomorphic to Split-Quaternions (Coquaternions) / $\text{Mat}(2, \mathbb{R})$.
*   $Cl(3, 0)$: Algebra of Physical Space (APS) / Pauli Algebra.
*   $Cl(1, 3)$ / $Cl(3, 1)$: Space-Time Algebra (STA) / Dirac Algebra (with two different conventions).
*   $Cl(4, 1)$: Conformal Geometric Algebra (CGA).

Type: `RealMultiVector`

| Algebra (Common Name) | Signature | Convention | Constructor / Alias |
|:------------------------------|:-----------|:-----------------| :--- |
| **Euclidean Vectors**         | $Cl(N, 0)$ | N-dim Euclidean | `RealMultiVector::new_euclidean` |
| **Complex Numbers**           | $Cl(0, 1)$ | | `RealMultiVector::new_complex_number` |
| **Split Complex Numbers**     | $Cl(1, 0)$ | | `RealMultiVector::new_split_complex` |
| **Quaternions**               | $Cl(0, 2)$ | | `RealMultiVector::new_quaternion` |
| **Split Quaternions**         | $Cl(2, 0)$ | | `RealMultiVector::new_split_quaternion` |
| **Pauli (APS)**               | $Cl(3, 0)$ | | `RealMultiVector::new_aps_vector` |
| **Spacetime (STA)**           | $Cl(1, 3)$ | Physics (+ - - -) | `RealMultiVector::new_spacetime_algebra_1_3` |
| **Spacetime (STA)**           | $Cl(3, 1)$ | Math/GR (- + + +) | `RealMultiVector::new_spacetime_algebra_3_1` |
| **Conformal (CGA)**           | $Cl(4, 1)$ | | `RealMultiVector::new_cga_vector` |


### 3D Projective Geometric Algebra

Type: PGA3DMultiVector

| Algebra | Signature            | Constructor / Alias               |
| :--- |:---------------------|:----------------------------------|
| **PGA 3D** | $Cl(3, 0, 1)$        | `PGA3DMultiVector::new_point`     |


## Custom Algebras

1) Define a matric
2) Instantiate either a real, complex, or custom typed MultiVector with the metric
3) Done

```rust
use deep_causality_multivector::{RealMultiVector, Metric};

   // Some data 
   let data = vec![0.0; 16];

   // Define a custom metric. See docs for Metrics about Generic or Custom metric type 
   let metric =  Metric::Custom {
                dim: 4,
                neg_mask: 1,
                zero_mask: 0,
            },

   // Instantaiate your custom algebra over a RealMultiVector
   let a = RealMultiVector::new(data_a,metric ).unwrap();
```

## Usage

Add this crate to your `Cargo.toml`.

```toml
deep_causality_multivector = {version = "0.1"}
```

### Basic Operations

```rust
use deep_causality_multivector::{CausalMultiVector, Metric};

fn main() {
    // Create two vectors in 2D Euclidean space
        
    let mut data_a = vec![0.0; 4];
    data_a[1] = 1.0; // 1.0 * e1
    let a = CausalMultiVector::new_euclidean(data_a).unwrap();

    let mut data_b = vec![0.0; 4];
    data_b[2] = 1.0; // 1.0 * e2
    let b = CausalMultiVector::new_euclidean(data_b).unwrap();

    // Geometric Product: e1 * e2 = e12
    let product = a * b;
    println!("e1 * e2 = e12 coefficient: {}", product.get(3).unwrap());
}
```

### Using Aliases (e.g., PGA)

```rust
use deep_causality_multivector::PGA3DMultiVector;

fn main() {
    // Create a point in 3D PGA (Dual representation)
    let point = PGA3DMultiVector::new_point(1.0, 2.0, 3.0);
    
    // Create a translator (Motor)
    let translator = PGA3DMultiVector::translator(2.0, 0.0, 0.0); // Shift x by 2
    
    // Apply transformation: P' = T * P * ~T
    let t_rev = translator.reversion();
    let transformed = translator.clone() * point * t_rev;
    
    println!("Transformed X: {}", transformed.get(13).unwrap()); // e032 component
}
```

### Higher-Kinded Types (HKT)

This crate implements HKT traits from `deep_causality_haft`.

*   **Functor**: Map a function over coefficients.
*   **Applicative**: Lift values and apply functions.
*   **Monad**: Tensor product of algebras.

```rust

use deep_causality_haft::{Applicative, Functor, Monad};
use deep_causality_multivector::{CausalMultiVector, Metric, CausalMultiVectorWitness};

fn main() {
    println!("=== Higher-Kinded Types (HKT) with CausalMultiVector ===");

    // 1. Functor: Mapping over coefficients
    println!("\n--- Functor (Map) ---");
    let v = CausalMultiVector::new_euclidean(vec![1.0, 2.0, 3.0, 4.0]).unwrap();
    println!("Original Vector: {:?}", v.data);

    // Scale by 2.0 using fmap
    let scaled = CausalMultiVectorWitness::fmap(v.clone(), |x| x * 2.0);
    println!("Scaled Vector (x2): {:?}", scaled.data);
    assert_eq!(scaled.data, vec![2.0, 4.0, 6.0, 8.0]);

    // 2. Applicative: Broadcasting a function
    println!("\n--- Applicative (Apply/Broadcast) ---");
    // Create a "pure" function wrapped in a scalar multivector
    let pure_fn = CausalMultiVectorWitness::pure(|x: f64| x + 10.0);

    // Apply it to our vector
    let shifted = CausalMultiVectorWitness::apply(pure_fn, v.clone());
    println!("Shifted Vector (+10): {:?}", shifted.data);
    assert_eq!(shifted.data, vec![11.0, 12.0, 13.0, 14.0]);

    // 3. Monad: Tensor Product via Bind    
    //...
    // See examples/hkt_usage.rs for full demonstration
}

```

## Benchmarks

Performance measured on Apple M3 Max.

| Operation | Metric | Time |
| :--- | :--- | :--- |
| **Geometric Product** | Euclidean 2D | ~119 ns |
| **Geometric Product** | PGA 3D | ~110 ns |
| **Addition** | Euclidean 3D | ~57 ns |
| **Reversion** | PGA 3D | ~56 ns |

To run benchmarks:
```bash
cargo bench -p deep_causality_multivector
```

## License

This project is licensed under the [MIT license](LICENSE).

## Security

For details about security, please read the [security policy](https://github.com/deepcausality-rs/deep_causality/blob/main/SECURITY.md).

## Author

* [Marvin Hansen](https://github.com/marvin-hansen).
* Github GPG key ID: 369D5A0B210D39BC
* GPG Fingerprint: 4B18 F7B2 04B9 7A72 967E 663E 369D 5A0B 210D 39BC

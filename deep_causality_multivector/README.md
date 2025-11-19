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
*   **Type Aliases & Constructors**: Pre-configured aliases for common algebras:
    *   `RealMultiVector`: Standard real Clifford algebras (Complex, Quaternions, Split-Complex, APS, STA, CGA).
    *   `ComplexMultiVector`: Complex Clifford algebras (Pauli, Octonion Operator).
    *   `PGA3DMultiVector`: 3D Projective Geometric Algebra.
    *   `DixonAlgebra`: $Cl_{\mathbb{C}}(6)$ (Dixon Algebra).

## Usage

Add this crate to your `Cargo.toml` (usually as part of the `deep_causality` workspace).

### Basic Operations

```rust
use deep_causality_multivector::{CausalMultiVector, Metric};

fn main() {
    // Create two vectors in 2D Euclidean space
    // e1 = (1, 0), e2 = (0, 1)
    // Indexing: Scalar=0, e1=1, e2=2, e12=3
    
    let mut data_a = vec![0.0; 4];
    data_a[1] = 1.0; // 1.0 * e1
    let a = CausalMultiVector::new(data_a, Metric::Euclidean(2)).unwrap();

    let mut data_b = vec![0.0; 4];
    data_b[2] = 1.0; // 1.0 * e2
    let b = CausalMultiVector::new(data_b, Metric::Euclidean(2)).unwrap();

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
    let m = Metric::Euclidean(2);
    let v = CausalMultiVector::new(vec![1.0, 2.0, 3.0, 4.0], m).unwrap();
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

## Supported Algebras

| Algebra | Signature | Constructor / Alias |
| :--- | :--- | :--- |
| **Complex Numbers** | $Cl(0, 1)$ | `RealMultiVector::new_complex_number` |
| **Quaternions** | $Cl(0, 2)$ | `RealMultiVector::new_quaternion` |
| **Split-Complex** | $Cl(1, 0)$ | `RealMultiVector::new_split_complex` |
| **Pauli (APS)** | $Cl(3, 0)$ | `RealMultiVector::new_aps_vector` |
| **Spacetime (STA)** | $Cl(1, 3)$ | `RealMultiVector::new_spacetime_vector` |
| **Conformal (CGA)** | $Cl(4, 1)$ | `RealMultiVector::new_cga_vector` |
| **PGA 3D** | $Cl(3, 0, 1)$ | `PGA3DMultiVector` |
| **Dixon** | $Cl_{\mathbb{C}}(6)$ | `DixonAlgebra` |

## License

This project is licensed under the [MIT license](LICENSE).

## Security

For details about security, please read the [security policy](https://github.com/deepcausality-rs/deep_causality/blob/main/SECURITY.md).

## Author

* [Marvin Hansen](https://github.com/marvin-hansen).
* Github GPG key ID: 369D5A0B210D39BC
* GPG Fingerprint: 4B18 F7B2 04B9 7A72 967E 663E 369D 5A0B 210D 39BC

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
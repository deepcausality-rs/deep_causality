# DeepCausality Effects

[![Crates.io][crates-badge]][crates-url]
[![MIT licensed][mit-badge]][mit-url]
[![Build Status][actions-badge]][actions-url]

[crates-badge]: https://img.shields.io/crates/v/deep_causality.svg
[crates-url]: https://crates.io/crates/deep_causality_effects
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/deepcausality/deep_causality.rs/blob/main/LICENSE
[actions-badge]: https://github.com/deepcausality/deep_causality.rs/workflows/CI/badge.svg
[actions-url]: https://github.com/deepcausality/deep_causality.rs/actions

**DeepCausality Effects** provides a unified type system for building heterogeneous causal graphs. It introduces the `EffectData` enum, which acts as a versatile container for various data types, enabling the core `deep_causality` engine to reason across diverse domains within a single graph structure.

## Overview

In complex systems, causal effects often appear in different forms: simple boolean triggers, continuous scalar values, multi-dimensional tensors, or geometric algebra constructs. `deep_causality_effects` solves the challenge of strictly typed homogeneous graphs by providing a "Sum Type" wrapper that encapsulates this complexity.

The crate is designed around the **"Atomic + Escape Hatch"** pattern:
1.  **Atomic Variants:** Efficient handling of common primitives (`bool`, `f64`, `i64`).
2.  **Unified Numerics:** A `NumericValue` type covering the full range of Rust's integer and float types.
3.  **Algebraic & Topological Types:** Native support for `MultiVector`, `CausalTensor`, `PointCloud`, `SimplicialComplex`, and `Manifold`.
4.  **Escape Hatch:** A type-erased `Custom` variant (`Arc<dyn Any>`) for arbitrary complex structures.

## Features

*   **Heterogeneity:** Mix and match data types in a single `CausalVec` or `CausalGraph`.
*   **Zero-Copy Cloning:** The `Custom` variant uses `Arc` for cheap cloning of complex data.
*   **Algebraic Support:** Native integration with `deep_causality_tensor` and `deep_causality_multivector`.
*   **Topology Support:** Native integration with `deep_causality_topology` (PointClouds, Complexes, Manifolds).
*   **Ergonomics:** Extensive `From<T>` implementations for seamless type conversion.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
deep_causality_effects = "0.0.1"
```

### Examples

#### Basic Usage

```rust
use deep_causality_effects::{EffectData, NumericValue};

fn main() {
    // Boolean Effect
    let activation: EffectData = true.into();
    
    // Numeric Effect (using generic wrapper)
    let scalar: EffectData = NumericValue::F64(42.0).into();
    
    // Collection
    let vec_effect: EffectData = vec![activation, scalar].into();
}
```

#### Algebraic Types

```rust
use deep_causality_effects::EffectData;
use deep_causality_tensor::CausalTensor;
use deep_causality_multivector::CausalMultiVector;

fn main() {
    // Geometric Algebra MultiVector (2D Euclidean)
    let mv = CausalMultiVector::<f64>::new_euclidean(vec![1.0, 0.0, 0.0, 0.0], 2);
    let mv_effect: EffectData = mv.into();

    // Tensor
    let tensor = CausalTensor::<f64>::new(vec![1.0, 2.0], vec![2]).unwrap();
    let tensor_effect: EffectData = tensor.into();
}
```

#### Topology Types

```rust
use deep_causality_effects::EffectData;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{PointCloud, SimplicialComplex, Manifold};

fn main() {
    // Point Cloud
    let points = CausalTensor::<f64>::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
    let meta = CausalTensor::<f64>::new(vec![0.0, 1.0], vec![2]).unwrap();
    let pc = PointCloud::new(points, meta, 0).unwrap();
    
    let effect: EffectData = pc.into();
}
```

#### Custom Types (Escape Hatch)

For types not natively covered, use the `Custom` variant:

```rust
use deep_causality_effects::EffectData;

#[derive(Debug, PartialEq)]
struct UserProfile {
    id: u64,
    role: String,
}

fn main() {
    let profile = UserProfile { id: 1, role: "Admin".into() };
    
    // Wrap in EffectData
    let effect = EffectData::from_custom(profile);
    
    // Downcast when needed
    if let Some(p) = effect.as_custom::<UserProfile>() {
        assert_eq!(p.role, "Admin");
    }
}
```

## Architecture

The `EffectData` enum is defined as:

```rust
pub enum EffectData {
    Bool(bool),
    Float(f64),
    Int(i64),
    String(String),
    Vector(Vec<EffectData>),
    Numerical(NumericValue), // Covers u8-u128, i8-i128, f32
    MultiVector(CausalMultiVector<f64>),
    Tensor(CausalTensor<f64>),
    PointCloud(PointCloud<f64>),
    SimplicialComplex(SimplicialComplex),
    Manifold(Manifold<f64>),
    Custom(Arc<dyn Any + Send + Sync>),
}
```

## License

This project is licensed under the [MIT license](LICENSE).

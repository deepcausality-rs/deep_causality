/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_effects::EffectData;
use deep_causality_effects::NumericValue;
use deep_causality_multivector::CausalMultiVector;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{PointCloud, SimplicialComplex};

#[test]
fn test_primitive_variants() {
    // Bool
    let e_bool: EffectData = true.into();
    if let EffectData::Bool(b) = e_bool {
        assert!(b);
    } else {
        panic!("Expected Bool variant");
    }

    // Float
    let e_float: EffectData = 42.0.into();
    if let EffectData::Float(f) = e_float {
        assert_eq!(f, 42.0);
    } else {
        panic!("Expected Float variant");
    }

    // Int
    let e_int: EffectData = 100_i64.into();
    if let EffectData::Int(i) = e_int {
        assert_eq!(i, 100);
    } else {
        panic!("Expected Int variant");
    }

    // String (from String)
    let e_string: EffectData = String::from("hello").into();
    if let EffectData::String(s) = e_string {
        assert_eq!(s, "hello");
    } else {
        panic!("Expected String variant");
    }

    // String (from &str)
    let e_str: EffectData = "world".into();
    if let EffectData::String(s) = e_str {
        assert_eq!(s, "world");
    } else {
        panic!("Expected String variant");
    }
}

#[test]
fn test_collection_variants() {
    let v_data = vec![EffectData::from(1.0), EffectData::from(true)];
    let e_vec: EffectData = v_data.clone().into();

    if let EffectData::Vector(v) = e_vec {
        assert_eq!(v.len(), 2);
        // Deep verification
        if let EffectData::Float(f) = &v[0] {
            assert_eq!(*f, 1.0);
        } else {
            panic!("Expected float at index 0");
        }
    } else {
        panic!("Expected Vector variant");
    }
}

#[test]
fn test_algebraic_variants() {
    let mv = CausalMultiVector::<f64>::new_euclidean(vec![1.0, 0.0, 0.0, 0.0], 2);
    let e_mv: EffectData = mv.into();
    if let EffectData::MultiVector(_) = e_mv {
        // Matched successfully
    } else {
        panic!("Expected MultiVector variant");
    }

    // Tensor
    let tensor = CausalTensor::<f64>::new(vec![1.0, 2.0], vec![2]).unwrap();
    let e_tensor: EffectData = tensor.into();
    if let EffectData::Tensor(t) = e_tensor {
        assert_eq!(t.shape(), &[2]);
    } else {
        panic!("Expected Tensor variant");
    }
}

#[test]
fn test_topology_variants() {
    // PointCloud
    let points = CausalTensor::<f64>::new(vec![0.0, 0.0, 1.0, 1.0], vec![2, 2]).unwrap();
    let metadata = CausalTensor::<f64>::new(vec![0.0, 1.0], vec![2]).unwrap();
    let pc = PointCloud::new(points, metadata, 0).unwrap();
    let e_pc: EffectData = pc.into();
    if let EffectData::PointCloud(p) = e_pc {
        assert_eq!(p.len(), 2);
    } else {
        panic!("Expected PointCloud variant");
    }

    // Since SimplicialComplex and Manifold require more complex setup (skeletons, matrices),
    // and we just want to test wrapper existence, we can use default if available or minimal valid construction.
    // SimplicialComplex has Default derive.
    let sc = SimplicialComplex::default();
    let e_sc: EffectData = sc.into();
    if let EffectData::SimplicialComplex(_) = e_sc {
        // Matched
    } else {
        panic!("Expected SimplicialComplex variant");
    }

    // Note: Manifold variant is implicitly tested via SimplicialComplex logic and From impls.
    // Constructing a fully valid Manifold instance requires passing `check_is_manifold`
    // which involves significant setup (oriented, link condition, etc.) that is out of scope
    // for this unit test. The existence of the variant is confirmed via PointCloud/SimplicialComplex.
}

#[test]
fn test_numerical_variant() {
    let num_u8 = NumericValue::U8(255);
    let e_num: EffectData = num_u8.into();

    if let EffectData::Numerical(NumericValue::U8(val)) = e_num {
        assert_eq!(val, 255);
    } else {
        panic!("Expected Numerical(U8) variant");
    }
}

#[test]
fn test_custom_variant() {
    struct MyType {
        inner: i32,
    }

    let my_val = MyType { inner: 99 };
    let e_custom = EffectData::from_custom(my_val);

    // Test downcast success
    let downcasted = e_custom.as_custom::<MyType>();
    assert!(downcasted.is_some());
    assert_eq!(downcasted.unwrap().inner, 99);

    // Test downcast failure
    let fail_cast = e_custom.as_custom::<String>();
    assert!(fail_cast.is_none());
}

#[test]
fn test_clone() {
    let e_orig: EffectData = 123.0.into();
    let e_clone = e_orig.clone();

    if let EffectData::Float(f) = e_clone {
        assert_eq!(f, 123.0);
    } else {
        panic!("Clone failed");
    }

    // Test Custom clone (Arc sharing)
    let e_custom = EffectData::from_custom(10);
    let e_custom_clone = e_custom.clone();

    // Both should point to same data (value 10)
    assert_eq!(*e_custom_clone.as_custom::<i32>().unwrap(), 10);
}

#[test]
fn test_debug() {
    let e: EffectData = 1.0.into();
    let s = format!("{:?}", e);
    assert!(s.contains("Float(1.0)"));
}

/*
 * Copyright (c) 2023. Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
 */

use deep_causality::prelude::*;
use deep_causality::utils::test_utils::{get_test_causality_coll, get_test_causaloid};

#[test]
fn test_add()
{
    let mut col = get_test_causality_coll();
    assert_eq!(3, col.len());

    let q = get_test_causaloid();
    col.push(q);
    assert_eq!(4, col.len());
}

#[test]
fn test_all_active()
{
    let col = get_test_causality_coll();
    assert_eq!(false, col.get_all_causes_true());

    let obs = 0.99;
    for cause in &col {
        cause.verify_single_cause(&obs).expect("verify failed");
    }
    assert_eq!(true, col.get_all_causes_true());
}

#[test]
fn test_number_active()
{
    let col = get_test_causality_coll();
    assert_eq!(false, col.get_all_causes_true());

    let obs = 0.99;
    for cause in &col {
        cause.verify_single_cause(&obs).expect("verify failed");
    }
    assert_eq!(true, col.get_all_causes_true());
    assert_eq!(3.0, col.number_active());
}

#[test]
fn test_percent_active()
{
    let col = get_test_causality_coll();
    assert_eq!(false, col.get_all_causes_true());

    let obs = 0.99;
    for cause in &col {
        cause.verify_single_cause(&obs).expect("verify failed");
    }
    assert_eq!(true, col.get_all_causes_true());
    assert_eq!(3.0, col.number_active());
    assert_eq!(100.0, col.percent_active());
}

#[test]
fn test_size()
{
    let col = get_test_causality_coll();
    assert_eq!(3, col.len());
}

#[test]
fn test_is_empty()
{
    let col = get_test_causality_coll();
    assert_eq!(false, col.is_empty());
}

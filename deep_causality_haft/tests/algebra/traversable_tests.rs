/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{OptionWitness, Traversable, VecWitness};

#[test]
fn test_traversable_vec_option() {
    // Sequence: Vec<Option<T>> -> Option<Vec<T>>
    let list_of_opts = vec![Some(1), Some(2), Some(3)];
    let opt_of_list = VecWitness::sequence::<i32, OptionWitness>(list_of_opts);
    assert_eq!(opt_of_list, Some(vec![1, 2, 3]));

    let list_with_none = vec![Some(1), None, Some(3)];
    let opt_none = VecWitness::sequence::<i32, OptionWitness>(list_with_none);
    assert_eq!(opt_none, None);
}

/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_haft::{
    BTreeMapWitness, BoxWitness, Functor, HKT, HashMapWitness, LinkedListWitness, OptionWitness,
    ResultWitness, VecDequeWitness, VecWitness,
};
use std::collections::{BTreeMap, HashMap, LinkedList, VecDeque};

// A generic function that doubles the value inside *any* Functor
fn double_value<F>(m_a: F::Type<i32>) -> F::Type<i32>
where
    F: Functor<F> + HKT,
{
    F::fmap(m_a, |x| x * 2)
}

fn main() {
    println!("--- Functor Example: Doubling values in different containers ---");

    // Using double_value with Option
    let opt = Some(5);
    println!("Original Option: {:?}", opt);
    let doubled_opt = double_value::<OptionWitness>(opt);
    println!("Doubled Option: {:?}", doubled_opt);
    assert_eq!(doubled_opt, Some(10));

    // Using double_value with Result
    let res = Ok(5);
    println!("Original Result: {:?}", res);
    let doubled_res = double_value::<ResultWitness<i32>>(res);
    println!("Doubled Result: {:?}", doubled_res);
    assert_eq!(doubled_res, Ok(10));

    // Using double_value with Box
    let b = Box::new(7);
    println!("Original Box: {:?}", b);
    let doubled_box = double_value::<BoxWitness>(b);
    println!("Doubled Box: {:?}", doubled_box);
    assert_eq!(doubled_box, Box::new(14));

    // Using double_value with LinkedList
    let list = LinkedList::<i32>::from([1, 2, 3]);
    println!("Original List: {:?}", list);
    let double_list = double_value::<LinkedListWitness>(list);
    println!("Doubled List: {:?}", double_list);

    // Using double_value with Vec
    let vec = vec![1, 2, 3];
    println!("Original Vec: {:?}", vec);
    let doubled_vec = double_value::<VecWitness>(vec);
    println!("Doubled Vec: {:?}", doubled_vec);
    assert_eq!(doubled_vec, vec![2, 4, 6]);

    // Using double_value with VecDeque
    let vec_dec = VecDeque::<i32>::from(vec![2, 4, 6]);
    println!("Original VecDec: {:?}", vec_dec);
    let doubled_vec_dec = double_value::<VecDequeWitness>(vec_dec);
    println!("Doubled VecDec: {:?}", doubled_vec_dec);
    assert_eq!(doubled_vec_dec, vec![4, 8, 12]);

    // Using double_value with HashMap
    let mut map = HashMap::new();
    map.insert(1, 2);
    map.insert(2, 3);
    map.insert(3, 4);
    println!("Original HashMap: {:?}", map);
    let double_map = double_value::<HashMapWitness<i32>>(map);
    println!("Doubled HashMap: {:?}", double_map);

    // Using double_value with BTreeMap
    let mut b_tree_map = BTreeMap::new();
    b_tree_map.insert(1, 6);
    b_tree_map.insert(2, 8);
    b_tree_map.insert(3, 10);
    println!("Original BTreeMap: {:?}", b_tree_map);
    let double_map_b_tree = double_value::<BTreeMapWitness<i32>>(b_tree_map);
    println!("Doubled BTreeMap: {:?}", double_map_b_tree);

    println!("\nExample finished successfully!");
}

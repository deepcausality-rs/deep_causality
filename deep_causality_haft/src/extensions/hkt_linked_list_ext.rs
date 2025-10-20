/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Applicative, Foldable, Functor, HKT, Monad};
use std::collections::LinkedList;

/// `LinkedListWitness` is a zero-sized type that acts as a Higher-Kinded Type (HKT) witness
/// for the `LinkedList<T>` type constructor. It allows `LinkedList` to be used with generic
/// functional programming traits like `Functor`, `Applicative`, `Foldable`, and `Monad`.
pub struct LinkedListWitness;

impl HKT for LinkedListWitness {
    /// Specifies that `LinkedListWitness` represents the `LinkedList<T>` type constructor.
    type Type<T> = LinkedList<T>;
}

// Implementation of Functor for LinkedListWitness
impl Functor<LinkedListWitness> for LinkedListWitness {
    /// Implements the `fmap` operation for `LinkedList<T>`.
    fn fmap<A, B, Func>(m_a: LinkedList<A>, f: Func) -> LinkedList<B>
    where
        Func: FnMut(A) -> B,
    {
        m_a.into_iter().map(f).collect()
    }
}

// Implementation of Foldable for LinkedListWitness
impl Foldable<LinkedListWitness> for LinkedListWitness {
    /// Folds (reduces) a `LinkedList` into a single value.
    fn fold<A, B, Func>(fa: LinkedList<A>, init: B, f: Func) -> B
    where
        Func: FnMut(B, A) -> B,
    {
        fa.into_iter().fold(init, f)
    }
}

// Implementation of Applicative for LinkedListWitness
impl Applicative<LinkedListWitness> for LinkedListWitness {
    /// Lifts a pure value into a `LinkedList` containing only that value.
    fn pure<T>(value: T) -> LinkedList<T> {
        let mut list = LinkedList::new();
        list.push_back(value);
        list
    }

    /// Applies a list of functions to a list of values.
    fn apply<A, B, Func>(f_ab: LinkedList<Func>, f_a: LinkedList<A>) -> LinkedList<B>
    where
        Func: FnMut(A) -> B,
        A: Clone,
    {
        f_ab.into_iter()
            .flat_map(|mut f_val| {
                f_a.iter()
                    .map(move |a_val| f_val(a_val.clone()))
                    .collect::<LinkedList<B>>()
            })
            .collect()
    }
}

// Implementation of Monad for LinkedListWitness
impl Monad<LinkedListWitness> for LinkedListWitness {
    /// Implements the `bind` (or `flat_map`) operation for `LinkedList<T>`.
    fn bind<A, B, Func>(m_a: LinkedList<A>, f: Func) -> LinkedList<B>
    where
        Func: FnMut(A) -> LinkedList<B>,
    {
        m_a.into_iter().flat_map(f).collect()
    }
}

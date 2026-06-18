/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! A sorted set of `usize` indices used throughout the clique-picking counter.
//!
//! Ported (counting-only API) from the authoritative `cliquepicking_rs::index_set`.
//! The set stores its elements in ascending order with no duplicates, which lets
//! membership use binary search and intersection/subset run as a linear merge.
//! Only the operations exercised by the counting path are kept here; the sampler
//! can grow the remaining ones later without disturbing this surface.

/// A set of indices stored as a sorted, duplicate-free `Vec<usize>`.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct IndexSet(Vec<usize>);

// Invariant: the backing vector is sorted ascending with no duplicate elements.
impl IndexSet {
    /// Creates an empty set.
    pub(crate) fn new() -> IndexSet {
        IndexSet(Vec::new())
    }

    /// Creates a set from an arbitrary vector, sorting it into the canonical
    /// ascending order. Duplicates are assumed absent.
    pub(crate) fn from(mut set: Vec<usize>) -> IndexSet {
        set.sort_unstable();
        IndexSet(set)
    }

    /// Creates a set from a vector already known to be sorted ascending and
    /// duplicate-free; no sorting is performed.
    pub(crate) fn from_sorted(set: Vec<usize>) -> IndexSet {
        IndexSet(set)
    }

    /// Iterates over the elements in ascending order.
    pub(crate) fn iter(&self) -> std::slice::Iter<'_, usize> {
        self.0.iter()
    }

    /// Returns the number of elements.
    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns the smallest element, or `None` if the set is empty.
    pub(crate) fn first(&self) -> Option<usize> {
        self.iter().copied().next()
    }

    /// Returns the element at position `pos` in ascending order.
    ///
    /// Panics if `pos` is out of bounds. Used by the sampler to map a flower-local
    /// candidate index back to a clique id.
    pub(crate) fn get(&self, pos: usize) -> usize {
        self.0[pos]
    }

    /// Returns `true` if `x` is a member, via binary search.
    pub(crate) fn contains(&self, x: usize) -> bool {
        self.0.binary_search(&x).is_ok()
    }

    /// Returns `true` if every element of `self` is in `other` (`self ⊆ other`),
    /// via a linear merge that exploits the sorted invariant.
    pub(crate) fn is_subset(&self, other: &IndexSet) -> bool {
        if self.len() > other.len() {
            return false;
        }
        let mut it = other.iter();
        for &el in self {
            loop {
                match it.next() {
                    Some(&x) => {
                        if x > el {
                            return false;
                        } else if x == el {
                            break;
                        }
                    }
                    None => return false,
                };
            }
        }
        true
    }

    /// Returns the intersection `self ∩ other` as a new sorted set.
    pub(crate) fn intersection(&self, other: &IndexSet) -> IndexSet {
        let mut intersection_vec = Vec::new();
        let mut it = other.iter().peekable();
        for &el in self {
            while let Some(&&x) = it.peek() {
                if x < el {
                    it.next();
                } else if x == el {
                    intersection_vec.push(el);
                    it.next();
                } else {
                    break;
                }
            }
        }
        IndexSet::from_sorted(intersection_vec)
    }

    /// Returns the set difference `self \ other` as a new sorted set, via a
    /// linear merge that exploits the sorted invariant.
    pub(crate) fn set_difference(&self, other: &IndexSet) -> IndexSet {
        let mut set_difference_vec = Vec::new();
        let mut it = other.iter().peekable();
        for &el in self {
            while let Some(&&x) = it.peek() {
                if x < el {
                    it.next();
                } else if x == el {
                    break;
                } else {
                    set_difference_vec.push(el);
                    break;
                }
            }
            if it.peek().is_none() {
                set_difference_vec.push(el);
            }
        }
        IndexSet::from_sorted(set_difference_vec)
    }

    /// Returns `true` if this set, viewed as a multiset of distinct indices,
    /// equals the elements of `vec` (order-insensitive).
    pub(crate) fn equal_to_vec(&self, vec: &[usize]) -> bool {
        if self.len() != vec.len() {
            return false;
        }
        for &el in vec {
            if !self.contains(el) {
                return false;
            }
        }
        true
    }

    /// Returns a cloned vector of the elements in ascending order.
    pub(crate) fn to_vec(&self) -> Vec<usize> {
        self.0.clone()
    }
}

impl<'a> IntoIterator for &'a IndexSet {
    type Item = &'a usize;
    type IntoIter = std::slice::Iter<'a, usize>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl Default for IndexSet {
    fn default() -> Self {
        IndexSet::new()
    }
}

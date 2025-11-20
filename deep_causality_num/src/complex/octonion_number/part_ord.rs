/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::complex::octonion_number::Octonion;
use crate::float::Float;
use std::cmp::Ordering;

// PartialOrd
impl<F: Float> PartialOrd for Octonion<F> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // Lexicographical comparison, not mathematically standard for octonions.
        self.s
            .partial_cmp(&other.s)
            .and_then(|ord| {
                if ord == Ordering::Equal {
                    self.e1.partial_cmp(&other.e1)
                } else {
                    Some(ord)
                }
            })
            .and_then(|ord| {
                if ord == Ordering::Equal {
                    self.e2.partial_cmp(&other.e2)
                } else {
                    Some(ord)
                }
            })
            .and_then(|ord| {
                if ord == Ordering::Equal {
                    self.e3.partial_cmp(&other.e3)
                } else {
                    Some(ord)
                }
            })
            .and_then(|ord| {
                if ord == Ordering::Equal {
                    self.e4.partial_cmp(&other.e4)
                } else {
                    Some(ord)
                }
            })
            .and_then(|ord| {
                if ord == Ordering::Equal {
                    self.e5.partial_cmp(&other.e5)
                } else {
                    Some(ord)
                }
            })
            .and_then(|ord| {
                if ord == Ordering::Equal {
                    self.e6.partial_cmp(&other.e6)
                } else {
                    Some(ord)
                }
            })
            .and_then(|ord| {
                if ord == Ordering::Equal {
                    self.e7.partial_cmp(&other.e7)
                } else {
                    Some(ord)
                }
            })
    }
}

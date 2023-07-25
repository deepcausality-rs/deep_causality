// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.


use std::fmt::{Display, Formatter};
use std::sync::atomic::{self, AtomicU64};

// Only the memory directly touched by the operation is synchronized.
// https://doc.rust-lang.org/nomicon/atomics.html#data-accesses
const ORDER: atomic::Ordering = atomic::Ordering::Relaxed;

#[derive(Debug)]
pub struct RelaxedAtomicCounter(AtomicU64);

impl RelaxedAtomicCounter
{
    /// Creates a new counter with atomic increment operation.
    pub fn new()
        -> Self
    {
        // No ordering constraints, only atomic operations.
        // https://doc.rust-lang.org/std/sync/atomic/enum.Ordering.html#variant.Relaxed
        RelaxedAtomicCounter(AtomicU64::new(0))
    }

    /// Increment and return new value.
    pub fn increment_and_get(
        &self
    )
        -> u64
    {
        self.0.fetch_add(1, ORDER) + 1
    }
}

impl Display for RelaxedAtomicCounter
{
    fn fmt(&self, f: &mut Formatter<'_>)
           -> std::fmt::Result
    {
        write!(f, "{}", self.0.load(ORDER))
    }
}
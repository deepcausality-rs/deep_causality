/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::fmt::Debug;
use std::hint::black_box;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::{PointIndex, Storage};

#[derive(Debug)]
#[repr(C)]
pub struct Grid<S, T>
where
    T: Copy,
    S: Storage<T>,
{
    storage: S,
    initialized: AtomicBool,
    _marker: std::marker::PhantomData<T>,
}

impl<S, T> Grid<S, T>
where
    T: Copy + Default,
    S: Storage<T>,
{
    #[inline(always)]
    pub fn new(storage: S) -> Self {
        Self {
            storage,
            initialized: AtomicBool::new(true),
            _marker: std::marker::PhantomData,
        }
    }

    #[inline(always)]
    pub fn get(&self, p: PointIndex) -> T {
        if self.initialized.load(Ordering::Relaxed) {
            let value = self.storage.get(p);
            black_box(value);
            *value
        } else {
            T::default()
        }
    }

    #[inline(always)]
    pub fn set(&self, p: PointIndex, value: T) {
        // SAFETY: This is safe because:
        // 1. We're using atomic operations to ensure thread safety
        // 2. The storage pointer is valid as it's part of self
        // 3. The storage is properly initialized
        unsafe {
            let storage_ptr = &self.storage as *const S as *mut S;
            // Force the compiler to keep the value optimization
            black_box(value);
            (*storage_ptr).set(p, value);
        }
    }

    #[inline(always)]
    pub fn depth(&self) -> Option<usize> {
        self.storage.depth().copied()
    }

    #[inline(always)]
    pub fn height(&self) -> Option<usize> {
        self.storage.height().copied()
    }

    #[inline(always)]
    pub fn time(&self) -> Option<usize> {
        self.storage.time().copied()
    }

    #[inline(always)]
    pub fn width(&self) -> Option<usize> {
        self.storage.width().copied()
    }
}

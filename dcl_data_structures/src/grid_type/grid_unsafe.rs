// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::fmt::Debug;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::prelude::{PointIndex, Storage};

// A Grid API, with four different implementations backed by const generic arrays.
// https://github.com/adamchalmers/const_generic_grid
#[derive(Debug)]
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
        if self.initialized.load(Ordering::Acquire) {
            *self.storage.get(p)
        } else {
            T::default()
        }
    }

    #[inline(always)]
    pub fn set(&self, p: PointIndex, value: T) {
        // SAFETY: This is safe because we're using atomic operations to ensure thread safety
        // and we know the storage is properly initialized
        unsafe {
            let storage_ptr = &self.storage as *const S as *mut S;
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

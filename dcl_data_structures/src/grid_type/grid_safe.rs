/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::cell::RefCell;
use std::fmt::Debug;

use crate::prelude::{PointIndex, Storage};

// A Grid API, with four different implementations backed by const generic arrays.
// https://github.com/adamchalmers/const_generic_grid
#[derive(Debug)]
pub struct Grid<S, T>
where
    T: Copy,
    S: Storage<T>,
{
    storage: RefCell<S>,
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
            storage: RefCell::new(storage),
            _marker: std::marker::PhantomData,
        }
    }

    #[inline(always)]
    pub fn get(&self, p: PointIndex) -> T {
        *self.storage.borrow().get(p)
    }

    #[inline(always)]
    pub fn set(&self, p: PointIndex, value: T) {
        self.storage.borrow_mut().set(p, value);
    }

    #[inline(always)]
    pub fn depth(&self) -> Option<usize> {
        self.storage.borrow().depth().copied()
    }

    #[inline(always)]
    pub fn height(&self) -> Option<usize> {
        self.storage.borrow().height().copied()
    }

    #[inline(always)]
    pub fn time(&self) -> Option<usize> {
        self.storage.borrow().time().copied()
    }

    #[inline(always)]
    pub fn width(&self) -> Option<usize> {
        self.storage.borrow().width().copied()
    }
}

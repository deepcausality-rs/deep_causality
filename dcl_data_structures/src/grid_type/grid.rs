// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::cell::RefCell;
use std::fmt::Debug;
use std::marker::PhantomData;

use crate::prelude::{PointIndex, Storage};

// A Grid API, with four different implementations backed by const generic arrays.
// https://github.com/adamchalmers/const_generic_grid
// https://practice.rs/generics-traits/const-generics.html
// Note, this API is not intended to be used directly, but rather through the GridArray Enum.
// Also, the storage API abstract over the underlying array type. See storage.rs for more details.

#[derive(Debug, Clone)]
pub struct Grid<S, T>
    where
        T: Copy,
        S: Storage<T>,
{
    inner: RefCell<S>,
    ty: PhantomData<T>,
}

impl<S, T> Grid<S, T>
    where
        T: Copy + Default,
        S: Storage<T>,
{
    pub fn new(storage: S) -> Self {
        Self
        {
            // interior mutability https://doc.rust-lang.org/book/ch15-05-interior-mutability.html
            inner: RefCell::new(storage),
            ty: PhantomData,
        }
    }

    pub fn get(&self, p: PointIndex) -> T { self.inner.borrow().get(p).to_owned() }

    pub fn set(&self, p: PointIndex, value: T) { self.inner.borrow_mut().set(p, value); }

    pub fn depth(&self) -> Option<usize> {
        // we have to deref inner value
        if self.inner.borrow().depth().is_some() {
            Some(*self.inner.borrow().depth().unwrap())
        } else {
            None
        }
    }

    pub fn height(&self) -> Option<usize> {
        if self.inner.borrow().height().is_some() {
            Some(*self.inner.borrow().height().unwrap())
        } else {
            None
        }
    }

    pub fn time(&self) -> Option<usize> {
        if self.inner.borrow().time().is_some() {
            Some(*self.inner.borrow().time().unwrap())
        } else {
            None
        }
    }

    pub fn width(&self) -> Option<usize> {
        if self.inner.borrow().width().is_some() {
            Some(*self.inner.borrow().width().unwrap())
        } else {
            None
        }
    }
}

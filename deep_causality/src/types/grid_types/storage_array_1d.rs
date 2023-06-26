// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.


use crate::prelude::{PointIndex, Storage};

// T Type
// H Height
impl<T, const H: usize> Storage<T> for [T; H]
    where
        T: Copy,
        [T; H]: Sized,
{
    fn get(&self, p: PointIndex) -> &T {
        &self[p.x]
    }

    fn set(&mut self, p: PointIndex, elem: T) {
        self[p.x] = elem
    }

    fn height(&self) -> Option<usize> {
        Some(H)
    }
}
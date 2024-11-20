// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use std::mem::align_of;
use crate::prelude::WindowStorage;

const ERR_EMPTY: &str = "Array is empty. Add some elements to the array first";
const ERR_NOT_FILLED: &str = "Array is not yet filled. Add some elements to the array first";

#[repr(C, align(64))]  // Cache line alignment
#[derive(Debug)]
pub struct UnsafeArrayStorage<T, const SIZE: usize, const CAPACITY: usize>
where
    T: PartialEq + Copy + Default,
    [T; CAPACITY]: Sized,
{
    arr: [T; CAPACITY],
    ptr: *mut T,        // Cached pointer to array
    size: usize,
    head: usize,
    tail: usize,
}

impl<T, const SIZE: usize, const CAPACITY: usize> UnsafeArrayStorage<T, SIZE, CAPACITY>
where
    T: PartialEq + Copy + Default,
    [T; CAPACITY]: Sized,
{
    #[inline(always)]
    pub fn new() -> Self {
        assert!(CAPACITY > SIZE, "CAPACITY must be greater than SIZE");
        assert!(align_of::<T>() >= 4, "Type must be at least 4-byte aligned");
        let mut storage = Self {
            arr: [T::default(); CAPACITY],
            ptr: std::ptr::null_mut(),
            size: SIZE,
            head: 0,
            tail: 0,
        };
        storage.ptr = storage.arr.as_mut_ptr();
        storage
    }

    #[inline(always)]
    const fn is_full(&self) -> bool {
        self.tail >= CAPACITY
    }

    #[inline(always)]
    const fn needs_head_adjustment(&self) -> bool {
        self.tail.saturating_sub(self.head) > self.size
    }

    #[inline(always)]
    unsafe fn rewind(&mut self) {
        // Use optimized copy for larger types
        if std::mem::size_of::<T>() >= 4 && align_of::<T>() >= 4 {
            let src = self.ptr.add(self.head);
            let dst = self.ptr;
            
            // Copy in chunks of 16 bytes when possible
            let simd_chunks = (self.size - 1) / 4;
            if simd_chunks > 0 {
                std::ptr::copy_nonoverlapping(
                    src as *const u8,
                    dst as *mut u8,
                    simd_chunks * 16
                );
                
                // Copy remaining elements
                let remaining = (self.size - 1) % 4;
                if remaining > 0 {
                    std::ptr::copy_nonoverlapping(
                        src.add(simd_chunks * 4),
                        dst.add(simd_chunks * 4),
                        remaining
                    );
                }
            } else {
                std::ptr::copy_nonoverlapping(src, dst, self.size - 1);
            }
        } else {
            // Fallback for smaller types
            std::ptr::copy_nonoverlapping(
                self.ptr.add(self.head),
                self.ptr,
                self.size - 1
            );
        }
        
        self.head = 0;
        self.tail = self.size;
    }
}

impl<T, const SIZE: usize, const CAPACITY: usize> Default for UnsafeArrayStorage<T, SIZE, CAPACITY>
where
    T: PartialEq + Copy + Default,
    [T; SIZE]: Sized,
{
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const SIZE: usize, const CAPACITY: usize> WindowStorage<T>
    for UnsafeArrayStorage<T, SIZE, CAPACITY>
where
    T: PartialEq + Copy + Default,
    [T; SIZE]: Sized,
{
    #[inline(always)]
    fn push(&mut self, value: T) {
        unsafe {
            if self.is_full() {
                self.rewind();
            }

            *self.ptr.add(self.tail) = value;

            if self.needs_head_adjustment() {
                self.head += 1;
            }

            self.tail += 1;
        }
    }

    #[inline(always)]
    fn first(&self) -> Result<T, String> {
        if self.tail == 0 {
            return Err(ERR_EMPTY.to_string());
        }

        unsafe {
            Ok(if self.tail > self.size {
                *self.ptr.add(self.head + 1)
            } else {
                *self.ptr.add(self.head)
            })
        }
    }

    #[inline(always)]
    fn last(&self) -> Result<T, String> {
        if !self.filled() {
            return Err(ERR_NOT_FILLED.to_string());
        }

        unsafe {
            Ok(*self.ptr.add(self.tail - 1))
        }
    }

    #[inline(always)]
    fn tail(&self) -> usize {
        self.tail
    }

    #[inline(always)]
    fn size(&self) -> usize {
        self.size
    }

    #[inline(always)]
    fn get_slice(&self) -> &[T] {
        unsafe {
            if self.tail > self.size {
                std::slice::from_raw_parts(
                    self.ptr.add(self.head + 1),
                    self.tail - (self.head + 1)
                )
            } else {
                std::slice::from_raw_parts(
                    self.ptr.add(self.head),
                    self.tail - self.head
                )
            }
        }
    }
}

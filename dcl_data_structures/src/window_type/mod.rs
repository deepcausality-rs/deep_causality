// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::marker::PhantomData;

use crate::prelude::{ArrayStorage, VectorStorage, WindowStorage};

#[cfg(feature = "unsafe")]
use crate::window_type::storage_unsafe::{
    unsafe_storage_array::UnsafeArrayStorage, unsafe_storage_vec::UnsafeVectorStorage,
};

pub(crate) mod storage;
pub(crate) mod storage_safe;
pub(crate) mod storage_unsafe;

/// Returns a new sliding window with a vector storage and the size and capacity given as parameters.
///
/// # Arguments
///
/// * `size: usize` - Maximum number of elements held in the sliding window.
/// * `multiple: usize` - Multiplier to calculate capacity as size * multiple
///
/// Capacity refers to the maximum number of elements before a rewind occurs.
///
/// # Example
///
/// ```
/// use dcl_data_structures::prelude::{VectorStorage,window_type, SlidingWindow};
///
/// // Size refers to the maximum number of elements the sliding window can store.
/// const SIZE: usize = 4;
/// // Multiplier to calculate capacity as size * multiple.
/// const MULT: usize = 300; // Capacity: 4 * 300 = 1200
///
/// // SlidingWindow requires PartialEq + Copy + Default
/// #[derive(Default, Debug, Copy, Clone, Hash, Eq, PartialEq)]
/// pub struct Data {
///    dats: i32,
/// }
///
/// let mut window: SlidingWindow<VectorStorage<Data>, Data>=window_type::new_with_vector_storage(SIZE, MULT);
///
/// assert!(window.empty());
/// assert_eq!(window.size(), SIZE);
/// ```
pub fn new_with_vector_storage<T>(
    size: usize,
    multiple: usize,
) -> SlidingWindow<VectorStorage<T>, T>
where
    T: PartialEq + Copy + Default,
{
    let storage = VectorStorage::new(size, multiple);
    SlidingWindow::with_storage(storage)
}

/// Creates a new [`SlidingWindow`] with an unsafe vector-based storage implementation.
///
/// This function provides a more performant but unsafe alternative to the safe vector storage.
/// It uses raw pointers internally for faster access but requires careful handling of memory.
///
/// # Feature Flag
///
/// This function requires the `unsafe` feature flag to be enabled:
/// ```toml
/// [dependencies]
/// dcl_data_structures = { version = "0.4.7", features = ["unsafe"] }
/// ```
///
/// # Arguments
///
/// * `size` - The fixed size of the sliding window
/// * `multiple` - The growth factor for internal buffer allocation
///
/// # Type Parameters
///
/// * `T` - The type of elements stored in the window, must implement `PartialEq + Copy + Default`
///
/// # Returns
///
/// Returns a new [`SlidingWindow`] instance with unsafe vector storage
///
/// # Examples
///
/// ```rust
/// # #[cfg(feature = "unsafe")]
/// # {
/// use dcl_data_structures::window_type::new_with_unsafe_vector_storage;
///
/// let mut window = new_with_unsafe_vector_storage::<f64>(5, 2);
/// window.push(1.0);
/// assert_eq!(window.first().unwrap(), 1.0);
/// # }
/// ```
///
/// # Safety
///
/// While the public interface is safe, the internal implementation uses unsafe code for
/// performance optimization. The unsafe implementation maintains the following invariants:
/// * Memory is properly allocated and deallocated
/// * No out-of-bounds access occurs
/// * No uninitialized memory is read
#[cfg(feature = "unsafe")]
pub fn new_with_unsafe_vector_storage<T>(
    size: usize,
    multiple: usize,
) -> SlidingWindow<UnsafeVectorStorage<T>, T>
where
    T: PartialEq + Copy + Default,
{
    let storage = UnsafeVectorStorage::new(size, multiple);
    SlidingWindow::with_storage(storage)
}

/// Returns a new sliding window with an const generic array storage and
/// the size and capacity given as generic parameters.
///
/// # Const Generic Arguments
/// *  SIZE: usize - Maximum number of elements held in the sliding window
/// * `CAPACITY: usize` - Maximum number of elements before a rewind occurs
///
/// Note, CAPACITY > SIZE and capacity should be a multiple of size.
/// For example, size 4 should be stored 300 times before rewind;
/// 4 * 300 = 1200
///
/// # Example
/// ```
/// use dcl_data_structures::prelude::{ArrayStorage, SlidingWindow,window_type};
///
/// // Size refers to the maximum number of elements the sliding window can store.
/// const SIZE: usize = 4;
/// // Capacity refers to the maximum number of elements before a rewind occurs.
/// // Note, CAPACITY > SIZE and capacity should be a multiple of size.
/// // For example, size 4 should be stored 300 times before rewind;
/// // 4 * 300 = 1200
/// const CAPACITY: usize = 1200;
///
/// // SlidingWindow requires PartialEq + Copy + Default
/// #[derive(Default, Debug, Copy, Clone, Hash, Eq, PartialEq)]
/// pub struct Data {
///    dats: i32,
/// }
///
/// // Util function that helps with type inference.
/// fn get_sliding_window() -> SlidingWindow<ArrayStorage<Data, SIZE, CAPACITY>, Data> {
///     window_type::new_with_array_storage()
/// }
///
///   let mut window = get_sliding_window();
///   assert_eq!(window.size(), SIZE);
///
/// ```
///
pub fn new_with_array_storage<T, const SIZE: usize, const CAPACITY: usize>(
) -> SlidingWindow<ArrayStorage<T, SIZE, CAPACITY>, T>
where
    T: PartialEq + Copy + Default,
    [T; SIZE]: Sized,
    [T; CAPACITY]: Sized,
{
    let storage = ArrayStorage::new();
    SlidingWindow::with_storage(storage)
}

/// Creates a new [`SlidingWindow`] with an unsafe array-based storage implementation.
///
/// This function provides a more performant but unsafe alternative to the safe array storage.
/// It uses raw pointers internally for faster access and stack allocation for better cache locality.
///
/// # Feature Flag
///
/// This function requires the `unsafe` feature flag to be enabled:
/// ```toml
/// [dependencies]
/// dcl_data_structures = { version = "0.4.7", features = ["unsafe"] }
/// ```
///
/// # Type Parameters
///
/// * `T` - The type of elements stored in the window, must implement `PartialEq + Copy + Default`
/// * `SIZE` - The fixed size of the sliding window, must be a const generic parameter
/// * `CAPACITY` - The total capacity of the internal buffer, must be greater than or equal to `SIZE`
///
/// # Generic Constraints
///
/// * `T: PartialEq + Copy + Default` - Elements must be comparable, copyable, and have a default value
/// * `[T; SIZE]: Sized` - The window size array must have a known size at compile time
/// * `[T; CAPACITY]: Sized` - The capacity array must have a known size at compile time
///
/// # Returns
///
/// Returns a new [`SlidingWindow`] instance with unsafe array storage
///
/// # Examples
///
/// ```rust
/// # #[cfg(feature = "unsafe")]
/// # {
/// use dcl_data_structures::window_type::new_with_unsafe_array_storage;
///
/// // Create a sliding window of size 3 with capacity 5
/// let mut window = new_with_unsafe_array_storage::<f64, 3, 5>();
/// assert!(window.empty());
///
/// // Fill the window
/// window.push(1.0);
/// window.push(2.0);
/// window.push(3.0);
///
/// // Now we can safely check first and last elements
/// assert!(window.first().is_ok());
/// assert_eq!(window.first().unwrap(), 1.0);
/// assert!(window.last().is_ok());
/// assert_eq!(window.last().unwrap(), 3.0);
/// # }
/// ```
///
/// # Safety
///
/// While the public interface is safe, the internal implementation uses unsafe code for
/// performance optimization. The unsafe implementation maintains the following invariants:
/// * All array accesses are bounds-checked at compile time through const generics
/// * No uninitialized memory is read
/// * Memory is stack-allocated, providing better cache locality
///
/// # Compile-time Requirements
///
/// * `CAPACITY` must be greater than or equal to `SIZE`
/// * Both `SIZE` and `CAPACITY` must be non-zero
#[cfg(feature = "unsafe")]
pub fn new_with_unsafe_array_storage<T, const SIZE: usize, const CAPACITY: usize>(
) -> SlidingWindow<UnsafeArrayStorage<T, SIZE, CAPACITY>, T>
where
    T: PartialEq + Copy + Default,
    [T; SIZE]: Sized,
    [T; CAPACITY]: Sized,
{
    let storage = UnsafeArrayStorage::new();
    SlidingWindow::with_storage(storage)
}

pub fn default_array_storage<
    T: PartialEq + Copy + Default,
    const SIZE: usize,
    const CAPACITY: usize,
>() -> SlidingWindow<ArrayStorage<T, SIZE, CAPACITY>, T> {
    assert!(CAPACITY > SIZE);

    SlidingWindow::with_storage(ArrayStorage::default())
}

pub struct SlidingWindow<S, T>
where
    T: PartialEq + Copy + Default,
    S: WindowStorage<T>,
{
    storage: S,
    ty: PhantomData<T>,
}

impl<S, T> SlidingWindow<S, T>
where
    T: PartialEq + Copy + Default,
    S: WindowStorage<T>,
{
    pub(crate) fn with_storage(storage: S) -> Self {
        Self {
            storage,
            ty: Default::default(),
        }
    }
}

impl<S, T> SlidingWindow<S, T>
where
    T: PartialEq + Copy + Default,
    S: WindowStorage<T>,
{
    /// Pushes a new element to the beginning of the sliding window.
    /// If the window is filled, the last element will be dropped.
    pub fn push(&mut self, value: T) {
        self.storage.push(value)
    }
    /// Returns the first element in the sliding window
    pub fn first(&self) -> Result<T, String> {
        self.storage.first()
    }
    /// Returns the last element in the sliding window
    pub fn last(&self) -> Result<T, String> {
        self.storage.last()
    }
    /// Returns true if the window is empty.
    pub fn empty(&self) -> bool {
        self.storage.empty()
    }
    /// Returns true if the window is filled.
    pub fn filled(&self) -> bool {
        self.storage.filled()
    }
    /// Returns the window size
    pub fn size(&self) -> usize {
        self.storage.size()
    }
    /// Returns the sliding window as a fixed size static array.
    pub fn arr<const SIZE: usize>(&self) -> Result<[T; SIZE], String> {
        self.storage.arr()
    }
    /// Returns sliding window as slice
    pub fn slice(&self) -> Result<&[T], String> {
        self.storage.slice()
    }
    /// Returns the sliding window as a vector.
    pub fn vec(&self) -> Result<Vec<T>, String> {
        self.storage.vec()
    }
}

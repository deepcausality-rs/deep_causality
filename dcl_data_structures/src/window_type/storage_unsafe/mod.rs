#[cfg(feature = "unsafe")]
pub(crate) mod unsafe_storage_array;
#[cfg(feature = "unsafe")]
pub(crate) mod unsafe_storage_vec;

#[cfg(feature = "unsafe")]
pub use unsafe_storage_array::UnsafeArrayStorage;
#[cfg(feature = "unsafe")]
pub use unsafe_storage_vec::UnsafeVectorStorage;

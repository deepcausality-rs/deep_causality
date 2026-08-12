# LLVM libc runtime headers

This Bazel package declares the subset of LLVM libc internal headers that
`libcxx` and `libcxxabi` require.

Those headers are consumed from the LLVM source set selected by
`llvm_source.version(...)` through the `@llvm-project//libc` package.

The list in `BUILD.bazel` is intentionally explicit to only expose the required
files and to keep runtime builds tied to the selected LLVM version.

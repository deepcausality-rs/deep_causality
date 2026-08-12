# GNU C Library (glibc) Support

#TODO(cerisier): Describe how compiling against any version of the glibc works here.

## Glibc stubs

The file `libc/glibc/abilist` is a Zig-specific binary blob that defines the supported
glibc versions and the set of symbols each version must define. 
See https://github.com/ziglang/glibc-abi-tool for the tooling to generate this blob.
The `glibc-stubs-generator` project uses this file to generate version-specific
stub libraries on demand.

The generated stub library is used for compile-time linking, with the expectation
that at run-time the real glibc library will provide the actual symbol implementations.

## Notes

When supporting new versions of the glibc, here are the steps required:
1. Regenerate the `abilist` file as described in https://github.com/ziglang/glibc-abi-tool
3. Ensure that headers for that version of the glibc is available in https://github.com/cerisier/glibc-headers
4. Ensure that that the kernel headers associated with that version of the glibc is
   available in https://github.com/cerisier/kernel-headers
5. Add the version in the various files that list glibc versions.
6. Ensure that files under `//third-party/libc/glibc` are still valid for this
   new version. 

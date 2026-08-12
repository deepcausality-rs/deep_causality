GCC_BUILTIN_INCLUDE_PATHS = [
    # Keep sorted
    "/usr/include",
    "/usr/include/aarch64-linux-gnu",
    "/usr/include/aarch64-linux-gnu/c++/%{gcc_version}",
    "/usr/include/c++/%{gcc_version}",
    "/usr/include/c++/%{gcc_version}/backward",
    "/usr/include/x86_64-linux-gnu",
    "/usr/include/x86_64-linux-gnu/c++/%{gcc_version}",
    "/usr/lib/gcc/aarch64-linux-gnu/%{gcc_version}/include",
    "/usr/lib/gcc/aarch64-linux-gnu/%{gcc_version}/include-fixed",
    "/usr/lib/gcc/x86_64-linux-gnu/%{gcc_version}/include",
    "/usr/lib/gcc/x86_64-linux-gnu/%{gcc_version}/include-fixed",
    "/usr/local/include",
    %{extra_cxx_builtin_include_directories}
]

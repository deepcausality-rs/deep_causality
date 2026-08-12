load("@bazel_skylib//rules:write_file.bzl", "write_file")

def make_glibc_linker_script(name, lib_name, lib_version, additional_linker_inputs = []):
    write_file(
        name = name,
        out = "lib{lib}.so".format(lib = lib_name),
        # GROUP(file) is the same as INPUT(file) when there is just one file.
        # Keep one input per line so callers can add configurable inputs.
        content = ["GROUP(", "lib{lib}.so.{version}".format(
            lib = lib_name,
            version = lib_version,
        )] + additional_linker_inputs + [")"],
    )
    return name

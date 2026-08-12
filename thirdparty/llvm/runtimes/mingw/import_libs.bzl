load("@bazel_lib//lib:run_binary.bzl", "run_binary")

def _generate_def_impl(ctx):
    # The templates have `@N` decorations which are only relevant to i386
    # and cause "symbol not found" on 64-bit, so we convert them to comments.
    # Ideally `llvm-dlltool --kill-at` would work, but it is ignored on non-i386.
    # TODO(zbarsky): Fix llvm-dlltool upstream.
    undecorated = ctx.actions.declare_file(ctx.file.src.path + "_undecorated")
    ctx.actions.expand_template(
        template = ctx.file.src,
        output = undecorated,
        substitutions = {
            "@": ";",
        },
    )

    args = ctx.actions.args()
    args.add_all(["-E", "-P", "-xc"])
    if ctx.attr.target_triple:
        args.add("-target", ctx.attr.target_triple)
    args.add("-D{}=1".format(ctx.attr.arch_macro))

    include_dirs = [
        # TODO(zbarsky): See if we can remove include_anchor...
        ctx.file.include_anchor.dirname,
        ctx.file.src.dirname,
    ]
    args.add_all(include_dirs, before_each = "-I")

    # ucrtbase-common.def.in:1800:28: warning: missing terminating ' character [-Winvalid-pp-token]
    # 1800 | F_LD64(_o_remainderl) ; Can't use long double functions from the CRT on x86
    args.add("-Wno-invalid-pp-token")

    args.add("-o", ctx.outputs.out)
    args.add(undecorated)

    inputs = [undecorated, ctx.file.include_anchor] + ctx.files.additional_includes

    ctx.actions.run(
        outputs = [ctx.outputs.out],
        inputs = inputs,
        executable = ctx.executable.tool,
        arguments = [args],
        mnemonic = "MingwGenerateDef",
        execution_requirements = {"supports-path-mapping": "1"},
    )

_generate_def = rule(
    implementation = _generate_def_impl,
    attrs = {
        "src": attr.label(
            allow_single_file = True,
            mandatory = True,
        ),
        "include_anchor": attr.label(
            allow_single_file = True,
            mandatory = True,
        ),
        "additional_includes": attr.label_list(
            allow_files = True,
        ),
        "arch_macro": attr.string(mandatory = True),
        "target_triple": attr.string(),
        "tool": attr.label(
            executable = True,
            allow_files = True,
            cfg = "exec",
            mandatory = True,
        ),
        "out": attr.output(mandatory = True),
    },
)

def _collect_definitions(directory):
    mappings = {}

    for path in sorted(native.glob([directory + "/*.def"])):
        name = path[path.rfind("/") + 1:-len(".def")]
        mappings[name] = path

    def_ins = native.glob([directory + "/*.def.in"], allow_empty = True)
    additional_includes = ["mingw-w64-crt/def-include/crt-aliases.def.in"] + def_ins

    for path in def_ins:
        base = path[path.rfind("/") + 1:-len(".def.in")]

        if "-common" in base:
            continue

        out = base + ".def"

        _generate_def(
            name = "generate_def_{}".format(base),
            src = path,
            include_anchor = "mingw-w64-crt/def-include/func.def.in",
            additional_includes = additional_includes,
            tool = "@llvm//tools:clang",
            arch_macro = select({
                "@platforms//cpu:x86_64": "__x86_64__",
                "@platforms//cpu:aarch64": "__aarch64__",
            }),
            target_triple = select({
                "@platforms//cpu:x86_64": "x86_64-w64-windows-gnu",
                "@platforms//cpu:aarch64": "aarch64-w64-windows-gnu",
            }),
            out = out,
        )

        mappings[base] = out

    return mappings

def mingw_import_libraries(name, directory):
    defs = _collect_definitions(directory)
    import_targets = []

    for lib_name, src in defs.items():
        target = "import_lib_" + lib_name
        run_binary(
            name = target,
            srcs = [src],
            outs = ["lib{}.a".format(lib_name)],
            tool = "@llvm//tools:llvm-dlltool",
            args = select({
                "@platforms//cpu:x86_64": ["-m", "i386:x86-64"],
                "@platforms//cpu:aarch64": ["-m", "arm64"],
            }) + [
                # The mingw def.in files are still decorated with stdcall @N suffixes;
                # strip them for 64-bit import libs so the symbols match Rust/LLVM output.
                # TODO(zbarsky): This doesn't actually work; fix llvm-dlltool upstream.
                #"--kill-at",
                "-d",
                "$(location %s)" % src,
                "-l",
                "$@",
            ],
        )

        # ucrtbase / ucrtbased are merged with extra objects elsewhere; keep
        # the generated targets available for direct deps, but do not expose
        # them via the directory filegroup to avoid shadowing the merged libs.
        # libws2_32 is merged with additional objects similarly.
        if lib_name not in ["ucrtbase", "ucrtbased", "ws2_32", "kernel32"]:
            import_targets.append(target)

    native.filegroup(
        name = name,
        srcs = import_targets,
    )

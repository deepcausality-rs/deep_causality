"""Bazel rules for Lean 4.

User-facing rules:
  lean_toolchain         — registers a Lean compiler binary + runtime tree.
                           Normally produced by `lake_workspace` (see lake.bzl);
                           can also be declared by hand against a hermetic
                           lean tarball.
  lean_prebuilt_library  — exposes a tree of prebuilt .olean files as a
                           LeanInfo provider consumable via the `deps` attr.
                           The `path_marker` file's parent directory becomes
                           the LEAN_PATH entry.
  lean_library           — compile a set of .lean sources to a persistent
                           .olean import-root tree (build outputs) and expose
                           it as LeanInfo. Lets one module be a *compiled*
                           dep of another (no source re-sharing). Transitive:
                           its LeanInfo carries its deps' closure too.
  lean_olean_archive     — bundle a lean_library's own .olean tree into a
                           tarball — the deployable cross-repo release artifact.
  lean_imported_library  — expose an unpacked .olean tarball (e.g. from an
                           `http_archive` of a release asset) as LeanInfo,
                           with NO recompile. The cross-repo consume side.
  lean_test              — stages a set of .lean sources into a module-path
                           layout and invokes the compiler on an entry point.
                           Returns 0 if all type-check, nonzero otherwise.
                           Accepts `deps = [LeanInfo]` and prepends each
                           dep's import root to LEAN_PATH.
  lean_emit              — like lean_test, but the entry file defines
                           `main : IO Unit`; runs it and captures stdout to
                           a declared output file. The Lean kernel becomes
                           the source of truth for emitted artifacts (SQL,
                           TTL, Markdown). Same `deps` plumbing as lean_test.

`lean_library`/`lean_olean_archive`/`lean_imported_library` (added 0.4.0) are
the cross-repo compiled-artifact seam: split a monolithic Lean library into
modules, publish each module's `.olean` tree as a per-`(lean-version, os, arch)`
release tarball, and have downstreams consume the prebuilt oleans without
recompiling. `.olean` is neither Lean-version- nor architecture-portable (it is
a compacted heap image), so a consumer must pin the SAME `lean-toolchain` and
`select()` the matching-platform artifact; Lean itself rejects a mismatched
olean loudly at use.
"""

# Used by `lean_regen_test` (see bottom of this file) — kept up here
# to satisfy Bazel's "all load()s before any other top-level statement"
# rule.
load("@bazel_skylib//rules:diff_test.bzl", _diff_test = "diff_test")

LeanToolchainInfo = provider(
    doc = "Lean 4 compiler binary + runtime tree.",
    fields = {
        "lean": "File: the lean compiler binary (executable).",
        "runtime": "depset[File]: stdlib oleans, shared libs, etc.",
    },
)

LeanInfo = provider(
    doc = "A Lean library: a directory of importable .olean files, exposed " +
          "via a marker file whose parent directory is the LEAN_PATH entry.",
    fields = {
        "markers": "depset[File]: each marker's parent directory IS a LEAN_PATH entry.",
        "files": "depset[File]: all .olean files (and the marker) needed when this lib is consumed.",
    },
)

def _lean_prebuilt_library_impl(ctx):
    marker = ctx.file.path_marker
    files = ctx.files.srcs + [marker]
    info = LeanInfo(
        markers = depset([marker]),
        files = depset(files),
    )
    return [
        DefaultInfo(files = depset(files)),
        info,
    ]

lean_prebuilt_library = rule(
    implementation = _lean_prebuilt_library_impl,
    attrs = {
        "srcs": attr.label_list(
            allow_files = True,
            mandatory = True,
            doc = "All files in the prebuilt-olean tree (typically `glob([\"lib/**\"])`).",
        ),
        "path_marker": attr.label(
            allow_single_file = True,
            mandatory = True,
            doc = "Anchor file inside the import-root directory. The marker's parent is the LEAN_PATH entry.",
        ),
    },
)

def _collect_dep_lean_info(deps):
    """Aggregate LeanInfo across deps. Returns (markers, files) depsets."""
    markers = []
    files = []
    for dep in deps:
        info = dep[LeanInfo]
        markers.append(info.markers)
        files.append(info.files)
    return depset(transitive = markers), depset(transitive = files)

def _dep_manifest_lines(dep_files, dep_marker_dirs, consumer_tops):
    """Topo-compile manifest lines for deps: oleans sharing one of the
    consumer's top-level namespaces are `stage`d into the compile root (Lean
    won't fall through LEAN_PATH roots for a namespace); disjoint deps go on
    `leanpath` (no copy)."""
    lines = []
    leanpath = {}
    for f in dep_files.to_list():
        if not f.path.endswith(".olean"):
            continue
        for d in dep_marker_dirs:
            if f.path.startswith(d + "/"):
                drel = f.path[len(d) + 1:]
                if drel.split("/")[0] in consumer_tops:
                    lines.append("stage\t" + f.path + "\t" + drel)
                else:
                    leanpath[d] = True
                break
    for d in leanpath:
        lines.append("leanpath\t" + d)
    return lines

def _lean_toolchain_impl(ctx):
    return [platform_common.ToolchainInfo(
        leantc = LeanToolchainInfo(
            lean = ctx.executable.lean,
            runtime = ctx.attr.runtime[DefaultInfo].files,
        ),
    )]

lean_toolchain = rule(
    implementation = _lean_toolchain_impl,
    attrs = {
        "lean": attr.label(
            executable = True,
            cfg = "exec",
            allow_single_file = True,
            mandatory = True,
        ),
        "runtime": attr.label(mandatory = True),
    },
)

def _module_path(src_short_path, file_package):
    """Relativize a source file against its OWN package to get its module path.

    The result is used to stage the file + derive its Lean module name.
    `file_package` is the package of the file itself (`src.owner.package`),
    NOT the consuming rule's package. This lets a rule mix sources from
    different packages — in particular cross-repo kernel sources like
    `@polyglot_ast//lean:Polyglot/C/Ast.lean` (package `lean`) consumed by
    a rule in a differently-named package (e.g. `//engine`). Both stage to
    `Polyglot/C/Ast.lean` regardless of the consumer's package.

    Handles external-repo sources where Bazel produces a short_path like
    `../<repo>+/<package>/<file>`; the `../<repo>+/` prefix is stripped
    first, leaving `<package>/<file>` which is then package-relativized.
    """
    if src_short_path.startswith("../"):
        rest = src_short_path[len("../"):]
        slash = rest.find("/")
        if slash >= 0:
            src_short_path = rest[slash + 1:]
    if file_package == "":
        return src_short_path
    if not src_short_path.startswith(file_package + "/"):
        fail("source %s is not inside its package %s" % (src_short_path, file_package))
    return src_short_path[len(file_package) + 1:]

def _lean_test_impl(ctx):
    tc = ctx.toolchains["@rules_lean//lean:toolchain_type"].leantc
    name = ctx.label.name

    rel_paths = []
    entry_rel = None
    consumer_tops = {}
    for src in ctx.files.srcs:
        rel = _module_path(src.short_path, src.owner.package)
        rel_paths.append((src, rel))
        consumer_tops[rel.split("/")[0]] = True
        if rel == ctx.attr.entry:
            entry_rel = rel

    if entry_rel == None:
        fail("entry %r not found among srcs (got %s)" % (ctx.attr.entry, [r for (_, r) in rel_paths]))

    dep_markers, dep_files = _collect_dep_lean_info(ctx.attr.deps)
    dep_marker_dirs = [m.path[:m.path.rfind("/")] for m in dep_markers.to_list()]

    # The type-check IS the test: compile every src (import-topological) at build
    # time via the driver. If anything fails to type-check, the marker action
    # fails → the test target fails to build → red. The test executable is then a
    # trivial pass (the marker is a build prerequisite). No compile shell.
    marker = ctx.actions.declare_file(name + ".typecheck_ok")
    lines = [
        "lean\t" + tc.lean.path,
        "work\t" + name + ".topo_work",
        "marker\t" + marker.path,
    ]
    for src, rel in rel_paths:
        lines.append("stage\t" + src.path + "\t" + rel)
        lines.append("module\t" + rel)
    lines += _dep_manifest_lines(dep_files, dep_marker_dirs, consumer_tops)

    manifest = ctx.actions.declare_file(name + ".topo_manifest")
    ctx.actions.write(output = manifest, content = "\n".join(lines) + "\n")
    ctx.actions.run(
        executable = tc.lean,
        arguments = ["--run", ctx.file._driver.path, manifest.path],
        outputs = [marker],
        inputs = depset(
            direct = [ctx.file._driver, manifest, tc.lean] + [s for (s, _) in rel_paths],
            transitive = [tc.runtime, dep_files],
        ),
        mnemonic = "LeanTest",
        progress_message = "Lean test %s" % name,
    )

    runner = ctx.actions.declare_file(name + ".sh")
    ctx.actions.write(
        output = runner,
        is_executable = True,
        content = "#!/bin/sh\nexit 0\n",
    )
    return [DefaultInfo(executable = runner, runfiles = ctx.runfiles(files = [marker]))]

lean_test = rule(
    implementation = _lean_test_impl,
    test = True,
    attrs = {
        "srcs": attr.label_list(
            allow_files = [".lean"],
            mandatory = True,
            doc = "All .lean files in the proof tree. Module path is derived from the file's path relative to this BUILD.bazel's package. Compiled in import-topological order, so list order is irrelevant — `glob([\"**/*.lean\"])` is fine.",
        ),
        "entry": attr.string(
            mandatory = True,
            doc = "Path of the entry-point .lean file relative to the package.",
        ),
        "deps": attr.label_list(
            providers = [LeanInfo],
            doc = "Prebuilt Lean libraries. Same-top-namespace deps are staged into the compile root; disjoint ones are on LEAN_PATH.",
        ),
        "_driver": attr.label(
            default = "@rules_lean//lean/private:topo_compile.lean",
            allow_single_file = True,
        ),
    },
    toolchains = ["@rules_lean//lean:toolchain_type"],
)

def _lean_emit_impl(ctx):
    tc = ctx.toolchains["@rules_lean//lean:toolchain_type"].leantc
    name = ctx.label.name
    output = ctx.outputs.out

    rel_paths = []
    entry_rel = None
    for src in ctx.files.srcs:
        rel = _module_path(src.short_path, src.owner.package)
        rel_paths.append((src, rel))
        if rel == ctx.attr.entry:
            entry_rel = rel

    if entry_rel == None:
        fail("entry %r not found among srcs (got %s)" %
             (ctx.attr.entry, [r for (_, r) in rel_paths]))

    # `data` files: staged alongside srcs in the work dir but NOT
    # compiled. Lets the entry script open them at runtime via a
    # workspace-relative path (the action runs from $WORK). Used e.g.
    # for `.dat` / `.txt` fixture inputs.
    #
    # External-repo data files (e.g. `@some_repo//path:file`) have
    # short_paths like `../+canon+some_repo/path/file`. We strip the
    # leading `../<repo>/` so the file lands under $WORK at its
    # natural workspace-relative path. Workspace-local data uses its
    # short_path verbatim. No package-prefix check (data files are
    # arbitrary fixtures, not Lean modules — they don't need to live
    # inside the rule's package).
    data_paths = []
    for d in ctx.files.data:
        sp = d.short_path
        if sp.startswith("../"):
            rest = sp[len("../"):]
            slash = rest.find("/")
            if slash >= 0:
                sp = rest[slash + 1:]
        data_paths.append((d, sp))

    dep_markers, dep_files = _collect_dep_lean_info(ctx.attr.deps)
    dep_marker_dirs = [m.path[:m.path.rfind("/")] for m in dep_markers.to_list()]
    consumer_tops = {rel.split("/")[0]: True for (_, rel) in rel_paths}

    # Manifest: compile the srcs (topo), stage `data` (uncompiled), then `--run`
    # the entry with its stdout captured to `out`. (See topo_compile.lean.)
    lines = [
        "lean\t" + tc.lean.path,
        "work\t" + name + ".topo_work",
        "entry\t" + entry_rel,
        "stdout\t" + output.path,
    ]
    for src, rel in rel_paths:
        lines.append("stage\t" + src.path + "\t" + rel)
        lines.append("module\t" + rel)
    for src, rel in data_paths:
        lines.append("stage\t" + src.path + "\t" + rel)
    lines += _dep_manifest_lines(dep_files, dep_marker_dirs, consumer_tops)

    manifest = ctx.actions.declare_file(name + ".topo_manifest")
    ctx.actions.write(output = manifest, content = "\n".join(lines) + "\n")

    ctx.actions.run(
        executable = tc.lean,
        arguments = ["--run", ctx.file._driver.path, manifest.path],
        outputs = [output],
        inputs = depset(
            direct = (
                [ctx.file._driver, manifest, tc.lean] +
                [src for (src, _) in rel_paths] +
                [src for (src, _) in data_paths]
            ),
            transitive = [tc.runtime, dep_files],
        ),
        mnemonic = "LeanEmit",
        progress_message = "Lean emit %s" % name,
    )

    return [DefaultInfo(files = depset([output]))]

lean_emit = rule(
    implementation = _lean_emit_impl,
    attrs = {
        "srcs": attr.label_list(
            allow_files = [".lean"],
            mandatory = True,
        ),
        "entry": attr.string(
            mandatory = True,
            doc = "Path of the entry-point .lean file (relative to the package) defining `main : IO Unit`. Stdout is captured to `out`.",
        ),
        "out": attr.output(
            mandatory = True,
            doc = "The emitted artifact (one file). Filename should reflect the artifact kind.",
        ),
        "deps": attr.label_list(providers = [LeanInfo]),
        "data": attr.label_list(
            allow_files = True,
            doc = "Non-Lean files staged alongside `srcs` in the action's work directory (NOT compiled). The Lean entry runs from that directory, so it can `IO.FS.readFile` them by their package-relative path. Typical use: fixture `.dat` / `.txt` / `.json` inputs the entry processes.",
        ),
        "_driver": attr.label(
            default = "@rules_lean//lean/private:topo_compile.lean",
            allow_single_file = True,
        ),
    },
    toolchains = ["@rules_lean//lean:toolchain_type"],
)

# =============================================================================
# lean_regen_test: assert a committed file matches the current
# `lean_emit` output for a given Lean main. Captures the "Lean spec is
# the source of truth; the committed Rust/C/whatever was emitted from
# it" idiom that consumers like rules_postgres' Pg.Ir cluster gates
# build their `Gate 1 — regen idempotence` checks on.
#
# Expands to a `lean_emit` (running Lean as a sandboxed Bazel action +
# capturing stdout) plus a skylib `diff_test` (byte-exact comparison
# against the committed `expected` label). Fails the build whenever
# the committed file has drifted from what the Lean source-of-truth
# currently emits — exactly the failure mode `Lean spec edited, regen
# forgotten` introduces.
#
# Usage:
#
#   load("@rules_lean//lean:lean.bzl", "lean_regen_test")
#
#   lean_regen_test(
#       name = "regen_int_arith",                # diff_test target name
#       srcs = [...],                            # ordered .lean deps
#       entry = "Pg/Ir/Emit/IntArith.lean",      # has `main : IO Unit`
#       expected = "//rust/pg_int4_arith:lib_rs",
#   )
#
# `bazel test //path:regen_int_arith` fails with the diff if the Lean
# emit and `expected` disagree.
# =============================================================================
def lean_regen_test(name, srcs, entry, expected, out = None, deps = None, data = None, tags = None):
    """Assert a committed file matches the current `lean_emit` output.

    Args:
      name: target name for the generated diff_test (e.g.
        `regen_int_arith`). The helper `lean_emit` is named
        `<name>_emit`.
      srcs: list of `.lean` source labels needed to compile the
        entry. Compiled in import-topological order, so list order
        is irrelevant (a `glob()` is fine). Must include the entry.
      entry: path of the entry-point `.lean` file (relative to the
        rule's package) defining `main : IO Unit`. Stdout is captured.
      expected: Bazel label of the committed file the lean_emit
        output is diffed against.
      out: optional filename for the emitted artifact (defaults to
        `<name>_emit.out`).
      deps: optional list of `LeanInfo`-providing deps for prebuilt
        olean closures (passed through to `lean_emit`).
      tags: optional tags propagated to the generated `diff_test`
        target only.
    """
    if out == None:
        out = name + "_emit.out"

    emit_name = name + "_emit"

    lean_emit(
        name = emit_name,
        srcs = srcs,
        entry = entry,
        out = out,
        deps = deps if deps else [],
        data = data if data else [],
    )

    _diff_test(
        name = name,
        file1 = ":" + emit_name,
        file2 = expected,
        tags = tags if tags else [],
    )

# =============================================================================
# lean_main_test: compile + run a Lean entry as a test. Passes iff
# the entry's `main : IO UInt32` exits 0. No expected-output diff
# needed — exit code IS the test result.
#
# Use case: gates that check a Lean script self-validates (e.g.
# round-trip stability, structural equivalence) where the script
# already returns the right exit code. Drops the need for a
# committed `expected.txt` fixture.
# =============================================================================

def _lean_main_test_impl(ctx):
    tc = ctx.toolchains["@rules_lean//lean:toolchain_type"].leantc
    name = ctx.label.name

    rel_paths = []
    entry_rel = None
    consumer_tops = {}
    for src in ctx.files.srcs:
        rel = _module_path(src.short_path, src.owner.package)
        rel_paths.append((src, rel))
        consumer_tops[rel.split("/")[0]] = True
        if rel == ctx.attr.entry:
            entry_rel = rel

    if entry_rel == None:
        fail("entry %r not found among srcs (got %s)" %
             (ctx.attr.entry, [r for (_, r) in rel_paths]))

    data_paths = []
    for d in ctx.files.data:
        sp = d.short_path
        if sp.startswith("../"):
            rest = sp[len("../"):]
            slash = rest.find("/")
            if slash >= 0:
                sp = rest[slash + 1:]
        data_paths.append((d, sp))

    dep_markers, dep_files = _collect_dep_lean_info(ctx.attr.deps)
    dep_marker_dirs = [m.path[:m.path.rfind("/")] for m in dep_markers.to_list()]

    # Compile every src + `--run` the entry at build time via the driver. If the
    # entry's `main` exits non-zero (or anything fails to compile), the action
    # fails → the test target fails to build → red. The marker (written last)
    # proves a clean run; the test executable is a trivial pass. No shell.
    marker = ctx.actions.declare_file(name + ".run_ok")
    lines = [
        "lean\t" + tc.lean.path,
        "work\t" + name + ".topo_work",
        "entry\t" + entry_rel,
        "marker\t" + marker.path,
    ]
    for src, rel in rel_paths:
        lines.append("stage\t" + src.path + "\t" + rel)
        lines.append("module\t" + rel)
    for src, rel in data_paths:
        lines.append("stage\t" + src.path + "\t" + rel)
    lines += _dep_manifest_lines(dep_files, dep_marker_dirs, consumer_tops)

    manifest = ctx.actions.declare_file(name + ".topo_manifest")
    ctx.actions.write(output = manifest, content = "\n".join(lines) + "\n")
    ctx.actions.run(
        executable = tc.lean,
        arguments = ["--run", ctx.file._driver.path, manifest.path],
        outputs = [marker],
        inputs = depset(
            direct = (
                [ctx.file._driver, manifest, tc.lean] +
                [src for (src, _) in rel_paths] +
                [src for (src, _) in data_paths]
            ),
            transitive = [tc.runtime, dep_files],
        ),
        mnemonic = "LeanMainTest",
        progress_message = "Lean main test %s" % name,
    )

    runner = ctx.actions.declare_file(name + ".sh")
    ctx.actions.write(output = runner, is_executable = True, content = "#!/bin/sh\nexit 0\n")
    return [DefaultInfo(executable = runner, runfiles = ctx.runfiles(files = [marker]))]

lean_main_test = rule(
    implementation = _lean_main_test_impl,
    test = True,
    attrs = {
        "srcs": attr.label_list(
            allow_files = [".lean"],
            mandatory = True,
            doc = "All .lean files needed to compile the entry. Compiled in import-topological order, so list order is irrelevant (a `glob()` is fine).",
        ),
        "entry": attr.string(
            mandatory = True,
            doc = "Path of the entry-point .lean file (relative to the package) defining `main : IO UInt32` (test result = exit code).",
        ),
        "deps": attr.label_list(providers = [LeanInfo]),
        "data": attr.label_list(
            allow_files = True,
            doc = "Non-Lean files staged at their workspace-relative path in the action's work directory. The Lean entry runs from that directory, so it can `IO.FS.readFile` them.",
        ),
        "_driver": attr.label(
            default = "@rules_lean//lean/private:topo_compile.lean",
            allow_single_file = True,
        ),
    },
    toolchains = ["@rules_lean//lean:toolchain_type"],
)

# =============================================================================
# lean_library: compile .lean sources to a persistent .olean import-root tree
# (build outputs) and expose it as LeanInfo, so one module can be a *compiled*
# dependency of another. DefaultInfo carries only THIS library's own tree (the
# unit a `lean_olean_archive` packages); LeanInfo carries the transitive
# closure (own + deps) so downstream consumers list only direct deps.
# =============================================================================

_MARKER_NAME = ".lean_root"

def _lean_library_impl(ctx):
    tc = ctx.toolchains["@rules_lean//lean:toolchain_type"].leantc
    name = ctx.label.name
    root_dir = name + "_lib"

    dep_markers, dep_files = _collect_dep_lean_info(ctx.attr.deps)
    dep_marker_dirs = [m.path[:m.path.rfind("/")] for m in dep_markers.to_list()]

    # srcs → (rel, declared .olean output); collect this lib's top-level
    # namespaces (used to decide which deps must share the compile root).
    units = []  # (src File, rel, olean File)
    consumer_tops = {}
    for src in ctx.files.srcs:
        rel = _module_path(src.short_path, src.owner.package)
        if not rel.endswith(".lean"):
            fail("lean_library srcs must be .lean files; got %s" % rel)
        consumer_tops[rel.split("/")[0]] = True
        olean = ctx.actions.declare_file("{}/{}".format(root_dir, rel[:-len(".lean")] + ".olean"))
        units.append((src, rel, olean))

    marker = ctx.actions.declare_file("{}/{}".format(root_dir, _MARKER_NAME))

    # Build the topo-compile driver manifest (see lean/private/topo_compile.lean).
    lines = [
        "lean\t" + tc.lean.path,
        "work\t" + name + ".topo_work",
        "marker\t" + marker.path,
    ]
    for src, rel, olean in units:
        lines.append("stage\t" + src.path + "\t" + rel)
        lines.append("module\t" + rel)
        lines.append("output\t" + rel + "\t" + olean.path)

    lines += _dep_manifest_lines(dep_files, dep_marker_dirs, consumer_tops)

    manifest = ctx.actions.declare_file(name + ".topo_manifest")
    ctx.actions.write(output = manifest, content = "\n".join(lines) + "\n")

    own_files = [olean for (_, _, olean) in units] + [marker]
    ctx.actions.run(
        executable = tc.lean,
        arguments = ["--run", ctx.file._driver.path, manifest.path],
        outputs = own_files,
        inputs = depset(
            direct = [ctx.file._driver, manifest, tc.lean] + [s for (s, _, _) in units],
            transitive = [tc.runtime, dep_files],
        ),
        mnemonic = "LeanLibrary",
        progress_message = "Lean library %s" % name,
    )

    return [
        DefaultInfo(files = depset(own_files)),
        LeanInfo(
            markers = depset([marker], transitive = [dep_markers]),
            files = depset(own_files, transitive = [dep_files]),
        ),
    ]

lean_library = rule(
    implementation = _lean_library_impl,
    attrs = {
        "srcs": attr.label_list(
            allow_files = [".lean"],
            mandatory = True,
            doc = "All .lean files in this library. Module path is derived from the file's path relative to its own package. Compiled in import-topological order, so list order is irrelevant (a `glob()` is fine).",
        ),
        "deps": attr.label_list(
            providers = [LeanInfo],
            doc = "Compiled Lean libraries this one imports. Same-top-namespace deps are staged into the compile root; disjoint ones are on LEAN_PATH. All propagate transitively in this library's LeanInfo.",
        ),
        "_driver": attr.label(
            default = "@rules_lean//lean/private:topo_compile.lean",
            allow_single_file = True,
            doc = "The Lean topo-compile driver (run via `lean --run`).",
        ),
    },
    toolchains = ["@rules_lean//lean:toolchain_type"],
    doc = "Compile .lean sources to a persistent .olean import-root tree and expose it as LeanInfo.",
)

# =============================================================================
# lean_binary: a runnable Lean executable. Compiles all srcs to an .olean
# import root (like lean_library), then emits a runner that `lean --run`s the
# entry against that root, forwarding command-line args to the program's
# `main (args : List String)`. Unlike lean_emit (build-time, fixed stdout) /
# lean_main_test (build-time test), this runs at runtime with real argv — so a
# Lean lowering can be a real CLI tool (e.g. a codec deparse reading a file).
# =============================================================================

def _lean_binary_impl(ctx):
    tc = ctx.toolchains["@rules_lean//lean:toolchain_type"].leantc
    name = ctx.label.name
    root_dir = name + "_bin"

    entry_rel = None
    entry_src = None
    units = []  # (src, rel, olean)
    consumer_tops = {}
    for src in ctx.files.srcs:
        rel = _module_path(src.short_path, src.owner.package)
        if not rel.endswith(".lean"):
            fail("lean_binary srcs must be .lean files; got %s" % rel)
        consumer_tops[rel.split("/")[0]] = True
        olean = ctx.actions.declare_file("{}/{}".format(root_dir, rel[:-len(".lean")] + ".olean"))
        units.append((src, rel, olean))
        if rel == ctx.attr.entry:
            entry_rel = rel
            entry_src = src
    if entry_rel == None:
        fail("entry %r not found among srcs" % ctx.attr.entry)

    dep_markers, dep_files = _collect_dep_lean_info(ctx.attr.deps)
    dep_marker_dirs = [m.path[:m.path.rfind("/")] for m in dep_markers.to_list()]

    marker = ctx.actions.declare_file("{}/{}".format(root_dir, _MARKER_NAME))
    lines = [
        "lean\t" + tc.lean.path,
        "work\t" + name + ".topo_work",
        "marker\t" + marker.path,
    ]
    for src, rel, olean in units:
        lines.append("stage\t" + src.path + "\t" + rel)
        lines.append("module\t" + rel)
        lines.append("output\t" + rel + "\t" + olean.path)
    lines += _dep_manifest_lines(dep_files, dep_marker_dirs, consumer_tops)

    manifest = ctx.actions.declare_file(name + ".topo_manifest")
    ctx.actions.write(output = manifest, content = "\n".join(lines) + "\n")
    own_oleans = [olean for (_, _, olean) in units]
    ctx.actions.run(
        executable = tc.lean,
        arguments = ["--run", ctx.file._driver.path, manifest.path],
        outputs = own_oleans + [marker],
        inputs = depset(
            direct = [ctx.file._driver, manifest, tc.lean] + [s for (s, _, _) in units],
            transitive = [tc.runtime, dep_files],
        ),
        mnemonic = "LeanBinary",
        progress_message = "Lean binary %s" % name,
    )

    # Runner: `lean --run <entry-source> "$@"` with LEAN_PATH = the olean root.
    lean_rf = tc.lean.short_path
    if lean_rf.startswith("../"):
        lean_rf = lean_rf[len("../"):]
    pkg = ctx.label.package
    root_rf = (pkg + "/" if pkg else "") + root_dir
    runner = ctx.actions.declare_file(name)
    ctx.actions.write(
        output = runner,
        is_executable = True,
        content = """#!/usr/bin/env bash
set -euo pipefail
if [[ -n "${{RUNFILES_DIR:-}}" ]]; then rf="$RUNFILES_DIR"
elif [[ -d "$0.runfiles" ]]; then rf="$0.runfiles"
else rf="${{BASH_SOURCE[0]}}.runfiles"; fi
ws="{ws}"
exec env LEAN_PATH="$rf/$ws/{root}" "$rf/{lean}" --run "$rf/$ws/{entry}" "$@"
""".format(ws = ctx.workspace_name, root = root_rf, lean = lean_rf, entry = entry_src.short_path),
    )

    runfiles = ctx.runfiles(
        files = own_oleans + [marker, entry_src, tc.lean],
        transitive_files = depset(transitive = [tc.runtime, dep_files]),
    )
    return [DefaultInfo(executable = runner, runfiles = runfiles)]

lean_binary = rule(
    implementation = _lean_binary_impl,
    executable = True,
    attrs = {
        "srcs": attr.label_list(allow_files = [".lean"], mandatory = True),
        "entry": attr.string(mandatory = True, doc = "Module-path of the src whose `main` is the entry point."),
        "deps": attr.label_list(providers = [LeanInfo]),
        "_driver": attr.label(
            default = "@rules_lean//lean/private:topo_compile.lean",
            allow_single_file = True,
        ),
    },
    toolchains = ["@rules_lean//lean:toolchain_type"],
    doc = "A runnable Lean executable: compiles srcs to an olean root and `lean --run`s the entry with runtime argv.",
)

# =============================================================================
# lean_olean_archive: tar a lean_library's OWN .olean import-root tree into a
# deployable artifact. The tarball unpacks to an import root (`Foo/Bar.olean`,
# `.lean_root` at top) consumable by `lean_imported_library`. One archive per
# `(lean-version, os, arch)` — build it on each target platform (oleans are not
# cross-compilable); the release/upload step names the asset per-platform.
# =============================================================================

def _lean_olean_archive_impl(ctx):
    own_files = ctx.attr.library[DefaultInfo].files.to_list()
    marker = None
    for f in own_files:
        if f.basename == _MARKER_NAME:
            marker = f
    if marker == None:
        fail("library %s has no %s marker; is it a lean_library?" % (ctx.attr.library.label, _MARKER_NAME))
    root = marker.dirname

    out = ctx.actions.declare_file(ctx.attr.out if ctx.attr.out else (ctx.label.name + ".tar.gz"))

    # `tar -C <root> .` packs the import-root contents at the tarball top.
    # -h dereferences: in the build action the import-root is a symlink farm into
    # bazel-out, so without -h the archive would hold dangling links to build-time
    # paths instead of the real .olean bytes. Portable across GNU and bsd tar
    # (macOS); entries are gzip-compressed.
    ctx.actions.run_shell(
        outputs = [out],
        inputs = depset(own_files),
        command = 'tar -czhf "{out}" -C "{root}" .'.format(out = out.path, root = root),
        mnemonic = "LeanOleanArchive",
        progress_message = "Lean olean archive %s" % ctx.label.name,
    )
    return [DefaultInfo(files = depset([out]))]

lean_olean_archive = rule(
    implementation = _lean_olean_archive_impl,
    attrs = {
        "library": attr.label(
            providers = [LeanInfo],
            mandatory = True,
            doc = "The `lean_library` whose own .olean tree is archived.",
        ),
        "out": attr.string(doc = "Output tarball name (default `<name>.tar.gz`)."),
    },
    doc = "Bundle a lean_library's .olean import-root tree into a deployable tarball.",
)

# =============================================================================
# lean_imported_library: expose an unpacked .olean tarball (e.g. extracted by
# an `http_archive` of a release asset) as LeanInfo, with NO recompile. This is
# the cross-repo consume side of lean_olean_archive. Identical mechanics to
# lean_prebuilt_library; named + documented for the import-from-release case.
# The consumer must pin the SAME lean-toolchain version and `select()` the
# matching-platform archive — Lean rejects a mismatched olean loudly at use.
# =============================================================================

lean_imported_library = rule(
    implementation = _lean_prebuilt_library_impl,
    attrs = {
        "srcs": attr.label_list(
            allow_files = True,
            mandatory = True,
            doc = "All files of the unpacked .olean tree (typically `@<archive_repo>//:all` or a `glob`).",
        ),
        "path_marker": attr.label(
            allow_single_file = True,
            mandatory = True,
            doc = "Anchor file inside the unpacked import root (the archive's `.lean_root`). Its parent dir becomes the LEAN_PATH entry.",
        ),
    },
    doc = "Expose an unpacked .olean release tarball as LeanInfo (no recompile).",
)

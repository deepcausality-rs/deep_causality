#!/usr/bin/env python3
#
# SPDX-License-Identifier: MIT
# Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
#
# Pack the prebuilt Lean olean trees into the archive that lean/lake-lock.json pins.
#
# Invoked by `scripts/lean_lock.sh oleans`. Two jobs:
#
#   1. Compute the transitive import closure of the tracked proofs, so the archive
#      carries the modules the proofs reach rather than all of Mathlib. Measured at
#      the pinned revs: 1,918 of 9,450 modules, 1.46 GB instead of 6.1 GB.
#
#   2. Emit a tar rooted at `.lake/packages`, i.e. entries look like
#      `mathlib/.lake/build/lib/lean/...`, which is what the repo rule extracts over
#      the sources fetched from the lock.
#
# The closure follows EVERY import edge, not only `public import`. Lean needs
# `.olean.private` and `.ir` for transitively reached modules, not just for direct
# imports -- verified by pruning to the public-only set and watching the type-check
# fail on `Batteries/Logic.olean.private`. Likewise every artifact per module is
# packed, not a chosen subset: pruning to `.olean` fails on `.olean.server`, adding
# that fails on `.olean.private`, adding that fails on `.ir`. Per-module-everything
# is the rule that type-checks all 65 proofs.

import gzip
import os
import re
import subprocess
import sys
import tarfile

# Lean 4's module system: `module` headers with `public`/`private`/`meta` import
# modifiers and `import all`. A bare-`import` regex silently matches nothing in
# Mathlib at these revs, which reads as a tiny closure rather than a parse failure.
EDGE = re.compile(
    r'^[ \t]*(?:(?:public|private|meta|protected)[ \t]+)*import[ \t]+(?:all[ \t]+)?'
    r'([A-Za-z0-9_.À-￿]+)',
    re.M,
)


def read(path):
    return open(path, encoding="utf-8", errors="replace").read()


def module_index(packages_dir):
    """module name -> (package, source file), over every Lake package."""
    index = {}
    for pkg in sorted(os.listdir(packages_dir)):
        root = os.path.join(packages_dir, pkg)
        if not os.path.isdir(root):
            continue
        for dirpath, dirnames, files in os.walk(root):
            dirnames[:] = [d for d in dirnames if d != ".lake"]
            for fn in files:
                if fn.endswith(".lean"):
                    rel = os.path.relpath(os.path.join(dirpath, fn), root)[: -len(".lean")]
                    index.setdefault(rel.replace(os.sep, "."), (pkg, os.path.join(dirpath, fn)))
    return index


def closure(index, roots):
    seen, stack, cache = set(), list(roots), {}
    while stack:
        mod = stack.pop()
        if mod in seen:
            continue
        seen.add(mod)
        if mod not in cache:
            cache[mod] = [m for m in EDGE.findall(read(index[mod][1])) if m in index]
        stack.extend(m for m in cache[mod] if m not in seen)
    return seen


def main() -> int:
    packages_dir, out_path = sys.argv[1], sys.argv[2]

    index = module_index(packages_dir)
    tracked = subprocess.run(
        ["git", "ls-files", "lean/**/*.lean"], capture_output=True, text=True
    ).stdout.split()
    roots = {m for f in tracked for m in EDGE.findall(read(f)) if m in index}
    if not roots:
        print("lean_oleans: no external imports resolved -- refusing to pack an empty "
              "archive. Check that %s is a materialized workspace." % packages_dir,
              file=sys.stderr)
        return 1

    mods = closure(index, roots)
    print("lean_oleans: %d direct imports -> %d modules of %d (%.1f%%)"
          % (len(roots), len(mods), len(index), 100 * len(mods) / len(index)), file=sys.stderr)

    # Deterministic archive: the same workspace must hash to the same sha256, or the
    # lock pins an artifact nobody can reproduce or audit. tar records mtime/uid/gid
    # per entry and gzip stamps its own header, so all four are normalised and the
    # entries are emitted in sorted order.
    def normalise(info):
        info.mtime = 0
        info.uid = info.gid = 0
        info.uname = info.gname = ""
        info.mode = 0o755 if info.isdir() else 0o644
        return info

    # `filename=""` matters as much as `mtime=0`: given a fileobj, GzipFile defaults to
    # storing that file's name in the FNAME header field, so an archive packed to
    # `a.tar.gz` and the same archive packed to `b.tar.gz` hash differently. The pin
    # must depend on the content, not on where it was written.
    total = 0
    with open(out_path, "wb") as raw, \
            gzip.GzipFile(filename="", fileobj=raw, mode="wb", compresslevel=6, mtime=0) as gz, \
            tarfile.open(fileobj=gz, mode="w", format=tarfile.GNU_FORMAT) as tar:
        for mod in sorted(mods):
            pkg = index[mod][0]
            parts = mod.split(".")
            base = os.path.join(packages_dir, pkg, ".lake", "build", "lib", "lean", *parts)
            d = os.path.dirname(base)
            if not os.path.isdir(d):
                continue
            stem = parts[-1] + "."
            for fn in sorted(os.listdir(d)):
                if not fn.startswith(stem):
                    continue
                src = os.path.join(d, fn)
                if not os.path.isfile(src):
                    continue
                arc = os.path.join(pkg, ".lake", "build", "lib", "lean", *parts[:-1], fn)
                tar.add(src, arcname=arc, filter=normalise)
                total += os.path.getsize(src)

    print("lean_oleans: packed %.1f MB uncompressed -> %s (%.1f MB)"
          % (total / 1e6, out_path, os.path.getsize(out_path) / 1e6), file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())

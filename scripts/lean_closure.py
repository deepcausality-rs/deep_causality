#!/usr/bin/env python3
#
# SPDX-License-Identifier: MIT
# Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
#
# Guard the tree-shaken Lean artifacts against import drift.
#
# WHAT GOES WRONG WITHOUT THIS
#
# Two artifacts are cut to the set of Mathlib modules these proofs reach:
#
#   * `cache_roots` in //MODULE.bazel, which tree-shakes `lake exe cache get`
#   * the `oleans` archive pinned in lean/lake-lock.json (1,918 of 9,450 modules)
#
# Add an import that falls outside that set and neither covers it. The failure
# surfaces as `unknown module` from Lean, or as a missing `.olean.private` deep in a
# build, pointing at a Mathlib file nobody touched. This turns that into a named
# error at the point of change.
#
# WHY IT NEEDS NEITHER LEAN NOR A WORKSPACE
#
# The closure is a function of exactly two inputs:
#
#   closure = f(direct imports of the tracked proofs, package sources at pinned revs)
#
# Both are in the repo: the imports are readable from lean/**/*.lean, and the revs
# are pinned in the lock (which rules_lean already validates against
# lake-manifest.json on every fetch). So a fingerprint over those two inputs detects
# every case where the closure could have moved, without resolving the module graph,
# materializing a Lake workspace, or touching the network. `check` runs in about a
# second on a bare checkout.
#
# It is deliberately conservative: it fingerprints the INPUTS, so it fires on any
# import change, including ones that happen not to widen the closure. A false alarm
# costs a repack; a miss costs a confusing build failure.

import hashlib
import json
import os
import re
import subprocess
import sys

# Lean 4's module system: `module` headers with `public`/`private`/`meta` import
# modifiers and `import all`. A bare-`import` regex silently matches nothing in
# Mathlib at these revs, which reads as "no imports" rather than as a parse failure.
EDGE = re.compile(
    r'^[ \t]*(?:(?:public|private|meta|protected)[ \t]+)*import[ \t]+(?:all[ \t]+)?'
    r'([A-Za-z0-9_.À-￿]+)',
    re.M,
)

CACHE_ROOTS = re.compile(r'cache_roots\s*=\s*\[(.*?)\]', re.S)
QUOTED = re.compile(r'"([^"]+)"')


def repo_root():
    return os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))


def project_namespace(root):
    """The Lake package name, e.g. DeepCausalityFormal — imports under it are ours."""
    text = open(os.path.join(root, "lean", "lakefile.toml"), encoding="utf-8").read()
    m = re.search(r'^\s*name\s*=\s*"([^"]+)"', text, re.M)
    if not m:
        raise SystemExit("lean_closure: no package name in lean/lakefile.toml")
    return m.group(1)


def direct_imports(root, namespace):
    """Sorted external imports of every tracked proof file."""
    # Pathspec is the plain directory, with the extension filtered here: `lean/**/*.lean`
    # relies on git's glob-pathspec handling, and this is a CI gate that has to behave
    # identically on whatever git the runner ships.
    tracked = [
        p for p in subprocess.run(
            ["git", "ls-files", "--", "lean"],
            cwd=root, capture_output=True, text=True, check=True,
        ).stdout.splitlines()
        if p.endswith(".lean")
    ]
    if not tracked:
        raise SystemExit("lean_closure: git listed no proof files under lean/")
    found = set()
    for rel in tracked:
        text = open(os.path.join(root, rel), encoding="utf-8", errors="replace").read()
        for mod in EDGE.findall(text):
            if mod != namespace and not mod.startswith(namespace + "."):
                found.add(mod)
    return sorted(found)


def fingerprint(imports, lock):
    """Digest over the two inputs the closure depends on."""
    h = hashlib.sha256()
    h.update(("toolchain\t%s\n" % lock.get("lean_toolchain", "")).encode())
    for pkg in sorted(lock.get("packages", []), key=lambda p: p["name"]):
        h.update(("package\t%s\t%s\n" % (pkg["name"], pkg.get("rev", ""))).encode())
    for mod in imports:
        h.update(("import\t%s\n" % mod).encode())
    return h.hexdigest()


def load(root):
    lock_path = os.path.join(root, "lean", "lake-lock.json")
    if not os.path.exists(lock_path):
        raise SystemExit("lean_closure: no lock at %s; run `lean_lock.sh sources`" % lock_path)
    lock = json.load(open(lock_path))
    ns = project_namespace(root)
    return lock_path, lock, direct_imports(root, ns)


def cmd_fingerprint(root):
    _, lock, imports = load(root)
    print(json.dumps({"inputs_sha256": fingerprint(imports, lock), "imports": imports}))
    return 0


def cmd_check(root):
    lock_path, lock, imports = load(root)
    problems = []

    # 1. cache_roots must list every Mathlib module the proofs import directly.
    #    `filterByRootModules` expands each root to its closure, so listing the direct
    #    imports is sufficient; a module absent from the list is never fetched.
    module_bazel = open(os.path.join(root, "MODULE.bazel"), encoding="utf-8").read()
    m = CACHE_ROOTS.search(module_bazel)
    roots = set(QUOTED.findall(m.group(1))) if m else set()
    mathlib_imports = {i for i in imports if i.startswith("Mathlib")}
    missing = sorted(mathlib_imports - roots)
    if missing:
        problems.append(
            "cache_roots in MODULE.bazel is missing %d import(s):\n    %s\n"
            "  Add them, or the Mathlib fetch will not include them."
            % (len(missing), "\n    ".join(missing))
        )
    stale = sorted(roots - mathlib_imports)
    if stale:
        print(
            "lean_closure: note — cache_roots lists %d module(s) nothing imports any more:\n"
            "    %s\n  Harmless, but they widen the fetch."
            % (len(stale), "\n    ".join(stale)),
            file=sys.stderr,
        )

    # 2. Every pinned URL must be fetchable from somewhere other than one laptop.
    #    A `file://` pin is what you get while bootstrapping an archive locally; it
    #    builds green on the machine that made it and fails everywhere else, so it must
    #    never reach main.
    local = [
        "%s: %s" % (name, url)
        for name, url in
        [("oleans", lock.get("oleans", {}).get("url", ""))]
        + [(p["name"], p.get("url", "")) for p in lock.get("packages", [])]
        if url and not url.startswith(("http://", "https://"))
    ]
    if local:
        problems.append(
            "lean/lake-lock.json pins %d machine-local URL(s):\n    %s\n"
            "  These resolve only on the machine that wrote them. Upload the artifact and\n"
            "  pin the hosted URL; the sha256 is unchanged, the archive is reproducible."
            % (len(local), "\n    ".join(local))
        )

    # 3. The pinned olean archive must have been cut from these same inputs.
    oleans = lock.get("oleans", {})
    if oleans.get("url"):
        want = fingerprint(imports, lock)
        got = oleans.get("inputs_sha256", "")
        if not got:
            problems.append(
                "lean/lake-lock.json pins an olean archive with no `inputs_sha256`, so\n"
                "  there is no way to tell whether it still covers these imports.\n"
                "  Repack with `scripts/lean_lock.sh oleans <URL>`."
            )
        elif got != want:
            problems.append(
                "the pinned olean archive is stale: it was cut for a different set of\n"
                "  imports or package revs.\n"
                "    archive: %s\n    current: %s\n"
                "  Repack with `scripts/lean_lock.sh oleans <URL>`." % (got, want)
            )

    if problems:
        print("\nlean_closure: FAILED\n", file=sys.stderr)
        for p in problems:
            print("  - %s\n" % p, file=sys.stderr)
        return 1

    print("lean_closure: OK — %d direct imports (%d Mathlib), cache_roots covers them%s"
          % (len(imports), len(mathlib_imports),
             ", olean archive matches" if oleans.get("url") else ", no olean archive pinned"))
    return 0


def main() -> int:
    root = repo_root()
    cmd = sys.argv[1] if len(sys.argv) > 1 else "check"
    if cmd == "check":
        return cmd_check(root)
    if cmd == "fingerprint":
        return cmd_fingerprint(root)
    raise SystemExit("usage: lean_closure.py {check|fingerprint}")


if __name__ == "__main__":
    sys.exit(main())

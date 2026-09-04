#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
# Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
"""Check the documented dependency tiers against the manifests.

Three places state the tier of a crate: the tier block in ``AGENTS.md``, and the ASCII block and
the crate table in ``deep_causality_unified_math/README.md``. This derives the tiers from the
``[dependencies]`` tables of the workspace members and compares, so the documents are checked
rather than read.

A crate's tier is ``1 + max(tier of its internal dependencies)``, or 0 when it has none.
Dev- and build-dependencies are excluded, matching what ``AGENTS.md`` says it lists.

The two documents scope their tiers differently and are checked against their own scope:
``AGENTS.md`` covers the library crates of the whole workspace, while the README covers the
sixteen crates under ``deep_causality_unified_math/`` alone. The same crate therefore holds
different tier numbers in the two, which is correct.

``graph.png`` is not checked here. It is a rendering, and its tiers are verified by eye against
this output.

Usage: python3 scripts/check_tiers.py [repo-root]
Exit status is 0 when every representation agrees with the manifests, 1 otherwise.
"""

import glob
import os
import re
import sys

DEP_SECTION = re.compile(r"^\[([^\]]+)\]")
DEP_ENTRY = re.compile(r"^([A-Za-z0-9_\-]+)\s*=")
OPTIONAL = re.compile(r"optional\s*=\s*true")


def read_manifests(root):
    """Return {crate: {"deps": set, "opt": set, "path": str}} for every workspace member."""
    members = re.search(
        r"members\s*=\s*\[(.*?)\]", open(os.path.join(root, "Cargo.toml")).read(), re.S
    ).group(1)

    paths = set()
    for pattern in re.findall(r'"([^"]+)"', members):
        for hit in glob.glob(os.path.join(root, pattern, "Cargo.toml")):
            if "/yanked/" in hit or "/reverted/" in hit or "/.claude/" in hit:
                continue
            paths.add(hit)

    crates = {}
    for path in sorted(paths):
        text = open(path).read()
        name = re.search(r'^\s*name\s*=\s*"([^"]+)"', text, re.M)
        if not name:
            continue
        section, deps, opt = None, set(), set()
        for line in text.splitlines():
            stripped = line.strip()
            header = DEP_SECTION.match(stripped)
            if header:
                section = header.group(1)
                continue
            # Both spellings occur: `foo = { ... }` under [dependencies], and a
            # [dependencies.foo] section of its own. Missing the second reads as absence.
            if section == "dependencies":
                entry = DEP_ENTRY.match(stripped)
                if entry:
                    deps.add(entry.group(1))
                    if OPTIONAL.search(stripped):
                        opt.add(entry.group(1))
            elif section and section.startswith("dependencies.") and section.count(".") == 1:
                dep = section.split(".", 1)[1]
                deps.add(dep)
                if OPTIONAL.match(stripped):
                    opt.add(dep)
        crates[name.group(1)] = {
            "deps": deps,
            "opt": opt,
            "path": os.path.relpath(path, root),
        }
    return crates


def tiers(manifests, scope):
    """Tier of every crate in `scope`, counting only dependencies inside `scope`."""
    memo = {}

    def tier(name):
        if name not in memo:
            inner = manifests[name]["deps"] & scope
            memo[name] = 1 + max(tier(d) for d in inner) if inner else 0
        return memo[name]

    return {n: tier(n) for n in scope}


def parse_agents_block(text):
    """Parse the fenced tier block in AGENTS.md into {crate: (tier, deps)}."""
    block = re.search(r"```\n(Tier 0 .*?)\n```", text, re.S).group(1)
    out, tier, current = {}, None, None
    for line in block.split("\n"):
        header = re.match(r"^Tier (\d+)", line)
        if header:
            tier = int(header.group(1))
            continue
        entry = re.match(r"^  (\S+)\s*(?:→\s*(.*))?$", line)
        if entry:
            current = entry.group(1)
            out[current] = [tier, entry.group(2) or ""]
        elif line.startswith("     ") and line.strip() and current:
            out[current][1] += " " + line.strip()
    return {
        name: (tier, {d.strip().replace(" (opt)", "") for d in deps.split(",") if d.strip()})
        for name, (tier, deps) in out.items()
    }


def main():
    root = sys.argv[1] if len(sys.argv) > 1 else "."
    manifests = read_manifests(root)
    failures = []

    def check(cond, message):
        if not cond:
            failures.append(message)

    # AGENTS.md: the library crates of the whole workspace.
    agents = parse_agents_block(open(os.path.join(root, "AGENTS.md")).read())
    scope = set(agents)
    check(
        scope <= set(manifests),
        f"AGENTS.md lists non-members: {sorted(scope - set(manifests))}",
    )
    scope &= set(manifests)
    derived = tiers(manifests, scope)
    for name, (tier, deps) in agents.items():
        if name not in manifests:
            continue
        check(tier == derived[name], f"AGENTS.md {name}: tier {tier}, derived {derived[name]}")
        expected = manifests[name]["deps"] & scope
        check(
            deps == expected,
            f"AGENTS.md {name}: missing={sorted(expected - deps)} extra={sorted(deps - expected)}",
        )

    # README: the crates under deep_causality_unified_math/ alone.
    readme_path = os.path.join(root, "deep_causality_unified_math", "README.md")
    readme = open(readme_path).read()
    math = {n for n, c in manifests.items() if c["path"].startswith("deep_causality_unified_math/")}
    derived = tiers(manifests, math)

    ascii_block = {}
    for line in re.search(r"```\n(tier \d+.*?)\n```", readme, re.S).group(1).split("\n"):
        row = re.match(r"^tier (\d+)\s+(.*)$", line)
        for crate in row.group(2).split():
            ascii_block["deep_causality_" + crate] = int(row.group(1))

    table = {
        m.group(1): int(m.group(2))
        for m in re.finditer(r"^\| `(deep_causality_\w+)` \| (\d+) \|", readme, re.M)
    }

    for label, found in (("block", ascii_block), ("table", table)):
        check(
            set(found) == math,
            f"README {label} crate set differs from the folder: {sorted(set(found) ^ math)}",
        )
        for name, tier in found.items():
            if name in derived:
                check(
                    tier == derived[name],
                    f"README {label} {name}: tier {tier}, derived {derived[name]}",
                )
    check(ascii_block == table, "README block and table disagree with each other")

    # The README states how many dependencies leave the folder, and over how many edges.
    leaving = sorted({d for n in math for d in manifests[n]["deps"] if d in manifests and d not in math})
    edges = sorted((n, d) for n in math for d in manifests[n]["deps"] if d in leaving)
    claim = re.search(r"(\w+) dependencies leave the folder, over (\w+) edges", readme, re.I)
    words = {"one": 1, "two": 2, "three": 3, "four": 4, "five": 5, "six": 6}
    check(claim is not None, "README no longer states what leaves the folder")
    if claim:
        check(
            words.get(claim.group(1).lower()) == len(leaving)
            and words.get(claim.group(2).lower()) == len(edges),
            f"README claims {claim.group(1)} deps over {claim.group(2)} edges; "
            f"found {len(leaving)} over {len(edges)}: {leaving}",
        )

    if failures:
        print("\n".join(failures))
        print(f"\n{len(failures)} mismatch(es) between the documented tiers and the manifests.")
        return 1

    print(
        f"tiers agree with the manifests: AGENTS.md {len(agents)} crates, "
        f"README {len(math)} crates (block and table), "
        f"{len(leaving)} dependencies leaving the folder over {len(edges)} edges"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())

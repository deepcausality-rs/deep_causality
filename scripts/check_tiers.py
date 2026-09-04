#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
# Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
"""Check the documented dependency tiers against the manifests.

Three places state the tier of a crate: the tier block in ``AGENTS.md``, and the ASCII block and
the crate table in ``deep_causality_unified_math/README.md``. This derives the tiers from the
runtime dependencies of the workspace members, as reported by ``cargo metadata``, and compares,
so the documents are checked rather than read.

A crate's tier is ``1 + max(tier of its internal dependencies)``, or 0 when it has none.
Dev- and build-dependencies are excluded, matching what ``AGENTS.md`` says it lists.

The two documents scope their tiers differently and are checked against their own scope:
``AGENTS.md`` covers the library crates of the whole workspace, while the README covers the
sixteen crates under ``deep_causality_unified_math/`` alone. The same crate therefore holds
different tier numbers in the two, which is correct.

Both scopes are checked in both directions. A workspace member outside ``examples/`` that no
tier block names fails just as loudly as a documented crate whose tier has drifted, so adding a
crate without documenting it cannot pass.

``graph.png`` is not checked here. It is a rendering, and its tiers are verified by eye against
this output.

Usage: python3 scripts/check_tiers.py [repo-root]
Exit status is 0 when every representation agrees with the manifests, 1 otherwise.
"""

import json
import os
import re
import subprocess
import sys


def read_manifests(root):
    """Return {crate: {"deps": set, "opt": set, "path": str}} for every workspace member.

    ``cargo metadata`` is the resolver rather than a regex over the manifests. It reports the
    *real* package name of a renamed dependency (``foo = { package = "bar" }``), and it reports
    dependencies declared under ``[target.'cfg(...)'.dependencies]`` — two spellings a
    section-and-key regex silently misreads or drops, either of which would let the tier check
    pass on an incomplete graph. ``--no-deps --offline`` restricts it to the workspace's own
    manifests, so it needs neither the network nor a registry index.

    Only runtime dependencies count (``kind`` is null); dev- and build-dependencies are excluded,
    matching what ``AGENTS.md`` says it lists.
    """
    command = ["cargo", "metadata", "--no-deps", "--offline", "--format-version", "1"]
    try:
        out = subprocess.run(
            command, cwd=root, check=True, capture_output=True, text=True
        ).stdout
    except OSError as e:
        sys.exit(f"cannot run `{' '.join(command)}`: {e}")
    except subprocess.CalledProcessError as e:
        sys.exit(f"`{' '.join(command)}` failed with status {e.returncode}:\n{e.stderr}")

    meta = json.loads(out)
    members = set(meta["workspace_members"])
    root = os.path.abspath(root)

    crates = {}
    for pkg in meta["packages"]:
        if pkg["id"] not in members:
            continue
        runtime = [d for d in pkg["dependencies"] if d["kind"] is None]
        crates[pkg["name"]] = {
            "deps": {d["name"] for d in runtime},
            "opt": {d["name"] for d in runtime if d["optional"]},
            "path": os.path.relpath(pkg["manifest_path"], root),
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

    # AGENTS.md: the library crates of the whole workspace — every workspace member that is not
    # an example. Checked in both directions, so a crate added to the workspace without an
    # AGENTS.md entry fails here instead of being quietly left out of the derived graph.
    agents = parse_agents_block(open(os.path.join(root, "AGENTS.md")).read())
    library = {n for n, c in manifests.items() if not c["path"].startswith("examples/")}
    check(
        set(agents) == library,
        f"AGENTS.md scope differs from the workspace library members: "
        f"undocumented={sorted(library - set(agents))} "
        f"not a library member={sorted(set(agents) - library)}",
    )
    scope = set(agents) & set(manifests)
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

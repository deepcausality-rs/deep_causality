#!/usr/bin/env python3
"""Fetch the Reservoir package index and emit a manifest of Lean packages.

Reservoir (https://reservoir.lean-lang.org) publishes the entire registry as a
single JSON bundle at /index/manifest.json. This script downloads it, wraps
the payload with a small provenance header (fetch URL + timestamp), and writes
it to disk. Uses only the Python standard library.

Usage:
  tools/reservoir_manifest.py                       # write ./reservoir-manifest.json
  tools/reservoir_manifest.py -o out.json --pretty  # pretty-printed
  tools/reservoir_manifest.py --compact             # drop versions/builds/dependents
  tools/reservoir_manifest.py --source file.json    # read local file instead of HTTP
"""

from __future__ import annotations

import argparse
import datetime
import gzip
import io
import json
import os
import sys
import urllib.request

DEFAULT_SOURCE = "https://reservoir.lean-lang.org/index/manifest.json"

# Fields stripped in --compact mode. Each is a list-valued field on a package
# entry that can balloon the output (builds is ~70% of the bundle size).
HEAVY_PACKAGE_FIELDS = ("versions", "builds", "dependents")


def fetch(source: str) -> bytes:
    if "://" not in source:
        with open(source, "rb") as f:
            return f.read()
    req = urllib.request.Request(
        source,
        headers={
            "Accept": "application/json",
            "Accept-Encoding": "gzip",
            "User-Agent": "rules_lean-reservoir-manifest/1",
        },
    )
    with urllib.request.urlopen(req) as resp:
        raw = resp.read()
        if resp.headers.get("Content-Encoding", "").lower() == "gzip":
            raw = gzip.decompress(raw)
        return raw


def compact_packages(packages: list[dict]) -> list[dict]:
    out = []
    for pkg in packages:
        slim = {k: v for k, v in pkg.items() if k not in HEAVY_PACKAGE_FIELDS}
        slim["versionCount"] = len(pkg.get("versions") or ())
        slim["buildCount"] = len(pkg.get("builds") or ())
        slim["dependentCount"] = len(pkg.get("dependents") or ())
        latest = (pkg.get("versions") or [None])[-1]
        if isinstance(latest, dict):
            slim["latestVersion"] = {
                k: latest.get(k)
                for k in ("version", "revision", "tag", "date", "toolchain")
            }
        out.append(slim)
    return out


def build_manifest(source: str, payload: dict, compact: bool) -> dict:
    packages = payload.get("packages") or []
    if compact:
        packages = compact_packages(packages)
    return {
        "schemaVersion": 1,
        "generator": "rules_lean/tools/reservoir_manifest.py",
        "source": source,
        "fetchedAt": datetime.datetime.now(datetime.timezone.utc)
        .replace(microsecond=0)
        .isoformat()
        .replace("+00:00", "Z"),
        "upstreamBundledAt": payload.get("bundledAt"),
        "compact": compact,
        "packageCount": len(packages),
        "toolchainCount": len(payload.get("toolchains") or ()),
        "toolchains": payload.get("toolchains") or [],
        "packages": packages,
        "packageAliases": payload.get("packageAliases") or {},
    }


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument(
        "-o",
        "--output",
        default="reservoir-manifest.json",
        help="Output path; '-' for stdout (default: ./reservoir-manifest.json)",
    )
    ap.add_argument(
        "--source",
        default=DEFAULT_SOURCE,
        help=f"URL or local path to the upstream bundle (default: {DEFAULT_SOURCE})",
    )
    ap.add_argument(
        "--compact",
        action="store_true",
        help="Drop versions/builds/dependents arrays from each package",
    )
    ap.add_argument(
        "--pretty",
        action="store_true",
        help="Pretty-print with indent=2 (default: single-line JSON)",
    )
    args = ap.parse_args(argv)

    raw = fetch(args.source)
    payload = json.loads(raw)
    manifest = build_manifest(args.source, payload, args.compact)

    indent = 2 if args.pretty else None
    serialized = json.dumps(manifest, indent=indent, sort_keys=False)
    if args.output == "-":
        sys.stdout.write(serialized)
        if indent is not None:
            sys.stdout.write("\n")
    else:
        with open(args.output, "w") as f:
            f.write(serialized)
            if indent is not None:
                f.write("\n")
        sys.stderr.write(
            f"wrote {args.output}: {len(manifest['packages'])} packages, "
            f"{len(manifest['toolchains'])} toolchains\n"
        )
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

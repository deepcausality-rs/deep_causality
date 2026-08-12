#!/usr/bin/env python3
"""Generate LLVM source archive index from GitHub releases."""

from __future__ import annotations

import argparse
import json
import os
import re
import sys
import urllib.error
import urllib.parse
import urllib.request
from typing import Any


RELEASES_API = "https://api.github.com/repos/llvm/llvm-project/releases?per_page=100"
TAG_RE = re.compile(r"^llvmorg-(\d+)\.(\d+)\.(\d+)(?:-rc(\d+))?$")


def _semver_key(version: str) -> tuple[int, int, int, int, int]:
    m = re.match(r"^(\d+)\.(\d+)\.(\d+)(?:-rc(\d+))?$", version)
    if not m:
        raise ValueError(f"invalid semver: {version}")
    major, minor, patch = (int(m.group(1)), int(m.group(2)), int(m.group(3)))
    rc = m.group(4)
    is_final = 1 if rc is None else 0
    rc_num = int(rc) if rc is not None else 0
    return (major, minor, patch, is_final, rc_num)


def _parse_next_url(link_header: str | None) -> str | None:
    if not link_header:
        return None
    for part in link_header.split(","):
        bits = [x.strip() for x in part.split(";")]
        if len(bits) != 2:
            continue
        url_part, rel_part = bits
        if rel_part != 'rel="next"':
            continue
        if url_part.startswith("<") and url_part.endswith(">"):
            return url_part[1:-1]
    return None


def _github_json(url: str, token: str | None) -> tuple[Any, str | None]:
    headers = {
        "Accept": "application/vnd.github+json",
        "User-Agent": "toolchains_llvm_bootstrapped/gen_llvm_versions_index.py",
    }
    if token:
        headers["Authorization"] = f"Bearer {token}"
    req = urllib.request.Request(url, headers=headers)
    try:
        with urllib.request.urlopen(req) as resp:
            data = json.loads(resp.read().decode("utf-8"))
            next_url = _parse_next_url(resp.headers.get("Link"))
            return data, next_url
    except urllib.error.HTTPError as exc:
        body = exc.read().decode("utf-8", errors="replace")
        raise RuntimeError(f"GitHub API request failed ({exc.code}) for {url}: {body}") from exc


def _extract_version_from_tag(tag: str) -> str | None:
    m = TAG_RE.match(tag)
    if not m:
        return None
    version = f"{m.group(1)}.{m.group(2)}.{m.group(3)}"
    if m.group(4):
        version += f"-rc{m.group(4)}"
    return version


def _build_index(min_major: int) -> tuple[dict[str, dict[str, str]], list[str]]:
    token = os.environ.get("GITHUB_TOKEN") or os.environ.get("GH_TOKEN")
    url = RELEASES_API
    result: dict[str, dict[str, str]] = {}
    warnings: list[str] = []

    while url:
        releases, url = _github_json(url, token)
        for release in releases:
            tag = release.get("tag_name", "")
            version = _extract_version_from_tag(tag)
            if not version:
                continue
            major = int(version.split(".", 1)[0])
            if major < min_major:
                continue
            src_asset_name = f"llvm-project-{version}.src.tar.xz"
            src_asset = next((a for a in release.get("assets", []) if a.get("name") == src_asset_name), None)
            if not src_asset:
                warnings.append(f"{version}: missing asset {src_asset_name}")
                continue
            digest = src_asset.get("digest", "")
            if not isinstance(digest, str) or not digest.startswith("sha256:"):
                warnings.append(f"{version}: missing sha256 digest for {src_asset_name}")
                continue
            sha256 = digest.split(":", 1)[1]
            result[version] = {
                "url": src_asset["browser_download_url"],
                "sha256": sha256,
            }

    sorted_items = sorted(result.items(), key=lambda item: _semver_key(item[0]))
    return dict(sorted_items), warnings


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--output",
        default="llvm_versions.json",
        help="Output JSON path (default: llvm_versions.json)",
    )
    parser.add_argument(
        "--min-major",
        type=int,
        default=21,
        help="Minimum LLVM major version to include (default: 21)",
    )
    args = parser.parse_args()

    index, warnings = _build_index(args.min_major)
    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(index, f, indent=2, sort_keys=False)
        f.write("\n")

    print(f"Wrote {len(index)} LLVM versions to {args.output}")
    for warning in warnings:
        print(f"warning: {warning}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

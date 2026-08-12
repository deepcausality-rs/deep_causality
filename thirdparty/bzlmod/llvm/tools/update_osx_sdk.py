#!/usr/bin/env python3

import re
import os
import sys
import urllib.request
import urllib.parse
import dataclasses
import argparse
import difflib
from dataclasses import dataclass
from typing import Any, Literal, get_args
from collections import defaultdict
import logging
import subprocess
import json
import hashlib
from pathlib import Path

logging.basicConfig(
    level=logging.INFO,
    datefmt="%Y-%m-%dT%H:%M:%S%z",
    format="[%(asctime)s][%(levelname)s] %(message)s",
)

logger = logging.getLogger(__name__)

# https://gist.github.com/meyer/b14c87d162366f0428a99cd2ff0d0b8b
# Updated for macOS 16
SUCATALOG_URL = "https://swscan.apple.com/content/catalogs/others/index-26-15-14-13-12-10.16-10.15-10.14-10.13-10.12-10.11-10.10-10.9-mountainlion-lion-snowleopard-leopard.merged-1.sucatalog"

# Where we put downloaded pkgs (stable across the run, easy to inspect/cache)
TMP_ROOT = Path("/tmp/apple_sucatalog")
TMP_ROOT.mkdir(parents=True, exist_ok=True)

# Packages we're interested in
PackageName = Literal[
    "CLTools_Executables",
    "CLTools_macOS_SDK",
    "CLTools_macOSLMOS_SDK",
    "CLTools_macOSNMOS_SDK",
]


class EnhancedJSONEncoder(json.JSONEncoder):
    def default(self, o: Any) -> Any:
        if dataclasses.is_dataclass(o) and not isinstance(o, type):
            return dataclasses.asdict(o)
        return super().default(o)


@dataclass
class SuCatalogPackage:
    url: str
    hash: str
    package_name: PackageName


@dataclass
class DownloadedPackage:
    store_path: str
    hash: str  # sha256 hex of the downloaded file


@dataclass
class FetchurlArgs:
    url: str
    hash: str  # sha256 hex


@dataclass
class MacOSSDKUpdate:
    sdk_version: str
    url: str
    sha256: str


ParsedSuCatalog = dict[str, dict[PackageName, SuCatalogPackage]]
AppleSDKReleases = dict[str, dict[PackageName, FetchurlArgs]]


def parse_sucatalog(catalog_url: str) -> ParsedSuCatalog:
    logger.info(f"Fetching catalog from {catalog_url}...")
    with urllib.request.urlopen(catalog_url) as sucatalog:
        logger.info("Reading catalog...")
        text = sucatalog.read().decode("utf-8", errors="replace")

        logger.info("Searching catalog for packages...")
        catalog: ParsedSuCatalog = defaultdict(dict)

        pat = re.compile(
            r"<string>(?P<url>https://swcdn\.apple\.com/content/downloads/.+/(?P<hash>[^/]+)/(?P<package_name>"
            + "|".join(get_args(PackageName))
            + r")\.pkg)</string>",
            re.VERBOSE,
        )

        matches = pat.finditer(text)
        count = 0
        for match in matches:
            package = SuCatalogPackage(**match.groupdict())  # type: ignore[arg-type]
            logger.info(f"Found {package.package_name} with hash {package.hash}")
            catalog[package.hash][package.package_name] = package
            count += 1

        logger.info(f"Discovered {count} package entries")
        return catalog


def _sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def _filename_from_url(url: str) -> str:
    # Keep the original basename; prefix with short hash of URL to avoid collisions
    base = os.path.basename(urllib.parse.urlparse(url).path)
    short = hashlib.sha1(url.encode("utf-8")).hexdigest()[:8]
    return f"{short}-{base}" if base else f"{short}.pkg"


def prefetch_to_tmp(url: str) -> DownloadedPackage:
    """
    Download URL into /tmp/apple_sucatalog/<short>-<basename>.pkg,
    compute sha256 (hex), and return the local path plus hash.
    """
    TMP_ROOT.mkdir(parents=True, exist_ok=True)
    fname = _filename_from_url(url)
    dest = TMP_ROOT / fname

    if dest.exists():
        logger.info(f"Reusing existing file {dest}")
    else:
        logger.info(f"Downloading {url} -> {dest}")
        # Stream download to avoid loading whole file in memory
        with urllib.request.urlopen(url) as r, open(dest, "wb") as out:
            while True:
                chunk = r.read(1024 * 1024)
                if not chunk:
                    break
                out.write(chunk)

    sha = _sha256_file(dest)
    logger.info(f"Stored at {dest} with sha256 {sha}")
    return DownloadedPackage(store_path=str(dest), hash=sha)


def get_macOS_SDK_version(pkg_path: str, *, normalize: bool = True) -> str:
    """
    Extract the Bom from the .pkg and sniff 'MacOSX<version>.sdk' inside.
    Requires `tar` on PATH. Decodes output as text with errors ignored.
    """
    logger.info(f"Getting macOS SDK version from {pkg_path}")
    tar_extract = subprocess.run(
        ["tar", "-xf", pkg_path, "--include", "Bom", "-O"],
        capture_output=True,
        text=False,
        check=False,
    )

    if tar_extract.returncode != 0:
        logger.error(
            "tar failed extracting Bom "
            f"(exit {tar_extract.returncode}). stderr: "
            f"{tar_extract.stderr.decode('utf-8', errors='ignore')}"
        )
        raise RuntimeError("tar failed extracting Bom")

    stdout_text = tar_extract.stdout.decode("utf-8", errors="ignore")

    pat = re.compile(r"MacOSX(?P<sdk_version>[\d.]+)\.sdk")
    matched = pat.search(stdout_text)
    assert matched is not None, "Could not find SDK version"
    sdk_version = matched.group("sdk_version")

    if not normalize:
        return sdk_version

    components = sdk_version.split(".")
    components += ["0"] * (3 - len(components))
    formatted = ".".join(components[:3])
    return formatted


def _version_key(version: str) -> tuple[int, ...]:
    return tuple(int(component) for component in version.split("."))


def generate_apple_sdk_releases(catalog: ParsedSuCatalog) -> AppleSDKReleases:
    apple_sdk_releases: AppleSDKReleases = defaultdict(dict)
    for hash_key, packages in catalog.items():
        # Copy structure so we can pop safely without mutating original map externally
        packages = dict(packages)

        # Get CLTools_macOS_SDK since it'll have the SDK version
        if "CLTools_macOS_SDK" not in packages:
            logger.warning(f"No CLTools_macOS_SDK in hash {hash_key}, skipping")
            continue

        macOS_SDK = packages.pop("CLTools_macOS_SDK")
        store_entry = prefetch_to_tmp(macOS_SDK.url)
        sdk_version = get_macOS_SDK_version(store_entry.store_path)
        logger.info(f"Found macOS SDK version {sdk_version} for {hash_key}")

        # Warn if we have duplicate packages for the same SDK version
        if sdk_version in apple_sdk_releases:
            logger.warning(
                f"Found macOS SDK version {sdk_version}, but with {hash_key}! "
                "Will overwrite existing entries!"
            )

        fetchurl_args = FetchurlArgs(url=macOS_SDK.url, hash=store_entry.hash)
        apple_sdk_releases[sdk_version]["CLTools_macOS_SDK"] = fetchurl_args
        logger.info(
            f"Added CLTools_macOS_SDK for {sdk_version} from {hash_key} to releases"
        )

        for package_name, package in packages.items():
            if package_name in apple_sdk_releases[sdk_version]:
                logger.warning(
                    f"Replacing existing macOS SDK {sdk_version} {package_name} with "
                    f"package of the same version and name from {hash_key}"
                )
            store_entry = prefetch_to_tmp(package.url)
            fetchurl_args = FetchurlArgs(url=package.url, hash=store_entry.hash)
            apple_sdk_releases[sdk_version][package_name] = fetchurl_args
            logger.info(
                f"Added {package_name} for {sdk_version} from {hash_key} to releases"
            )

    return apple_sdk_releases


def latest_nmos_sdk_update(catalog: ParsedSuCatalog) -> MacOSSDKUpdate:
    candidates: list[MacOSSDKUpdate] = []

    for hash_key, packages in catalog.items():
        package = packages.get("CLTools_macOSNMOS_SDK")
        if package is None:
            logger.debug(f"No CLTools_macOSNMOS_SDK in hash {hash_key}, skipping")
            continue

        store_entry = prefetch_to_tmp(package.url)
        sdk_version = get_macOS_SDK_version(store_entry.store_path, normalize=False)
        logger.info(
            "Found CLTools_macOSNMOS_SDK %s for %s",
            sdk_version,
            hash_key,
        )
        candidates.append(
            MacOSSDKUpdate(
                sdk_version=sdk_version,
                url=package.url,
                sha256=store_entry.hash,
            )
        )

    if not candidates:
        raise RuntimeError("No CLTools_macOSNMOS_SDK packages found in catalog")

    return max(
        candidates,
        key=lambda candidate: (_version_key(candidate.sdk_version), candidate.url),
    )


def module_path_from_env() -> Path:
    workspace = os.environ.get("BUILD_WORKSPACE_DIRECTORY")
    if workspace:
        return Path(workspace) / "MODULE.bazel"
    return Path.cwd() / "MODULE.bazel"


def replace_single(pattern: str, replacement: str, text: str, *, field_name: str) -> str:
    updated, count = re.subn(pattern, replacement, text, count=1, flags=re.MULTILINE)
    if count != 1:
        raise RuntimeError(f"Could not update {field_name} in osx.from_archive")
    return updated


def update_osx_from_archive(module_text: str, update: MacOSSDKUpdate) -> str:
    block_pattern = re.compile(
        r"(?P<block>osx\.from_archive\(\n.*?\n\))",
        re.DOTALL,
    )
    match = block_pattern.search(module_text)
    if match is None:
        raise RuntimeError("Could not find osx.from_archive block in MODULE.bazel")

    block = match.group("block")
    strip_prefix = (
        "Payload/Library/Developer/CommandLineTools/SDKs/"
        f"MacOSX{update.sdk_version}.sdk"
    )

    block = replace_single(
        r'^    sha256 = "[^"]+",$',
        f'    sha256 = "{update.sha256}",',
        block,
        field_name="sha256",
    )
    block = replace_single(
        r'^    strip_prefix = "[^"]+",$',
        f'    strip_prefix = "{strip_prefix}",',
        block,
        field_name="strip_prefix",
    )
    block = replace_single(
        (
            r'^        "https://swcdn\.apple\.com/content/downloads/[^"]+/'
            r'CLTools_macOSNMOS_SDK\.pkg",$'
        ),
        f'        "{update.url}",',
        block,
        field_name="url",
    )

    return module_text[: match.start("block")] + block + module_text[match.end("block") :]


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(
        description="Update MODULE.bazel to the latest CLTools_macOSNMOS_SDK package.",
    )
    parser.add_argument(
        "--catalog-url",
        default=SUCATALOG_URL,
        help="Apple software update catalog URL.",
    )
    parser.add_argument(
        "--module",
        type=Path,
        default=module_path_from_env(),
        help=(
            "MODULE.bazel path. Defaults to "
            "$BUILD_WORKSPACE_DIRECTORY/MODULE.bazel under bazel run."
        ),
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Print the MODULE.bazel diff without writing it.",
    )
    parser.add_argument(
        "--verbose",
        action="store_true",
        help="Enable debug logging.",
    )
    args = parser.parse_args(argv)

    if args.verbose:
        logging.getLogger().setLevel(logging.DEBUG)

    catalog = parse_sucatalog(args.catalog_url)
    update = latest_nmos_sdk_update(catalog)

    module_path = args.module
    module_text = module_path.read_text()
    updated = update_osx_from_archive(module_text, update)

    if args.dry_run:
        sys.stdout.writelines(
            difflib.unified_diff(
                module_text.splitlines(keepends=True),
                updated.splitlines(keepends=True),
                fromfile=str(module_path),
                tofile=str(module_path),
            )
        )
    elif updated != module_text:
        module_path.write_text(updated)

    print(
        f"MODULE.bazel osx.from_archive: macOS SDK {update.sdk_version} "
        f"{update.sha256} {update.url}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))

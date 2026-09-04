#
# SPDX-License-Identifier: MIT
# Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
#
set -o errexit
set -o nounset
set -o pipefail

: # cargo update stubbed for test

# ---------------------------------------------------------------------------------------------
# Websites
# ---------------------------------------------------------------------------------------------
#
# Every directory under website/ holding a package.json is an independent pnpm project with its
# own lockfile; website/web_design has none and is skipped.
#
# Each site gets one attempt at the bulk update. When `pnpm update --latest` resolves, builds and
# type-checks, the site is done. When any step fails the site is restored to the state it started
# in and named in the summary, because the failure means a dependency needs a decision this script
# cannot make: a major with a breaking change, a peer conflict, or an override in
# pnpm-workspace.yaml that no longer matches what the new version pins.
#
# The restore covers package.json, pnpm-lock.yaml and pnpm-workspace.yaml. pnpm writes to all
# three: package.json and the lockfile carry the versions, and pnpm appends accepted-early
# releases to minimumReleaseAgeExclude in the workspace file. node_modules is reconciled with the
# restored lockfile by a plain `pnpm install`.
#
# A failing site does not stop the others. Their results are worth having, and one summary listing
# every site that needs hand-work beats stopping at the first.

# Packages the bulk update must leave alone, passed to pnpm as negated selectors.
#
# typescript: TypeScript 7 ships the native compiler, which does not expose the programmatic API
# @astrojs/language-server uses, so `astro check` aborts on it. @astrojs/check declares
# `typescript: "^5.0.0 || ^6.0.0"` to match. Tracked at withastro/roadmap#1321; until that lands,
# the sites stay on 6.x and `pnpm outdated` reports typescript as behind on every one of them.
DC_NPM_HELD=(typescript)

DC_WEBSITE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/website"

if [ ! -d "${DC_WEBSITE_DIR}" ]; then
    command echo ""
    command echo "No website/ directory; skipping the website update."
elif ! command -v pnpm >/dev/null 2>&1; then
    command echo ""
    command echo "pnpm not found; skipping the website update. Install it with: npm install -g pnpm"
else
    DC_UPDATE_ARGS=()
    for DC_HELD in "${DC_NPM_HELD[@]}"; do
        DC_UPDATE_ARGS+=("!${DC_HELD}")
    done

    DC_SNAPSHOT_DIR="$(mktemp -d)"
    trap 'rm -rf "${DC_SNAPSHOT_DIR}"' EXIT

    DC_MANUAL_SITES=()

    for DC_SITE_PATH in "${DC_WEBSITE_DIR}"/*/; do
        [ -f "${DC_SITE_PATH}package.json" ] || continue
        DC_SITE="$(basename "${DC_SITE_PATH}")"

        command echo ""
        command echo "Updating website/${DC_SITE}"

        # Snapshot every file pnpm may rewrite, so a failed attempt leaves nothing behind.
        for DC_FILE in package.json pnpm-lock.yaml pnpm-workspace.yaml; do
            if [ -f "${DC_SITE_PATH}${DC_FILE}" ]; then
                cp "${DC_SITE_PATH}${DC_FILE}" "${DC_SNAPSHOT_DIR}/${DC_SITE}.${DC_FILE}"
            fi
        done

        # The fast path. Stops at the first failing step; `check:tokens` runs only where the site
        # defines it.
        DC_OK=0
        if (
            cd "${DC_SITE_PATH}" &&
            pnpm update --latest ${DC_UPDATE_ARGS[@]+"${DC_UPDATE_ARGS[@]}"} &&
            pnpm build &&
            pnpm check &&
            { ! grep -q '"check:tokens"' package.json || pnpm check:tokens; }
        ); then
            DC_OK=1
        fi

        if [ "${DC_OK}" -eq 1 ]; then
            command echo "website/${DC_SITE}: updated"
        else
            command echo "website/${DC_SITE}: update failed, restoring"
            for DC_FILE in package.json pnpm-lock.yaml pnpm-workspace.yaml; do
                if [ -f "${DC_SNAPSHOT_DIR}/${DC_SITE}.${DC_FILE}" ]; then
                    cp "${DC_SNAPSHOT_DIR}/${DC_SITE}.${DC_FILE}" "${DC_SITE_PATH}${DC_FILE}"
                fi
            done
            ( cd "${DC_SITE_PATH}" && pnpm install ) || true
            DC_MANUAL_SITES+=("${DC_SITE}")
        fi
    done

    if [ "${#DC_MANUAL_SITES[@]}" -ne 0 ]; then
        command echo ""
        command echo "These sites need a manual update: ${DC_MANUAL_SITES[*]}"
        command echo ""
        command echo "For each one, from its directory:"
        command echo "  pnpm outdated                      list what is behind"
        command echo "  pnpm add <pkg>@<version>           take one package at a time"
        command echo "  pnpm build && pnpm check           confirm after each"
        command echo ""
        command echo "A major version often also moves a pin in pnpm-workspace.yaml. When astro"
        command echo "bumps the version of @astrojs/markdown-satteri it depends on, the override"
        command echo "has to move with it, or pnpm resolves a version astro rejects."
        exit 1
    fi
fi

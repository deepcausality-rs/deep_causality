#
# SPDX-License-Identifier: MIT
# Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
#
set -o errexit
set -o nounset
set -o pipefail

command echo ""
command echo "Checking cargo update"

# `cargo upgrade` is `cargo-edit`, which no Rust toolchain ships and `scripts/install_deps.sh` does
# not install. Under `set -o errexit` a bare call ends `make update` on "no such subcommand", and
# the website updates below — the part that most often has work to do — never run. Skipped with an
# actionable message instead, the same way pnpm is below.
if command -v cargo-upgrade >/dev/null 2>&1; then
    command cargo upgrade
else
    command echo "cargo-upgrade not found; skipping the Rust dependency update."
    command echo "Install it with: cargo install cargo-edit --locked"
fi

command echo ""
command echo "Checking for rustup stable update"
command rustup update stable

command echo ""
command echo "Checking crate updates"
command cargo update

# ---------------------------------------------------------------------------------------------
# Websites
# ---------------------------------------------------------------------------------------------
#
# Every directory under website/ holding a package.json is an independent pnpm project with its
# own lockfile; website/web_design has none and is skipped.
#
# Each site runs these steps in order:
#
#   1. `pnpm update --latest` takes the newest version of every dependency not held below.
#   2. scripts/sync_pnpm_overrides.mjs moves each override in pnpm-workspace.yaml to the exact
#      version a direct dependency now pins; `pnpm update` moves the pins but never the overrides.
#   3. `pnpm install` re-resolves the lockfile under the synced overrides.
#   4. A frozen install in a scratch copy holding only the manifest, lockfile and workspace file
#      checks the lockfile the way the Cloudflare deploy does. In the site directory itself pnpm
#      sees an installed node_modules, reports "Already up to date" and compares nothing.
#   5. `pnpm build`, `pnpm check` and, where the site defines it, `pnpm check:tokens`.
#
# A failing step leaves a decision this script cannot make: a major with a breaking change, a peer
# conflict, or two dependencies pinning different versions of one overridden package. The site is
# restored to the state it started in and the update aborts, naming the site and the step.
#
# The restore covers package.json, pnpm-lock.yaml and pnpm-workspace.yaml. pnpm writes to all
# three: package.json and the lockfile carry the versions, and pnpm appends accepted-early
# releases to minimumReleaseAgeExclude in the workspace file. node_modules is reconciled with the
# restored lockfile by a plain `pnpm install`.

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

    DC_SCRIPTS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    DC_SNAPSHOT_DIR="$(mktemp -d)"
    trap 'rm -rf "${DC_SNAPSHOT_DIR}"' EXIT

    # Restores the site's snapshot, reconciles node_modules with it and ends the update, naming the
    # site and the failed step.
    dc_abort() {
        for DC_FILE in package.json pnpm-lock.yaml pnpm-workspace.yaml; do
            if [ -f "${DC_SNAPSHOT_DIR}/${DC_SITE}.${DC_FILE}" ]; then
                cp "${DC_SNAPSHOT_DIR}/${DC_SITE}.${DC_FILE}" "${DC_SITE_PATH}${DC_FILE}"
            fi
        done
        ( cd "${DC_SITE_PATH}" && pnpm install ) || true
        command echo ""
        command echo "ERROR: website/${DC_SITE}: '${DC_STEP}' failed; see its output above."
        command echo "website/${DC_SITE} is restored to its state before the update. Sites updated"
        command echo "before it keep their update; sites after it were not attempted."
        command echo ""
        command echo "To update it by hand, from website/${DC_SITE}:"
        command echo "  pnpm outdated                      list what is behind"
        command echo "  pnpm add <pkg>@<version>           take one package at a time"
        command echo "  pnpm build && pnpm check           confirm after each"
        exit 1
    }

    # Runs one step in the site directory; a failure aborts the update.
    dc_step() {
        DC_STEP="$1"
        shift
        ( cd "${DC_SITE_PATH}" && "$@" ) || dc_abort
    }

    # The frozen install the Cloudflare deploy runs, in a scratch copy without node_modules.
    dc_check_lockfile() {
        local scratch
        scratch="$(mktemp -d "${DC_SNAPSHOT_DIR}/lockfile.XXXXXX")"
        for DC_FILE in package.json pnpm-lock.yaml pnpm-workspace.yaml .npmrc; do
            if [ -f "${DC_FILE}" ]; then
                cp "${DC_FILE}" "${scratch}/"
            fi
        done
        ( cd "${scratch}" && pnpm install --frozen-lockfile --lockfile-only )
    }

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

        dc_step "pnpm update --latest" pnpm update --latest ${DC_UPDATE_ARGS[@]+"${DC_UPDATE_ARGS[@]}"}
        dc_step "sync overrides" node "${DC_SCRIPTS_DIR}/sync_pnpm_overrides.mjs" .
        dc_step "pnpm install" pnpm install
        dc_step "frozen lockfile check" dc_check_lockfile
        dc_step "pnpm build" pnpm build
        dc_step "pnpm check" pnpm check
        if grep -q '"check:tokens"' "${DC_SITE_PATH}package.json"; then
            dc_step "pnpm check:tokens" pnpm check:tokens
        fi

        command echo "website/${DC_SITE}: updated"
    done
fi

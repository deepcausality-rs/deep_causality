#
# SPDX-License-Identifier: MIT
# Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
#
# Asserts that crates.io holds every internal dev-dependency version this workspace requires.
#
# release-plz publishes crate by crate against the index, and its release order drops development
# edges (release_order.rs, `should_dep_be_released_before`). A dev-dependency that carries a
# version must therefore reach the index before the release starts, or `cargo publish -p <crate>`
# fails on a crate nothing ordered first.
#
# `cargo publish --workspace --dry-run` passes in that case, because it satisfies intra-workspace
# requirements from the workspace. Both checks gate a release; each sees what the other misses.
#
# Cargo drops a dev-dependency declared with a bare path from the published manifest, and
# `cargo metadata` reports its requirement as `*`. This skips those.
#
# Only a caret over a numeric `major[.minor[.patch]]` is evaluated. Any other requirement,
# including one with a pre-release or build-metadata suffix, fails the check instead of being
# guessed at.
set -o errexit
set -o nounset
set -o pipefail

source "$(dirname "${BASH_SOURCE[0]}")/crates.sh"
cd "$DC_REPO_ROOT"

DC_INDEX="https://index.crates.io"
DC_AGENT="deep_causality check_dev_deps (https://github.com/deepcausality-rs/deep_causality)"

for tool in cargo jq curl; do
    command -v "$tool" >/dev/null 2>&1 || {
        echo "check_dev_deps: $tool is required and not on PATH" >&2
        exit 1
    }
done

# Maps a crate name to its sparse-index path.
dc__index_path() {
    local n="$1"
    case ${#n} in
        1) echo "1/$n" ;;
        2) echo "2/$n" ;;
        3) echo "3/${n:0:1}/$n" ;;
        *) echo "${n:0:2}/${n:2:2}/$n" ;;
    esac
}

# Prints a crate's non-yanked release versions, newest first.
#
# An unpublished crate prints nothing and succeeds. A network or index failure exits non-zero, so
# an outage never reads as "no versions".
dc__published_versions() {
    local name="$1" url body code status=0
    url="$DC_INDEX/$(dc__index_path "$name")"
    body="$(curl -sS -w '\n%{http_code}' -A "$DC_AGENT" --max-time 30 "$url")" || status=$?
    if [ "$status" -ne 0 ]; then
        echo "check_dev_deps: cannot reach $url (curl exit $status)" >&2
        return 1
    fi
    code="${body##*$'\n'}"
    case "$code" in
        200) : ;;
        404) return 0 ;;
        *)
            echo "check_dev_deps: $url returned HTTP $code" >&2
            return 1
            ;;
    esac
    # jq selects the releases, so a crate holding only pre-releases yields empty output instead of
    # a non-zero grep that `pipefail` would raise as a hard failure.
    printf '%s' "${body%$'\n'*}" |
        jq -r 'select(.yanked == false) | .vers | select(test("^[0-9]+\\.[0-9]+\\.[0-9]+$"))' |
        sort -t. -k1,1nr -k2,2nr -k3,3nr
}

# Prints the half-open range of a caret requirement as six fields: lower major minor patch, then
# upper. Fails on any other form.
#
# How many fields the requirement states decides the upper bound, not just which field is
# non-zero: `^0` is `<1.0.0`, `^0.0` is `<0.1.0`, `^0.0.3` is `<0.0.4`.
dc__caret_bounds() {
    local req="${1#^}"
    [[ "$req" =~ ^[0-9]+(\.[0-9]+){0,2}$ ]] || return 1
    local major minor patch fields
    IFS=. read -r major minor patch <<<"$req"
    fields=1
    [ -n "${minor:-}" ] && fields=2
    [ -n "${patch:-}" ] && fields=3
    minor="${minor:-0}"
    patch="${patch:-0}"
    if [ "$major" -ne 0 ]; then
        echo "$major $minor $patch $((major + 1)) 0 0"
    elif [ "$fields" -eq 1 ]; then
        echo "0 0 0 1 0 0"
    elif [ "$minor" -ne 0 ]; then
        echo "0 $minor $patch 0 $((minor + 1)) 0"
    elif [ "$fields" -eq 2 ]; then
        echo "0 0 0 0 1 0"
    else
        echo "0 0 $patch 0 0 $((patch + 1))"
    fi
}

# Tests version $1.$2.$3 against the range [$4.$5.$6, $7.$8.$9), field by field so no field width
# is assumed.
dc__in_range() {
    local vM=$1 vm=$2 vp=$3 lM=$4 lm=$5 lp=$6 uM=$7 um=$8 up=$9
    if [ "$vM" -lt "$lM" ]; then return 1; fi
    if [ "$vM" -eq "$lM" ]; then
        if [ "$vm" -lt "$lm" ]; then return 1; fi
        if [ "$vm" -eq "$lm" ] && [ "$vp" -lt "$lp" ]; then return 1; fi
    fi
    if [ "$vM" -gt "$uM" ]; then return 1; fi
    if [ "$vM" -eq "$uM" ]; then
        if [ "$vm" -gt "$um" ]; then return 1; fi
        if [ "$vm" -eq "$um" ] && [ "$vp" -ge "$up" ]; then return 1; fi
    fi
    return 0
}

# Prints crate<TAB>dep<TAB>requirement for each internal dev-dependency that reaches the index.
# `cargo metadata` supplies the requirement, so `workspace = true` inheritance arrives resolved
# and nothing here parses TOML.
dc__edges() {
    cargo metadata --no-deps --format-version 1 | jq -r '
        ([.packages[].name]) as $members
        | .packages
        | sort_by(.name)[]
        | . as $p
        | .dependencies[]
        | select(.kind == "dev")
        | select(.req != "*")
        | select(.name as $dep | $members | index($dep))
        | [$p.name, .name, .req] | @tsv
    '
}

edges="$(dc__edges)"

# An empty list would pass forever while the invariant rotted. This workspace has such edges, so
# refuse: their disappearance is a change to argue with, not to read off a green tick.
if [ -z "$edges" ]; then
    echo "check_dev_deps: no internal dev-dependency carries a version requirement." >&2
    echo "  Either every one now uses a bare path, which makes this check moot, or" >&2
    echo "  'cargo metadata' stopped reporting them. Confirm which before deleting this." >&2
    exit 1
fi

checked=0
violations=0

while IFS=$'\t' read -r crate dep req; do
    [ -n "$crate" ] || continue
    checked=$((checked + 1))

    if ! bounds="$(dc__caret_bounds "$req")"; then
        printf '  %-4s %s -> %s %s\n' "FAIL" "$crate" "$dep" "$req"
        echo "       '$req' is not a caret over a numeric major[.minor[.patch]]. Extend dc__caret_bounds." >&2
        violations=$((violations + 1))
        continue
    fi
    read -r lM lm lp uM um up <<<"$bounds"

    versions="$(dc__published_versions "$dep")"
    highest="${versions%%$'\n'*}"
    [ -n "$highest" ] || highest="none published"

    match=""
    while IFS=. read -r vM vm vp; do
        [ -n "$vM" ] || continue
        if dc__in_range "$vM" "$vm" "$vp" "$lM" "$lm" "$lp" "$uM" "$um" "$up"; then
            match="$vM.$vm.$vp"
            break
        fi
    done <<<"$versions"

    if [ -n "$match" ]; then
        printf '  %-4s %s -> %s %s (crates.io: %s)\n' "ok" "$crate" "$dep" "$req" "$highest"
    else
        printf '  %-4s %s -> %s %s (crates.io: %s)\n' "FAIL" "$crate" "$dep" "$req" "$highest"
        violations=$((violations + 1))
    fi
done < <(printf '%s\n' "$edges")

echo
echo "checked $checked internal dev-dependency edges"

if [ "$violations" -ne 0 ]; then
    echo >&2
    echo "check_dev_deps: $violations blocker(s)." >&2
    echo "  Each FAIL above is a 'cargo publish -p <crate>' that fails mid-release, because" >&2
    echo "  release-plz publishes no dev-dependency before its dependent." >&2
    echo "  Publish the dependency, or give it a bare path so cargo drops it from the" >&2
    echo "  published manifest." >&2
    exit 1
fi

echo "every internal dev-dependency is satisfiable from crates.io"

# rules_lean

Bazel rules for [Lean 4](https://lean-lang.org/), with native [Lake](https://github.com/leanprover/lean4/tree/master/src/lake)
integration that reuses Mathlib's upstream Reservoir cache instead of forcing
each consumer to self-host a multi-gigabyte olean tarball.

- **rules**: `lean_test`, `lean_emit`, `lean_prebuilt_library`, `lean_toolchain` — see [docs/lean.md](docs/lean.md).
- **lake integration**: `lake_workspace` repository rule + `lake` module extension — see [docs/lake.md](docs/lake.md).
- **RulesLean Lean library** (`lean/lib/`): structured introspection of `.olean` files (`RulesLean.Olean`) and Lake workspaces (`RulesLean.Workspace`). Internal helpers under `RulesLean.Internal.*` are unstable; treat them as opt-in and expect API churn between releases. See [lean/lib/RulesLean.lean](lean/lib/RulesLean.lean) for the entry-point doc.
- **lake_imports_manifest** target: when `lake_workspace` materializes, it builds the RulesLean library + `oleanImports` CLI and runs it over every olean in the workspace. Result lands at `@<your-lake-deps>//:lake_imports_manifest` — a TSV of `<path>\t<imported-module>` edges (~5MB / 42k edges for full mathlib). Downstream consumers can use it for import-graph analysis, tree-shaking, dead-code detection.

## Install

Add the registry to your `.bazelrc`:

```
common --registry=https://registry.fastverk.com/
common --registry=https://bcr.bazel.build/
```

In your `MODULE.bazel`:

```python
bazel_dep(name = "rules_lean", version = "0.3.0")

lake = use_extension("@rules_lean//lean:lake.bzl", "lake")
lake.workspace(
    name = "lake_deps",
    lean_toolchain  = "//:lean-toolchain",
    lakefile        = "//:lakefile.lean",
    lake_manifest   = "//:lake-manifest.json",
)
use_repo(lake, "lake_deps")
register_toolchains("@lake_deps//:lean_toolchain_def")
```

## Quick start

Your repo root needs three Lake-convention files:

**`lean-toolchain`** — pins the Lean version (Lake and Bazel both honor it):

```
leanprover/lean4:v4.29.1
```

**`lakefile.lean`** — a deps-only lakefile listing Lake packages you want:

```lean
import Lake
open Lake DSL

package «my-project» where

require mathlib from git
  "https://github.com/leanprover-community/mathlib4.git" @ "v4.29.1"
```

**`lake-manifest.json`** — generate once with `elan`-installed `lake`, then commit:

```sh
lake update     # produces lake-manifest.json with all transitive revs pinned
```

Now any `BUILD.bazel` can typecheck Lean code against the resolved packages:

```python
load("@rules_lean//lean:lean.bzl", "lean_test")

lean_test(
    name  = "smoke",
    srcs  = ["Smoke.lean"],
    entry = "Smoke.lean",
    deps  = [
        "@lake_deps//:mathlib",
        "@lake_deps//:batteries",
    ],
)
```

```lean
-- Smoke.lean
import Mathlib.Data.Finset.Basic
example : (∅ : Finset Nat).card = 0 := Finset.card_empty
```

`bazel test //:smoke` will, on first run: download the Lean toolchain, run
`lake update`, run `lake exe cache get` (Reservoir-cached mathlib oleans),
and typecheck `Smoke.lean`.

## How it works

`lake_workspace` is a Bazel repository rule that:

1. Reads `lean-toolchain`, downloads the matching Lean tarball
   (sha256-pinned for [known versions](lean/private/known_lean_versions.bzl)).
2. Stages your `lakefile` + `lake-manifest.json` into the external repo.
3. Runs `lake update` to materialize all transitive Lake package checkouts at
   the manifest-pinned revs.
4. If mathlib is in the dep graph, runs `lake exe cache get` to fetch
   prebuilt oleans from the upstream Reservoir cache.
5. Generates a `BUILD.bazel` exposing each resolved Lake package as its own
   `lean_prebuilt_library` (target name = Lake's directory name:
   `:mathlib`, `:batteries`, `:Cli`, `:LeanSearchClient`, …).

### What's hermetic

| Layer                | Pinned by                                                |
| -------------------- | -------------------------------------------------------- |
| Lean toolchain       | `sha256` in [`lean/private/known_lean_versions.bzl`](lean/private/known_lean_versions.bzl) |
| Lake dep git revs    | Your committed `lake-manifest.json` (Lake's lockfile)    |
| Mathlib oleans       | Content-addressed by mathlib commit in Reservoir cache (verified by Lake) |

For Lake packages **not** covered by the Reservoir cache (anything outside
mathlib's transitive deps), pass `allow_source_build = True` to `lake.workspace`
— the rule then runs `lake build <pkg>` to compile oleans from source. Slow
but unavoidable for custom deps.

### What's not (yet) hermetic

- Lean versions that aren't pinned in `known_lean_versions.bzl` download
  unverified (with a warning). Add an entry — one line — for any new
  version you need.
- `lake update` reaches the network. The lake-manifest.json constrains
  *what* gets resolved, but the network has to be there. Bazel's normal
  repository-cache mitigates the cost on rebuilds.

## Compatibility

- **Bazel**: 7.4+, bzlmod required.
- **Lean**: 4.29+ tested. Other versions: add the platform sha256 to
  [`lean/private/known_lean_versions.bzl`](lean/private/known_lean_versions.bzl)
  (compute with `curl -fsSL <url> | shasum -a 256`).
- **Platforms**: darwin_aarch64, darwin_x86_64, linux_x86_64, linux_aarch64.

## Contributing

Rule reference docs (`docs/lean.md`, `docs/lake.md`) are stardoc-generated
from the `.bzl` docstrings and committed to source. After editing a rule
docstring, regenerate:

```sh
bazel run //docs:update
```

CI gates this via `bazel test //docs/...` (diff_test against the committed
output).

## License

MIT.

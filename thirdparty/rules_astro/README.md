# rules_astro

Build [Astro](https://astro.build) sites **hermetically under Bazel** — `astro build`
runs as a cacheable, RBE-eligible action producing a static `dist/` TreeArtifact.

Generic (no fastverk coupling); the reusable build layer for `fastverk/site`, docs
portals, and blogs. Sibling of `rules_fastverk_plugin` / `rules_ci` / `rules_meridian`
in `tomato-bazel`; publish to the bazel-registry for consumption.

> **Status: validated.** `astro_site` builds `fastverk/site` (19 pages) hermetically as a
> single `JsRunBinary` action on Bazel 7.4.1 + `aspect_rules_js` 2.1.3, producing the static
> `dist/` TreeArtifact — remotely cacheable / RBE-eligible.

## Use

`MODULE.bazel`:

```starlark
bazel_dep(name = "rules_astro", version = "0.0.1")
bazel_dep(name = "aspect_rules_js", version = "2.1.3")

npm = use_extension("@aspect_rules_js//npm:extensions.bzl", "npm")
npm.npm_translate_lock(name = "npm", pnpm_lock = "//:pnpm-lock.yaml")
use_repo(npm, "npm")
```

`site/BUILD.bazel`:

```starlark
load("@aspect_rules_js//npm:defs.bzl", "npm_link_all_packages")
load("@npm//:astro/package_json.bzl", astro_bin = "bin")
load("@rules_astro//astro:defs.bzl", "astro_site", "astro_dev")

npm_link_all_packages(name = "node_modules")
astro_bin.astro_binary(name = "astro_tool")   # the astro CLI as a js_binary

astro_site(
    name = "build",                     # -> dist/ (static, hermetic, RBE-cacheable)
    srcs = glob(["src/**", "public/**"]) + ["astro.config.mjs", "package.json", "tsconfig.json"],
    astro_bin = ":astro_tool",
    node_modules = ":node_modules",
    data = [":features_json"],          # generated inputs (feature catalog, OG cards)
)

astro_dev(name = "dev", srcs = glob(["src/**", "public/**"]), astro_bin = ":astro_tool", node_modules = ":node_modules")
```

```sh
bazel build //site:build     # -> bazel-bin/site/build/ (dist)
bazel run   //site:dev       # local preview
```

## API

- `astro_site(name, srcs, config, data, astro_bin, out_dir, chdir, env, **kwargs)`
- `astro_dev(name, srcs, config, data, astro_bin, **kwargs)`

See `astro/defs.bzl` for the full docstrings.

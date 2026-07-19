"""Public API for rules_astro.

    load("@rules_astro//astro:defs.bzl", "astro_site", "astro_dev")

`astro_site` runs `astro build` as a hermetic Bazel action and captures the static
output as a TreeArtifact (default `dist/`). `astro_dev` wraps `astro dev` for
`bazel run`.

Both are thin wrappers over aspect_rules_js. The consumer sets up node_modules via
`npm.npm_translate_lock` + `npm_link_all_packages`, materializes the astro CLI as a
`js_binary` from the generated npm `bin` struct, and passes it as `astro_bin`:

    load("@aspect_rules_js//npm:defs.bzl", "npm_link_all_packages")
    load("@npm//:astro/package_json.bzl", astro_bin = "bin")
    load("@rules_astro//astro:defs.bzl", "astro_site")

    npm_link_all_packages(name = "node_modules")
    astro_bin.astro_binary(name = "astro_tool")   # the js_binary tool

    astro_site(
        name = "build",
        srcs = glob(["src/**", "public/**"]) + ["astro.config.mjs", "package.json", "tsconfig.json"],
        astro_bin = ":astro_tool",
        node_modules = ":node_modules",
    )

Keeping the wrapper thin means Astro upgrades are just an npm lockfile bump.
"""

load("@aspect_rules_js//js:defs.bzl", "js_run_binary", "js_run_devserver")

def astro_site(
        name,
        srcs,
        astro_bin,
        config = "astro.config.mjs",
        data = [],
        node_modules = None,
        out_dir = "dist",
        chdir = None,
        env = {},
        **kwargs):
    """Build an Astro site into a static output directory (a TreeArtifact).

    Args:
      name: target name; produces `<name>` as an out_dir TreeArtifact.
      srcs: site sources (src/**, public/**, astro config, package.json, tsconfig).
      astro_bin: a `js_binary` for the astro CLI (from the npm `bin` struct).
      config: astro config file (default astro.config.mjs).
      data: extra runtime inputs (generated content like features.json, OG cards).
      node_modules: the `npm_link_all_packages` target, materialized for the build.
      out_dir: Astro's build output dir (default "dist").
      chdir: dir to run from (defaults to the target's package).
      env: extra environment for the build.
      **kwargs: passed through (visibility, tags, ...).
    """
    inputs = srcs + [config] + list(data)
    if node_modules:
        inputs = inputs + [node_modules]
    js_run_binary(
        name = name,
        srcs = inputs,
        tool = astro_bin,
        args = ["build"],
        chdir = chdir or native.package_name(),
        out_dirs = [out_dir],
        env = env,
        # astro build is deterministic given inputs; safe to cache remotely.
        **kwargs
    )

def astro_dev(name, srcs, astro_bin, config = "astro.config.mjs", data = [], node_modules = None, **kwargs):
    """`bazel run //site:dev` -> a local Astro dev server."""
    inputs = srcs + [config] + list(data)
    if node_modules:
        inputs = inputs + [node_modules]
    js_run_devserver(
        name = name,
        # js_run_devserver materializes inputs via `data`, not `srcs`.
        data = inputs,
        tool = astro_bin,
        args = ["dev"],
        chdir = native.package_name(),
        **kwargs
    )

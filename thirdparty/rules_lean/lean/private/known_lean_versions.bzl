"""Hash table for known Lean 4 release tarballs.

Bumping a Lean version requires adding an entry here. To compute sha256:

    curl -fsSL <url> | shasum -a 256

Unpinned versions can still be downloaded (unverified) — `lake_workspace`
will emit a warning. Always prefer pinning.
"""

# Map: lean version tag (with leading 'v') -> { platform -> sha256 hex }.
KNOWN_LEAN_VERSIONS = {
    # Pinned by DeepCausality (thirdparty/rules_lean is vendored). Matches
    # lean/lean-toolchain + the Mathlib rev in lean/lakefile.toml. Hashes of the
    # leanprover/lean4 v4.32.0 release zips (curl -fsSL <url> | shasum -a 256).
    "v4.32.0": {
        "darwin_aarch64": "ffd3410d554dcb2d83dd92f0b7b92a79a6d462d11765b2db4f291cdd66f90942",
        "darwin_x86_64": "f6af4fcf34c2966032065f109557e7363e3bd84fed84a77137752a345a3b7da0",
        "linux_x86_64": "5320dc308f108775904d865b05df386e6bc7dee254e030a90177e8fcc36f0fbe",
        "linux_aarch64": "efc019f0403c77300497ea33415e18e46deac4c7c4f6423934e518fa60ee6fba",
    },
    "v4.29.1": {
        "darwin_aarch64": "c15284adf88ad830c71775b9828cb81f49f7f262cbe1456b25d935855bd70975",
        "linux_x86_64": "357acb30fca2212986fdc8b83dbe88e8f5610efc060f6e3515079c56a92d276f",
    },
    "v4.30.0-rc2": {
        "darwin_aarch64": "1bda6929976b2a034985fdfc85faa5e757421f6542c5e59c644e44dc1132fe51",
        "darwin_x86_64": "822b5a802763c3833c748ba6dd781fdf16426a16b7b7b2b753783ff3435feb7b",
        "linux_x86_64": "0006942b918c7fb9751a5e50b9e5ad570c5cc6aa758c980a3abc054dd8739d35",
        "linux_aarch64": "62c60766b850e1d5b4405742c4aefff097441105e51f5fb5c1bf90434b8e0960",
    },
}

# Per-platform asset filename template. Lean release naming has shifted
# slightly across versions; this is the modern (4.20+) convention.
PLATFORM_ASSETS = {
    "darwin_aarch64": "lean-{v}-darwin_aarch64.zip",
    "darwin_x86_64": "lean-{v}-darwin.zip",
    "linux_x86_64": "lean-{v}-linux.zip",
    "linux_aarch64": "lean-{v}-linux_aarch64.zip",
}

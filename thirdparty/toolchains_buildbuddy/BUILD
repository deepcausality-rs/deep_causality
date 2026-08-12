load("@buildifier_prebuilt//:rules.bzl", "buildifier", "buildifier_test")

EXCLUDE_PATTERNS = [
    # keep sorted
    ".bazelbsp",
    ".bazelversion",
    ".git",
    ".ijwb",
]

LINT_WARNINGS = [
    "-module-docstring",
]

buildifier(
    name = "buildifier",
    exclude_patterns = EXCLUDE_PATTERNS,
    lint_mode = "fix",
    lint_warnings = LINT_WARNINGS,
)

buildifier_test(
    name = "buildifier.test",
    exclude_patterns = EXCLUDE_PATTERNS,
    lint_mode = "warn",
    lint_warnings = LINT_WARNINGS,
    no_sandbox = True,
    workspace = "//:MODULE.bazel",
)

exports_files(
    glob(
        ["*"],
        exclude = EXCLUDE_PATTERNS,
    ),
)

# This filegroup collects all parent workspace files needed by child workspaces
# for integration tests
filegroup(
    name = "local_repository_files",
    srcs = [
        ".bazelversion",
    ] + glob(
        ["*"],
        exclude = EXCLUDE_PATTERNS,
    ) + [
        "//platforms:all_files",
        "//templates:all_files",
        "//toolchains/cc:all_files",
    ],
    visibility = ["//:__subpackages__"],
)

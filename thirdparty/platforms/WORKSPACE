workspace(name = "platforms")

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

http_archive(
    name = "package_metadata",
    sha256 = "0e89367f1cb6d93a5a1afea4b55b11ea6b28f63f653b47154153677ca7d4afea",
    strip_prefix = "supply-chain-0.0.3/metadata",
    urls = [
        "https://mirror.bazel.build/github.com/bazel-contrib/supply-chain/releases/download/v0.0.3/supply-chain-v0.0.3.tar.gz",
        "https://github.com/bazel-contrib/supply-chain/releases/download/v0.0.3/supply-chain-v0.0.3.tar.gz",
    ],
)

http_archive(
    name = "rules_license",
    sha256 = "4531deccb913639c30e5c7512a054d5d875698daeb75d8cf90f284375fe7c360",
    urls = [
        "https://mirror.bazel.build/github.com/bazelbuild/rules_license/releases/download/0.0.7/rules_license-0.0.7.tar.gz",
        "https://github.com/bazelbuild/rules_license/releases/download/0.0.7/rules_license-0.0.7.tar.gz",
    ],
)

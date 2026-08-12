load("@bazel_features//:features.bzl", "bazel_features")
load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
load("//constraints/libc:libc_versions.bzl", "GLIBC_VERSIONS")
load("//platforms:common.bzl", "LIBC_SUPPORTED_TARGETS")

# glibc's source/header triples do not always match our (os, arch) names.
# For arches whose canonical glibc triple differs, list the override here.
_GLIBC_TRIPLE_OVERRIDES = {
    ("linux", "armv7"): "arm-linux-gnueabihf",
}

def glibc_triple(target_os, target_arch):
    return _GLIBC_TRIPLE_OVERRIDES.get(
        (target_os, target_arch),
        "{}-{}-gnu".format(target_arch, target_os),
    )

GLIBC_RELEASE_COMMITS = {
    "2.28": "92d25389c255b0da9b56bc05694f0702cd22921a",  # release/2.28/master
    "2.29": "2417ddb64590b7197d6df15b9df67866561713e0",  # release/2.29/master
    "2.30": "f8db9906d759152db9977c8774470d05a80519da",  # release/2.30/master
    "2.31": "7b27c450c34563a28e634cccb399cd415e71ebfe",  # release/2.31/master
    "2.32": "5ad449c398a845a9c84808e4ac603beaa1006909",  # release/2.32/master
    "2.33": "5f08d1df2c07904c1dc98bdf2b363c65874266f7",  # release/2.33/master
    "2.34": "26b14f335d53ce4bdaed5dbdd0e7268b8c7b5484",  # release/2.34/master
    "2.35": "d2febe7c407665c18cfea1930c65f41899ab3aa3",  # release/2.35/master
    "2.36": "03e0cad3a0d8cfb6e761e8e16cc09e6c96f9fd44",  # release/2.36/master
    "2.37": "032545ebd3ab2248b137bc92df0bd2864031cc8b",  # release/2.37/master
    "2.38": "5a08d049dc5037e89eb95bb1506652f0043fa39e",  # release/2.38/master
    "2.39": "68f3f1a1d08f7f3e0fb74391461699717efbb4bc",  # release/2.39/master
    "2.40": "8d3dd23e3de8b4c6e4b94f8bbfab971c3b8a55be",  # release/2.40/master
    "2.41": "5cf17ebc659c875aff3c49d2a59ce15f46167389",  # release/2.41/master
    "2.42": "8aaf4b732d7650c2db3beb4dc8bb70eab5b022c3",  # release/2.42/master
}

GLIBC_RELEASE_URLS = {
    "2.43": ["https://ftp.gnu.org/gnu/glibc/glibc-2.43.tar.xz"],
    "2.44": ["https://ftp.gnu.org/gnu/glibc/glibc-2.44.tar.xz"],
}

GLIBC_RELEASE_STRIP_PREFIXES = {
    "2.43": "glibc-2.43",
    "2.44": "glibc-2.44",
}

GLIBC_RELEASE_INTEGRITY = {
    "2.28": "f4cf37f12e0c3a81485ea8743dc41dbcec31ae326b22d6265e724c7ee753529e",
    "2.29": "39cb3f84e92fe829f0b5d96d08437d18e90d17a23021119a7235e17d6bb25245",
    "2.30": "617511b27f473280597a1cc2828101a29b9f4c682ee99422b8c246c2f84f46e2",
    "2.31": "80329e5c6624ebeaa9c0d80f73dda8c28491114206420c44a5718f56302c44ec",
    "2.32": "51900a45edab4cc21059846fb875f3583948dc9449bad7dac997687cb998e8b2",
    "2.33": "9c8b9bea370bd6f03d154b6003542fa5b1d0b7124953ce5709b3859c3dade076",
    "2.34": "1a15c817f8d21ab0bc843f04bea3e4e33b91b640d7ce879d23b8d915de1ed03e",
    "2.35": "76e4f569b0058f46d9f2a83eba40342efda4414f2ba5337c1b947e930dcacc51",
    "2.36": "3c77f25a49c823ce0777191fafaf1ae65c0bd0d88e9f0080e592368c557af779",
    "2.37": "5b7e52f5679744b6b692b2eabe54bbb13283f9baec0e9e08f8c825a984989c55",
    "2.38": "b1aff01416addccf7970b69275ee9d66d094ef6b770bdc49dcb3f1737cb563a1",
    "2.39": "b3d0acbe514fec5352bc195a91513be9de26740b086700135e7528356f8a38cc",
    "2.40": "80d4e3a0846423a536a14da79d7b527a84b0f73df3f1d5beadf63c8d16fc2429",
    "2.41": "ae2ef9d50a04e5f4e7eece46455a21bf4e4e69518ed31496fdb4c0895f7e18fd",
    "2.42": "0a1427b902e6fe666508162d92d5be3eaff4d410fe66f3426ccaa9848ed90f05",
    "2.43": "d9c86c6b5dbddb43a3e08270c5844fc5177d19442cf5b8df4be7c07cd5fa3831",
    "2.44": "37f600f2bef3c5e8300147059568b2a2e40a7ad6ccc65ce942556d49429cc667",
}

def _glibc_release_urls(version):
    urls = GLIBC_RELEASE_URLS.get(version)
    if urls:
        return urls

    return ["https://github.com/bminor/glibc/archive/{}.tar.gz".format(GLIBC_RELEASE_COMMITS[version])]

def _glibc_release_strip_prefix(version):
    strip_prefix = GLIBC_RELEASE_STRIP_PREFIXES.get(version)
    if strip_prefix:
        return strip_prefix

    return "glibc-{}".format(GLIBC_RELEASE_COMMITS[version])

def _glibc_trampoline_repository_impl(repository_ctx):
    repository_ctx.template("BUILD.bazel", repository_ctx.attr._build_file)

_glibc_trampoline_repository = repository_rule(
    implementation = _glibc_trampoline_repository_impl,
    attrs = {
        "_build_file": attr.label(
            allow_single_file = True,
            default = ":trampoline.BUILD.bazel",
        ),
    },
)

def _glibc_impl(module_ctx):
    """glibc sources and headers extension."""

    index = {}
    for mod in module_ctx.modules:
        for index in mod.tags.index:
            file_path = module_ctx.path(index.file)
            file_content = module_ctx.read(file_path)
            index = json.decode(file_content, default = None)

    for version in GLIBC_VERSIONS:
        for (target_os, target_arch) in LIBC_SUPPORTED_TARGETS:
            target = glibc_triple(target_os, target_arch)

            #TODO(cerisier): Share the repository between targets
            http_archive(
                name = "glibc_%s.%s" % (target, version),
                urls = _glibc_release_urls(version),
                strip_prefix = _glibc_release_strip_prefix(version),
                sha256 = GLIBC_RELEASE_INTEGRITY[version],
                build_file = "//3rd_party/libc/glibc:glibc.BUILD.bazel",
                patches = [
                    # This file is generated when compiling the glibc.
                    #
                    # We add it as an empty file because all the constant it
                    # normally defines are manually passed as `defines`
                    "//3rd_party/libc/glibc:0001-Add-empty-config.h.patch",

                    # This file is generated when compiling the glibc.
                    #
                    # It defines the ABI tag value used in the ELF note included
                    # in the startup code linked into every program.
                    #
                    # On linux, it is the same value all the time:
                    # .*-.*-linux.* 0 2.0.0 # earliest compatible kernel version
                    "//3rd_party/libc/glibc:0002-Add-abi-tag.h.patch",

                    # This file is generated when compiling the glibc.
                    #
                    # It defines constants and macros used during the glibc c
                    # files compilation. We can hardcode the values safely as
                    # long as every module that existed in the range of libc
                    # we support is listed in this file with a value associated.
                    "//3rd_party/libc/glibc:0003-Adding-libc-modules.h.patch",
                ],
                patch_args = [
                    "-p1",
                ],
            )

            index_entry = index.get(version, {}).get(target, None)
            if index_entry == None:
                fail("Missing index entry for %s %s in index.json" % (version, target))

            repo = "glibc_headers_%s.%s" % (target, version)
            http_archive(
                name = repo,
                url = index_entry.get("url"),
                sha256 = index_entry.get("sha256"),
                strip_prefix = target,
                build_file = ":glibc-headers.BUILD.bazel",
            )

    _glibc_trampoline_repository(
        name = "glibc",
    )

    repos = ["glibc"]
    is_non_dev_dependency = module_ctx.root_module_has_non_dev_dependency
    root_direct_deps = list(repos) if is_non_dev_dependency else []
    root_direct_dev_deps = list(repos) if not is_non_dev_dependency else []

    metadata_kwargs = {}
    if bazel_features.external_deps.extension_metadata_has_reproducible:
        metadata_kwargs["reproducible"] = True

    return module_ctx.extension_metadata(
        root_module_direct_deps = root_direct_deps,
        root_module_direct_dev_deps = root_direct_dev_deps,
        **metadata_kwargs
    )

glibc_index = tag_class(
    attrs = {
        "file": attr.label(
            allow_single_file = True,
            default = ":glibc_index.json",
            mandatory = True,
        ),
    },
)

glibc = module_extension(
    implementation = _glibc_impl,
    tag_classes = {
        "index": glibc_index,
    },
)

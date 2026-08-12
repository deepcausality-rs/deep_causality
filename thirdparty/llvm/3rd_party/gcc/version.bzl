GCC_VERSIONS = [
    "17.0.0",
    "16.1.0",
    "15.2.0",
    "15.1.0",
    "14.3.0",
    "14.2.0",
    "13.4.0",
    "13.3.0",
    "13.2.0",
    "13.1.0",
    "12.5.0",
    "12.4.0",
    "12.3.0",
    "12.2.0",
    "12.1.0",
    "11.5.0",
    "11.4.0",
    "11.3.0",
    "11.2.0",
    "11.1.0",
    "10.5.0",
    "10.4.0",
    "10.3.0",
    "10.2.0",
    "10.1.0",
    "9.5.0",
    "9.4.0",
    "9.3.0",
    "9.2.0",
    "9.1.0",
    "8.5.0",
    "8.4.0",
    "8.3.0",
    "8.2.0",
    "8.1.0",
]

DEFAULT_GCC_VERSION = "17.0.0"

GCC_RELEASES = {
    "17.0.0": {
        "commit": "2bfd402f8569511901ec8fe7628f57471e6d240a",
        "sha256": "dc033fdfd79caf199113446af6d082004534437b6ebd276f9732815d86cbe723",
    },
    "16.1.0": {
        "commit": "6afcc4f6da931eb93f3ab001a0dd9650ea71d1ea",
        "sha256": "b62f8feee8f9f6e3c9bb60e09546c869dacd2b1e2948a2da2c610ce76a3e5e89",
    },
    "15.1.0": {
        "commit": "1b306039ac49f8ad91ca71d3de3150a3c9fa792a",
        "sha256": "b45e06e72a265b129f0f68fc81b7be06c931e5ff7c2d47bafcd45c573eda96ad",
    },
    "15.2.0": {
        "commit": "5115c7e447fc07457443df874bf57840e8316d5f",
        "sha256": "0a1cfcef7d3903f706c94ceba9f49a08bf3acd8ce5258270f7f3016290f7c4ee",
    },
    "14.2.0": {
        "commit": "04696df09633baf97cdbbdd6e9929b9d472161d3",
        "sha256": "bc304fe24d9c046588d6336a283325eb86c85b88699111ae9cccc08ca16fbc66",
    },
    "14.3.0": {
        "commit": "c9cd41fba9ebd288c4f101e4b99da934bcb96a11",
        "sha256": "e15cb7bdf3f0cbda46d3bd58082abb5cd9bbbb4749c03292a19cf91c285555e1",
    },
    "13.1.0": {
        "commit": "cc035c5d8672f87dc8c2756d9f8367903aa72d93",
        "sha256": "decd4061fa41b28073c9e7a7b2c1b294c413f3b6efd0fff5552c77dbe40deaa8",
    },
    "13.2.0": {
        "commit": "c891d8dc23e1a46ad9f3e757d09e57b500d40044",
        "sha256": "47478252fe8b890a43396707349bfe4a013426dc63d43dd14a0c5dff2a6fe952",
    },
    "13.3.0": {
        "commit": "b71f1de6e9cf7181a288c0f39f9b1ef6580cf5c8",
        "sha256": "54e834fe573948905c934a494ae503d90b08b5607f8f1650ffddf052051b5339",
    },
    "13.4.0": {
        "commit": "99677969d463d75a562f94460ea75e9f6a016b4f",
        "sha256": "2a061f0d2afe337127d50f607644adfa0f4e1c019c829ade3643f1afb6702191",
    },
    "12.1.0": {
        "commit": "1ea978e3066ac565a1ec28a96a4d61eaf38e2726",
        "sha256": "37f4815d27485cd84bfd725598a5afe53714ce4ecdfdef4bc266c086ae165367",
    },
    "12.2.0": {
        "commit": "2ee5e4300186a92ad73f1a1a64cb918dc76c8d67",
        "sha256": "89c4ffb9aac6e10e4896529f608a02171e5b528b7ad91a69710d4e5b151175d1",
    },
    "12.3.0": {
        "commit": "8fc1a49c9312b05d925b7d21f1d2145d70818151",
        "sha256": "a2b2702a49f9edbf144565ea15b79919a0e970518eee363f246ef11f1dc08d7c",
    },
    "12.4.0": {
        "commit": "2bada4bc59bed4be34fab463bdb3c3ebfd2b41bb",
        "sha256": "1c53b5a5e16473dd9f7aedfbd70f005c80d83a20e6df0ab975956bebe8f69b9a",
    },
    "12.5.0": {
        "commit": "c17d40bb3778bca5e81595f033df9222b66658eb",
        "sha256": "9740c1b2b67745629154dc36e1eda4e3c1d6558328a614b75e61f134ac3f5a2b",
    },
    "11.1.0": {
        "commit": "50bc9185c2821350f0b785d6e23a6e9dcde58466",
        "sha256": "b632d97861ca71dd79e762e7e41b255a6bf0725b829c064873ce9013792c6856",
    },
    "11.2.0": {
        "commit": "7ca388565af176bd4efd4f8db1e5e9e11e98ef45",
        "sha256": "25d3e7e5df3ad89eaa07bb0ca523e0e42438d8f5ccda0bb8d9d4e5cd3688510a",
    },
    "11.3.0": {
        "commit": "2d280e7eafc086e9df85f50ed1a6526d6a3a204d",
        "sha256": "53b26b1c6bb35162a43e1260450407a39e833f770c8deb5a5a3b55da48687041",
    },
    "11.4.0": {
        "commit": "ff4bf326d03e750a8d4905ea49425fe7d15a04b8",
        "sha256": "4eb72119b70179379add1897bd3dad6aa24b2d36f683a4bf27e36bbcc785309b",
    },
    "11.5.0": {
        "commit": "5cc4c42a0d4de08715c2eef8715ad5b2e92a23b6",
        "sha256": "f74980dd3928f79376ca0ac9f53fe98e6509511ea53de8476465bf095d297c5b",
    },
    "10.1.0": {
        "commit": "6e6e3f144a33ae504149dc992453b4f6dea12fdb",
        "sha256": "3db01abaac3d4dde34272d44e724e28494ca55c87961b64703555122d5e6205b",
    },
    "10.2.0": {
        "commit": "ee5c3db6c5b2c3332912fb4c9cfa2864569ebd9a",
        "sha256": "13b4da7d6c12a096188640aafd047ffcac6f5731298d67818e6135af34864b54",
    },
    "10.3.0": {
        "commit": "f00b5710a30f22efc3171c393e56aeb335c3cd39",
        "sha256": "3d850029364cad899ed121af72e14533eee2cbb95745ff6027b4bf7e97554741",
    },
    "10.4.0": {
        "commit": "7ff47281ce4f3699185b06a3430968eac2a5b0c6",
        "sha256": "7f60d80d2054ef738733a4c996de046e055f1f1c085afccb3c2fc3c5b6ee1d84",
    },
    "10.5.0": {
        "commit": "d04fe5541c53cb16d1ca5c80da044b4c7633dbc6",
        "sha256": "ecace41b1e79da90f87d71c2bb6fce5c53eb126ec3a4c376bbc8d296c77753e9",
    },
    "9.1.0": {
        "commit": "c8913260b0756f977ab5e6e6392c51a83657fffc",
        "sha256": "b4ade971adfc94650c2af6d7daf7bac6dbf6c8c8131b1d63e139a6dc3160ecbb",
    },
    "9.2.0": {
        "commit": "a0c06cc27d2146b7d86758ffa236516c6143d62c",
        "sha256": "75871cd2d0fb91f108253bf271e0063fe68f5ad7fee4d2931f8ea2102b09dd5e",
    },
    "9.3.0": {
        "commit": "4212a6a3e44f870412d9025eeb323fd4f50a61da",
        "sha256": "e15f544b409cd90fe281989fad7d06475486961e553080ec0852153e7e7fa21f",
    },
    "9.4.0": {
        "commit": "13c83c4cc679ad5383ed57f359e53e8d518b7842",
        "sha256": "efc27f0a4a83f7bb534c195e16e63a367e109878ab98975f2731b47cbb35287a",
    },
    "9.5.0": {
        "commit": "7a15b5060a83ea8282323d92043c6152e6a3e22d",
        "sha256": "a41dba755e4cbcb96a984cd2284c37df4ddf0db094a9af3acd4e4647cd416848",
    },
    "8.1.0": {
        "commit": "406c2abec3f998e9064919b22db62f38a7c0e7b9",
        "sha256": "6f31c32ab844293951fe4f846dcec361ee24424cc416879a96176990a76ad4fe",
    },
    "8.2.0": {
        "commit": "ddeb81e76461fc0075542d436dc962f3cf6fac92",
        "sha256": "6c91235412ef1c5fc5e4c295f49b7079c879c2db8c64b008202e41f219dad918",
    },
    "8.3.0": {
        "commit": "4c44b708f11eec6fc02456e8577708d01ca92327",
        "sha256": "9a3d65f2ae7ed56ee601ba04cd8a2563dbb9409e7e98319d94c56d4366c418f0",
    },
    "8.4.0": {
        "commit": "8cd3bffead2ed1d1998c190865694f920fbc93ab",
        "sha256": "6d02f84e8b40e6b140a008cd261ed9bdd4c05f7df7692ef6d0a84cae37fd2118",
    },
    "8.5.0": {
        "commit": "eafe83f2f20ef0c1e7703c361ba314b44574523c",
        "sha256": "87bb3018c1523d7fc1d089c69beb93439f3c3675f3af16933d0259682956a151",
    },
}

GCC_VERSION = DEFAULT_GCC_VERSION
GCC_COMMIT = GCC_RELEASES[DEFAULT_GCC_VERSION]["commit"]
GCC_SHA256 = GCC_RELEASES[DEFAULT_GCC_VERSION]["sha256"]

def gcc_repo_name(version):
    return "gcc_" + version.replace(".", "_")

def gcc_repository_label(version, target):
    return "@gcc//:{}/{}".format(gcc_repo_name(version), target)

def gcc_config_toolexeclibdir_target(version):
    if gcc_version_at_least_for(version, "10.0.0"):
        return "config/toolexeclibdir.m4"
    return "gcc_config_toolexeclibdir_m4"

def libstdcxx_constraint_value(version):
    return "libstdcxx." + version

def libstdcxx_constraint_label(version):
    return "@llvm//constraints/cxxstdlib:" + libstdcxx_constraint_value(version)

def gcc_target_suffix(version):
    return "gcc_" + version.replace(".", "_")

def _version_tuple(version):
    parts = version.split(".")
    values = [int(part) for part in parts]
    if len(values) == 1:
        values.extend([0, 0])
    elif len(values) == 2:
        values.append(0)
    return values

def _compare_versions(lhs, rhs):
    lhs_values = _version_tuple(lhs)
    rhs_values = _version_tuple(rhs)
    for i in range(3):
        if lhs_values[i] < rhs_values[i]:
            return -1
        if lhs_values[i] > rhs_values[i]:
            return 1
    return 0

def gcc_version_at_least_for(gcc_version, version):
    return _compare_versions(gcc_version, version) >= 0

def gcc_version_less_than_for(gcc_version, version):
    return _compare_versions(gcc_version, version) < 0

def _libstdcxx_patch(path):
    return "//3rd_party/gcc/patches:" + path

GCC_PATCHES = {
    "10.5.0": [
        _libstdcxx_patch("10.x/libstdcxx-cow-string-inst.patch"),
        _libstdcxx_patch("10.x/libstdcxx-constinit.patch"),
    ],
    "10.4.0": [
        _libstdcxx_patch("10.x/libstdcxx-cow-string-inst.patch"),
        _libstdcxx_patch("10.x/libstdcxx-constinit.patch"),
    ],
    "10.3.0": [
        _libstdcxx_patch("10.x/libstdcxx-cow-string-inst.patch"),
        _libstdcxx_patch("10.x/libstdcxx-constinit.patch"),
    ],
    "10.2.0": [
        _libstdcxx_patch("10.x/libstdcxx-cow-string-inst.patch"),
        _libstdcxx_patch("10.1-10.2/libstdcxx-filesystem-source-noexcept.patch"),
        _libstdcxx_patch("10.x/libstdcxx-constinit.patch"),
    ],
    "10.1.0": [
        _libstdcxx_patch("10.x/libstdcxx-cow-string-inst.patch"),
        _libstdcxx_patch("10.1-10.2/libstdcxx-filesystem-source-noexcept.patch"),
        _libstdcxx_patch("10.x/libstdcxx-constinit.patch"),
    ],
    "9.5.0": [
        _libstdcxx_patch("9.3-9.5/libstdcxx-cow-string-inst.patch"),
        _libstdcxx_patch("9.5.0/libstdcxx-experimental-fs-path-noexcept.patch"),
        _libstdcxx_patch("9.x/libstdcxx-pstl-disable-parallel-policies.patch"),
    ],
    "9.4.0": [
        _libstdcxx_patch("9.3-9.5/libstdcxx-cow-string-inst.patch"),
        _libstdcxx_patch("9.x/libstdcxx-pstl-disable-parallel-policies.patch"),
    ],
    "9.3.0": [
        _libstdcxx_patch("9.3-9.5/libstdcxx-cow-string-inst.patch"),
        _libstdcxx_patch("9.3.0/libstdcxx-filesystem-source-noexcept.patch"),
        _libstdcxx_patch("9.x/libstdcxx-pstl-disable-parallel-policies.patch"),
    ],
    "9.2.0": [
        _libstdcxx_patch("9.1-9.2/libstdcxx-cow-string-inst.patch"),
        _libstdcxx_patch("9.1-9.2/libstdcxx-filesystem-source-noexcept.patch"),
        _libstdcxx_patch("9.x/libstdcxx-pstl-disable-parallel-policies.patch"),
    ],
    "9.1.0": [
        _libstdcxx_patch("9.1-9.2/libstdcxx-cow-string-inst.patch"),
        _libstdcxx_patch("9.1-9.2/libstdcxx-filesystem-source-noexcept.patch"),
        _libstdcxx_patch("9.x/libstdcxx-pstl-disable-parallel-policies.patch"),
    ],
}

def gcc_patches(version):
    return GCC_PATCHES.get(version, [])

def gcc_version_at_least(version):
    return gcc_version_at_least_for(GCC_VERSION, version)

def gcc_version_less_than(version):
    return gcc_version_less_than_for(GCC_VERSION, version)

def select_for_gcc_version(values):
    choices = {
        libstdcxx_constraint_label(version): values[version]
        for version in GCC_VERSIONS
    }
    choices["//conditions:default"] = values[DEFAULT_GCC_VERSION]
    return select(choices)

def select_gcc_version_at_least(version, then, otherwise):
    return select_for_gcc_version({
        gcc_version: then if gcc_version_at_least_for(gcc_version, version) else otherwise
        for gcc_version in GCC_VERSIONS
    })

def gcc_has_config_toolexeclibdir_m4(version):
    return gcc_version_at_least_for(version, "10.0.0")

def libstdcxx_has_atomic_builtins_define(version):
    return gcc_version_less_than_for(version, "16.0.0")

def libstdcxx_has_posix_semaphore_check(version):
    return gcc_version_at_least_for(version, "11.0.0") and gcc_version_less_than_for(version, "16.0.0")

def libstdcxx_has_debugging_checks(version):
    return gcc_version_at_least_for(version, "16.0.0")

def libstdcxx_has_stdio_locking_checks(version):
    return gcc_version_at_least_for(version, "16.0.0")

def libstdcxx_has_networking_o_nonblock_check(version):
    return (gcc_version_at_least_for(version, "12.5.0") and gcc_version_less_than_for(version, "13.0.0")) or (gcc_version_at_least_for(version, "13.4.0") and gcc_version_less_than_for(version, "14.0.0")) or gcc_version_at_least_for(version, "14.3.0")

def libstdcxx_has_struct_tm_tm_zone_check(version):
    return gcc_version_at_least_for(version, "15.0.0")

def libstdcxx_has_c99_cxx11_detail_checks(version):
    return gcc_version_at_least_for(version, "14.0.0")

def libstdcxx_has_fseeko_ftello_check(version):
    return gcc_version_at_least_for(version, "13.2.0")

def libstdcxx_has_filesystem_chdir_chmod_getcwd_mkdir_checks(version):
    return (gcc_version_at_least_for(version, "12.4.0") and gcc_version_less_than_for(version, "13.0.0")) or gcc_version_at_least_for(version, "13.3.0")

def libstdcxx_has_filesystem_openat_check(version):
    return (gcc_version_at_least_for(version, "11.5.0") and gcc_version_less_than_for(version, "12.0.0")) or gcc_version_at_least_for(version, "12.2.0")

def libstdcxx_has_filesystem_copy_file_range_check(version):
    return gcc_version_at_least_for(version, "14.0.0")

def libstdcxx_has_text_encoding_checks(version):
    return gcc_version_at_least_for(version, "14.0.0")

def libstdcxx_has_alignas_init_priority_checks(version):
    return gcc_version_at_least_for(version, "13.2.0")

def libstdcxx_has_zoneinfo_policy(version):
    return gcc_version_at_least_for(version, "13.0.0")

def libstdcxx_has_int128_float128_checks(version):
    return gcc_version_less_than_for(version, "12.0.0")

def libstdcxx_has_uchar_char8_checks(version):
    return gcc_version_at_least_for(version, "12.0.0")

def libstdcxx_has_int64_t_checks(version):
    return gcc_version_less_than_for(version, "12.0.0")

def libstdcxx_has_cxx11_no_sleep_define(version):
    return gcc_version_at_least_for(version, "11.3.0")

def libstdcxx_has_filesystem_dirfd_checks(version):
    return gcc_version_at_least_for(version, "11.5.0")

def libstdcxx_has_decl_strnlen_check(version):
    return gcc_version_at_least_for(version, "12.0.0")

def libstdcxx_has_arc4random_getentropy_checks(version):
    return gcc_version_at_least_for(version, "12.0.0")

def libstdcxx_has_stdlib_secure_getenv_check(version):
    return gcc_version_at_least_for(version, "11.4.0")

def libstdcxx_has_no_sleep_policy(version):
    return gcc_version_at_least_for(version, "11.0.0")

def libstdcxx_has_uselocale_check(version):
    return gcc_version_at_least_for(version, "11.0.0")

def libstdcxx_has_system_error_check(version):
    return gcc_version_less_than_for(version, "9.5.0") or (gcc_version_at_least_for(version, "10.1.0") and gcc_version_less_than_for(version, "10.3.0"))

def libstdcxx_has_pthread_clock_checks(version):
    return gcc_version_at_least_for(version, "10.0.0")

def libstdcxx_has_x86_rdseed_check(version):
    return gcc_version_at_least_for(version, "10.0.0")

def libstdcxx_has_dev_random_policy(version):
    return gcc_version_at_least_for(version, "9.0.0")

def libstdcxx_has_filesystem_extra_posix_checks(version):
    return gcc_version_at_least_for(version, "9.0.0")

def libstdcxx_has_sockatmark_wfopen_checks(version):
    return gcc_version_at_least_for(version, "9.0.0")

def libstdcxx_has_stdlib_timespec_get_check(version):
    return gcc_version_at_least_for(version, "9.0.0")

def libstdcxx_has_atomic_lock_policy_define(version):
    return gcc_version_at_least_for(version, "9.0.0")

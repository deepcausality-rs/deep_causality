# Generated from linux sysroot CSV data.
# Run: ./generate_constraints.py > constraints.bzl

# notes: Musl target.
# source_url: https://alpinelinux.org/posts/Alpine-3.12.0-released.html
ALPINE_3_12_CONSTRAINTS = [
    "@llvm//constraints/libc:musl",
    "@llvm//constraints/kernel/linux:5.4",
]

# notes: First common post-time64 musl baseline in this table.
# source_url: https://alpinelinux.org/posts/Alpine-3.13.0-released.html
ALPINE_3_13_CONSTRAINTS = [
    "@llvm//constraints/libc:musl",
    "@llvm//constraints/kernel/linux:5.10",
]

# notes: Musl target.
# source_url: https://alpinelinux.org/posts/Alpine-3.14.0-released.html
ALPINE_3_14_CONSTRAINTS = [
    "@llvm//constraints/libc:musl",
    "@llvm//constraints/kernel/linux:5.10",
]

# notes: Musl target.
# source_url: https://alpinelinux.org/posts/Alpine-3.15.0-released.html
ALPINE_3_15_CONSTRAINTS = [
    "@llvm//constraints/libc:musl",
    "@llvm//constraints/kernel/linux:5.15",
]

# notes: Musl target.
# source_url: https://alpinelinux.org/posts/Alpine-3.16.0-released.html
ALPINE_3_16_CONSTRAINTS = [
    "@llvm//constraints/libc:musl",
    "@llvm//constraints/kernel/linux:5.15",
]

# notes: Musl target.
# source_url: https://alpinelinux.org/posts/Alpine-3.17.0-released.html
ALPINE_3_17_CONSTRAINTS = [
    "@llvm//constraints/libc:musl",
    "@llvm//constraints/kernel/linux:5.15",
]

# notes: Release notes call out linux-lts 6.1 and musl 1.2.4.
# source_url: https://wiki.alpinelinux.org/wiki/Release_Notes_for_Alpine_3.18.0
ALPINE_3_18_CONSTRAINTS = [
    "@llvm//constraints/libc:musl",
    "@llvm//constraints/kernel/linux:6.1",
]

# notes: Musl target.
# source_url: https://alpinelinux.org/posts/Alpine-3.19.0-released.html
ALPINE_3_19_CONSTRAINTS = [
    "@llvm//constraints/libc:musl",
    "@llvm//constraints/kernel/linux:6.6",
]

# notes: Musl target.
# source_url: https://alpinelinux.org/posts/Alpine-3.20.0-released.html
ALPINE_3_20_CONSTRAINTS = [
    "@llvm//constraints/libc:musl",
    "@llvm//constraints/kernel/linux:6.6",
]

# notes: Alpine 3.21 announcement highlights Linux kernel 6.12.
# source_url: https://alpinelinux.org/posts/Alpine-3.21.0-released.html
ALPINE_3_21_CONSTRAINTS = [
    "@llvm//constraints/libc:musl",
    "@llvm//constraints/kernel/linux:6.12",
]

# notes: Musl target.
# source_url: https://www.alpinelinux.org/posts/Alpine-3.22.0-released.html
ALPINE_3_22_CONSTRAINTS = [
    "@llvm//constraints/libc:musl",
    "@llvm//constraints/kernel/linux:6.12",
]

# notes: Release notes call out Linux kernel 6.18 except Pi kernels still on 6.12.
# source_url: https://www.alpinelinux.org/posts/Alpine-3.23.0-released.html
ALPINE_3_23_CONSTRAINTS = [
    "@llvm//constraints/libc:musl",
    "@llvm//constraints/kernel/linux:6.18",
]

# notes: Amazon Linux 1 final line; older AMI snapshots had earlier kernels.
# source_url: https://docs.aws.amazon.com/linux/al1/ug/relnotes-2018.03.html
AMAZON_LINUX_AMI_2018_03_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.17",
    "@llvm//constraints/kernel/linux:4.14",
]

# notes: AL2 reaches EOL 2026-06-30; AWS encourages kernel 5.10 or AL2023.
# source_url: https://docs.aws.amazon.com/AL2/latest/relnotes/relnotes-20260427.html
AMAZON_LINUX_2_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.26",
    "@llvm//constraints/kernel/linux:4.14",
]

# notes: Use deterministic release snapshots; do not collapse AL2023 to one kernel line for all AMIs.
# source_url: https://docs.aws.amazon.com/linux/al2023/release-notes/relnotes-2023.11.20260427.html
AMAZON_LINUX_2023_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.34",
    "@llvm//constraints/kernel/linux:6.1",
]

# notes: Very old Debian ABI reference.
# source_url: https://www.debian.org/releases/lenny/
DEBIAN_5_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.7",
    "@llvm//constraints/kernel/linux:2.6",
]

# notes: Debian used eglibc packaging; ABI baseline is GLIBC_2.11 era.
# source_url: https://www.debian.org/doc/manuals/project-history/releases.en.html
DEBIAN_6_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.11",
    "@llvm//constraints/kernel/linux:2.6",
]

# notes: Oldstable-era floor; not a practical current target.
# source_url: https://wiki.debian.org/DebianWheezy
DEBIAN_7_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.13",
    "@llvm//constraints/kernel/linux:3.2",
]

# notes: Old compatibility reference.
# source_url: https://www.debian.org/releases/jessie/
DEBIAN_8_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.19",
    "@llvm//constraints/kernel/linux:3.16",
]

# notes: Useful only when targeting old appliances or fleets.
# source_url: https://wiki.debian.org/NewInStretch
DEBIAN_9_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.24",
    "@llvm//constraints/kernel/linux:4.9",
]

# notes: Matches RHEL 8 glibc but has newer kernel headers.
# source_url: https://www.debian.org/releases/buster/
DEBIAN_10_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.28",
    "@llvm//constraints/kernel/linux:4.19",
]

# notes: Good old Debian and Ubuntu-ish floor; does not cover RHEL 8 glibc 2.28.
# source_url: https://www.debian.org/releases/bullseye/amd64/release-notes.en.txt
DEBIAN_11_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.31",
    "@llvm//constraints/kernel/linux:5.10",
]

# notes: Too new for EL9 or AL2023 if GLIBC_2.35+ symbols leak.
# source_url: https://www.debian.org/releases/bookworm/amd64/release-notes/ch-whats-new.en.html
DEBIAN_12_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.36",
    "@llvm//constraints/kernel/linux:6.1",
]

# notes: Not broad compatibility; use only for Debian 13/newer class.
# source_url: https://www.debian.org/releases/stable/release-notes/whats-new.html
DEBIAN_13_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.41",
    "@llvm//constraints/kernel/linux:6.12",
]

# notes: Forky/testing package baselines; not frozen until Debian 14 release.
# source_url: https://packages.debian.org/en/forky/libc6; https://packages.debian.org/linux-libc-dev
DEBIAN_14_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.42",
    "@llvm//constraints/kernel/linux:6.19",
]

# notes: Release-media kernel/header baseline; Fedora moves quickly after release.
# source_url: https://fedoraproject.org/wiki/Releases/18/Release_Notes
FEDORA_18_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.16",
    "@llvm//constraints/kernel/linux:3.6",
]

# notes: Release-media kernel/header baseline; Fedora moves quickly after release.
# source_url: https://fedoraproject.org/wiki/Releases/19/Release_Notes
FEDORA_19_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.17",
    "@llvm//constraints/kernel/linux:3.9",
]

# notes: Release-media kernel/header baseline; Fedora moves quickly after release.
# source_url: https://fedoraproject.org/wiki/Releases/20/Release_Notes
FEDORA_20_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.18",
    "@llvm//constraints/kernel/linux:3.11",
]

# notes: Release-media kernel/header baseline; Fedora moves quickly after release.
# source_url: https://fedoraproject.org/wiki/Releases/21/Release_Notes
FEDORA_21_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.20",
    "@llvm//constraints/kernel/linux:3.17",
]

# notes: Release-media kernel/header baseline; Fedora moves quickly after release.
# source_url: https://fedoraproject.org/wiki/Releases/22/Release_Notes
FEDORA_22_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.21",
    "@llvm//constraints/kernel/linux:4.0",
]

# notes: Release-media kernel/header baseline; Fedora moves quickly after release.
# source_url: https://fedoraproject.org/wiki/Releases/23/Release_Notes
FEDORA_23_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.22",
    "@llvm//constraints/kernel/linux:4.2",
]

# notes: Release-media kernel/header baseline; Fedora moves quickly after release.
# source_url: https://fedoraproject.org/wiki/Releases/24/Release_Notes
FEDORA_24_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.23",
    "@llvm//constraints/kernel/linux:4.5",
]

# notes: Release-media kernel/header baseline; Fedora moves quickly after release.
# source_url: https://fedoraproject.org/wiki/Releases/25/Release_Notes
FEDORA_25_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.24",
    "@llvm//constraints/kernel/linux:4.8",
]

# notes: Release-media kernel/header baseline; Fedora moves quickly after release.
# source_url: https://fedoraproject.org/wiki/Releases/26/Release_Notes
FEDORA_26_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.25",
    "@llvm//constraints/kernel/linux:4.11",
]

# notes: Release-media kernel/header baseline; Fedora moves quickly after release.
# source_url: https://fedoraproject.org/wiki/Releases/27/Release_Notes
FEDORA_27_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.26",
    "@llvm//constraints/kernel/linux:4.13",
]

# notes: Release-media kernel/header baseline; Fedora moves quickly after release.
# source_url: https://fedoraproject.org/wiki/Releases/28/Release_Notes
FEDORA_28_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.27",
    "@llvm//constraints/kernel/linux:4.16",
]

# notes: Release-media kernel/header baseline; Fedora moves quickly after release.
# source_url: https://fedoraproject.org/wiki/Releases/29/Release_Notes
FEDORA_29_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.28",
    "@llvm//constraints/kernel/linux:4.18",
]

# notes: Release notes document glibc 2.29; kernel is release-media baseline.
# source_url: https://docs.fedoraproject.org/en-US/fedora/f30/release-notes/developers/Development_C/
FEDORA_30_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.29",
    "@llvm//constraints/kernel/linux:5.0",
]

# notes: Release-media kernel/header baseline; Fedora moves quickly after release.
# source_url: https://fedoraproject.org/wiki/Releases/31/Release_Notes
FEDORA_31_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.30",
    "@llvm//constraints/kernel/linux:5.3",
]

# notes: Release-media kernel/header baseline; Fedora moves quickly after release.
# source_url: https://fedoraproject.org/wiki/Releases/32/Release_Notes
FEDORA_32_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.31",
    "@llvm//constraints/kernel/linux:5.6",
]

# notes: Release-media kernel/header baseline; Fedora moves quickly after release.
# source_url: https://fedoraproject.org/wiki/Releases/33/Release_Notes
FEDORA_33_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.32",
    "@llvm//constraints/kernel/linux:5.8",
]

# notes: Release-media kernel/header baseline; Fedora moves quickly after release.
# source_url: https://fedoraproject.org/wiki/Releases/34/Release_Notes
FEDORA_34_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.33",
    "@llvm//constraints/kernel/linux:5.11",
]

# notes: Release-media kernel/header baseline; Fedora moves quickly after release.
# source_url: https://fedoraproject.org/wiki/Releases/35/Release_Notes
FEDORA_35_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.34",
    "@llvm//constraints/kernel/linux:5.14",
]

# notes: Release-media kernel/header baseline; Fedora moves quickly after release.
# source_url: https://fedoraproject.org/wiki/Releases/36/Release_Notes
FEDORA_36_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.35",
    "@llvm//constraints/kernel/linux:5.17",
]

# notes: Release-media kernel/header baseline; Fedora moves quickly after release.
# source_url: https://fedoraproject.org/wiki/Releases/37/Release_Notes
FEDORA_37_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.36",
    "@llvm//constraints/kernel/linux:6.0",
]

# notes: Release-media kernel/header baseline; Fedora moves quickly after release.
# source_url: https://fedoraproject.org/wiki/Releases/38/Release_Notes
FEDORA_38_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.37",
    "@llvm//constraints/kernel/linux:6.2",
]

# notes: Release-media kernel/header baseline; Fedora moves quickly after release.
# source_url: https://fedoraproject.org/wiki/Releases/39/Release_Notes
FEDORA_39_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.38",
    "@llvm//constraints/kernel/linux:6.5",
]

# notes: Release coverage and package set show glibc 2.39 and Linux 6.8 class.
# source_url: https://fedoraproject.org/wiki/Releases/40/ChangeSet
FEDORA_40_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.39",
    "@llvm//constraints/kernel/linux:6.8",
]

# notes: Fedora kernels move within a release; this is the release/initial sane default.
# source_url: https://fedoraproject.org/wiki/Releases/41/ChangeSet
FEDORA_41_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.40",
    "@llvm//constraints/kernel/linux:6.11",
]

# notes: Fedora package pages move after release; keep this as initial baseline.
# source_url: https://packages.fedoraproject.org/pkgs/kernel/kernel/fedora-42.html
FEDORA_42_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.41",
    "@llvm//constraints/kernel/linux:6.14",
]

# notes: Fedora package pages move after release; keep this as initial baseline.
# source_url: https://packages.fedoraproject.org/pkgs/kernel/kernel/fedora-43.html
FEDORA_43_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.42",
    "@llvm//constraints/kernel/linux:6.17",
]

# notes: Fedora package page currently shows Fedora 44 on the 6.19 kernel line.
# source_url: https://packages.fedoraproject.org/pkgs/kernel/kernel/fedora-44.html
FEDORA_44_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.43",
    "@llvm//constraints/kernel/linux:6.19",
]

# notes: Historical only; RHCK follows RHEL 6 class while UEK can be newer.
# source_url: https://docs.oracle.com/en/operating-systems/oracle-linux/6/relnotes6.10/E96232.pdf
ORACLE_LINUX_6_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.12",
    "@llvm//constraints/kernel/linux:2.6",
]

# notes: User-space ABI is EL7-class; UEK choice is separate.
# source_url: https://blogs.oracle.com/scoter/oracle-linux-and-unbreakable-enterprise-kernel-uek-releases
ORACLE_LINUX_7_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.17",
    "@llvm//constraints/kernel/linux:3.10",
]

# notes: Oracle documents RHCK and UEK availability separately from user-space ABI.
# source_url: https://docs.oracle.com/en/operating-systems/oracle-linux/8/boot/oracle_linux8_kernel_version_matrix.html
ORACLE_LINUX_8_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.28",
    "@llvm//constraints/kernel/linux:4.18",
]

# notes: User-space compatibility is independent of kernel choice.
# source_url: https://docs.oracle.com/en/operating-systems/oracle-linux/9/boot/oracle_linux9_kernel_version_matrix.html
ORACLE_LINUX_9_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.34",
    "@llvm//constraints/kernel/linux:5.14",
]

# notes: Oracle Linux 10 ships UEK by default on x86_64 plus RHCK; both are 6.12 class.
# source_url: https://docs.oracle.com/en/operating-systems/oracle-linux/10/relnotes10.0/ol10.0-ShippedKernels.html
ORACLE_LINUX_10_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.39",
    "@llvm//constraints/kernel/linux:6.12",
]

# notes: Very old enterprise ABI floor.
# source_url: https://docs.redhat.com/en/documentation/Red_Hat_Enterprise_Linux/5/html/5.0_release_notes/index
RHEL_5_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.5",
    "@llvm//constraints/kernel/linux:2.6",
]

# notes: Old enterprise ABI floor; use only for explicit customer requirement.
# source_url: https://docs.redhat.com/en/documentation/red_hat_enterprise_linux/6/html/6.0_release_notes/index
RHEL_6_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.12",
    "@llvm//constraints/kernel/linux:2.6",
]

# notes: Ancient enterprise ABI floor.
# source_url: https://docs.redhat.com/en/documentation/red_hat_enterprise_linux/7/html-single/7.0_release_notes/index
RHEL_7_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.17",
    "@llvm//constraints/kernel/linux:3.10",
]

# notes: Choose this if RHEL 8 class compatibility matters.
# source_url: https://docs.redhat.com/en/documentation/red_hat_enterprise_linux/8/html-single/8.0_release_notes/index
RHEL_8_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.28",
    "@llvm//constraints/kernel/linux:4.18",
]

# notes: Good EL9 and AL2023 class baseline.
# source_url: https://docs.redhat.com/en/documentation/red_hat_enterprise_linux/9/html-single/9.0_release_notes/index
RHEL_9_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.34",
    "@llvm//constraints/kernel/linux:5.14",
]

# notes: RHEL 10 requires x86-64-v3 on x86_64; keep CPU baseline separate from libc/UAPI.
# source_url: https://docs.redhat.com/en/documentation/red_hat_enterprise_linux/10/html-single/10.0_release_notes/index
RHEL_10_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.39",
    "@llvm//constraints/kernel/linux:6.12",
]

# notes: Historical clone; not a current target.
# source_url: https://docs.redhat.com/en/documentation/Red_Hat_Enterprise_Linux/5/html/5.0_release_notes/index
CENTOS_LINUX_5_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.5",
    "@llvm//constraints/kernel/linux:2.6",
]

# notes: Historical clone; not a current target.
# source_url: https://docs.redhat.com/en/documentation/red_hat_enterprise_linux/6/html/6.0_release_notes/index
CENTOS_LINUX_6_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.12",
    "@llvm//constraints/kernel/linux:2.6",
]

# notes: Use only if explicit old fleet support is required.
# source_url: https://docs.redhat.com/en/documentation/red_hat_enterprise_linux/7/html-single/7.0_release_notes/index
CENTOS_LINUX_7_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.17",
    "@llvm//constraints/kernel/linux:3.10",
]

# notes: Treat as EL8-compatible for ABI; distro package revisions differ.
# source_url: https://docs.redhat.com/en/documentation/red_hat_enterprise_linux/8/html-single/8.0_release_notes/index
ROCKY_LINUX_ALMALINUX_8_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.28",
    "@llvm//constraints/kernel/linux:4.18",
]

# notes: Treat as EL9-compatible for ABI; distro package revisions differ.
# source_url: https://docs.redhat.com/en/documentation/red_hat_enterprise_linux/9/html-single/9.0_release_notes/index
ROCKY_LINUX_ALMALINUX_9_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.34",
    "@llvm//constraints/kernel/linux:5.14",
]

# notes: Keep x86-64-v3 CPU baseline separate from libc/UAPI.
# source_url: https://docs.redhat.com/en/documentation/red_hat_enterprise_linux/10/html-single/10.0_release_notes/index
ROCKY_LINUX_ALMALINUX_10_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.39",
    "@llvm//constraints/kernel/linux:6.12",
]

# notes: Stream is moving; use RHEL major for a frozen ABI label.
# source_url: https://docs.redhat.com/en/documentation/red_hat_enterprise_linux/8/html-single/8.0_release_notes/index
CENTOS_STREAM_8_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.28",
    "@llvm//constraints/kernel/linux:4.18",
]

# notes: Stream is moving; use RHEL major for a frozen ABI label.
# source_url: https://docs.redhat.com/en/documentation/red_hat_enterprise_linux/9/html-single/9.0_release_notes/index
CENTOS_STREAM_9_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.34",
    "@llvm//constraints/kernel/linux:5.14",
]

# notes: Useful for testing; less useful as a frozen compatibility label.
# source_url: https://docs.redhat.com/en/documentation/red_hat_enterprise_linux/10/html-single/10.0_release_notes/index
CENTOS_STREAM_10_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.39",
    "@llvm//constraints/kernel/linux:6.12",
]

# notes: Old SUSE enterprise floor; service pack matters.
# source_url: https://documentation.suse.com/releasenotes/sles/12-SP5/index.html
SLES_12_SP5_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.22",
    "@llvm//constraints/kernel/linux:4.12",
]

# notes: Do not collapse all SLES 15 service packs into one baseline.
# source_url: https://www.suse.com/releasenotes/x86_64/SUSE-SLES/15/index.html
SLES_15_GA_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.26",
    "@llvm//constraints/kernel/linux:4.12",
]

# notes: Kernel jumped while glibc stayed old.
# source_url: https://www.suse.com/releasenotes/x86_64/SUSE-SLES/15-SP2/index.html
SLES_15_SP2_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.26",
    "@llvm//constraints/kernel/linux:5.3",
]

# notes: Use SP-specific baseline.
# source_url: https://susedoc.github.io/release-notes/sles-15_SP4/html/release-notes/
SLES_15_SP4_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.31",
    "@llvm//constraints/kernel/linux:5.14",
]

# notes: Use SP-specific baseline.
# source_url: https://www.suse.com/releasenotes/x86_64/SUSE-SLES/15-SP5/index.html
SLES_15_SP5_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.31",
    "@llvm//constraints/kernel/linux:5.14",
]

# notes: SUSE release notes call out glibc 2.38 and Linux 6.4.
# source_url: https://documentation.suse.com/releasenotes/sles/15-SP6/index.html
SLES_15_SP6_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.38",
    "@llvm//constraints/kernel/linux:6.4",
]

# notes: SP7 remains 6.4 and glibc 2.38 class.
# source_url: https://www.suse.com/releasenotes/x86_64/SUSE-SLES/15-SP7/index.html
SLES_15_SP7_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.38",
    "@llvm//constraints/kernel/linux:6.4",
]

# notes: SLES 16.0 modern baseline.
# source_url: https://www.suse.com/c/what-is-new-in-suse-linux-enterprise-server-16-0/
SLES_16_0_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.40",
    "@llvm//constraints/kernel/linux:6.12",
]

# notes: Leap tracks SLE service-pack baselines.
# source_url: https://get.opensuse.org/leap/15.4/
OPENSUSE_LEAP_15_4_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.31",
    "@llvm//constraints/kernel/linux:5.14",
]

# notes: Leap tracks SLE service-pack baselines.
# source_url: https://get.opensuse.org/leap/15.5/
OPENSUSE_LEAP_15_5_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.31",
    "@llvm//constraints/kernel/linux:5.14",
]

# notes: Leap 15.6 is built on SLE 15 SP6; use SP-specific baseline.
# source_url: https://news.opensuse.org/2024/03/07/leap-reaches-beta-phase/
OPENSUSE_LEAP_15_6_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.38",
    "@llvm//constraints/kernel/linux:6.4",
]

# notes: First Ubuntu LTS class baseline; too old for normal targets.
# source_url: https://wiki.ubuntu.com/DapperReleaseNotes
UBUNTU_6_06_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.3",
    "@llvm//constraints/kernel/linux:2.6",
]

# notes: Old LTS compatibility reference.
# source_url: https://wiki.ubuntu.com/HardyReleaseNotes
UBUNTU_8_04_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.7",
    "@llvm//constraints/kernel/linux:2.6",
]

# notes: Ubuntu used eglibc packaging; ABI baseline is GLIBC_2.11 era.
# source_url: https://wiki.ubuntu.com/LucidLynx/ReleaseNotes
UBUNTU_10_04_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.11",
    "@llvm//constraints/kernel/linux:2.6",
]

# notes: GA kernel only; later point-release kernels intentionally excluded.
# source_url: https://wiki.ubuntu.com/PrecisePangolin/ReleaseNotes/UbuntuDesktop
UBUNTU_12_04_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.15",
    "@llvm//constraints/kernel/linux:3.2",
]

# notes: GA kernel only; later point-release kernels intentionally excluded.
# source_url: https://wiki.ubuntu.com/TrustyTahr/ReleaseNotes
UBUNTU_14_04_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.19",
    "@llvm//constraints/kernel/linux:3.13",
]

# notes: GA kernel only; useful as an old Linux floor if still required.
# source_url: https://wiki.ubuntu.com/XenialXerus/ReleaseNotes/16.04
UBUNTU_16_04_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.23",
    "@llvm//constraints/kernel/linux:4.4",
]

# notes: Use only if intentionally supporting 18.04 or similarly old fleets.
# source_url: https://wiki.ubuntu.com/BionicBeaver/ReleaseNotes/18.04
UBUNTU_18_04_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.27",
    "@llvm//constraints/kernel/linux:4.15",
]

# notes: GA kernel only; later point-release kernels intentionally excluded.
# source_url: https://wiki.ubuntu.com/FocalFossa/ReleaseNotes
UBUNTU_20_04_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.31",
    "@llvm//constraints/kernel/linux:5.4",
]

# notes: Too new for RHEL 8 or Ubuntu 20.04 if GLIBC_2.32+ symbols leak.
# source_url: https://documentation.ubuntu.com/release-notes/22.04/
UBUNTU_22_04_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.35",
    "@llvm//constraints/kernel/linux:5.15",
]

# notes: Good 2024+ target; not broad backwards compatibility.
# source_url: https://documentation.ubuntu.com/release-notes/24.04/
UBUNTU_24_04_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.39",
    "@llvm//constraints/kernel/linux:6.8",
]

# notes: Released 2026-04-23; too new as a general compatibility floor.
# source_url: https://documentation.ubuntu.com/release-notes/26.04/changes-since-previous-interim/; https://launchpad.net/ubuntu/resolute/+source/glibc
UBUNTU_26_04_CONSTRAINTS = [
    "@llvm//constraints/libc:gnu.2.42",
    "@llvm//constraints/kernel/linux:7.0",
]

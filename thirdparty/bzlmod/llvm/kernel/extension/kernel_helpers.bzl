def arch_to_kernel_arch(arch):
    """Convert the architecture name used in the glibc to the one used in the kernel.

    Args:
        arch: The architecture name used in the glibc.

    Returns:
        The architecture name used in the kernel.
    """
    if arch == "x86_64":
        return "x86"
    elif arch == "aarch64":
        return "arm64"
    elif arch == "riscv64":
        return "riscv"
    elif arch == "s390x":
        return "s390"
    elif arch == "armv7":
        return "arm"
    else:
        return arch

ARCH_ALIASES = {
    "x86_64": ["amd64"],
    "aarch64": ["arm64"],
    "riscv64": [],
    "s390x": [],
    "armv7": [],
}

SUPPORTED_TARGETS = [
    ("macos", "x86_64"),
    ("macos", "aarch64"),
    ("linux", "x86_64"),
    ("linux", "aarch64"),
    ("linux", "riscv64"),
    ("linux", "s390x"),
    ("linux", "armv7"),
    ("windows", "x86_64"),
    ("windows", "aarch64"),
    ("none", "bpfeb"),
    ("none", "bpfel"),
    ("none", "wasm32"),
    ("none", "wasm64"),
]

SUPPORTED_EXECS = [
    ("macos", "x86_64"),
    ("macos", "aarch64"),
    ("linux", "x86_64"),
    ("linux", "aarch64"),
    ("windows", "x86_64"),
    ("windows", "aarch64"),
]

LIBC_SUPPORTED_TARGETS = [
    ("linux", "x86_64"),
    ("linux", "aarch64"),
    ("linux", "riscv64"),
    ("linux", "s390x"),
    ("linux", "armv7"),
]

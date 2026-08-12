# Partly from https://github.com/cerisier/glibc-headers/blob/main/glibc_kernel_versions.txt

LIBC_KERNEL_VERSIONS = {
    "gnu.2.17": "3.9.11",
    "gnu.2.18": "3.12.74",
    "gnu.2.19": "3.15.10",
    "gnu.2.20": "3.17.8",
    "gnu.2.21": "4.0.9",
    "gnu.2.22": "4.3.6",
    "gnu.2.23": "4.6.7",
    "gnu.2.24": "4.8.17",
    "gnu.2.25": "4.11.12",
    "gnu.2.26": "4.14.336",
    "gnu.2.27": "4.16.18",
    "gnu.2.28": "4.19.325",
    "gnu.2.29": "5.1.21",
    "gnu.2.30": "5.4.293",
    "gnu.2.31": "5.7.19",
    "gnu.2.32": "5.9.16",
    "gnu.2.33": "5.12.19",
    "gnu.2.34": "5.15.182",
    "gnu.2.35": "5.17.15",
    "gnu.2.36": "6.0.19",
    "gnu.2.37": "6.3.13",
    "gnu.2.38": "6.6.90",
    "gnu.2.39": "6.9.12",
    "gnu.2.40": "6.12.28",
    "gnu.2.41": "6.14.6",
    "gnu.2.42": "6.16.12",
    "gnu.2.43": "6.18.15",
    "gnu.2.44": "7.0.10",
    "musl": "6.16.12",  # Latest for musl always
}

LIBC_KERNEL_VERSIONS["unconstrained"] = LIBC_KERNEL_VERSIONS["gnu.2.28"]

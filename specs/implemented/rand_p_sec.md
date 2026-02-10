# Random Number Seeding Security Analysis

## Current Implementation

The `deep_causality_rand` crate employs two distinct strategies for random number generation and seeding, controlled by the `os-random` feature flag.

### 1. Default Configuration (Internal PRNG)
**Feature Flag**: `os-random` is **DISABLED** (default).

*   **Generator**: `Xoshiro256**` (xoshiro256-star-star).
*   **Storage**: Thread-local (`thread_local!`). Each thread maintains its own independent generator state.
*   **Seeding Mechanism**:
    *   **Source**: A combination of a fixed 64-bit constant (`0x736f6d6570736575`) and the cryptographic hash of the current **Thread ID**.
    *   **Entropy**: The hashing uses `std::collections::hash_map::RandomState`. This inputs process-level entropy because `RandomState` is randomized by the OS at process initialization.
    *   **Expansion**: The initial 64-bit seed (derived from the hash) is expanded into the 256-bit state required by Xoshiro256 using the **SplitMix64** algorithm.
*   **Security Properties**:
    *   **Uniqueness**: Guaranteed unique streams per thread (via Thread ID) and unique seeds per process run (via `RandomState`).
    *   **Predictability**: `Xoshiro256**` is a **non-cryptographic** PRNG. If the internal state is recovered, all future (and past) output can be predicted with 100% accuracy.
    *   **Weaknesses**: Susceptible to correlation attacks (linear complexity) if not for the scramblers, but state recovery is trivial given sufficient consecutive outputs (e.g., 256 bits of output). It is **UNSAFE** for key generation, nonces, or any security-sensitive operation.

### 2. Secure Configuration (OS CSPRNG)
**Feature Flag**: `os-random` is **ENABLED**.

*   **Generator**: `OsRandomRng`.
*   **Storage**: Stateless.
*   **Seeding Mechanism**:
    *   **Mechanism**: No user-space seeding. Calls `getrandom` (OS CSPRNG) directly.
*   **Security Properties**:
    *   Relies entirely on the operating system's CSPRNG (Cryptographically Secure Pseudo-Random Number Generator).
    *   Suitable for cryptographic keys when the underlying OS RNG is secure.

## Underlying CSPRNG Implementation (macOS)

The `OsRandomRng` relies on the `getrandom` crate (version **0.4.1**). On **macOS**, this resolves to the following kernel interface:

*   **Interface**: `getentropy(2)` (available in `<sys/random.h>`).
    *   Check preference: `getrandom` prioritizes `getentropy` over the older `/dev/urandom` file interface on supported macOS versions (10.12+).
*   **Source**: The macOS kernel's entropy pool.
    *   **Algorithm**: The kernel uses a Yarrow-based or Fortuna-based CSPRNG (depending on the exact macOS kernel version).
    *   **Security**: This source is considered cryptographically secure and is suitable for generating long-term cryptographic keys.
    *   **Blocking**: `getentropy` may block if the kernel's entropy pool has not yet been initialized (typically only relevant at very early boot).

## ChaCha20 CSPRNG Implementation

To address the need for a cryptographically secure userspace PRNG (e.g., for reproducible simulations requiring security or massive throughput without syscall overhead), we rely on the audited AED crate (`chacha20poly1305`).

### Feature Flags
This implementation will be guarded by a new feature flag:
*   **Feature**: `aead-random`
*   **Behavior**: When enabled, exposes the `ChaCha20Rng` struct.

### Design
To utilize the audited AEAD implementation as a CSPRNG:

*   **Crate**: `chacha20poly1305` (Pure Rust, constant-time, explicitly audited by NCC Group).
*   **Algorithm**: `ChaCha20Poly1305` (IETF Interface).
*   **Dependencies**:
    *   `chacha20poly1305` (Optional, enabled by `aead-random`)
    *   `zeroize` (Optional, enabled by `aead-random`) - **Critical for Security**
    *   `getrandom` (Optional, enabled by `aead-random`)
*   **Mechanism**:
    *   **Seed**: 32-byte Key (from OS or user).
    *   **State**: 12-byte Nonce (Counter) + Internal Buffer.
    *   **Step Function**:
        1.  Increment Nonce (Counter).
        2.  Encrypt a block of Zeros (`[0u8; 64]` or larger).
        3.  The resulting **Ciphertext** (stripped of the Poly1305 tag) is the keystream (random output).
        4.  Authenticate Tag is discarded.

### Hybrid Seeding (Defense in Depth)

To mitigate potential hardware backdoors in the OS RNG (e.g., weakened RDRAND or compromise of `/dev/urandom`), the `aead-random` implementation employs a **Hybrid Seeding Strategy**.

*   **Hardware Entropy**: 32 bytes from `getrandom` (OS CSPRNG).
*   **Software Entropy**: A 64-bit entropy derived from:
    *   `SystemTime` (absolute epoch time, nanosecond precision).
    *   `Instant` (monotonic uptime, nanosecond precision).
    *   `ThreadId` (hashed process/thread identity).
    *   `Stack Address` (ASLR memory layout).
    *   `Heap Address` (ASLR heap layout).
    *   `RDTSC` (x86_64) or `CNTVCT_EL0` (aarch64) - High-resolution CPU cycle counter.
*   **Mixing**: The software entropy is XORed into the hardware seed with rotation. This ensures that even if the hardware seed is predictable (backdoored), the final seed depends on the exact nanosecond execution time and deep microarchitectural state, making remote prediction computationally infeasible.

### Memory Security (Zeroization)
To prevent key material from lingering in memory after use or drop:
*   **Requirement**: The internal state containing the Key (Seed) MUST be zeroized on Drop.
*   **Implementation**: Use the `zeroize` crate to derive `ZeroizeOnDrop` for the RNG struct (specifically the key storage).
*   **Tag Handling**: The Poly1305 Authentication Tag is public information in many contexts, but in an RNG context, it is a byproduct. While less critical than the key, zeroizing the intermediate buffer (which holds the plaintext/ciphertext/tag) is a best practice to prevent any potential state leakage.

### Verified Properties
*   **Audited**: References the NCC Group audit of the `chacha20poly1305` crate.
*   **Constant-Time**: The implementation is designed to be constant-time.
*   **Forward Secrecy (Partial)**: Systematic reseeding (e.g., Ratcheting) provides forward secrecy. Without ratcheting, compromise of the current state (Key + Nonce) reveals all future outputs.
    *   *Note*: Zeroization only protects "dead" states (dropped pointers). It does not protect the "live" state in registers during execution.

/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_rand::RngCore;

// Mock RngCore for deterministic testing
struct MockRngCore {
    values_u64: Vec<u64>,
    index: usize,
}

impl MockRngCore {
    fn new(values_u64: Vec<u64>) -> Self {
        MockRngCore {
            values_u64,
            index: 0,
        }
    }
}

impl RngCore for MockRngCore {
    fn next_u64(&mut self) -> u64 {
        let val = self.values_u64[self.index];
        self.index = (self.index + 1) % self.values_u64.len();
        val
    }

    fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32 // Take lower 32 bits of the next u64
    }
}

#[test]
fn test_rng_core_next_u64() {
    let mut rng = MockRngCore::new(vec![1, 2, 3]);
    assert_eq!(rng.next_u64(), 1);
    assert_eq!(rng.next_u64(), 2);
    assert_eq!(rng.next_u64(), 3);
    assert_eq!(rng.next_u64(), 1); // Loop back
}

#[test]
fn test_rng_core_next_u32() {
    // next_u32 uses the default implementation, which calls next_u64
    let mut rng = MockRngCore::new(vec![0x00000000FFFFFFFF, 0xFFFFFFFF00000000]);
    assert_eq!(rng.next_u32(), 4294967295);
    assert_eq!(rng.next_u32(), 0);
}

#[test]
fn test_rng_core_fill_bytes() {
    let mut rng = MockRngCore::new(vec![0x0102030405060708, 0x090A0B0C0D0E0F10]);
    let mut buffer = [0u8; 16];
    rng.fill_bytes(&mut buffer);
    assert_eq!(
        buffer,
        [
            0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01, 0x10, 0x0F, 0x0E, 0x0D, 0x0C, 0x0B,
            0x0A, 0x09
        ]
    );

    // Test with a smaller buffer
    let mut rng = MockRngCore::new(vec![0x1122334455667788]);
    let mut buffer_small = [0u8; 4];
    rng.fill_bytes(&mut buffer_small);
    assert_eq!(buffer_small, [0x88, 0x77, 0x66, 0x55]);

    // Test with a buffer that's not a multiple of 8
    let mut rng = MockRngCore::new(vec![0xAABBCCDDEEFF0011, 0x2233445566778899]);
    let mut buffer_odd = [0u8; 10];
    rng.fill_bytes(&mut buffer_odd);
    assert_eq!(
        buffer_odd,
        [0x11, 0x00, 0xFF, 0xEE, 0xDD, 0xCC, 0xBB, 0xAA, 0x99, 0x88]
    );
}

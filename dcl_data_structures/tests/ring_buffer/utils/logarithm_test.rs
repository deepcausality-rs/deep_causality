use dcl_data_structures::ring_buffer::utils::logarithm::log2;

#[test]
fn test_log2_zero() {
    assert_eq!(log2(0), 0);
}

#[test]
fn test_log2_one() {
    assert_eq!(log2(1), 0);
}

#[test]
fn test_log2_powers_of_two() {
    assert_eq!(log2(2), 1);
    assert_eq!(log2(4), 2);
    assert_eq!(log2(8), 3);
    assert_eq!(log2(16), 4);
    assert_eq!(log2(32), 5);
    assert_eq!(log2(64), 6);
    assert_eq!(log2(128), 7);
    assert_eq!(log2(256), 8);
}

#[test]
fn test_log2_non_powers_of_two() {
    assert_eq!(log2(3), 1);
    assert_eq!(log2(5), 2);
    assert_eq!(log2(7), 2);
    assert_eq!(log2(9), 3);
    assert_eq!(log2(15), 3);
    assert_eq!(log2(31), 4);
    assert_eq!(log2(63), 5);
}

#[test]
fn test_log2_large_numbers() {
    assert_eq!(log2(1u64 << 63), 63);
    assert_eq!(log2((1u64 << 63) - 1), 62);
    assert_eq!(log2(u64::MAX), 63);
}

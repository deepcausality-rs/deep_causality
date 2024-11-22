use std::num::NonZeroUsize;
use std::sync::Arc;
use std::thread;
use dcl_data_structures::ring_buffer::utils::bit_map::BitMap;

#[test]
fn test_bitmap_new() {
    let capacity = NonZeroUsize::new(64).unwrap();
    let bitmap = BitMap::new(capacity);
    assert!(!bitmap.is_set(0));
}

#[test]
fn test_bitmap_set_and_check() {
    let capacity = NonZeroUsize::new(128).unwrap();
    let bitmap = BitMap::new(capacity);
    
    bitmap.set(42);
    assert!(bitmap.is_set(42));
    assert!(!bitmap.is_set(43));
}

#[test]
fn test_bitmap_unset() {
    let capacity = NonZeroUsize::new(256).unwrap();
    let bitmap = BitMap::new(capacity);
    
    bitmap.set(100);
    assert!(bitmap.is_set(100));
    
    bitmap.unset(100);
    assert!(!bitmap.is_set(100));
}

#[test]
fn test_bitmap_multiple_operations() {
    let capacity = NonZeroUsize::new(512).unwrap();
    let bitmap = BitMap::new(capacity);
    
    // Set multiple bits
    for i in 0..10 {
        bitmap.set(i);
    }
    
    // Verify all bits are set
    for i in 0..10 {
        assert!(bitmap.is_set(i));
    }
    
    // Unset even numbers
    for i in (0..10).step_by(2) {
        bitmap.unset(i);
    }
    
    // Verify even numbers are unset and odd numbers are still set
    for i in 0..10 {
        if i % 2 == 0 {
            assert!(!bitmap.is_set(i));
        } else {
            assert!(bitmap.is_set(i));
        }
    }
}

#[test]
fn test_bitmap_boundary_conditions() {
    // Test with minimum capacity
    let min_capacity = NonZeroUsize::new(1).unwrap();
    let bitmap = BitMap::new(min_capacity);
    bitmap.set(0);
    assert!(bitmap.is_set(0));
    
    // Test with capacity that's not a power of 2
    let odd_capacity = NonZeroUsize::new(63).unwrap();
    let bitmap = BitMap::new(odd_capacity);
    bitmap.set(62);
    assert!(bitmap.is_set(62));
    
    // Test with large capacity
    let large_capacity = NonZeroUsize::new(1024).unwrap();
    let bitmap = BitMap::new(large_capacity);
    bitmap.set(1023);
    assert!(bitmap.is_set(1023));
}

#[test]
fn test_bitmap_word_boundary() {
    let capacity = NonZeroUsize::new(128).unwrap();
    let bitmap = BitMap::new(capacity);
    
    // Test bits at word boundaries (64-bit boundaries)
    bitmap.set(63);  // Last bit of first word
    bitmap.set(64);  // First bit of second word
    
    assert!(bitmap.is_set(63));
    assert!(bitmap.is_set(64));
    
    bitmap.unset(63);
    bitmap.unset(64);
    
    assert!(!bitmap.is_set(63));
    assert!(!bitmap.is_set(64));
}

#[test]
fn test_bitmap_set_unset_patterns() {
    let capacity = NonZeroUsize::new(256).unwrap();
    let bitmap = BitMap::new(capacity);
    
    // Test alternating pattern
    for i in 0..64 {
        if i % 2 == 0 {
            bitmap.set(i);
        }
    }
    
    for i in 0..64 {
        assert_eq!(bitmap.is_set(i), i % 2 == 0);
    }
    
    // Test set-all-unset-all pattern
    for i in 0..64 {
        bitmap.set(i);
    }
    
    for i in 0..64 {
        assert!(bitmap.is_set(i));
    }
    
    for i in 0..64 {
        bitmap.unset(i);
    }
    
    for i in 0..64 {
        assert!(!bitmap.is_set(i));
    }
}

#[test]
fn test_bitmap_concurrent_access() {
    let capacity = NonZeroUsize::new(1024).unwrap();
    let bitmap = Arc::new(BitMap::new(capacity));
    let mut handles = vec![];
    
    // Spawn threads to set bits
    for t in 0..8 {
        let bitmap = Arc::clone(&bitmap);
        let handle = thread::spawn(move || {
            for i in 0..64 {
                let index = t * 64 + i;
                bitmap.set(index as u64);
            }
        });
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Verify all bits were set
    for i in 0..512 {
        assert!(bitmap.is_set(i));
    }
    
    // Test concurrent unset operations
    let mut handles = vec![];
    for t in 0..8 {
        let bitmap = Arc::clone(&bitmap);
        let handle = thread::spawn(move || {
            for i in 0..64 {
                let index = t * 64 + i;
                bitmap.unset(index as u64);
            }
        });
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Verify all bits were unset
    for i in 0..512 {
        assert!(!bitmap.is_set(i));
    }
}

#[test]
fn test_bitmap_sequential_wrap() {
    let capacity = NonZeroUsize::new(64).unwrap();
    let bitmap = BitMap::new(capacity);
    
    // Test wrapping behavior
    for i in 0..128 {
        bitmap.set(i);
        assert!(bitmap.is_set(i));
        bitmap.unset(i);
        assert!(!bitmap.is_set(i));
    }
}



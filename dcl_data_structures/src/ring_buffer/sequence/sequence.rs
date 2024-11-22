use std::sync::atomic::{AtomicU64, Ordering};

pub type Sequence = u64;

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
const CACHE_LINE_SIZE: usize = 128;

#[cfg(target_arch = "x86_64")]
const CACHE_LINE_SIZE: usize = 64;

const CACHE_LINE_PADDING: usize = CACHE_LINE_SIZE - size_of::<AtomicU64>();

#[repr(align(64))]
pub struct AtomicSequence {
    _pad: [u8; CACHE_LINE_PADDING],
    offset: AtomicU64,
}

impl AtomicSequence {
    pub fn get(&self) -> Sequence {
        self.offset.load(Ordering::Acquire)
    }

    pub fn set(&self, value: Sequence) {
        self.offset.store(value, Ordering::Release);
    }

    pub fn compare_exchange(&self, current: Sequence, new: Sequence) -> bool {
        self.offset
            .compare_exchange(current, new, Ordering::SeqCst, Ordering::Acquire)
            .is_ok()
    }
}

impl Default for AtomicSequence {
    fn default() -> Self {
        Self {
            _pad: [0; CACHE_LINE_PADDING],
            offset: AtomicU64::default(),
        }
    }
}

impl From<Sequence> for AtomicSequence {
    fn from(value: Sequence) -> Self {
        Self {
            _pad: [0; CACHE_LINE_PADDING],
            offset: AtomicU64::new(value),
        }
    }
}

impl Into<Sequence> for AtomicSequence {
    fn into(self) -> Sequence {
        self.offset.into_inner()
    }
}

#[inline(always)]
pub fn log2(mut n: u64) -> u64 {
    let mut r = 0;
    loop {
        n >>= 1;
        if n == 0 {
            return r;
        }
        r += 1;
    }
}

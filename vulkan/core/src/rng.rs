//! xorshift32 PRNG standing in for `Math.random()`.

pub struct Rng(u32);

impl Rng {
    pub fn new(seed: u32) -> Self {
        Rng(if seed == 0 { 0x9E37_79B9 } else { seed })
    }

    pub fn from_time() -> Self {
        let n = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(12345);
        Rng::new((n ^ (n >> 32)) as u32)
    }

    fn next_u32(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        x
    }

    /// Uniform in [0, 1).
    pub fn random(&mut self) -> f64 {
        self.next_u32() as f64 / 4_294_967_296.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn in_range_and_deterministic() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        for _ in 0..1000 {
            let v = a.random();
            assert!((0.0..1.0).contains(&v));
            assert_eq!(v, b.random());
        }
    }
}

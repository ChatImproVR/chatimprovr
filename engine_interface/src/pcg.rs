extern "C" {
    fn _random() -> u64;
}

/// https://en.wikipedia.org/wiki/Permuted_congruential_generator
pub struct Pcg {
    state: u64,
    multiplier: u64,
    increment: u64,
}

impl Pcg {
    pub fn new() -> Self {
        let seed = unsafe { _random() };
        Self::new_detailed(seed, 6364136223846793005, 1442695040888963407)
    }

    fn new_detailed(seed: u64, multiplier: u64, increment: u64) -> Self {
        Self {
            state: seed + increment,
            multiplier,
            increment,
        }
    }

    fn u64_to_u32(x: u64) -> u32 {
        (x % u64::from(u32::MAX)) as u32
    }

    fn rotr32(x: u32, r: u32) -> u32 {
        x >> r | x << (r.wrapping_neg() & 31)
    }

    pub fn gen_u32(&mut self) -> u32 {
        let mut x = self.state;
        let count = x >> 59;
        self.state = x.wrapping_mul(self.multiplier).wrapping_add(self.increment);
        x ^= x >> 18;
        Self::rotr32(Self::u64_to_u32(x >> 27), Self::u64_to_u32(count))
    }

    pub fn gen_u128(&mut self) -> u128 {
        let mut n: u128 = 0;
        for _ in 0..4 {
            n <<= 32;
            n |= u128::from(self.gen_u32());
        }
        n
    }
}

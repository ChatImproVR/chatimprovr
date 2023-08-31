/*
extern "C" {
    #[cfg(target_family = "wasm")]
    fn _random() -> u64;
}
*/

/// https://en.wikipedia.org/wiki/Permuted_congruential_generator
#[derive(Default)]
pub struct Pcg {
    state: u64,
    multiplier: u64,
    increment: u64,
}

impl Pcg {
    pub fn new() -> Self {
        // #[cfg(target_family = "wasm")]
        // let seed = unsafe { _random() };

        // TODO: Handle this in a better way!
        // #[cfg(not(target_family = "wasm"))]
        let seed = 0;

        Self::new_detailed(seed, 6364136223846793005, 1442695040888963407)
    }

    fn new_detailed(seed: u64, multiplier: u64, increment: u64) -> Self {
        Self {
            state: seed.wrapping_add(increment),
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

    /// Generates a random u32
    pub fn gen_u32(&mut self) -> u32 {
        let mut x = self.state;
        let count = x >> 59;
        self.state = x.wrapping_mul(self.multiplier).wrapping_add(self.increment);
        x ^= x >> 18;
        Self::rotr32(Self::u64_to_u32(x >> 27), Self::u64_to_u32(count))
    }

    /// This is kind of bad. Whatever. Generates (MOST) f32s between 0 and 1.
    ///
    /// The actual distribution might be shit though.
    pub fn gen_f32(&mut self) -> f32 {
        self.gen_u32() as f32 / u32::MAX as f32
    }

    /// Generates a boolean
    pub fn gen_bool(&mut self) -> bool {
        self.gen_u32() & 1 == 0
    }

    /// Generates a u128
    pub fn gen_u128(&mut self) -> u128 {
        let mut n: u128 = 0;
        for _ in 0..4 {
            n <<= 32;
            n |= u128::from(self.gen_u32());
        }
        n
    }
}

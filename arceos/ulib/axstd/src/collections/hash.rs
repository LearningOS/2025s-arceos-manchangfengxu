use core::hash::{BuildHasher, Hasher};
use arceos_api::random::random;

pub struct RandomState;

impl BuildHasher for RandomState {
    type Hasher = SimpleHasher;

    fn build_hasher(&self) -> Self::Hasher {
        let seed = random();
        SimpleHasher { state: seed }
    }
}

pub struct SimpleHasher {
    state: u128,
}

impl Hasher for SimpleHasher {
    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.state = self.state.wrapping_mul(31).wrapping_add(*byte as u128);
        }
    }

    fn finish(&self) -> u64 {
        self.state as u64
    }
}

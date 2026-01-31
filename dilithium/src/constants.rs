// 𝑞 - modulus
pub const Q: u32 = 8380417;

// 𝜂 - private key range
pub const ETA: u32 = 2;

// 𝜏 (tau) - number of ±1's in polynomial 𝑐 (challenge)
pub const TAU: usize = 39; // For ML-DSA-44

// 𝜔 (omega) - max # of 1's in the hint 𝐡
pub const OMEGA: usize = 80; // For ML-DSA-44

// (𝑘, ℓ) - dimensions of 𝐀
pub const K: usize = 4;
pub const L: usize = 4;

// 𝛾1 - coefficient range of 𝐲
pub const GAMMA1: u32 = 2u32.pow(17);

// 𝛾2 - low-order rounding range
pub const GAMMA2: u32 = (Q - 1) / 88; // For ML-DSA-44

// 𝜆 - collision strength of 𝑐̃
pub const LAMBDA: usize = 128;

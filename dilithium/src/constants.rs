// 𝑞 - modulus
pub const Q: u32 = 8380417;

// 𝜂 - private key range
pub const ETA: u32 = 2;

// 𝜔 (omega) - Total nonzero coefficients allowed
pub const OMEGA: usize = 80;

// (𝑘, ℓ) - dimensions of 𝐀
pub const K: usize = 4;
pub const L: usize = 4;

// 𝛾1 - coefficient range of 𝐲
pub const GAMMA1: u32 = 2u32.pow(17);

// 𝛾2 - low-order rounding range
pub const GAMMA2: u32 = 95232;

// 𝜆 - collision strength of 𝑐
pub const LAMBDA: usize = 128;

use crate::constants::{OMEGA, Q};

type Polynomial = [u8; 256];

/// Algorithm 9
pub fn integer_to_bits(x: u32, alpha: usize) -> Vec<u8> {
    let mut y = Vec::with_capacity(alpha);
    let mut x_prime = x;

    for _ in 0..alpha {
        y.push((x_prime % 2) as u8);
        x_prime = x_prime / 2;
    }

    y
}

/// Algorithm 10
pub fn bits_to_integer(y: &[u8], alpha: usize) -> u32 {
    let mut x = 0u32;

    for i in 1..(alpha + 1) {
        x = 2 * x + y[alpha - i] as u32;
    }

    x
}

/// Algorithm 11
pub fn integer_to_bytes(x: u32, alpha: usize) -> Vec<u8> {
    let mut y = Vec::with_capacity(alpha);
    let mut x_prime = x;

    for _ in 0..alpha {
        y.push((x_prime % 256) as u8);
        x_prime = x_prime / 256;
    }

    y
}

/// Algorithm 12
pub fn bits_to_bytes(y: &[u8]) -> Vec<u8> {
    let alpha = y.len();
    let len = (alpha + 7) / 8;
    let mut z = vec![0u8; len];

    for i in 0..alpha {
        z[i / 8] += y[i] << (i % 8)
    }

    z
}

/// Algorithm 13
pub fn bytes_to_bits(z: Vec<u8>) -> Vec<u8> {
    let alpha = z.len();
    let len = 8 * alpha;
    let mut y = vec![0u8; len];
    let mut z_prime = z;

    for i in 0..alpha {
        for j in 0..8 {
            y[8 * i + j] = z_prime[i] % 2;
            z_prime[i] = z_prime[i] / 2;
        }
    }

    y
}

/// Algorithm 14
pub fn coeff_from_three_bytes(b0: u8, b1: u8, b2: u8) -> Option<u32> {
    let mut b2_prime = b2;
    if b2_prime > 127 {
        b2_prime -= 128;
    }

    let z = 65536 * (b2_prime as u32) + 256 * (b1 as u32) + (b0 as u32);

    if z < Q { Some(z) } else { None }
}

/// Algorithm 15
pub fn coeff_from_half_byte(b: u8, eta: u32) -> Option<i32> {
    if eta == 2 && b < 15 {
        Some(2 - (b % 5) as i32)
    } else if eta == 4 && b < 9 {
        Some(4 - b as i32)
    } else {
        None
    }
}

fn bitlen(x: u32) -> usize {
    if x == 0 {
        1
    } else {
        32 - x.leading_zeros() as usize
    }
}

/// Algorithm 16
pub fn simple_bit_pack(w: &[u32; 256], b: u32) -> Vec<u8> {
    let bits_per_coeff = bitlen(b);

    let mut z = Vec::new();

    for i in 0..256 {
        z.extend_from_slice(&integer_to_bits(w[i], bits_per_coeff));
    }

    bits_to_bytes(&z)
}

/// Algorithm 17
pub fn bit_pack(w: &[i32; 256], a: u32, b: u32) -> Vec<u8> {
    let bits_per_coeff = bitlen((a + b) as u32);

    let mut z = Vec::new();

    for i in 0..256 {
        z.extend_from_slice(&integer_to_bits((b as i32 - w[i]) as u32, bits_per_coeff));
    }

    bits_to_bytes(&z)
}

/// Algorithm 18
pub fn simple_bit_unpack(v: &[u8], b: u32) -> [u32; 256] {
    let c = bitlen(b);
    let z = bytes_to_bits(v.to_vec());
    let mut w = [0u32; 256];

    for i in 0..256 {
        let start = i * c;
        let end = start + c;
        let bits: Vec<u8> = z[start..end].to_vec();
        w[i] = bits_to_integer(&bits, c);
    }

    w
}

/// Algorithm 19
pub fn bit_unpack(v: &[u8], a: u32, b: u32) -> [i32; 256] {
    let c = bitlen(a + b);
    let z = bytes_to_bits(v.to_vec());
    let mut w = [0i32; 256];

    for i in 0..256 {
        let start = i * c;
        let end = start + c;
        let bits: Vec<u8> = z[start..end].to_vec();
        w[i] = b as i32 - bits_to_integer(&bits, c) as i32;
    }

    w
}

/// Algorithm 20
pub fn hint_bit_pack(h: &[Polynomial]) -> Vec<u8> {
    let k = h.len();
    let mut y = vec![0u8; OMEGA + k];
    let mut index = 0;

    for i in 0..k {
        for j in 0..256 {
            if h[i][j] != 0 {
                y[index] = j as u8;
                index += 1;
            }
        }
        y[OMEGA + i] = index as u8;
    }

    y
}

/// Algorithm 21
pub fn hint_bit_unpack(y: &[u8], k: usize, omega: usize) -> Option<Vec<Polynomial>> {
    let mut h = vec![[0u8; 256]; k];
    let mut index = 0usize;

    for i in 0..k {
        if (y[omega + i] as usize) < index || (y[omega + i] as usize) > omega {
            return None;
        }

        let first = index;

        while index < y[omega + i] as usize {
            if index > first {
                if y[index - 1] >= y[index] {
                    return None;
                }
            }

            h[i][y[index] as usize] = 1;
            index += 1;
        }
    }

    for i in index..omega {
        if y[i] != 0 {
            return None;
        }
    }

    Some(h)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integer_to_bits_test() {
        let x = 16;
        let alpha = 8;
        let result = integer_to_bits(x, alpha);
        let expected = vec![0, 0, 0, 0, 1, 0, 0, 0];

        assert_eq!(result, expected);
    }

    #[test]
    fn bits_to_integer_test() {
        let y = vec![0, 0, 0, 0, 1, 0, 0, 0];
        let alpha = 8;
        let result = bits_to_integer(&y, alpha);
        let expected = 16;

        assert_eq!(result, expected);
    }

    #[test]
    fn integer_to_bytes_test() {
        let x = 356;
        let alpha = 8;
        let result = integer_to_bytes(x, alpha);
        let expected = vec![100, 1, 0, 0, 0, 0, 0, 0];

        assert_eq!(result, expected);
    }

    #[test]
    fn bits_to_bytes_test() {
        let y = [0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0];
        let result = bits_to_bytes(&y);
        let expected = vec![102, 6];

        assert_eq!(result, expected);
    }

    #[test]
    fn bytes_to_bits_test() {
        let z = vec![102, 6];
        let result = bytes_to_bits(z);
        let expected = [0, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 0, 0, 0, 0];

        assert_eq!(result, expected);
    }

    #[test]
    fn coeff_from_three_bytes_test() {
        assert_eq!(coeff_from_three_bytes(0, 0, 0), Some(0));
        assert_eq!(coeff_from_three_bytes(1, 0, 0), Some(1));
        assert_eq!(coeff_from_three_bytes(1, 0, 128), Some(1));

        // Maximum valid value
        let result = coeff_from_three_bytes(0x00, 0xE0, 0x7F);
        assert!(result.is_some());

        // z = 127*65536 + 255*256 + 255 = 8,388,607 > q
        let result = coeff_from_three_bytes(255, 255, 127);
        assert_eq!(result, None);
    }

    #[test]
    fn coeff_from_half_byte_test() {
        // η = 2 tests
        assert_eq!(coeff_from_half_byte(0, 2), Some(2));
        assert_eq!(coeff_from_half_byte(1, 2), Some(1));
        assert_eq!(coeff_from_half_byte(2, 2), Some(0));
        assert_eq!(coeff_from_half_byte(3, 2), Some(-1));
        assert_eq!(coeff_from_half_byte(4, 2), Some(-2));
        assert_eq!(coeff_from_half_byte(14, 2), Some(-2));
        assert_eq!(coeff_from_half_byte(15, 2), None);

        // η = 4 tests
        assert_eq!(coeff_from_half_byte(0, 4), Some(4));
        assert_eq!(coeff_from_half_byte(4, 4), Some(0));
        assert_eq!(coeff_from_half_byte(8, 4), Some(-4));
        assert_eq!(coeff_from_half_byte(9, 4), None);

        // Edge case: b > 15
        assert_eq!(coeff_from_half_byte(16, 2), None);
        assert_eq!(coeff_from_half_byte(255, 4), None);

        // Edge case: invalid eta
        assert_eq!(coeff_from_half_byte(5, 3), None);
    }

    #[test]
    fn simple_bit_pack_test() {
        let w = [0u32; 256];
        let result = simple_bit_pack(&w, 1);
        assert_eq!(result.len(), 32);
        assert_eq!(result, vec![0u8; 32]);

        // b = 15 (4 bits per coefficient)
        let mut w = [0u32; 256];
        w[0] = 9;
        w[1] = 10;
        let result = simple_bit_pack(&w, 15);
        assert_eq!(result.len(), 128);
        assert_eq!(result[0], 169);
    }

    #[test]
    fn bit_pack_test() {
        let w = [0i32; 256];
        let result = bit_pack(&w, 5, 5);

        // Should be 128 bytes: 32 * bitlen(10) = 32 * 4 = 128
        assert_eq!(result.len(), 128);
        assert_eq!(result[0], 85);
    }

    #[test]
    fn simple_bit_unpack_test() {
        let w = [9u32; 256];
        let packed = simple_bit_pack(&w, 15);
        let unpacked = simple_bit_unpack(&packed, 15);

        assert_eq!(unpacked[0], 9);
        assert_eq!(unpacked[255], 9);
    }

    #[test]
    fn bit_unpack_test() {
        let w = [0i32; 256];
        let packed = bit_pack(&w, 5, 5);
        let unpacked = bit_unpack(&packed, 5, 5);

        assert_eq!(unpacked[0], 0);
        assert_eq!(unpacked[255], 0);
    }

    #[test]
    fn hint_bit_pack_test() {
        // Create a vector with k=2 polynomials
        let mut h = vec![[0u8; 256]; 2];

        // Set a few coefficients to 1
        h[0][5] = 1;
        h[0][10] = 1;
        h[1][3] = 1;

        let packed = hint_bit_pack(&h);

        // Output length should be ω + k = 80 + 2 = 82
        assert_eq!(packed.len(), 82);

        // First bytes should contain the positions: 5, 10, 3
        assert_eq!(packed[0], 5);
        assert_eq!(packed[1], 10);
        assert_eq!(packed[2], 3);

        // y[ω + 0] should be 2 (two nonzeros in h[0])
        assert_eq!(packed[80], 2);

        // y[ω + 1] should be 3 (total of 3 nonzeros after h[1])
        assert_eq!(packed[81], 3);
    }

    #[test]
    fn hint_bit_pack_unpack_roundtrip_test() {
        let mut h = vec![[0u8; 256]; 3];

        h[0][1] = 1;
        h[0][50] = 1;
        h[0][100] = 1;
        h[1][25] = 1;
        h[2][200] = 1;
        h[2][255] = 1;

        let omega = 80;
        let packed = hint_bit_pack(&h);
        let unpacked = hint_bit_unpack(&packed, 3, omega).unwrap();

        // Verify all the 1s are in the right places
        assert_eq!(unpacked[0][1], 1);
        assert_eq!(unpacked[0][50], 1);
        assert_eq!(unpacked[0][100], 1);
        assert_eq!(unpacked[1][25], 1);
        assert_eq!(unpacked[2][200], 1);
        assert_eq!(unpacked[2][255], 1);

        // Verify other positions are 0
        assert_eq!(unpacked[0][0], 0);
        assert_eq!(unpacked[1][0], 0);
        assert_eq!(unpacked[2][0], 0);
    }

    #[test]
    fn hint_bit_unpack_malformed_test() {
        let omega = 80;
        let k = 2;
        let mut y = vec![0u8; omega + k];

        y[80] = 2;
        y[81] = 100; // Invalid: > ω

        let result = hint_bit_unpack(&y, k, omega);
        assert!(result.is_none()); // Should return ⊥
    }
}

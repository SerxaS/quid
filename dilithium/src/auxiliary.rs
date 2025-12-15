use crate::constants::Q;

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

        let result = coeff_from_three_bytes(1, 0, 128);
        assert_eq!(result, Some(1));

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
}

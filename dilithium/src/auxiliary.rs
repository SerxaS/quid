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
}

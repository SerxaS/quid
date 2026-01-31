use sha3::{
    Shake128, Shake256,
    digest::{ExtendableOutput, Update, XofReader},
};

use crate::constants::{ETA, GAMMA1, GAMMA2, K, L, LAMBDA, OMEGA, Q, TAU};

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
pub fn coeff_from_half_byte(b: u8) -> Option<i32> {
    if ETA == 2 && b < 15 {
        Some(2 - (b % 5) as i32)
    } else if ETA == 4 && b < 9 {
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

/// Algorithm 22
pub fn pk_encode(rho: &[u8; 32], t1: &[Polynomial]) -> Vec<u8> {
    let mut pk = rho.to_vec();

    for i in 0..K {
        let mut w = [0u32; 256];
        for j in 0..256 {
            w[j] = t1[i][j] as u32;
        }

        let b = (1u32 << (bitlen(Q - 1) - 13)) - 1;
        pk.extend_from_slice(&simple_bit_pack(&w, b));
    }

    pk
}

/// Algorithm 23
pub fn pk_decode(pk: &[u8]) -> ([u8; 32], Vec<Polynomial>) {
    let mut rho = [0u8; 32];
    rho.copy_from_slice(&pk[0..32]);

    let b = (1u32 << (bitlen(Q - 1) - 13)) - 1;
    let chunk_size = 32 * bitlen(b);
    let mut t1 = vec![[0u8; 256]; K];

    for i in 0..K {
        let start = 32 + i * chunk_size;
        let end = start + chunk_size;
        let w = simple_bit_unpack(&pk[start..end], b);

        for j in 0..256 {
            t1[i][j] = w[j] as u8;
        }
    }

    (rho, t1)
}

/// Algorithm 24
pub fn sk_encode(
    rho: &[u8; 32],
    k_seed: &[u8; 32],
    tr: &[u8; 64],
    s1: &[Polynomial],
    s2: &[Polynomial],
    t0: &[Polynomial],
) -> Vec<u8> {
    let mut sk = Vec::new();

    sk.extend_from_slice(rho);
    sk.extend_from_slice(k_seed);
    sk.extend_from_slice(tr);

    for i in 0..L {
        let mut w = [0i32; 256];

        for j in 0..256 {
            w[j] = s1[i][j] as i8 as i32;
        }
        sk.extend_from_slice(&bit_pack(&w, ETA, ETA));
    }

    for i in 0..K {
        let mut w = [0i32; 256];

        for j in 0..256 {
            w[j] = s2[i][j] as i8 as i32;
        }
        sk.extend_from_slice(&bit_pack(&w, ETA, ETA));
    }

    for i in 0..K {
        let mut w = [0i32; 256];

        for j in 0..256 {
            w[j] = t0[i][j] as i8 as i32;
        }
        let a = (1u32 << 12) - 1;
        let b = 1u32 << 12;
        sk.extend_from_slice(&bit_pack(&w, a, b));
    }

    sk
}

/// Algorithm 25
pub fn sk_decode(
    sk: &[u8],
) -> (
    [u8; 32],
    [u8; 32],
    [u8; 64],
    Vec<Polynomial>,
    Vec<Polynomial>,
    Vec<Polynomial>,
) {
    let mut rho = [0u8; 32];
    let mut k_seed = [0u8; 32];
    let mut tr = [0u8; 64];

    rho.copy_from_slice(&sk[0..32]);
    k_seed.copy_from_slice(&sk[32..64]);
    tr.copy_from_slice(&sk[64..128]);

    let mut pos = 128;

    let mut s1 = vec![[0u8; 256]; L];
    let s_chunk_size = 32 * bitlen(2 * ETA);

    for i in 0..L {
        let chunk = &sk[pos..pos + s_chunk_size];
        let w = bit_unpack(chunk, ETA, ETA);

        for j in 0..256 {
            s1[i][j] = w[j] as i8 as u8;
        }
        pos += s_chunk_size;
    }
    let mut s2 = vec![[0u8; 256]; K];

    for i in 0..K {
        let chunk = &sk[pos..pos + s_chunk_size];
        let w = bit_unpack(chunk, ETA, ETA);
        for j in 0..256 {
            s2[i][j] = w[j] as i8 as u8;
        }
        pos += s_chunk_size;
    }

    let mut t0 = vec![[0u8; 256]; K];
    let a = (1u32 << 12) - 1;
    let b = 1u32 << 12;
    let t_chunk_size = 32 * bitlen(a + b);

    for i in 0..K {
        let chunk = &sk[pos..pos + t_chunk_size];
        let w = bit_unpack(chunk, a, b);
        for j in 0..256 {
            t0[i][j] = w[j] as i8 as u8;
        }
        pos += t_chunk_size;
    }

    (rho, k_seed, tr, s1, s2, t0)
}

/// Algorithm 26
pub fn sig_encode(c_tilde: &[u8], z: &[Polynomial], h: &[Polynomial]) -> Vec<u8> {
    let mut sigma = c_tilde.to_vec();

    for i in 0..L {
        let mut w = [0i32; 256];
        for j in 0..256 {
            w[j] = z[i][j] as i8 as i32;
        }
        sigma.extend_from_slice(&bit_pack(&w, GAMMA1 - 1, GAMMA1));
    }

    sigma.extend_from_slice(&hint_bit_pack(h));

    sigma
}

/// Algorithm 27
pub fn sig_decode(sigma: &[u8]) -> Option<(Vec<u8>, Vec<Polynomial>, Vec<Polynomial>)> {
    let c_tilde_len = LAMBDA / 4;
    let c_tilde = sigma[0..c_tilde_len].to_vec();

    let mut pos = c_tilde_len;

    let mut z = vec![[0u8; 256]; L];
    let z_chunk_size = 32 * (1 + bitlen(GAMMA1 - 1));
    for i in 0..L {
        let chunk = &sigma[pos..pos + z_chunk_size];
        let w = bit_unpack(chunk, GAMMA1 - 1, GAMMA1);
        for j in 0..256 {
            z[i][j] = w[j] as i8 as u8;
        }
        pos += z_chunk_size;
    }

    // Decode h
    let h_bytes = &sigma[pos..];
    let h = hint_bit_unpack(h_bytes, K, OMEGA)?;

    Some((c_tilde, z, h))
}

/// Algorithm 28
pub fn w1_encode(w1: &[Polynomial]) -> Vec<u8> {
    let mut w1_tilde = Vec::new();

    for i in 0..K {
        let mut w = [0u32; 256];
        for j in 0..256 {
            w[j] = w1[i][j] as u32;
        }

        let b = (Q - 1) / (2 * GAMMA2) - 1;
        w1_tilde.extend_from_slice(&simple_bit_pack(&w, b));
    }

    w1_tilde
}

/// Algorithm 29
pub fn sample_in_ball(rho: &[u8]) -> [i32; 256] {
    let mut c = [0i32; 256];

    let mut hasher = Shake256::default();
    hasher.update(rho);
    let mut ctx = hasher.finalize_xof();

    let mut s = [0u8; 8];
    ctx.read(&mut s);
    let h = bytes_to_bits(s.to_vec());

    for i in (256 - TAU)..256 {
        let mut j_bytes = [0u8; 1];
        ctx.read(&mut j_bytes);
        let mut j = j_bytes[0] as usize;

        while j > i {
            ctx.read(&mut j_bytes);
            j = j_bytes[0] as usize;
        }

        c[i] = c[j];
        c[j] = if h[i + TAU - 256] == 0 { 1 } else { -1 };
    }

    c
}

/// Algorithm 30
pub fn rej_ntt_poly(rho: &[u8; 34]) -> [u32; 256] {
    let mut a_hat = [0u32; 256];
    let mut j = 0;

    let mut hasher = Shake128::default();
    hasher.update(rho);
    let mut ctx = hasher.finalize_xof();

    while j < 256 {
        let mut s = [0u8; 3];
        ctx.read(&mut s);

        if let Some(coeff) = coeff_from_three_bytes(s[0], s[1], s[2]) {
            a_hat[j] = coeff;
            j += 1;
        }
    }

    a_hat
}

/// Algorithm 31
pub fn rej_bounded_poly(rho: &[u8; 66]) -> [i32; 256] {
    let mut a = [0i32; 256];
    let mut j = 0;

    let mut hasher = Shake256::default();
    hasher.update(rho);
    let mut ctx = hasher.finalize_xof();

    while j < 256 {
        let mut z_byte = [0u8; 1];
        ctx.read(&mut z_byte);
        let z = z_byte[0];

        let z0 = coeff_from_half_byte(z & 0x0F);
        let z1 = coeff_from_half_byte(z >> 4);

        if let Some(coeff) = z0 {
            a[j] = coeff;
            j += 1;
        }

        if let Some(coeff) = z1 {
            if j < 256 {
                a[j] = coeff;
                j += 1;
            }
        }
    }

    a
}

/// Algorithm 32
pub fn expand_a(rho: &[u8; 32]) -> Vec<Vec<[u32; 256]>> {
    let mut a_hat = vec![vec![[0u32; 256]; L]; K];

    for r in 0..K {
        for s in 0..L {
            let mut rho_prime = rho.to_vec();
            rho_prime.extend_from_slice(&integer_to_bytes(s as u32, 1));
            rho_prime.extend_from_slice(&integer_to_bytes(r as u32, 1));

            let mut rho_prime_array = [0u8; 34];
            rho_prime_array.copy_from_slice(&rho_prime);

            a_hat[r][s] = rej_ntt_poly(&rho_prime_array);
        }
    }

    a_hat
}

/// Algorithm 33
pub fn expand_s(rho: &[u8; 64]) -> (Vec<[i32; 256]>, Vec<[i32; 256]>) {
    let mut s1 = vec![[0i32; 256]; L];
    let mut s2 = vec![[0i32; 256]; K];

    let mut rho_array = [0u8; 66];
    rho_array[..64].copy_from_slice(rho);

    for r in 0..L {
        let r_bytes = integer_to_bytes(r as u32, 2);
        rho_array[64] = r_bytes[0];
        rho_array[65] = r_bytes[1];

        s1[r] = rej_bounded_poly(&rho_array);
    }

    for r in 0..K {
        let r_bytes = integer_to_bytes((r + L) as u32, 2);
        rho_array[64] = r_bytes[0];
        rho_array[65] = r_bytes[1];

        s2[r] = rej_bounded_poly(&rho_array);
    }

    (s1, s2)
}

/// Algorithm 34
pub fn expand_mask(rho: &[u8; 64], mu: usize) -> Vec<[i32; 256]> {
    let c = 1 + bitlen(GAMMA1 - 1);
    let mut y = vec![[0i32; 256]; L];

    let mut rho_prime = [0u8; 66];
    rho_prime[..64].copy_from_slice(rho);

    let mut v = vec![0u8; 32 * c];

    for r in 0..L {
        let r_bytes = integer_to_bytes((mu + r) as u32, 2);
        rho_prime[64] = r_bytes[0];
        rho_prime[65] = r_bytes[1];

        let mut hasher = Shake256::default();
        hasher.update(&rho_prime);
        let mut ctx = hasher.finalize_xof();

        ctx.read(&mut v);

        y[r] = bit_unpack(&v, GAMMA1 - 1, GAMMA1);
    }

    y
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
        assert_eq!(coeff_from_half_byte(0), Some(2));
        assert_eq!(coeff_from_half_byte(1), Some(1));
        assert_eq!(coeff_from_half_byte(2), Some(0));
        assert_eq!(coeff_from_half_byte(3), Some(-1));
        assert_eq!(coeff_from_half_byte(4), Some(-2));
        assert_eq!(coeff_from_half_byte(14), Some(-2));
        assert_eq!(coeff_from_half_byte(15), None);

        // Edge case: b > 15
        assert_eq!(coeff_from_half_byte(16), None);
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

    #[test]
    fn pk_encode_decode_test() {
        let rho = [42u8; 32];
        let t1 = vec![[5u8; 256]; K];

        let encoded = pk_encode(&rho, &t1);
        let (rho_decoded, t1_decoded) = pk_decode(&encoded);

        assert_eq!(rho, rho_decoded);
        assert_eq!(t1[0][0], t1_decoded[0][0]);
    }

    #[test]
    fn sk_encode_decode_test() {
        let rho = [1u8; 32];
        let k_seed = [2u8; 32];
        let tr = [3u8; 64];
        let s1 = vec![[0u8; 256]; L];
        let s2 = vec![[0u8; 256]; K];
        let t0 = vec![[0u8; 256]; K];

        let encoded = sk_encode(&rho, &k_seed, &tr, &s1, &s2, &t0);
        let (rho_dec, k_dec, tr_dec, _, _, _) = sk_decode(&encoded);

        assert_eq!(rho, rho_dec);
        assert_eq!(k_seed, k_dec);
        assert_eq!(tr, tr_dec);
    }

    #[test]
    fn sig_encode_decode_test() {
        let c_tilde = vec![7u8; 32];
        let z = vec![[1u8; 256]; L];
        let h = vec![[0u8; 256]; K];

        let encoded = sig_encode(&c_tilde, &z, &h);
        let decoded = sig_decode(&encoded);

        assert!(decoded.is_some());
        let (c_dec, _, _) = decoded.unwrap();
        assert_eq!(c_tilde, c_dec);
    }

    #[test]
    fn w1_encode_test() {
        let w1 = vec![[5u8; 256]; K];
        let encoded = w1_encode(&w1);

        let b = (Q - 1) / (2 * GAMMA2) - 1;
        let expected_len = 32 * K * bitlen(b);
        assert_eq!(encoded.len(), expected_len);
    }

    #[test]
    fn sample_in_ball_test() {
        let rho = [0u8; 32];
        let c = sample_in_ball(&rho);

        let mut count = 0;
        for &coeff in &c {
            assert!(coeff == -1 || coeff == 0 || coeff == 1);
            if coeff != 0 {
                count += 1;
            }
        }

        assert_eq!(count, TAU);
    }

    #[test]
    fn rej_ntt_poly_test() {
        let rho = [1u8; 34];
        let a_hat = rej_ntt_poly(&rho);

        for &coeff in &a_hat {
            assert!(coeff < Q);
        }
    }

    #[test]
    fn rej_bounded_poly_test() {
        let rho = [2u8; 66];
        let a = rej_bounded_poly(&rho);

        for &coeff in &a {
            assert!(coeff >= -(ETA as i32) && coeff <= (ETA as i32));
        }
    }

    #[test]
    fn expand_a_test() {
        let rho = [3u8; 32];
        let a_hat = expand_a(&rho);

        assert_eq!(a_hat.len(), K);
        assert_eq!(a_hat[0].len(), L);

        for i in 0..K {
            for j in 0..L {
                for &coeff in &a_hat[i][j] {
                    assert!(coeff < Q);
                }
            }
        }
    }

    #[test]
    fn expand_s_test() {
        let rho = [4u8; 64];
        let (s1, s2) = expand_s(&rho);

        assert_eq!(s1.len(), L);
        assert_eq!(s2.len(), K);

        for poly in &s1 {
            for &coeff in poly {
                assert!(coeff >= -(ETA as i32) && coeff <= (ETA as i32));
            }
        }

        for poly in &s2 {
            for &coeff in poly {
                assert!(coeff >= -(ETA as i32) && coeff <= (ETA as i32));
            }
        }
    }

    #[test]
    fn expand_mask_test() {
        let rho = [8u8; 64];
        let mu = 0;
        let y = expand_mask(&rho, mu);

        assert_eq!(y.len(), L);

        for poly in &y {
            for &coeff in poly {
                assert!(coeff >= -(GAMMA1 as i32) + 1 && coeff <= GAMMA1 as i32);
            }
        }
    }
}

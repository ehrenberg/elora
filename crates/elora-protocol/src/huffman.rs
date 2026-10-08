//! Static Huffman code (stage 3 of E-063).
//!
//! The frequency table is trained on our own traffic (`cargo xtask
//! train-huffman`) and lives in `huffman_table.rs`. The code is canonical, so that
//! sender and receiver derive it identically from the table.

use crate::codec::{DecodeError, DecodeResult};

/// 256 byte values + end symbol.
const SYMBOLS: usize = 257;
const EOF: usize = 256;
/// Longest allowed code (limited by smoothing the frequencies).
const MAX_LEN: u8 = 24;

#[derive(Debug, Clone)]
pub struct Huffman {
    /// Code and length per symbol.
    codes: Vec<(u32, u8)>,
    /// For decoding: per length the first code, the first index and the count.
    first_code: [u32; MAX_LEN as usize + 1],
    first_index: [usize; MAX_LEN as usize + 1],
    count: [usize; MAX_LEN as usize + 1],
    /// Symbols in canonical order.
    sorted: Vec<u16>,
}

impl Huffman {
    /// Builds the code from the frequencies of the 256 byte values (0 is treated like 1).
    ///
    /// # Panics
    /// Never for valid tables (code lengths are limited).
    pub fn new(freq: &[u32; 256]) -> Self {
        let mut weights: Vec<u64> = freq.iter().map(|&f| u64::from(f.max(1))).collect();
        weights.push(1); // EOF
        let lengths = loop {
            let l = code_lengths(&weights);
            if l.iter().all(|&x| x <= MAX_LEN) {
                break l;
            }
            // codes too long: smooth the frequencies
            for w in &mut weights {
                *w = (*w >> 1) + 1;
            }
        };
        // canonical codes
        let mut sorted: Vec<u16> = (0..SYMBOLS as u16).collect();
        sorted.sort_by_key(|&s| (lengths[s as usize], s));
        let mut codes = vec![(0u32, 0u8); SYMBOLS];
        let mut count = [0usize; MAX_LEN as usize + 1];
        for &l in &lengths {
            count[l as usize] += 1;
        }
        let mut first_code = [0u32; MAX_LEN as usize + 1];
        let mut first_index = [0usize; MAX_LEN as usize + 1];
        let (mut code, mut index) = (0u32, 0usize);
        for len in 1..=MAX_LEN as usize {
            code = (code + count[len - 1] as u32) << 1;
            first_code[len] = code;
            first_index[len] = index;
            index += count[len];
        }
        let mut next = first_code;
        for &s in &sorted {
            let l = lengths[s as usize] as usize;
            codes[s as usize] = (next[l], l as u8);
            next[l] += 1;
        }
        Self {
            codes,
            first_code,
            first_index,
            count,
            sorted,
        }
    }

    pub fn encode(&self, data: &[u8]) -> Vec<u8> {
        let mut out = Vec::with_capacity(data.len());
        let (mut acc, mut bits) = (0u64, 0u32);
        let mut put = |code: u32, len: u8, out: &mut Vec<u8>| {
            acc = (acc << len) | u64::from(code);
            bits += u32::from(len);
            while bits >= 8 {
                bits -= 8;
                out.push((acc >> bits) as u8);
            }
        };
        for &b in data {
            let (c, l) = self.codes[b as usize];
            put(c, l, &mut out);
        }
        let (c, l) = self.codes[EOF];
        put(c, l, &mut out);
        if bits > 0 {
            out.push((acc << (8 - bits)) as u8);
        }
        out
    }

    /// # Errors
    /// On invalid data or if the result exceeds `max` bytes.
    pub fn decode(&self, data: &[u8], max: usize) -> DecodeResult<Vec<u8>> {
        let mut out = Vec::with_capacity(data.len() * 2);
        let (mut code, mut len) = (0u32, 0usize);
        for &byte in data {
            for bit in (0..8).rev() {
                code = (code << 1) | u32::from((byte >> bit) & 1);
                len += 1;
                if len > MAX_LEN as usize {
                    return Err(DecodeError::Invalid("Huffman"));
                }
                let offset = code.wrapping_sub(self.first_code[len]) as usize;
                if code >= self.first_code[len] && offset < self.count[len] {
                    let sym = self.sorted[self.first_index[len] + offset] as usize;
                    if sym == EOF {
                        return Ok(out);
                    }
                    if out.len() >= max {
                        return Err(DecodeError::Invalid("too large"));
                    }
                    out.push(sym as u8);
                    code = 0;
                    len = 0;
                }
            }
        }
        Err(DecodeError::UnexpectedEnd)
    }
}

/// Code lengths via a classic Huffman tree.
fn code_lengths(weights: &[u64]) -> Vec<u8> {
    use std::cmp::Reverse;
    use std::collections::BinaryHeap;
    let n = weights.len();
    let mut parent = vec![usize::MAX; 2 * n];
    let mut heap: BinaryHeap<Reverse<(u64, usize)>> = weights
        .iter()
        .enumerate()
        .map(|(i, &w)| Reverse((w, i)))
        .collect();
    let mut next = n;
    while heap.len() > 1 {
        let Reverse((wa, a)) = heap.pop().expect("≥2");
        let Reverse((wb, b)) = heap.pop().expect("≥2");
        parent[a] = next;
        parent[b] = next;
        heap.push(Reverse((wa + wb, next)));
        next += 1;
    }
    (0..n)
        .map(|i| {
            let (mut d, mut p) = (0u8, i);
            while parent[p] != usize::MAX {
                p = parent[p];
                d = d.saturating_add(1);
            }
            d.max(1)
        })
        .collect()
}

/// The trained code of the game protocol.
pub fn game() -> &'static Huffman {
    static CODE: std::sync::OnceLock<Huffman> = std::sync::OnceLock::new();
    CODE.get_or_init(|| Huffman::new(&crate::huffman_table::FREQUENCIES))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_all_bytes_and_empty() {
        let h = game();
        let data: Vec<u8> = (0..=255u8).chain([0, 0, 0, 1, 1, 255]).collect();
        assert_eq!(h.decode(&h.encode(&data), 1024).unwrap(), data);
        assert_eq!(h.decode(&h.encode(&[]), 1024).unwrap(), Vec::<u8>::new());
    }

    #[test]
    fn skewed_data_gets_smaller() {
        let mut freq = [1u32; 256];
        freq[0] = 1000;
        freq[1] = 500;
        let h = Huffman::new(&freq);
        let data = vec![0u8; 100];
        assert!(h.encode(&data).len() < 30);
        assert_eq!(h.decode(&h.encode(&data), 1000).unwrap(), data);
    }

    #[test]
    fn rejects_garbage_and_limits_size() {
        let h = game();
        assert!(h.decode(&[], 10).is_err());
        let big = h.encode(&[7u8; 100]);
        assert!(h.decode(&big, 50).is_err());
    }
}

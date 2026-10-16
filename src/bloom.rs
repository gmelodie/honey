//! Scalable Bloom filter: slice `i` gets error `p * (1 - TIGHTENING) * TIGHTENING^i`, which sums to `p`.

use std::f64::consts::LN_2;
use std::io::{self, Read, Write};

const MAGIC: &[u8; 8] = b"HONEYBF1";
const INITIAL_CAPACITY: u64 = 100_000;
const GROWTH: u64 = 4;
const TIGHTENING: f64 = 0.9;

struct Slice {
    words: Vec<u64>,
    nbits: u64,
    k: u32,
    capacity: u64,
    count: u64,
}

impl Slice {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "sizes are positive and far below 2^52"
    )]
    fn new(capacity: u64, error_rate: f64) -> Self {
        let nbits = ((capacity as f64) * -error_rate.ln() / (LN_2 * LN_2)).ceil() as u64;
        let k = ((nbits as f64 / capacity as f64) * LN_2).round().max(1.0) as u32;
        Self {
            words: vec![0; nbits.div_ceil(64) as usize],
            nbits,
            k,
            capacity,
            count: 0,
        }
    }

    fn bit(&self, (h1, h2): (u64, u64), i: u32) -> u64 {
        h1.wrapping_add(u64::from(i).wrapping_mul(h2)) % self.nbits
    }

    fn contains(&self, h: (u64, u64)) -> bool {
        (0..self.k).all(|i| {
            let b = self.bit(h, i);
            self.words[(b / 64) as usize] >> (b % 64) & 1 == 1
        })
    }

    fn insert(&mut self, h: (u64, u64)) {
        for i in 0..self.k {
            let b = self.bit(h, i);
            self.words[(b / 64) as usize] |= 1 << (b % 64);
        }
        self.count += 1;
    }
}

pub struct ScalableBloom {
    error_rate: f64,
    slices: Vec<Slice>,
}

/// FNV-1a plus a splitmix64 finaliser: stable across Rust releases, unlike `DefaultHasher`.
fn hash(item: &str) -> (u64, u64) {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in item.bytes() {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0100_0000_01b3);
    }

    let mut z = h.wrapping_add(0x9e37_79b9_7f4a_7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    (h, (z ^ (z >> 31)) | 1)
}

fn read_u64(r: &mut impl Read) -> io::Result<u64> {
    let mut b = [0; 8];
    r.read_exact(&mut b)?;
    Ok(u64::from_le_bytes(b))
}

fn read_u32(r: &mut impl Read) -> io::Result<u32> {
    let mut b = [0; 4];
    r.read_exact(&mut b)?;
    Ok(u32::from_le_bytes(b))
}

fn invalid(msg: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, msg)
}

impl ScalableBloom {
    pub fn new(error_rate: f64) -> Self {
        Self {
            error_rate,
            slices: Vec::new(),
        }
    }

    pub fn len(&self) -> u64 {
        self.slices.iter().map(|s| s.count).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn contains(&self, item: &str) -> bool {
        let h = hash(item);
        self.slices.iter().any(|s| s.contains(h))
    }

    /// Returns false when the item (or a false positive) is already present.
    pub fn insert(&mut self, item: &str) -> bool {
        let h = hash(item);
        if self.slices.iter().any(|s| s.contains(h)) {
            return false;
        }

        if self.slices.last().is_none_or(|s| s.count >= s.capacity) {
            let n = u32::try_from(self.slices.len()).expect("slice count fits u32");
            let capacity = INITIAL_CAPACITY * GROWTH.pow(n);
            let error = self.error_rate * (1.0 - TIGHTENING) * TIGHTENING.powf(f64::from(n));
            self.slices.push(Slice::new(capacity, error));
        }
        self.slices.last_mut().unwrap().insert(h);
        true
    }

    pub fn write_to(&self, w: &mut impl Write) -> io::Result<()> {
        w.write_all(MAGIC)?;
        w.write_all(&self.error_rate.to_le_bytes())?;
        let nslices = u32::try_from(self.slices.len()).map_err(|_| invalid("too many slices"))?;
        w.write_all(&nslices.to_le_bytes())?;
        for s in &self.slices {
            w.write_all(&s.nbits.to_le_bytes())?;
            w.write_all(&s.k.to_le_bytes())?;
            w.write_all(&s.capacity.to_le_bytes())?;
            w.write_all(&s.count.to_le_bytes())?;
            for word in &s.words {
                w.write_all(&word.to_le_bytes())?;
            }
        }
        Ok(())
    }

    pub fn read_from(r: &mut impl Read) -> io::Result<Self> {
        let mut magic = [0; 8];
        r.read_exact(&mut magic)?;
        if &magic != MAGIC {
            return Err(invalid("not a honey bloom filter"));
        }

        let error_rate = f64::from_bits(read_u64(r)?);
        let nslices = read_u32(r)?;
        let mut slices = Vec::new();
        for _ in 0..nslices {
            let nbits = read_u64(r)?;
            let k = read_u32(r)?;
            let capacity = read_u64(r)?;
            let count = read_u64(r)?;
            if nbits == 0 || k == 0 {
                return Err(invalid("empty bloom slice"));
            }
            let words = (0..nbits.div_ceil(64))
                .map(|_| read_u64(r))
                .collect::<io::Result<_>>()?;
            slices.push(Slice {
                words,
                nbits,
                k,
                capacity,
                count,
            });
        }
        Ok(Self { error_rate, slices })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bytes(b: &ScalableBloom) -> Vec<u8> {
        let mut v = Vec::new();
        b.write_to(&mut v).unwrap();
        v
    }

    #[test]
    fn insert_reports_new_then_duplicate_and_contains_matches() {
        let mut b = ScalableBloom::new(0.001);
        assert!(b.is_empty());
        assert!(!b.contains("hunter2"));
        assert!(b.insert("hunter2"));
        assert!(!b.insert("hunter2"));
        assert!(b.contains("hunter2"));
        assert!(b.insert(""));
        assert!(b.contains(""));
        assert_eq!(b.len(), 2);
    }

    #[test]
    fn grows_a_second_slice_after_initial_capacity() {
        let mut b = ScalableBloom::new(0.001);
        let mut stored = 0;
        let mut i = 0;
        while b.slices.len() < 2 {
            stored += u64::from(b.insert(&format!("pw{i}")));
            i += 1;
            assert!(i < 200_000, "never grew");
        }
        assert_eq!(b.slices[0].count, INITIAL_CAPACITY);
        assert_eq!(b.slices[1].capacity, INITIAL_CAPACITY * GROWTH);
        assert_eq!(b.slices[1].count, 1);
        assert_eq!(b.len(), stored);
        assert!(b.contains("pw0") && b.contains(&format!("pw{}", i - 1)));
    }

    #[test]
    fn round_trip_preserves_membership_and_bytes() {
        let mut b = ScalableBloom::new(0.01);
        for i in 0..500 {
            b.insert(&format!("w{i}"));
        }
        let data = bytes(&b);
        let r = ScalableBloom::read_from(&mut data.as_slice()).unwrap();
        assert_eq!(r.len(), b.len());
        assert!((0..500).all(|i| r.contains(&format!("w{i}"))));
        assert_eq!(bytes(&r), data);
    }

    #[test]
    fn empty_filter_round_trips() {
        let data = bytes(&ScalableBloom::new(0.5));
        assert_eq!(data.len(), 8 + 8 + 4);
        let r = ScalableBloom::read_from(&mut data.as_slice()).unwrap();
        assert!(r.is_empty() && !r.contains("x"));
    }

    #[test]
    fn bad_magic_is_invalid_data() {
        let mut data = bytes(&ScalableBloom::new(0.01));
        data[0] = b'X';
        let e = ScalableBloom::read_from(&mut data.as_slice())
            .err()
            .unwrap();
        assert_eq!(e.kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn truncated_file_is_unexpected_eof_at_every_cut() {
        let mut b = ScalableBloom::new(0.01);
        b.insert("a");
        let data = bytes(&b);
        for cut in [0, 4, 8, 15, 19, 30, data.len() - 1] {
            let e = ScalableBloom::read_from(&mut &data[..cut]).err().unwrap();
            assert_eq!(e.kind(), io::ErrorKind::UnexpectedEof, "cut {cut}");
        }
    }

    #[test]
    fn zero_bit_slice_is_rejected() {
        let mut data = Vec::new();
        data.extend_from_slice(MAGIC);
        data.extend_from_slice(&0.01f64.to_le_bytes());
        data.extend_from_slice(&1u32.to_le_bytes());
        data.extend_from_slice(&[0; 8 + 4 + 8 + 8]);
        let e = ScalableBloom::read_from(&mut data.as_slice())
            .err()
            .unwrap();
        assert_eq!(e.kind(), io::ErrorKind::InvalidData);
    }
}

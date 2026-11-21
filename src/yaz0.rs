//! Support for Yaz0 decompression and fast compression, based on the
//! `oead::yaz0` module.
use alloc::borrow::Cow;
use alloc::vec::Vec;

use binrw::binrw;

use crate::{Error, Result, TinyString};

const HEADER_LEN: usize = 0x10;
const WINDOW: usize = 0x1000;
const MIN_MATCH: usize = 3;
const MAX_MATCH: usize = 0xFF + 0x12;
// A three byte chunk expands to at most MAX_MATCH bytes.
const MAX_EXPANSION: usize = MAX_MATCH / 3;

/// The header of Yaz0 compressed data.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[binrw]
#[brw(big)]
pub struct Header {
    /// Should be "Yaz0".
    pub magic: [u8; 4],
    /// The size of the uncompressed data.
    pub uncompressed_size: u32,
    /// [Newer files only] Required buffer alignment
    pub data_alignment: u32,
    #[doc(hidden)]
    reserved: [u8; 4],
}

/// Get the header of Yaz0 compressed data, if it exists.
pub fn get_header(data: impl AsRef<[u8]>) -> Option<Header> {
    binrw::BinRead::read(&mut binrw::io::Cursor::new(data.as_ref())).ok()
}

fn checked_header(data: &[u8]) -> Result<Header> {
    if data.len() < HEADER_LEN {
        return Err(Error::InsufficientData(data.len(), HEADER_LEN));
    }
    let header = get_header(data).ok_or(Error::InvalidData("Missing or corrupt Yaz0 header"))?;
    if &header.magic != b"Yaz0" {
        return Err(Error::BadMagic(
            TinyString::from_lossy(&header.magic),
            "Yaz0",
        ));
    }
    if header.uncompressed_size as usize > (data.len() - HEADER_LEN).saturating_mul(MAX_EXPANSION) {
        return Err(Error::InvalidData(
            "Yaz0 size exceeds what the data can hold",
        ));
    }
    Ok(header)
}

fn decode(src: &[u8], dst: &mut [u8]) -> Result<()> {
    const TRUNCATED: Error = Error::InvalidData("Yaz0 data ended early");
    const OUT_OF_BOUNDS: Error = Error::InvalidData("Copy is out of bounds");

    let mut input = src.iter().skip(HEADER_LEN).copied();
    let mut next = || input.next().ok_or(TRUNCATED);
    let end = dst.len();
    let mut out = 0;
    while out < end {
        let group = next()?;
        for bit in (0..8).rev() {
            if out == end {
                break;
            }
            if group >> bit & 1 == 1 {
                dst[out] = next()?;
                out += 1;
                continue;
            }
            let pair = u16::from_be_bytes([next()?, next()?]) as usize;
            let distance = (pair & 0x0FFF) + 1;
            let length = match pair >> 12 {
                0 => next()? as usize + 0x12,
                n => n + 2,
            };
            let base = out.checked_sub(distance).ok_or(OUT_OF_BOUNDS)?;
            if out + length > end {
                return Err(OUT_OF_BOUNDS);
            }
            match distance {
                d if d >= length => dst.copy_within(base..base + length, out),
                1 => {
                    let byte = dst[base];
                    dst[out..out + length].fill(byte);
                }
                // overlapping copies repeat the source
                _ => (0..length).for_each(|i| dst[out + i] = dst[base + i]),
            }
            out += length;
        }
    }
    Ok(())
}

/// Decompress Yaz0 data to vector.
pub fn decompress(data: impl AsRef<[u8]>) -> Result<Vec<u8>> {
    let data = data.as_ref();
    let header = checked_header(data)?;
    let mut out = alloc::vec![0; header.uncompressed_size as usize];
    decode(data, &mut out)?;
    Ok(out)
}

/// Decompress Yaz0 data into an existing buffer, returning the number of
/// bytes written.
pub fn decompress_into(data: impl AsRef<[u8]>, mut buffer: impl AsMut<[u8]>) -> Result<usize> {
    let data = data.as_ref();
    let header = checked_header(data)?;
    let size = header.uncompressed_size as usize;
    let buffer = buffer.as_mut();
    let capacity = buffer.len();
    let target = buffer
        .get_mut(..size)
        .ok_or(Error::InsufficientData(capacity, size))?;
    decode(data, target)?;
    Ok(size)
}

/// Decompress Yaz0 data into an existing buffer, returning the number of
/// bytes written.
///
/// # Safety
/// This function is extremely unsafe. The caller must needs be quite
/// certain that the provided slice is fully valid and complete Yaz0 data.
/// The destination buffer must also be large enough to hold the decompressed
/// data. **Do not use this function on untrusted data.**
pub unsafe fn decompress_unchecked(data: impl AsRef<[u8]>, mut buffer: impl AsMut<[u8]>) -> usize {
    let data = data.as_ref();
    let size = unsafe {
        u32::from_be_bytes(data.get_unchecked(0x4..0x8).try_into().unwrap_unchecked()) as usize
    };
    let mut src = unsafe { data.as_ptr().add(HEADER_LEN) };
    let mut out = buffer.as_mut().as_mut_ptr();
    let end = unsafe { out.add(size) };
    let mut group = 0u8;
    let mut remaining = 0;
    while out < end {
        if remaining == 0 {
            group = unsafe { *src };
            src = unsafe { src.add(1) };
            remaining = 8;
        }
        if group & 0x80 != 0 {
            unsafe { *out = *src };
            out = unsafe { out.add(1) };
            src = unsafe { src.add(1) };
        } else {
            let pair = unsafe { u16::from_be_bytes([*src, *src.add(1)]) } as usize;
            src = unsafe { src.add(2) };
            let distance = (pair & 0x0FFF) + 1;
            let length = match pair >> 12 {
                0 => {
                    let extra = unsafe { *src as usize };
                    src = unsafe { src.add(1) };
                    extra + 0x12
                }
                n => n + 2,
            };
            let base = unsafe { out.sub(distance) };
            if distance >= length {
                unsafe { core::ptr::copy_nonoverlapping(base, out, length) };
            } else {
                for i in 0..length {
                    unsafe { *out.add(i) = *base.add(i) };
                }
            }
            out = unsafe { out.add(length) };
        }
        group <<= 1;
        remaining -= 1;
    }
    size
}

/// Conditionally decompress Yaz0 data to a vector. Returns a [`Cow`] which
/// contains the original data if the data is not Yaz0 compressed or
/// decompression fails, or containing the decompressed data otherwise.
#[inline]
pub fn decompress_if(data: &[u8]) -> Cow<'_, [u8]> {
    if !data.starts_with(b"Yaz0") {
        return Cow::Borrowed(data);
    }
    decompress(data).map_or(Cow::Borrowed(data), Cow::Owned)
}

/// Compress data with default compression settings (no alignment, compression
/// level 7).
pub fn compress(data: impl AsRef<[u8]>) -> Vec<u8> {
    compress_with_options(data, CompressOptions::default())
}

/// Yaz0 compression options.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct CompressOptions {
    /// Buffer alignment hint for decompression
    pub alignment: u8,
    /// Compression level (6 to 9; 6 is fastest and 9 is slowest)
    pub compression_level: u8,
}

impl Default for CompressOptions {
    fn default() -> Self {
        Self {
            alignment: 0,
            compression_level: 7,
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct Effort {
    max_chain: u32,
    lazy_below: usize,
    good: usize,
}

impl From<u8> for Effort {
    fn from(level: u8) -> Self {
        let (max_chain, lazy_below, good) = match level.clamp(6, 9) {
            6 => (32, 0, 0),
            7 => (32, 32, 0),
            8 => (128, 32, 8),
            _ => (256, 64, 8),
        };
        Self {
            max_chain,
            lazy_below,
            good,
        }
    }
}

/// Compress data with custom compression settings.
///
/// Automatically clamps the compression level to 6 to 9.
///
/// # Panics
/// If `data` is 4 GiB or larger.
pub fn compress_with_options(data: impl AsRef<[u8]>, options: CompressOptions) -> Vec<u8> {
    let data = data.as_ref();
    let size = u32::try_from(data.len()).expect("Yaz0 cannot hold 4 GiB or more");
    let mut out = ChunkWriter::with_capacity(data.len() / 2 + 32);
    out.raw(b"Yaz0");
    out.raw(&size.to_be_bytes());
    out.raw(&u32::from(options.alignment).to_be_bytes());
    out.raw(&[0; 4]);

    let mut matcher = Matcher::new(data, options.compression_level.into());
    let mut i = 0;
    let mut carried = None;
    while i < data.len() {
        let found = carried.take().unwrap_or_else(|| matcher.find(i, 0));
        let Some(Match { len, dist }) = found else {
            out.literal(data[i]);
            matcher.insert(i);
            i += 1;
            continue;
        };
        if len < matcher.effort.lazy_below && i + 1 < data.len() {
            matcher.insert(i);
            let next = matcher.find(i + 1, len);
            // a literal now beats a shorter match if a longer one starts next
            if next.is_some_and(|m| m.len > len) {
                out.literal(data[i]);
                carried = Some(next);
                i += 1;
                continue;
            }
            out.back_ref(len, dist);
            (i + 1..i + len).for_each(|k| matcher.insert(k));
        } else {
            out.back_ref(len, dist);
            (i..i + len).for_each(|k| matcher.insert(k));
        }
        i += len;
    }
    out.finish()
}

#[derive(Debug, Copy, Clone)]
struct Match {
    len: usize,
    dist: usize,
}

struct Matcher<'a> {
    data: &'a [u8],
    effort: Effort,
    // positions are stored plus one so that zero means empty
    head: Vec<u32>,
    prev: Vec<u32>,
}

impl<'a> Matcher<'a> {
    const HASH_BITS: u32 = 15;

    fn new(data: &'a [u8], effort: Effort) -> Self {
        Self {
            data,
            effort,
            head: alloc::vec![0; 1 << Self::HASH_BITS],
            prev: alloc::vec![0; WINDOW],
        }
    }

    #[inline(always)]
    fn hash(&self, i: usize) -> usize {
        let d = &self.data[i..i + MIN_MATCH];
        let v = u32::from(d[0]) | u32::from(d[1]) << 8 | u32::from(d[2]) << 16;
        (v.wrapping_mul(0x9E37_79B1) >> (32 - Self::HASH_BITS)) as usize
    }

    #[inline(always)]
    fn insert(&mut self, i: usize) {
        if i + MIN_MATCH <= self.data.len() {
            let h = self.hash(i);
            self.prev[i % WINDOW] = self.head[h];
            self.head[h] = i as u32 + 1;
        }
    }

    #[inline(always)]
    fn common_len(&self, a: usize, b: usize, max: usize) -> usize {
        let word =
            |at: usize| u64::from_le_bytes(self.data[at..at + 8].try_into().unwrap_or_default());
        let mut l = 0;
        while l + 8 <= max {
            let diff = word(a + l) ^ word(b + l);
            if diff != 0 {
                return l + (diff.trailing_zeros() / 8) as usize;
            }
            l += 8;
        }
        while l < max && self.data[a + l] == self.data[b + l] {
            l += 1;
        }
        l
    }

    fn find(&self, i: usize, prev_len: usize) -> Option<Match> {
        if i + MIN_MATCH > self.data.len() {
            return None;
        }
        let max = MAX_MATCH.min(self.data.len() - i);
        let mut chain = self.effort.max_chain;
        if self.effort.good != 0 && prev_len >= self.effort.good {
            chain >>= 2;
        }
        let mut best = Match {
            len: MIN_MATCH - 1,
            dist: 0,
        };
        let mut candidate = self.head[self.hash(i)] as usize;
        for _ in 0..chain {
            let Some(c) = candidate.checked_sub(1) else {
                break;
            };
            let dist = i - c;
            if dist > WINDOW {
                break;
            }
            if self.data[c + best.len] == self.data[i + best.len] {
                let len = self.common_len(c, i, max);
                if len > best.len {
                    best = Match { len, dist };
                    if len == max {
                        break;
                    }
                }
            }
            let next = self.prev[c % WINDOW] as usize;
            // stale ring slot
            if next > c {
                break;
            }
            candidate = next;
        }
        (best.dist != 0).then_some(best)
    }
}

struct ChunkWriter {
    buf: Vec<u8>,
    flag_at: usize,
    flags: u8,
    free: u8,
}

impl ChunkWriter {
    fn with_capacity(cap: usize) -> Self {
        Self {
            buf: Vec::with_capacity(cap),
            flag_at: 0,
            flags: 0,
            free: 0,
        }
    }

    fn raw(&mut self, bytes: &[u8]) {
        self.buf.extend_from_slice(bytes);
    }

    #[inline(always)]
    fn chunk(&mut self, literal: bool, write: impl FnOnce(&mut Vec<u8>)) {
        if self.free == 0 {
            self.flag_at = self.buf.len();
            self.buf.push(0);
            (self.flags, self.free) = (0, 8);
        }
        self.free -= 1;
        self.flags |= u8::from(literal) << self.free;
        write(&mut self.buf);
        self.buf[self.flag_at] = self.flags;
    }

    #[inline(always)]
    fn literal(&mut self, byte: u8) {
        self.chunk(true, |b| b.push(byte));
    }

    #[inline(always)]
    fn back_ref(&mut self, len: usize, dist: usize) {
        let d = dist - 1;
        self.chunk(false, |b| {
            if len < 0x12 {
                b.extend_from_slice(&[((len - 2) << 4 | d >> 8) as u8, d as u8]);
            } else {
                b.extend_from_slice(&[(d >> 8) as u8, d as u8, (len - 0x12) as u8]);
            }
        });
    }

    fn finish(self) -> Vec<u8> {
        self.buf
    }
}

/// Compress data conditionally, if an associated path has a Yaz0-associated
/// file extension (starts with `s`, but does not equal `sarc`). Returns a
/// [`Cow`] which contains the original data if the data does not need to be
/// compressed, or containing the compressed data otherwise.
///
/// Requires `std` because it inspects the file path.
#[inline]
#[cfg(feature = "std")]
pub fn compress_if(data: &[u8], path: impl AsRef<std::path::Path>) -> Cow<'_, [u8]> {
    if path
        .as_ref()
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.starts_with('s') && e != "sarc")
        .unwrap_or(false)
    {
        Cow::Owned(compress(data))
    } else {
        Cow::Borrowed(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static FILES: &[(&str, [u8; 4], usize)] = &[
        ("ActorInfo.product.sbyml", [b'Y', b'B', 0x02, 0x0], 1963604),
        ("Demo344_1.sbeventpack", [b'S', b'A', b'R', b'C'], 2847908),
        (
            "ResourceSizeTable.product.srsizetable",
            [b'R', b'S', b'T', b'B'],
            526804,
        ),
        ("0-0.shknm2", [0x57, 0xE0, 0xE0, 0x57], 17584),
    ];

    #[test]
    fn test_header() {
        for (file, _, len) in FILES {
            let path = std::path::Path::new("test/yaz0").join(file);
            let data = std::fs::read(path).unwrap();
            let header = super::get_header(data.as_slice()).unwrap();
            assert_eq!(header.uncompressed_size, *len as u32);
        }
    }

    #[test]
    fn test_decompress() {
        for (file, magic, len) in FILES {
            let path = std::path::Path::new("test/yaz0").join(file);
            let data = std::fs::read(path).unwrap();
            let decompressed = super::decompress(data).unwrap();
            assert_eq!(&decompressed[..4], magic.as_slice());
            assert_eq!(decompressed.len(), *len);
            println!("{} is good", file);
        }
    }

    #[test]
    fn test_roundtrip() {
        for (file, ..) in FILES {
            let path = std::path::Path::new("test/yaz0").join(file);
            let data = std::fs::read(path).unwrap();
            let decompressed = super::decompress(data).unwrap();
            let compressed = super::compress(decompressed.as_slice());
            let decompressed2 = super::decompress(compressed).unwrap();
            assert_eq!(decompressed, decompressed2);
        }
    }

    #[test]
    fn test_unchecked() {
        let data = b"Nothing you have not given away will ever really be yours.";
        let compressed = super::compress(data);
        let mut buffer = alloc::vec![0; data.len()];
        let size = unsafe { super::decompress_unchecked(compressed, &mut buffer) };
        assert_eq!(data.as_slice(), &buffer[..size]);
    }

    #[test]
    fn unchecked_matches_checked() {
        for (file, ..) in FILES {
            let path = std::path::Path::new("test/yaz0").join(file);
            let data = std::fs::read(path).unwrap();
            let expected = decompress(&data).unwrap();
            let mut buffer = alloc::vec![0; expected.len()];
            let size = unsafe { decompress_unchecked(&data, &mut buffer) };
            assert_eq!(size, expected.len());
            assert_eq!(buffer, expected);
        }
    }

    fn xorshift(state: &mut u64) -> u64 {
        *state ^= *state << 13;
        *state ^= *state >> 7;
        *state ^= *state << 17;
        *state
    }

    #[test]
    fn roundtrip_every_level_and_edge_case() {
        let mut cases: Vec<Vec<u8>> = vec![
            vec![],
            vec![0],
            vec![1, 2],
            vec![7; 3],
            vec![7; MAX_MATCH],
            vec![7; MAX_MATCH + 1],
            vec![7; 100_000],
            (0..=255u8).cycle().take(70_000).collect(),
        ];
        let mut seed = 0x9E37_79B9_7F4A_7C15;
        for _ in 0..100 {
            let alphabet = 1 + xorshift(&mut seed) % 6;
            let len = (xorshift(&mut seed) % 20_000) as usize;
            cases.push(
                (0..len)
                    .map(|_| (xorshift(&mut seed) % alphabet) as u8)
                    .collect(),
            );
        }
        for level in 6..=9 {
            for case in &cases {
                let options = CompressOptions {
                    alignment: 0,
                    compression_level: level,
                };
                let packed = compress_with_options(case, options);
                assert_eq!(
                    &decompress(&packed).unwrap(),
                    case,
                    "level {level}, {} bytes",
                    case.len()
                );
                let mut buffer = vec![0; case.len()];
                let size = unsafe { decompress_unchecked(&packed, &mut buffer) };
                assert_eq!(&buffer[..size], case.as_slice());
            }
        }
    }

    #[test]
    fn out_of_range_levels_are_clamped() {
        let data = b"abcabcabcabcabcabcabc".repeat(50);
        for level in [0, 5, 10, 255] {
            let options = CompressOptions {
                alignment: 0,
                compression_level: level,
            };
            assert_eq!(
                decompress(compress_with_options(&data, options)).unwrap(),
                data
            );
        }
    }

    #[test]
    fn alignment_is_written_to_the_header() {
        let packed = compress_with_options(
            b"hello",
            CompressOptions {
                alignment: 32,
                compression_level: 7,
            },
        );
        assert_eq!(get_header(&packed).unwrap().data_alignment, 32);
    }

    #[test]
    fn decompress_into_accepts_oversized_buffers() {
        let data = b"Nothing you have not given away will ever really be yours.";
        let mut buffer = vec![0xAA; data.len() + 100];
        let n = decompress_into(compress(data), &mut buffer).unwrap();
        assert_eq!(&buffer[..n], data.as_slice());
        assert!(buffer[n..].iter().all(|&b| b == 0xAA));
        assert!(decompress_into(compress(data), vec![0; data.len() - 1]).is_err());
    }

    #[test]
    fn rejects_corrupt_streams() {
        let header = |size: u32| {
            let mut h = b"Yaz0".to_vec();
            h.extend_from_slice(&size.to_be_bytes());
            h.extend_from_slice(&[0; 8]);
            h
        };
        assert!(matches!(
            decompress(b"Yaz0"),
            Err(Error::InsufficientData(..))
        ));
        assert!(matches!(
            decompress([b"Yaz1".as_slice(), &[0; 12]].concat()),
            Err(Error::BadMagic(..))
        ));
        assert!(decompress([header(4), vec![0x00, 0x10, 0x05]].concat()).is_err());
        assert!(decompress([header(2), vec![0b1000_0000, b'A', 0x30, 0x00]].concat()).is_err());
        assert!(decompress([header(8), vec![0xFF, 1, 2, 3]].concat()).is_err());
        assert!(decompress([header(u32::MAX), vec![0xFF, 1]].concat()).is_err());
        let mut packed = compress(b"the quick brown fox ".repeat(100));
        packed.truncate(packed.len() - 3);
        assert!(decompress(&packed).is_err());
    }

    #[test]
    fn overlapping_copy_repeats_the_source() {
        let mut v = b"Yaz0".to_vec();
        v.extend_from_slice(&6u32.to_be_bytes());
        v.extend_from_slice(&[0; 8]);
        v.extend_from_slice(&[0b1000_0000, b'A', 0x30, 0x00]);
        assert_eq!(decompress(v).unwrap(), b"AAAAAA");
    }

    #[cfg(feature = "std")]
    #[test]
    fn conditional_helpers() {
        let data = b"some bytes, some bytes, some bytes, some bytes".to_vec();
        assert!(matches!(compress_if(&data, "Foo.sbyml"), Cow::Owned(_)));
        assert!(matches!(compress_if(&data, "Foo.sarc"), Cow::Borrowed(_)));
        assert!(matches!(compress_if(&data, "Foo.byml"), Cow::Borrowed(_)));
        let packed = compress(&data);
        assert_eq!(&*decompress_if(&packed), data.as_slice());
        assert_eq!(&*decompress_if(&data), data.as_slice());
        assert!(matches!(
            decompress_if(b"Yaz0 but not really"),
            Cow::Borrowed(_)
        ));
    }
}

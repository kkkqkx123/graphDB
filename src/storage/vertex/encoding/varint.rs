//! Varint Encoding
//!
//! Variable-length integer encoding for compact storage of small integers.
//! Based on SQLite's varint format.
//!
//! # Format
//!
//! - Values 0-127: 1 byte (high bit = 0)
//! - Values 128-16383: 2 bytes (high bit = 1, second byte high bit = 0)
//! - Larger values: up to 9 bytes
//!
//! # Examples
//!
//! ```
//! use graphdb::storage::vertex::encoding::varint::Varint;
//!
//! let encoded = Varint::encode(42);
//! assert_eq!(encoded.len(), 1);
//!
//! let (value, len) = Varint::decode(&encoded);
//! assert_eq!(value, 42);
//! assert_eq!(len, 1);
//! ```

#[derive(Debug, Clone, Copy)]
pub struct Varint;

impl Varint {
    pub fn encode(value: u64) -> Vec<u8> {
        if value < 0x80 {
            return vec![value as u8];
        }

        let mut result = Vec::new();
        let mut v = value;

        while v >= 0x80 {
            result.push((v as u8) | 0x80);
            v >>= 7;
        }
        result.push(v as u8);

        result
    }

    pub fn encode_into(value: u64, buffer: &mut Vec<u8>) {
        if value < 0x80 {
            buffer.push(value as u8);
            return;
        }

        let mut v = value;
        let start_len = buffer.len();

        while v >= 0x80 {
            buffer.push((v as u8) | 0x80);
            v >>= 7;
        }
        buffer.push(v as u8);
    }

    pub fn decode(data: &[u8]) -> (u64, usize) {
        if data.is_empty() {
            return (0, 0);
        }

        let mut result = 0u64;
        let mut shift = 0;
        let mut bytes_read = 0;

        for &byte in data {
            bytes_read += 1;
            result |= ((byte & 0x7F) as u64) << shift;

            if byte & 0x80 == 0 {
                break;
            }
            shift += 7;

            if shift >= 64 {
                break;
            }
        }

        (result, bytes_read)
    }

    pub fn decode_at(data: &[u8], offset: usize) -> (u64, usize) {
        if offset >= data.len() {
            return (0, 0);
        }

        Self::decode(&data[offset..])
    }

    pub fn encoded_len(value: u64) -> usize {
        if value == 0 {
            return 1;
        }

        let bits = 64 - value.leading_zeros();
        ((bits + 6) / 7) as usize
    }

    pub fn max_encoded_len() -> usize {
        9
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SignedVarint;

impl SignedVarint {
    pub fn encode(value: i64) -> Vec<u8> {
        let zigzag = Self::zigzag_encode(value);
        Varint::encode(zigzag)
    }

    pub fn encode_into(value: i64, buffer: &mut Vec<u8>) {
        let zigzag = Self::zigzag_encode(value);
        Varint::encode_into(zigzag, buffer);
    }

    pub fn decode(data: &[u8]) -> (i64, usize) {
        let (zigzag, len) = Varint::decode(data);
        (Self::zigzag_decode(zigzag), len)
    }

    pub fn decode_at(data: &[u8], offset: usize) -> (i64, usize) {
        let (zigzag, len) = Varint::decode_at(data, offset);
        (Self::zigzag_decode(zigzag), len)
    }

    pub fn encoded_len(value: i64) -> usize {
        let zigzag = Self::zigzag_encode(value);
        Varint::encoded_len(zigzag)
    }

    fn zigzag_encode(value: i64) -> u64 {
        ((value << 1) ^ (value >> 63)) as u64
    }

    fn zigzag_decode(value: u64) -> i64 {
        ((value >> 1) as i64) ^ (-((value & 1) as i64))
    }
}

#[derive(Debug, Clone)]
pub struct VarintWriter {
    buffer: Vec<u8>,
}

impl VarintWriter {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
        }
    }

    pub fn write_u64(&mut self, value: u64) {
        Varint::encode_into(value, &mut self.buffer);
    }

    pub fn write_i64(&mut self, value: i64) {
        SignedVarint::encode_into(value, &mut self.buffer);
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.write_u64(bytes.len() as u64);
        self.buffer.extend_from_slice(bytes);
    }

    pub fn write_str(&mut self, s: &str) {
        self.write_bytes(s.as_bytes());
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn finish(self) -> Vec<u8> {
        self.buffer
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.buffer
    }
}

impl Default for VarintWriter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct VarintReader<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> VarintReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }

    pub fn read_u64(&mut self) -> Option<u64> {
        if self.offset >= self.data.len() {
            return None;
        }

        let (value, len) = Varint::decode_at(self.data, self.offset);
        if len == 0 {
            return None;
        }

        self.offset += len;
        Some(value)
    }

    pub fn read_i64(&mut self) -> Option<i64> {
        if self.offset >= self.data.len() {
            return None;
        }

        let (value, len) = SignedVarint::decode_at(self.data, self.offset);
        if len == 0 {
            return None;
        }

        self.offset += len;
        Some(value)
    }

    pub fn read_bytes(&mut self) -> Option<Vec<u8>> {
        let len = self.read_u64()? as usize;
        if self.offset + len > self.data.len() {
            return None;
        }

        let bytes = self.data[self.offset..self.offset + len].to_vec();
        self.offset += len;
        Some(bytes)
    }

    pub fn read_str(&mut self) -> Option<String> {
        let bytes = self.read_bytes()?;
        String::from_utf8(bytes).ok()
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.offset)
    }

    pub fn is_empty(&self) -> bool {
        self.offset >= self.data.len()
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn seek(&mut self, offset: usize) {
        self.offset = offset.min(self.data.len());
    }
}

pub fn varint_encode(value: u64) -> Vec<u8> {
    Varint::encode(value)
}

pub fn varint_decode(data: &[u8]) -> (u64, usize) {
    Varint::decode(data)
}

pub fn signed_varint_encode(value: i64) -> Vec<u8> {
    SignedVarint::encode(value)
}

pub fn signed_varint_decode(data: &[u8]) -> (i64, usize) {
    SignedVarint::decode(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_varint_encode_small() {
        assert_eq!(Varint::encode(0), vec![0x00]);
        assert_eq!(Varint::encode(1), vec![0x01]);
        assert_eq!(Varint::encode(127), vec![0x7F]);
    }

    #[test]
    fn test_varint_encode_medium() {
        assert_eq!(Varint::encode(128), vec![0x80, 0x01]);
        assert_eq!(Varint::encode(300), vec![0xAC, 0x02]);
        assert_eq!(Varint::encode(16383), vec![0xFF, 0x7F]);
    }

    #[test]
    fn test_varint_encode_large() {
        let encoded = Varint::encode(u64::MAX);
        assert!(encoded.len() <= 10);

        let (decoded, _) = Varint::decode(&encoded);
        assert_eq!(decoded, u64::MAX);
    }

    #[test]
    fn test_varint_decode() {
        let (value, len) = Varint::decode(&[0x7F]);
        assert_eq!(value, 127);
        assert_eq!(len, 1);

        let (value, len) = Varint::decode(&[0x80, 0x01]);
        assert_eq!(value, 128);
        assert_eq!(len, 2);
    }

    #[test]
    fn test_varint_roundtrip() {
        for value in [0, 1, 127, 128, 300, 16383, 16384, u32::MAX as u64, u64::MAX] {
            let encoded = Varint::encode(value);
            let (decoded, _) = Varint::decode(&encoded);
            assert_eq!(decoded, value);
        }
    }

    #[test]
    fn test_varint_encoded_len() {
        assert_eq!(Varint::encoded_len(0), 1);
        assert_eq!(Varint::encoded_len(127), 1);
        assert_eq!(Varint::encoded_len(128), 2);
        assert_eq!(Varint::encoded_len(16383), 2);
        assert_eq!(Varint::encoded_len(16384), 3);
    }

    #[test]
    fn test_signed_varint() {
        for value in [0, 1, -1, 63, -64, 64, -65, i32::MAX as i64, i32::MIN as i64, i64::MAX, i64::MIN] {
            let encoded = SignedVarint::encode(value);
            let (decoded, _) = SignedVarint::decode(&encoded);
            assert_eq!(decoded, value);
        }
    }

    #[test]
    fn test_zigzag() {
        assert_eq!(SignedVarint::zigzag_encode(0), 0);
        assert_eq!(SignedVarint::zigzag_encode(-1), 1);
        assert_eq!(SignedVarint::zigzag_encode(1), 2);
        assert_eq!(SignedVarint::zigzag_encode(-2), 3);
        assert_eq!(SignedVarint::zigzag_encode(2), 4);

        assert_eq!(SignedVarint::zigzag_decode(0), 0);
        assert_eq!(SignedVarint::zigzag_decode(1), -1);
        assert_eq!(SignedVarint::zigzag_decode(2), 1);
        assert_eq!(SignedVarint::zigzag_decode(3), -2);
        assert_eq!(SignedVarint::zigzag_decode(4), 2);
    }

    #[test]
    fn test_varint_writer_reader() {
        let mut writer = VarintWriter::new();

        writer.write_u64(42);
        writer.write_i64(-100);
        writer.write_str("hello");
        writer.write_bytes(&[1, 2, 3, 4, 5]);

        let data = writer.finish();
        let mut reader = VarintReader::new(&data);

        assert_eq!(reader.read_u64(), Some(42));
        assert_eq!(reader.read_i64(), Some(-100));
        assert_eq!(reader.read_str(), Some("hello".to_string()));
        assert_eq!(reader.read_bytes(), Some(vec![1, 2, 3, 4, 5]));
        assert!(reader.is_empty());
    }

    #[test]
    fn test_varint_writer_bytes() {
        let mut writer = VarintWriter::new();

        writer.write_bytes(b"test");

        let data = writer.finish();
        assert_eq!(data, vec![4, b't', b'e', b's', b't']);
    }

    #[test]
    fn test_varint_reader_remaining() {
        let mut writer = VarintWriter::new();
        writer.write_u64(42);
        writer.write_u64(100);

        let data = writer.finish();
        let mut reader = VarintReader::new(&data);

        assert_eq!(reader.remaining(), data.len());
        reader.read_u64();
        assert!(reader.remaining() < data.len());
        reader.read_u64();
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn test_varint_space_savings() {
        let original_size = 8;

        let small_values = [0u64, 1, 50, 100, 127];
        for &v in &small_values {
            let encoded = Varint::encode(v);
            assert!(encoded.len() < original_size);
        }

        let medium_values = [128u64, 1000, 10000, 16383];
        for &v in &medium_values {
            let encoded = Varint::encode(v);
            assert!(encoded.len() < original_size);
        }
    }
}

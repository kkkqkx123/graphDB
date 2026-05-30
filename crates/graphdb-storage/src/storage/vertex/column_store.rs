//! Column Store
//!
//! Columnar storage for vertex properties.
//! Each column stores values of a single property type.
//!
//! The storage is split into two variants:
//! - `FixedWidthColumn`: For fixed-length types (Bool, SmallInt, Int, BigInt, Float, Double, Date, Time, Uuid)
//! - `VariableWidthColumn`: For variable-length types (String)
//! - `Column`: Public wrapper that selects the appropriate variant at construction time

use super::encoding::{
    ColumnEncoding, ColumnStats, CompressionSelector, EncodingType, FsstColumn, FsstEncoder,
};
use crate::core::value::{DateValue, DateTimeValue, TimeValue};
use crate::core::{DataType, StorageError, StorageResult, Value};
use crate::utils::NullBitmap;
use bitvec::prelude::*;

/// Unified column storage interface.
pub trait ColumnStorage: Send + Sync + std::fmt::Debug {
    fn get(&self, row_idx: usize) -> Option<Value>;
    fn set(&mut self, row_idx: usize, value: Option<&Value>) -> StorageResult<()>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn is_null(&self, row_idx: usize) -> bool;
    fn memory_usage(&self) -> usize;
    fn clear(&mut self);
    fn resize(&mut self, new_count: usize);
    fn data(&self) -> &[u8];
    fn data_size(&self) -> usize;
    fn null_bitmap(&self) -> Option<&BitVec<u8, Lsb0>>;
    fn null_bitmap_raw(&self) -> Option<&[u8]>;
    fn null_count(&self) -> usize;
    fn load_data(
        &mut self,
        data: Vec<u8>,
        offsets: Option<Vec<usize>>,
        null_bitmap: Option<BitVec<u8, Lsb0>>,
    );
    fn load_data_from_raw(
        &mut self,
        data: Vec<u8>,
        offsets: Vec<u64>,
        null_bitmap_raw: Option<Vec<u8>>,
        bitmap_bit_len: usize,
    );
    fn get_flush_data(&self) -> (Vec<u8>, Vec<u64>, Option<BitVec<u8, Lsb0>>);
    /// Extract data for a specific row range [start_row, end_row).
    /// Returns the same format as `get_flush_data()` but only for the given rows.
    fn get_flush_data_range(
        &self,
        start_row: usize,
        end_row: usize,
    ) -> (Vec<u8>, Vec<u64>, Option<BitVec<u8, Lsb0>>);
}

/// Returns the element size for fixed-width data types.
/// Returns 0 for variable-length types.
pub fn element_size(data_type: &DataType) -> usize {
    match data_type {
        DataType::Bool => 1,
        DataType::SmallInt => 2,
        DataType::Int => 4,
        DataType::BigInt => 8,
        DataType::Float => 4,
        DataType::Double => 8,
        DataType::Date => 12,
        DataType::Time => 8,
        DataType::DateTime | DataType::Timestamp => 28,
        DataType::Uuid => 16,
        _ => 0,
    }
}

/// Returns true if the data type is variable-length.
pub fn is_variable_length_type(data_type: &DataType) -> bool {
    matches!(data_type, DataType::String | DataType::Geography | DataType::List | DataType::Map | DataType::Set | DataType::Vertex | DataType::Edge | DataType::Path | DataType::Vector | DataType::DataSet | DataType::Json | DataType::JsonB | DataType::Interval | DataType::Null)
}

// ---------------------------------------------------------------------------
// FixedWidthColumn
// ---------------------------------------------------------------------------

/// Column storage for fixed-width (primitive) types.
///
/// Values are stored in a flat `Vec<u8>` with direct offset calculation:
/// `offset = row_idx * element_size`.
/// This provides O(1) random access without any branching on type.
#[derive(Debug, Clone)]
pub struct FixedWidthColumn {
    data: Vec<u8>,
    data_type: DataType,
    element_size: usize,
    null_bitmap: Option<BitVec<u8, Lsb0>>,
    row_count: usize,
}

impl FixedWidthColumn {
    pub fn new(data_type: DataType, nullable: bool) -> Self {
        let elem_size = element_size(&data_type);
        Self {
            data: Vec::new(),
            data_type: data_type.clone(),
            element_size: elem_size,
            null_bitmap: if nullable { Some(BitVec::new()) } else { None },
            row_count: 0,
        }
    }

    pub fn with_capacity(data_type: DataType, nullable: bool, capacity: usize) -> Self {
        let elem_size = element_size(&data_type);
        Self {
            data: Vec::with_capacity(capacity * elem_size),
            data_type: data_type.clone(),
            element_size: elem_size,
            null_bitmap: if nullable {
                Some(BitVec::with_capacity(capacity))
            } else {
                None
            },
            row_count: 0,
        }
    }
}

impl ColumnStorage for FixedWidthColumn {
    fn get(&self, row_idx: usize) -> Option<Value> {
        if self.is_null(row_idx) {
            return None;
        }

        let offset = row_idx * self.element_size;
        if offset + self.element_size > self.data.len() {
            return None;
        }

        let raw = read_fixed_value(&self.data, offset, self.element_size)?;
        Some(convert_to_type(raw, &self.data_type))
    }

    fn set(&mut self, row_idx: usize, value: Option<&Value>) -> StorageResult<()> {
        let offset = row_idx * self.element_size;
        if offset + self.element_size > self.data.len() {
            self.data.resize(offset + self.element_size, 0);
        }

        match value {
            Some(v) => {
                write_fixed_value(&mut self.data, offset, self.element_size, v)?;
                if let Some(ref mut bitmap) = self.null_bitmap {
                    ensure_bitmap_len(bitmap, row_idx + 1);
                    bitmap.set(row_idx, false);
                }
            }
            None => {
                if let Some(ref mut bitmap) = self.null_bitmap {
                    ensure_bitmap_len(bitmap, row_idx + 1);
                    bitmap.set(row_idx, true);
                }
            }
        }

        if row_idx >= self.row_count {
            self.row_count = row_idx + 1;
        }

        Ok(())
    }

    fn len(&self) -> usize {
        self.row_count
    }

    fn is_null(&self, row_idx: usize) -> bool {
        self.null_bitmap
            .as_ref()
            .map(|b| row_idx < b.len() && b[row_idx])
            .unwrap_or(false)
    }

    fn memory_usage(&self) -> usize {
        let data_size = self.data.len();
        let bitmap_size = self
            .null_bitmap
            .as_ref()
            .map(|b| b.as_raw_slice().len())
            .unwrap_or(0);
        data_size + bitmap_size
    }

    fn clear(&mut self) {
        self.data.clear();
        if let Some(ref mut bitmap) = self.null_bitmap {
            bitmap.clear();
        }
        self.row_count = 0;
    }

    fn resize(&mut self, new_count: usize) {
        let old_count = self.row_count;
        self.data.resize(new_count * self.element_size, 0);
        if let Some(ref mut bitmap) = self.null_bitmap {
            bitmap.resize(new_count, false);
            for i in old_count..new_count {
                bitmap.set(i, true);
            }
        }
        self.row_count = new_count;
    }

    fn data(&self) -> &[u8] {
        &self.data
    }

    fn data_size(&self) -> usize {
        self.data.len()
    }

    fn null_bitmap(&self) -> Option<&BitVec<u8, Lsb0>> {
        self.null_bitmap.as_ref()
    }

    fn null_bitmap_raw(&self) -> Option<&[u8]> {
        self.null_bitmap.as_ref().map(|b| b.as_raw_slice())
    }

    fn null_count(&self) -> usize {
        self.null_bitmap
            .as_ref()
            .map(|b| b.count_ones())
            .unwrap_or(0)
    }

    fn load_data(
        &mut self,
        data: Vec<u8>,
        _offsets: Option<Vec<usize>>,
        null_bitmap: Option<BitVec<u8, Lsb0>>,
    ) {
        self.data = data;
        let elem_size = self.element_size.max(1);
        let remainder = self.data.len() % elem_size;
        if remainder != 0 {
            self.data.resize(self.data.len() + (elem_size - remainder), 0);
        }
        self.row_count = self.data.len() / elem_size;
        self.null_bitmap = null_bitmap;
    }

    fn load_data_from_raw(
        &mut self,
        data: Vec<u8>,
        _offsets: Vec<u64>,
        null_bitmap_raw: Option<Vec<u8>>,
        bitmap_bit_len: usize,
    ) {
        self.data = data;
        let elem_size = self.element_size.max(1);
        let remainder = self.data.len() % elem_size;
        if remainder != 0 {
            self.data.resize(self.data.len() + (elem_size - remainder), 0);
        }
        self.null_bitmap = null_bitmap_raw.map(|raw| {
            let mut bv = BitVec::from_vec(raw);
            bv.resize(bitmap_bit_len, false);
            bv
        });
        self.row_count = self.data.len() / elem_size;
    }

    fn get_flush_data(&self) -> (Vec<u8>, Vec<u64>, Option<BitVec<u8, Lsb0>>) {
        (self.data.clone(), Vec::new(), self.null_bitmap.clone())
    }

    fn get_flush_data_range(
        &self,
        start_row: usize,
        end_row: usize,
    ) -> (Vec<u8>, Vec<u64>, Option<BitVec<u8, Lsb0>>) {
        let start_byte = start_row * self.element_size;
        let end_byte = std::cmp::min(end_row * self.element_size, self.data.len());
        let data = if end_byte > start_byte {
            self.data[start_byte..end_byte].to_vec()
        } else {
            Vec::new()
        };

        let bitmap = self.null_bitmap.as_ref().map(|b| {
            let mut chunk = BitVec::with_capacity(end_row - start_row);
            for i in start_row..std::cmp::min(end_row, b.len()) {
                chunk.push(b[i]);
            }
            chunk.resize(end_row - start_row, false);
            chunk
        });

        (data, Vec::new(), bitmap)
    }
}

// ---------------------------------------------------------------------------
// VariableWidthColumn
// ---------------------------------------------------------------------------

/// Column storage for variable-length types (String, and future Bytes/JSON).
///
/// Values are stored as concatenated byte data with an offsets array.
/// Each value is prefixed with its length (8 bytes, little-endian).
/// O(1) random access via the offsets array.
#[derive(Debug, Clone)]
pub struct VariableWidthColumn {
    data: Vec<u8>,
    offsets: Vec<usize>,
    null_bitmap: Option<BitVec<u8, Lsb0>>,
    row_count: usize,
}

impl VariableWidthColumn {
    pub fn new(nullable: bool) -> Self {
        Self {
            data: Vec::new(),
            offsets: Vec::new(),
            null_bitmap: if nullable { Some(BitVec::new()) } else { None },
            row_count: 0,
        }
    }

    pub fn with_capacity(nullable: bool, capacity: usize) -> Self {
        Self {
            data: Vec::new(),
            offsets: Vec::with_capacity(capacity),
            null_bitmap: if nullable {
                Some(BitVec::with_capacity(capacity))
            } else {
                None
            },
            row_count: 0,
        }
    }
}

impl ColumnStorage for VariableWidthColumn {
    fn get(&self, row_idx: usize) -> Option<Value> {
        if self.is_null(row_idx) {
            return None;
        }

        if row_idx >= self.offsets.len() {
            return None;
        }

        let start = self.offsets[row_idx];
        if start == usize::MAX {
            return None;
        }

        if start + 8 > self.data.len() {
            return None;
        }

        let len_bytes: [u8; 8] = self.data[start..start + 8].try_into().ok()?;
        let len = u64::from_le_bytes(len_bytes) as usize;

        if start + 8 + len > self.data.len() {
            return None;
        }

        let bytes = &self.data[start + 8..start + 8 + len];
        String::from_utf8(bytes.to_vec())
            .ok()
            .map(Value::String)
    }

    fn set(&mut self, row_idx: usize, value: Option<&Value>) -> StorageResult<()> {
        while self.offsets.len() <= row_idx {
            self.offsets.push(self.data.len());
        }

        match value {
            Some(v) => {
                let start = self.data.len();
                write_variable_value(&mut self.data, v)?;
                self.offsets[row_idx] = start;

                if let Some(ref mut bitmap) = self.null_bitmap {
                    ensure_bitmap_len(bitmap, row_idx + 1);
                    bitmap.set(row_idx, false);
                }
            }
            None => {
                self.offsets[row_idx] = usize::MAX;

                if let Some(ref mut bitmap) = self.null_bitmap {
                    ensure_bitmap_len(bitmap, row_idx + 1);
                    bitmap.set(row_idx, true);
                }
            }
        }

        if row_idx >= self.row_count {
            self.row_count = row_idx + 1;
        }

        Ok(())
    }

    fn len(&self) -> usize {
        self.row_count
    }

    fn is_null(&self, row_idx: usize) -> bool {
        self.null_bitmap
            .as_ref()
            .map(|b| row_idx < b.len() && b[row_idx])
            .unwrap_or(false)
    }

    fn memory_usage(&self) -> usize {
        let data_size = self.data.len();
        let offsets_size = self.offsets.len() * std::mem::size_of::<usize>();
        let bitmap_size = self
            .null_bitmap
            .as_ref()
            .map(|b| b.as_raw_slice().len())
            .unwrap_or(0);
        data_size + offsets_size + bitmap_size
    }

    fn clear(&mut self) {
        self.data.clear();
        self.offsets.clear();
        if let Some(ref mut bitmap) = self.null_bitmap {
            bitmap.clear();
        }
        self.row_count = 0;
    }

    fn resize(&mut self, new_count: usize) {
        let old_count = self.row_count;
        self.offsets.resize(new_count, self.data.len());
        if let Some(ref mut bitmap) = self.null_bitmap {
            bitmap.resize(new_count, false);
            for i in old_count..new_count {
                bitmap.set(i, true);
            }
        }
        self.row_count = new_count;
    }

    fn data(&self) -> &[u8] {
        &self.data
    }

    fn data_size(&self) -> usize {
        self.data.len()
    }

    fn null_bitmap(&self) -> Option<&BitVec<u8, Lsb0>> {
        self.null_bitmap.as_ref()
    }

    fn null_bitmap_raw(&self) -> Option<&[u8]> {
        self.null_bitmap.as_ref().map(|b| b.as_raw_slice())
    }

    fn null_count(&self) -> usize {
        self.null_bitmap
            .as_ref()
            .map(|b| b.count_ones())
            .unwrap_or(0)
    }

    fn load_data(
        &mut self,
        data: Vec<u8>,
        offsets: Option<Vec<usize>>,
        null_bitmap: Option<BitVec<u8, Lsb0>>,
    ) {
        self.data = data;
        if let Some(offs) = offsets {
            self.offsets = offs;
            self.row_count = self.offsets.len();
        } else {
            self.offsets.clear();
            self.row_count = 0;
        }
        self.null_bitmap = null_bitmap;
    }

    fn load_data_from_raw(
        &mut self,
        data: Vec<u8>,
        offsets: Vec<u64>,
        null_bitmap_raw: Option<Vec<u8>>,
        bitmap_bit_len: usize,
    ) {
        self.data = data;
        self.null_bitmap = null_bitmap_raw.map(|raw| {
            let mut bv = BitVec::from_vec(raw);
            bv.resize(bitmap_bit_len, false);
            bv
        });
        if !offsets.is_empty() {
            self.offsets = offsets.into_iter().map(|o| o as usize).collect();
            self.row_count = self.offsets.len();
        } else {
            self.offsets.clear();
            self.row_count = 0;
        }
    }

    fn get_flush_data(&self) -> (Vec<u8>, Vec<u64>, Option<BitVec<u8, Lsb0>>) {
        let offsets: Vec<u64> = self.offsets.iter().map(|&o| o as u64).collect();
        (self.data.clone(), offsets, self.null_bitmap.clone())
    }

    fn get_flush_data_range(
        &self,
        start_row: usize,
        end_row: usize,
    ) -> (Vec<u8>, Vec<u64>, Option<BitVec<u8, Lsb0>>) {
        let mut data = Vec::new();
        let mut offsets: Vec<u64> = Vec::new();
        let mut null_flags: Vec<bool> = Vec::new();

        for row in start_row..end_row {
            if row < self.offsets.len() && !self.is_null(row) {
                let entry_start = self.offsets[row];
                let entry_len = if row + 1 < self.offsets.len() && self.offsets[row + 1] != usize::MAX && self.offsets[row + 1] > 0 {
                    self.offsets[row + 1] - entry_start
                } else {
                    self.data.len() - entry_start
                };
                offsets.push(data.len() as u64);
                data.extend_from_slice(&self.data[entry_start..entry_start + entry_len]);
                null_flags.push(false);
            } else {
                offsets.push(data.len() as u64);
                null_flags.push(true);
            }
        }

        let bitmap = self.null_bitmap.as_ref().map(|_| {
            let mut chunk = BitVec::with_capacity(null_flags.len());
            for &flag in &null_flags {
                chunk.push(flag);
            }
            chunk
        });

        (data, offsets, bitmap)
    }
}

// ---------------------------------------------------------------------------
// Internal helpers (shared between Fixed and Variable)
// ---------------------------------------------------------------------------

fn ensure_bitmap_len(bitmap: &mut BitVec<u8, Lsb0>, min_len: usize) {
    if bitmap.len() < min_len {
        bitmap.resize(min_len, false);
    }
}

fn write_fixed_value(
    data: &mut [u8],
    offset: usize,
    element_size: usize,
    value: &Value,
) -> StorageResult<()> {
    let required_size = match value {
        Value::Bool(_) => 1,
        Value::SmallInt(_) => 2,
        Value::Int(_) => 4,
        Value::BigInt(_) => 8,
        Value::Float(_) => 4,
        Value::Double(_) => 8,
        Value::Date(_) => 12,
        Value::Time(_) => 8,
        Value::DateTime(_) => 28,
        _ => {
            return Err(StorageError::type_mismatch(
                value.data_type(),
                value.data_type(),
            ));
        }
    };
    
    if offset + required_size > data.len() {
        return Err(StorageError::invalid_input(format!(
            "Column data buffer too small: offset={}, required_size={}, data_len={}, element_size={}",
            offset, required_size, data.len(), element_size
        )));
    }
    
    match value {
        Value::Bool(b) => {
            data[offset] = if *b { 1 } else { 0 };
        }
        Value::SmallInt(i) => {
            data[offset..offset + 2].copy_from_slice(&i.to_le_bytes());
        }
        Value::Int(i) => {
            data[offset..offset + 4].copy_from_slice(&i.to_le_bytes());
        }
        Value::BigInt(i) => {
            data[offset..offset + 8].copy_from_slice(&i.to_le_bytes());
        }
        Value::Float(f) => {
            data[offset..offset + 4].copy_from_slice(&f.to_le_bytes());
        }
        Value::Double(d) => {
            data[offset..offset + 8].copy_from_slice(&d.to_le_bytes());
        }
        Value::Date(d) => {
            data[offset..offset + 4].copy_from_slice(&d.year.to_le_bytes());
            data[offset + 4..offset + 8].copy_from_slice(&d.month.to_le_bytes());
            data[offset + 8..offset + 12].copy_from_slice(&d.day.to_le_bytes());
        }
        Value::Time(t) => {
            let micros = t.hour as u64 * 3_600_000_000
                + t.minute as u64 * 60_000_000
                + t.sec as u64 * 1_000_000
                + t.microsec as u64;
            data[offset..offset + 8].copy_from_slice(&micros.to_le_bytes());
        }
        Value::DateTime(dt) => {
            data[offset..offset + 4].copy_from_slice(&dt.year.to_le_bytes());
            data[offset + 4..offset + 8].copy_from_slice(&dt.month.to_le_bytes());
            data[offset + 8..offset + 12].copy_from_slice(&dt.day.to_le_bytes());
            data[offset + 12..offset + 16].copy_from_slice(&dt.hour.to_le_bytes());
            data[offset + 16..offset + 20].copy_from_slice(&dt.minute.to_le_bytes());
            data[offset + 20..offset + 24].copy_from_slice(&dt.sec.to_le_bytes());
            data[offset + 24..offset + 28].copy_from_slice(&dt.microsec.to_le_bytes());
        }
        _ => {
            return Err(StorageError::type_mismatch(
                value.data_type(),
                value.data_type(),
            ));
        }
    }
    Ok(())
}

fn write_variable_value(data: &mut Vec<u8>, value: &Value) -> StorageResult<()> {
    match value {
        Value::String(s) => {
            let bytes = s.as_bytes();
            let len = bytes.len() as u64;
            data.extend_from_slice(&len.to_le_bytes());
            data.extend_from_slice(bytes);
        }
        _ => {
            return Err(StorageError::type_mismatch(
                value.data_type(),
                value.data_type(),
            ));
        }
    }
    Ok(())
}

fn read_fixed_value(data: &[u8], offset: usize, element_size: usize) -> Option<Value> {
    if offset + element_size > data.len() {
        return None;
    }

    match element_size {
        1 => Some(Value::Bool(data[offset] != 0)),
        2 => {
            let bytes: [u8; 2] = data[offset..offset + 2].try_into().ok()?;
            Some(Value::SmallInt(i16::from_le_bytes(bytes)))
        }
        4 => {
            let bytes: [u8; 4] = data[offset..offset + 4].try_into().ok()?;
            Some(Value::Int(i32::from_le_bytes(bytes)))
        }
        8 => {
            let bytes: [u8; 8] = data[offset..offset + 8].try_into().ok()?;
            Some(Value::BigInt(i64::from_le_bytes(bytes)))
        }
        12 => {
            let year_bytes: [u8; 4] = data[offset..offset + 4].try_into().ok()?;
            let month_bytes: [u8; 4] = data[offset + 4..offset + 8].try_into().ok()?;
            let day_bytes: [u8; 4] = data[offset + 8..offset + 12].try_into().ok()?;
            Some(Value::Date(DateValue {
                year: i32::from_le_bytes(year_bytes),
                month: u32::from_le_bytes(month_bytes),
                day: u32::from_le_bytes(day_bytes),
            }))
        }
        28 => {
            let year_bytes: [u8; 4] = data[offset..offset + 4].try_into().ok()?;
            let month_bytes: [u8; 4] = data[offset + 4..offset + 8].try_into().ok()?;
            let day_bytes: [u8; 4] = data[offset + 8..offset + 12].try_into().ok()?;
            let hour_bytes: [u8; 4] = data[offset + 12..offset + 16].try_into().ok()?;
            let minute_bytes: [u8; 4] = data[offset + 16..offset + 20].try_into().ok()?;
            let sec_bytes: [u8; 4] = data[offset + 20..offset + 24].try_into().ok()?;
            let microsec_bytes: [u8; 4] = data[offset + 24..offset + 28].try_into().ok()?;
            Some(Value::DateTime(DateTimeValue {
                year: i32::from_le_bytes(year_bytes),
                month: u32::from_le_bytes(month_bytes),
                day: u32::from_le_bytes(day_bytes),
                hour: u32::from_le_bytes(hour_bytes),
                minute: u32::from_le_bytes(minute_bytes),
                sec: u32::from_le_bytes(sec_bytes),
                microsec: u32::from_le_bytes(microsec_bytes),
            }))
        }
        _ => None,
    }
}

/// Convert a raw read_fixed_value result to the correct Value variant based on the declared DataType.
/// This handles ambiguous element sizes where multiple types share the same width.
fn convert_to_type(raw: Value, data_type: &DataType) -> Value {
    match (data_type, &raw) {
        (DataType::Double, Value::BigInt(n)) => Value::Double(f64::from_bits(*n as u64)),
        (DataType::Float, Value::Int(n)) => Value::Float(f32::from_bits(*n as u32)),
        (DataType::Float, Value::BigInt(n)) => Value::Float(f32::from_bits(*n as u32)),
        (DataType::Time, Value::BigInt(n)) => {
            let micros = *n as u64;
            let hour = (micros / 3_600_000_000) as u32;
            let rem = micros % 3_600_000_000;
            let minute = (rem / 60_000_000) as u32;
            let rem = rem % 60_000_000;
            let sec = (rem / 1_000_000) as u32;
            let microsec = (rem % 1_000_000) as u32;
            Value::Time(TimeValue { hour, minute, sec, microsec })
        }
        _ => raw,
    }
}

// ---------------------------------------------------------------------------
// Column (public wrapper enum)
// ---------------------------------------------------------------------------

/// Internal dispatch between fixed-width and variable-width storage.
#[derive(Debug, Clone)]
enum ColumnInner {
    Fixed(FixedWidthColumn),
    Variable(VariableWidthColumn),
}

/// Column storage that automatically selects fixed-width or variable-width
/// layout based on the `DataType` at construction time.
///
/// # Variant Selection
///
/// | `DataType` | Storage variant |
/// |---|---|
/// | Bool, SmallInt, Int, BigInt, Float, Double, Date, Time, Uuid | `FixedWidthColumn` |
/// | String | `VariableWidthColumn` |
#[derive(Debug, Clone)]
pub struct Column {
    pub name: String,
    pub col_id: i32,
    pub data_type: DataType,
    pub nullable: bool,
    inner: ColumnInner,
    encoding: ColumnEncoding,
}

impl Column {
    pub fn new(name: String, col_id: i32, data_type: DataType, nullable: bool) -> Self {
        let inner = if is_variable_length_type(&data_type) {
            ColumnInner::Variable(VariableWidthColumn::new(nullable))
        } else {
            ColumnInner::Fixed(FixedWidthColumn::new(data_type.clone(), nullable))
        };

        Self {
            name,
            col_id,
            data_type,
            nullable,
            inner,
            encoding: ColumnEncoding::None,
        }
    }

    pub fn with_capacity(
        name: String,
        col_id: i32,
        data_type: DataType,
        nullable: bool,
        capacity: usize,
    ) -> Self {
        let inner = if is_variable_length_type(&data_type) {
            ColumnInner::Variable(VariableWidthColumn::with_capacity(nullable, capacity))
        } else {
            ColumnInner::Fixed(FixedWidthColumn::with_capacity(data_type.clone(), nullable, capacity))
        };

        Self {
            name,
            col_id,
            data_type,
            nullable,
            inner,
            encoding: ColumnEncoding::None,
        }
    }

    pub fn element_size(data_type: &DataType) -> usize {
        element_size(data_type)
    }

    fn inner(&self) -> &dyn ColumnStorage {
        match &self.inner {
            ColumnInner::Fixed(c) => c,
            ColumnInner::Variable(c) => c,
        }
    }

    fn inner_mut(&mut self) -> &mut dyn ColumnStorage {
        match &mut self.inner {
            ColumnInner::Fixed(c) => c,
            ColumnInner::Variable(c) => c,
        }
    }

    // -----------------------------------------------------------------------
    // Core read / write
    // -----------------------------------------------------------------------

    pub fn set(&mut self, row_idx: usize, value: Option<&Value>) -> StorageResult<()> {
        if self.encoding.is_encoded() {
            self.encoding.set(row_idx, value)?;
            if row_idx >= self.len() {
                self.sync_row_count_from_encoding();
            }
            return Ok(());
        }

        if let Some(v) = value {
            if v.is_null() {
                if !self.nullable {
                    return Err(StorageError::null_value_not_allowed(self.name.clone()));
                }
                self.inner_mut().set(row_idx, None)?;
            } else {
                self.inner_mut().set(row_idx, Some(v))?;
            }
        } else {
            if !self.nullable {
                return Err(StorageError::null_value_not_allowed(self.name.clone()));
            }
            self.inner_mut().set(row_idx, None)?;
        }

        Ok(())
    }

    pub fn get(&self, row_idx: usize) -> Option<Value> {
        if self.encoding.is_encoded() {
            return self.encoding.get(row_idx);
        }
        self.inner().get(row_idx)
    }

    pub fn is_null(&self, row_idx: usize) -> bool {
        self.inner().is_null(row_idx)
    }

    pub fn null_count(&self) -> usize {
        self.inner().null_count()
    }

    pub fn len(&self) -> usize {
        self.inner().len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner().is_empty()
    }

    pub fn data_size(&self) -> usize {
        self.inner().data_size()
    }

    pub fn data(&self) -> &[u8] {
        self.inner().data()
    }

    pub fn null_bitmap(&self) -> Option<&BitVec<u8, Lsb0>> {
        self.inner().null_bitmap()
    }

    pub fn null_bitmap_raw(&self) -> Option<&[u8]> {
        self.inner().null_bitmap_raw()
    }

    pub fn memory_usage(&self) -> usize {
        self.inner().memory_usage() + self.encoding.memory_usage()
    }

    pub fn memory_size(&self) -> usize {
        self.memory_usage() + std::mem::size_of::<Self>()
    }

    pub fn used_memory_size(&self) -> usize {
        let non_null_count = self.len() - self.null_count();
        let elem_size = element_size(&self.data_type);
        non_null_count * elem_size + std::mem::size_of::<Self>()
    }

    pub fn clear(&mut self) {
        self.inner_mut().clear();
        self.encoding = ColumnEncoding::None;
    }

    /// Reset encoding: decode all values from encoding back to raw storage,
    /// then clear the encoding to `None`.
    ///
    /// This is the inverse of `auto_compress()` and allows re-encoding
    /// with a different algorithm (e.g. when data characteristics change).
    pub fn reset_encoding(&mut self) -> StorageResult<()> {
        if !self.encoding.is_encoded() {
            return Ok(());
        }

        let row_count = self.encoding.len();
        let mut all_values: Vec<Option<Value>> = Vec::with_capacity(row_count);

        for i in 0..row_count {
            if self.encoding.is_null(i) {
                all_values.push(None);
            } else {
                all_values.push(self.encoding.get(i));
            }
        }

        self.inner_mut().clear();
        self.encoding = ColumnEncoding::None;

        self.inner_mut().resize(row_count);
        for (i, val) in all_values.into_iter().enumerate() {
            self.set(i, val.as_ref())?;
        }

        Ok(())
    }

    /// Recompress: reset encoding first, then re-apply compression.
    pub fn recompress(&mut self) -> StorageResult<()> {
        self.reset_encoding()?;
        self.auto_compress()
    }

    pub fn resize(&mut self, new_count: usize) {
        self.inner_mut().resize(new_count);
    }

    pub fn load_data(
        &mut self,
        data: Vec<u8>,
        offsets: Option<Vec<usize>>,
        null_bitmap: Option<BitVec<u8, Lsb0>>,
    ) {
        self.inner_mut().load_data(data, offsets, null_bitmap);
    }

    pub fn load_data_from_raw(
        &mut self,
        data: Vec<u8>,
        offsets: Vec<u64>,
        null_bitmap_raw: Option<Vec<u8>>,
        bitmap_bit_len: usize,
    ) {
        self.inner_mut()
            .load_data_from_raw(data, offsets, null_bitmap_raw, bitmap_bit_len);
    }

    pub fn get_flush_data(&self) -> (Vec<u8>, Vec<u64>, Option<BitVec<u8, Lsb0>>) {
        if !self.encoding.is_encoded() {
            return self.inner().get_flush_data();
        }

        let row_count = self.len();
        let mut new_data = Vec::new();
        let mut new_offsets = Vec::new();
        let mut new_bitmap = self
            .null_bitmap()
            .map(|_| BitVec::with_capacity(row_count));

        let is_var = is_variable_length_type(&self.data_type);

        for i in 0..row_count {
            let value = self.encoding.get(i);
            match value {
                Some(v) => {
                    if let Some(ref mut bm) = new_bitmap {
                        bm.push(false);
                    }
                    if is_var {
                        new_offsets.push(new_data.len() as u64);
                        match &v {
                            Value::String(s) => {
                                let bytes = s.as_bytes();
                                new_data.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
                                new_data.extend_from_slice(bytes);
                            }
                            _ => {
                                new_offsets.pop();
                                new_offsets.push(u64::MAX);
                            }
                        }
                    } else {
                        let elem_size = element_size(&self.data_type);
                        let start = new_data.len();
                        new_data.resize(start + elem_size, 0);
                        let _ = write_fixed_value(&mut new_data, start, elem_size, &v);
                    }
                }
                None => {
                    if let Some(ref mut bm) = new_bitmap {
                        bm.push(true);
                    }
                    if is_var {
                        new_offsets.push(u64::MAX);
                    }
                }
            }
        }

        (new_data, new_offsets, new_bitmap)
    }

    /// Extract data for a specific row range [start_row, end_row).
    /// Returns column data in the same format as `get_flush_data()`.
    pub fn get_flush_data_range(
        &self,
        start_row: usize,
        end_row: usize,
    ) -> (Vec<u8>, Vec<u64>, Option<BitVec<u8, Lsb0>>) {
        if !self.encoding.is_encoded() {
            return self.inner().get_flush_data_range(start_row, end_row);
        }

        let row_count = self.len();
        let end = std::cmp::min(end_row, row_count);
        let start = std::cmp::min(start_row, end);

        let mut new_data = Vec::new();
        let mut new_offsets = Vec::new();
        let mut new_bitmap = self
            .null_bitmap()
            .map(|_| BitVec::with_capacity(end - start));

        let is_var = is_variable_length_type(&self.data_type);

        for i in start..end {
            let value = self.encoding.get(i);
            match value {
                Some(v) => {
                    if let Some(ref mut bm) = new_bitmap {
                        bm.push(false);
                    }
                    if is_var {
                        new_offsets.push(new_data.len() as u64);
                        match &v {
                            Value::String(s) => {
                                let bytes = s.as_bytes();
                                new_data.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
                                new_data.extend_from_slice(bytes);
                            }
                            _ => {
                                new_offsets.pop();
                                new_offsets.push(u64::MAX);
                            }
                        }
                    } else {
                        let elem_size = element_size(&self.data_type);
                        let offset = new_data.len();
                        new_data.resize(offset + elem_size, 0);
                        let _ = write_fixed_value(&mut new_data, offset, elem_size, &v);
                    }
                }
                None => {
                    if let Some(ref mut bm) = new_bitmap {
                        bm.push(true);
                    }
                    if is_var {
                        new_offsets.push(u64::MAX);
                    }
                }
            }
        }

        (new_data, new_offsets, new_bitmap)
    }

    // -----------------------------------------------------------------------
    // Statistics
    // -----------------------------------------------------------------------

    pub fn compute_stats(&self) -> ColumnStats {
        let mut stats = ColumnStats::new(self.data_type.clone());
        stats.row_count = self.len();
        stats.null_count = self.null_count();

        let mut distinct_values = std::collections::HashSet::new();
        let mut total_length: usize = 0;
        let mut run_count: usize = 0;
        let mut prev_value: Option<Value> = None;

        for i in 0..self.len() {
            if let Some(value) = self.get(i) {
                if prev_value.as_ref() != Some(&value) {
                    run_count += 1;
                }
                prev_value = Some(value.clone());
                distinct_values.insert(value.clone());
                if matches!(self.data_type, DataType::String) {
                    if let Value::String(s) = &value {
                        total_length += s.len();
                    }
                }
            }
        }

        stats.distinct_count = distinct_values.len();
        stats.run_count = run_count.max(1);
        stats.avg_length = if !self.is_empty() {
            total_length as f64 / self.len() as f64
        } else {
            0.0
        };

        stats
    }

    pub fn collect_stats(&self) -> ColumnStats {
        let mut stats = ColumnStats::new(self.data_type.clone());
        stats.row_count = self.len();
        stats.null_count = self.null_count();

        let mut distinct_values: std::collections::HashSet<String> =
            std::collections::HashSet::new();
        let mut total_length = 0usize;
        let mut run_count = 1usize;
        let mut prev_value: Option<Value> = None;

        for i in 0..self.len() {
            if let Some(value) = self.get(i) {
                match &value {
                    Value::String(s) => {
                        distinct_values.insert(s.clone());
                        total_length += s.len();
                    }
                    Value::SmallInt(v) => {
                        Self::update_int_stats(&mut stats, *v as i64);
                    }
                    Value::Int(v) => {
                        Self::update_int_stats(&mut stats, *v as i64);
                    }
                    Value::BigInt(v) => {
                        Self::update_int_stats(&mut stats, *v);
                    }
                    _ => {}
                }

                if let Some(ref prev) = prev_value {
                    if *prev != value {
                        run_count += 1;
                    }
                }
                prev_value = Some(value);
            }
        }

        stats.distinct_count = distinct_values.len();
        stats.avg_length = if !self.is_empty() {
            total_length as f64 / self.len() as f64
        } else {
            0.0
        };
        stats.run_count = run_count;

        stats
    }

    fn update_int_stats(stats: &mut ColumnStats, value: i64) {
        match (&stats.min_value, &stats.max_value) {
            (None, None) => {
                stats.min_value = Some(Value::BigInt(value));
                stats.max_value = Some(Value::BigInt(value));
            }
            (Some(min), Some(max)) => {
                if let Value::BigInt(min_val) = min {
                    if value < *min_val {
                        stats.min_value = Some(Value::BigInt(value));
                    }
                }
                if let Value::BigInt(max_val) = max {
                    if value > *max_val {
                        stats.max_value = Some(Value::BigInt(value));
                    }
                }
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // Encoding
    // -----------------------------------------------------------------------

    pub fn encoding_type(&self) -> EncodingType {
        self.encoding.encoding_type()
    }

    pub fn encoding(&self) -> &ColumnEncoding {
        &self.encoding
    }

    fn sync_row_count_from_encoding(&mut self) {
        let encoded_len = self.encoding.len();
        self.inner_mut().resize(encoded_len);
    }

    pub fn apply_fsst_encoding(&mut self, max_symbols: usize) -> StorageResult<()> {
        if self.data_type != DataType::String {
            return Err(StorageError::not_supported(
                "FSST encoding only supports String type".to_string(),
            ));
        }

        let mut strings: Vec<Option<String>> = Vec::with_capacity(self.len());
        for i in 0..self.len() {
            if self.is_null(i) {
                strings.push(None);
            } else {
                match self.get(i) {
                    Some(Value::String(s)) => strings.push(Some(s)),
                    _ => strings.push(None),
                }
            }
        }

        let string_refs: Vec<Option<&str>> = strings.iter().map(|s| s.as_deref()).collect();
        let non_null: Vec<&str> = string_refs.iter().filter_map(|s| *s).collect();

        if non_null.is_empty() {
            return Ok(());
        }

        let encoder = FsstEncoder::train(&non_null, max_symbols);

        let mut encoded_data = Vec::with_capacity(self.len());
        let mut null_bitmap = NullBitmap::with_capacity(self.len());

        for s in &string_refs {
            match s {
                Some(val) => {
                    encoded_data.push(encoder.encode(val));
                    null_bitmap.push(false);
                }
                None => {
                    encoded_data.push(Vec::new());
                    null_bitmap.push(true);
                }
            }
        }

        let fsst_col = FsstColumn {
            encoder,
            encoded_data,
            null_bitmap,
        };

        self.encoding = ColumnEncoding::Fsst(fsst_col);

        Ok(())
    }

    pub fn decode_fsst_value(&self, row_idx: usize) -> Option<Value> {
        if self.encoding.encoding_type() != EncodingType::Fsst {
            return None;
        }

        match &self.encoding {
            ColumnEncoding::Fsst(col) => {
                if row_idx >= col.len() || col.is_null(row_idx) {
                    return None;
                }
                col.get(row_idx).map(Value::String)
            }
            _ => None,
        }
    }

    pub fn get_encoded_fsst(&self, row_idx: usize) -> Option<&[u8]> {
        match &self.encoding {
            ColumnEncoding::Fsst(col) => {
                if row_idx >= col.encoded_data.len() {
                    return None;
                }
                Some(&col.encoded_data[row_idx])
            }
            _ => None,
        }
    }

    pub fn fsst_encoder(&self) -> Option<&FsstEncoder> {
        match &self.encoding {
            ColumnEncoding::Fsst(col) => Some(&col.encoder),
            _ => None,
        }
    }

    pub fn fsst_column(&self) -> Option<&FsstColumn> {
        match &self.encoding {
            ColumnEncoding::Fsst(col) => Some(col),
            _ => None,
        }
    }

    pub fn fsst_symbol_table_bytes(&self) -> Option<Vec<u8>> {
        match &self.encoding {
            ColumnEncoding::Fsst(col) => {
                let encoder = &col.encoder;
                let table = encoder.table();
                let mut bytes = Vec::new();

                let symbol_count = table.len() as u32;
                bytes.extend_from_slice(&symbol_count.to_le_bytes());

                for code in 1..=255u8 {
                    if let Some(symbol_bytes) = table.get_by_code(code) {
                        bytes.push(code);
                        bytes.push(symbol_bytes.len() as u8);
                        bytes.extend_from_slice(symbol_bytes);
                    }
                }

                Some(bytes)
            }
            _ => None,
        }
    }

    pub fn load_fsst_from_data(
        &mut self,
        encoded_data: Vec<Vec<u8>>,
        null_bitmap: NullBitmap,
        symbol_table_bytes: &[u8],
    ) -> StorageResult<()> {
        let mut table = super::encoding::FsstSymbolTable::new();

        if symbol_table_bytes.len() >= 4 {
            let symbol_count = u32::from_le_bytes(
                symbol_table_bytes[0..4]
                    .try_into()
                    .map_err(|_| StorageError::deserialize_error("failed to read FSST symbol count"))?
            ) as usize;
            let mut offset = 4;

            for _ in 0..symbol_count {
                if offset + 2 > symbol_table_bytes.len() {
                    break;
                }

                let code = symbol_table_bytes[offset];
                let sym_len = symbol_table_bytes[offset + 1] as usize;
                offset += 2;

                if offset + sym_len > symbol_table_bytes.len() {
                    break;
                }

                let sym_bytes = symbol_table_bytes[offset..offset + sym_len].to_vec();
                offset += sym_len;

                table.insert(sym_bytes, code);
            }
        }

        let encoder = FsstEncoder::with_table(table);

        self.encoding = ColumnEncoding::Fsst(FsstColumn {
            encoder,
            encoded_data,
            null_bitmap,
        });

        Ok(())
    }

    pub fn append_fsst_value(&mut self, value: Option<&str>) -> StorageResult<()> {
        if self.data_type != DataType::String {
            return Err(StorageError::not_supported(
                "FSST encoding only supports String type".to_string(),
            ));
        }

        match &mut self.encoding {
            ColumnEncoding::Fsst(fsst_col) => {
                fsst_col.append_with_stats(value);
                let new_len = fsst_col.len();
                self.inner_mut().resize(new_len);
            }
            _ => {
                return Err(StorageError::invalid_operation(
                    "FSST encoding not initialized. Call apply_fsst_encoding first.".to_string(),
                ));
            }
        }

        Ok(())
    }

    pub fn can_append_fsst(&self) -> bool {
        matches!(self.encoding, ColumnEncoding::Fsst(_))
    }

    pub fn fsst_row_count(&self) -> usize {
        match &self.encoding {
            ColumnEncoding::Fsst(col) => col.len(),
            _ => 0,
        }
    }

    pub fn fsst_compression_ratio(&self) -> Option<f64> {
        match &self.encoding {
            ColumnEncoding::Fsst(col) => Some(col.fast_compression_ratio()),
            _ => None,
        }
    }

    // -----------------------------------------------------------------------
    // Encoding application (Dictionary, RLE, BitPacking, ALP)
    // -----------------------------------------------------------------------

    pub fn auto_compress(&mut self) -> StorageResult<()> {
        if self.encoding.is_encoded() {
            return Ok(());
        }

        let stats = self.collect_stats();
        let selector = CompressionSelector::new();
        let encoding_type = selector.select(&stats);

        match encoding_type {
            EncodingType::Fsst => {
                self.apply_fsst_encoding(255)?;
            }
            EncodingType::Dictionary => {
                self.apply_dictionary_encoding()?;
            }
            EncodingType::Rle => {
                self.apply_rle_encoding()?;
            }
            EncodingType::BitPacking => {
                self.apply_bitpacking_encoding()?;
            }
            EncodingType::Alp => {
                self.apply_alp_encoding()?;
            }
            EncodingType::None => {}
        }

        Ok(())
    }

    pub fn apply_dictionary_encoding(&mut self) -> StorageResult<()> {
        if self.data_type != DataType::String {
            return Err(StorageError::not_supported(
                "Dictionary encoding only supports String type".to_string(),
            ));
        }

        use super::encoding::DictionaryColumn;

        let mut dict_col = DictionaryColumn::new();
        for i in 0..self.len() {
            let value = self.get(i);
            dict_col.set(i, value.as_ref())?;
        }

        self.encoding = ColumnEncoding::Dictionary(dict_col);

        Ok(())
    }

    pub fn apply_rle_encoding(&mut self) -> StorageResult<()> {
        use super::encoding::{RleBoolColumn, RleIntColumn};

        match self.data_type {
            DataType::Bool => {
                let mut rle_col = RleBoolColumn::new();
                for i in 0..self.len() {
                    let value = self.get(i);
                    rle_col.append(value.as_ref())?;
                }
                self.encoding = ColumnEncoding::RleBool(rle_col);
            }
            DataType::SmallInt | DataType::Int | DataType::BigInt => {
                let mut rle_col = RleIntColumn::new();
                for i in 0..self.len() {
                    let value = self.get(i);
                    rle_col.append(value.as_ref())?;
                }
                self.encoding = ColumnEncoding::RleInt(rle_col);
            }
            _ => {
                return Err(StorageError::not_supported(format!(
                    "RLE encoding not supported for {:?}",
                    self.data_type
                )));
            }
        }

        Ok(())
    }

    pub fn apply_bitpacking_encoding(&mut self) -> StorageResult<()> {
        use super::encoding::BitPackedIntColumn;

        match self.data_type {
            DataType::SmallInt | DataType::Int | DataType::BigInt => {
                let mut values: Vec<Option<Value>> = Vec::with_capacity(self.len());
                for i in 0..self.len() {
                    values.push(self.get(i));
                }
                let bp_col = BitPackedIntColumn::analyze(&values, self.data_type.clone())?;
                self.encoding = ColumnEncoding::BitPacked(bp_col);
            }
            _ => {
                return Err(StorageError::not_supported(format!(
                    "BitPacking encoding not supported for {:?}",
                    self.data_type
                )));
            }
        }

        Ok(())
    }

    pub fn apply_alp_encoding(&mut self) -> StorageResult<()> {
        use super::encoding::AlpColumn;

        match self.data_type {
            DataType::Float | DataType::Double => {
                let mut values: Vec<Option<Value>> = Vec::with_capacity(self.len());
                for i in 0..self.len() {
                    values.push(self.get(i));
                }
                let alp_col = AlpColumn::analyze_values(&values, self.data_type.clone())?;
                self.encoding = ColumnEncoding::Alp(alp_col);
            }
            _ => {
                return Err(StorageError::not_supported(format!(
                    "ALP encoding not supported for {:?}",
                    self.data_type
                )));
            }
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// ColumnStore
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ColumnStore {
    columns: Vec<Column>,
    name_to_index: std::collections::HashMap<String, usize>,
}

impl ColumnStore {
    pub fn new() -> Self {
        Self {
            columns: Vec::new(),
            name_to_index: std::collections::HashMap::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            columns: Vec::with_capacity(capacity),
            name_to_index: std::collections::HashMap::with_capacity(capacity),
        }
    }

    pub fn add_column(&mut self, name: String, data_type: DataType, nullable: bool) -> i32 {
        let col_id = self.columns.len() as i32;
        let column = Column::new(name.clone(), col_id, data_type, nullable);
        self.name_to_index.insert(name, self.columns.len());
        self.columns.push(column);
        col_id
    }

    pub fn get_column(&self, name: &str) -> Option<&Column> {
        self.name_to_index
            .get(name)
            .and_then(|&idx| self.columns.get(idx))
    }

    pub fn get_column_mut(&mut self, name: &str) -> Option<&mut Column> {
        self.name_to_index
            .get(name)
            .and_then(|&idx| self.columns.get_mut(idx))
    }

    pub fn get_column_by_id(&self, col_id: i32) -> Option<&Column> {
        self.columns.get(col_id as usize)
    }

    pub fn get_column_by_id_mut(&mut self, col_id: i32) -> Option<&mut Column> {
        self.columns.get_mut(col_id as usize)
    }

    pub fn set(&mut self, row_idx: usize, values: &[(String, Value)]) -> StorageResult<()> {
        for (name, value) in values {
            if let Some(col) = self.get_column_mut(name) {
                col.set(row_idx, Some(value))?;
            }
        }
        Ok(())
    }

    pub fn get(&self, row_idx: usize) -> Vec<(String, Option<Value>)> {
        self.columns
            .iter()
            .map(|col| (col.name.clone(), col.get(row_idx)))
            .collect()
    }

    pub fn get_property(&self, row_idx: usize, col_name: &str) -> Option<Value> {
        self.get_column(col_name)?.get(row_idx)
    }

    pub fn set_property(
        &mut self,
        row_idx: usize,
        col_name: &str,
        value: Option<&Value>,
    ) -> StorageResult<()> {
        let col = self
            .get_column_mut(col_name)
            .ok_or_else(|| StorageError::column_not_found(col_name.to_string()))?;
        col.set(row_idx, value)
    }

    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    pub fn row_count(&self) -> usize {
        self.columns.first().map(|c| c.len()).unwrap_or(0)
    }

    pub fn clear(&mut self) {
        for col in &mut self.columns {
            col.clear();
        }
    }

    pub fn resize(&mut self, new_count: usize) {
        for col in &mut self.columns {
            col.resize(new_count);
        }
    }

    pub fn reset_encodings(&mut self) -> StorageResult<()> {
        for col in &mut self.columns {
            col.reset_encoding()?;
        }
        Ok(())
    }

    pub fn recompress_all(&mut self) -> StorageResult<()> {
        for col in &mut self.columns {
            col.recompress()?;
        }
        Ok(())
    }

    pub fn columns(&self) -> &[Column] {
        &self.columns
    }

    pub fn column_names(&self) -> Vec<&str> {
        self.columns.iter().map(|c| c.name.as_str()).collect()
    }

    pub fn load_column(
        &mut self,
        name: &str,
        data: Vec<u8>,
        offsets: Option<Vec<usize>>,
        null_bitmap: Option<BitVec<u8, Lsb0>>,
    ) -> StorageResult<()> {
        if let Some(col) = self.get_column_mut(name) {
            col.load_data(data, offsets, null_bitmap);
            Ok(())
        } else {
            Err(StorageError::column_not_found(name.to_string()))
        }
    }

    pub fn load_column_from_raw(
        &mut self,
        name: &str,
        data: Vec<u8>,
        offsets: Vec<u64>,
        null_bitmap_raw: Option<Vec<u8>>,
        bitmap_bit_len: usize,
    ) -> StorageResult<()> {
        if let Some(col) = self.get_column_mut(name) {
            col.load_data_from_raw(data, offsets, null_bitmap_raw, bitmap_bit_len);
            Ok(())
        } else {
            Err(StorageError::column_not_found(name.to_string()))
        }
    }

    pub fn iter_columns(&self) -> impl Iterator<Item = (&String, &Column)> {
        self.name_to_index
            .iter()
            .filter_map(|(name, &idx)| self.columns.get(idx).map(|col| (name, col)))
    }

    pub fn apply_fsst_to_string_columns(&mut self, max_symbols: usize) -> StorageResult<()> {
        for col in &mut self.columns {
            if col.data_type == DataType::String && !col.is_empty() {
                col.apply_fsst_encoding(max_symbols)?;
            }
        }
        Ok(())
    }

    pub fn apply_encoding_to_column(
        &mut self,
        col_name: &str,
        encoding_type: EncodingType,
    ) -> StorageResult<()> {
        let col = self
            .get_column_mut(col_name)
            .ok_or_else(|| StorageError::column_not_found(col_name.to_string()))?;

        if col.is_empty() {
            return Ok(());
        }

        match encoding_type {
            EncodingType::Fsst => {
                if col.data_type != DataType::String {
                    return Err(StorageError::not_supported(
                        "FSST encoding only supports String type".to_string(),
                    ));
                }
                col.apply_fsst_encoding(1024)?;
            }
            EncodingType::Dictionary => {
                col.apply_dictionary_encoding()?;
            }
            EncodingType::Rle => {
                col.apply_rle_encoding()?;
            }
            EncodingType::BitPacking => {
                col.apply_bitpacking_encoding()?;
            }
            EncodingType::Alp => {
                col.apply_alp_encoding()?;
            }
            EncodingType::None => {}
        }

        Ok(())
    }

    pub fn auto_apply_encodings(
        &mut self,
        config: Option<super::encoding::CompressionConfig>,
    ) -> StorageResult<()> {
        let selector = match config {
            Some(c) => super::encoding::CompressionSelector::with_config(c),
            None => super::encoding::CompressionSelector::new(),
        };

        for col in &mut self.columns {
            if col.is_empty() || col.encoding.is_encoded() {
                continue;
            }

            let stats = col.compute_stats();
            let encoding = selector.select(&stats);

            match encoding {
                EncodingType::Fsst => {
                    if col.data_type == DataType::String {
                        col.apply_fsst_encoding(1024)?;
                    }
                }
                EncodingType::Dictionary => {
                    col.apply_dictionary_encoding()?;
                }
                EncodingType::Rle => {
                    col.apply_rle_encoding()?;
                }
                EncodingType::BitPacking => {
                    col.apply_bitpacking_encoding()?;
                }
                EncodingType::Alp => {
                    col.apply_alp_encoding()?;
                }
                EncodingType::None => {}
            }
        }

        Ok(())
    }

    pub fn memory_size(&self) -> usize {
        let mut total = std::mem::size_of::<Self>();

        for col in &self.columns {
            total += col.memory_size();
        }

        total += self.name_to_index.len()
            * (std::mem::size_of::<String>() + std::mem::size_of::<usize>());

        total
    }

    pub fn used_memory_size(&self) -> usize {
        let mut total = std::mem::size_of::<Self>();

        for col in &self.columns {
            total += col.used_memory_size();
        }

        total
    }
}

impl Default for ColumnStore {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_basic() {
        let mut col = Column::new("age".to_string(), 0, DataType::Int, true);

        col.set(0, Some(&Value::Int(25))).unwrap();
        col.set(1, Some(&Value::Int(30))).unwrap();
        col.set(2, None).unwrap();

        assert_eq!(col.get(0), Some(Value::Int(25)));
        assert_eq!(col.get(1), Some(Value::Int(30)));
        assert!(col.is_null(2));
        assert_eq!(col.len(), 3);
    }

    #[test]
    fn test_column_string() {
        let mut col = Column::new("name".to_string(), 0, DataType::String, false);

        col.set(0, Some(&Value::String("Alice".to_string())))
            .unwrap();
        col.set(1, Some(&Value::String("Bob".to_string())))
            .unwrap();

        assert_eq!(
            col.get(0),
            Some(Value::String("Alice".to_string()))
        );
        assert_eq!(
            col.get(1),
            Some(Value::String("Bob".to_string()))
        );
        assert_eq!(col.len(), 2);
    }

    #[test]
    fn test_column_store() {
        let mut store = ColumnStore::new();

        store.add_column("name".to_string(), DataType::String, false);
        store.add_column("age".to_string(), DataType::Int, true);

        store
            .set(
                0,
                &[
                    ("name".to_string(), Value::String("Alice".to_string())),
                    ("age".to_string(), Value::Int(30)),
                ],
            )
            .unwrap();

        store
            .set(
                1,
                &[
                    ("name".to_string(), Value::String("Bob".to_string())),
                    ("age".to_string(), Value::Int(25)),
                ],
            )
            .unwrap();

        assert_eq!(store.get_property(0, "age"), Some(Value::Int(30)));
        assert_eq!(
            store.get_property(1, "name"),
            Some(Value::String("Bob".to_string()))
        );
    }

    #[test]
    fn test_fixed_width_multiple_types() {
        let mut col = Column::new("mixed".to_string(), 0, DataType::BigInt, false);
        col.set(0, Some(&Value::BigInt(100))).unwrap();
        col.set(1, Some(&Value::BigInt(200))).unwrap();
        assert_eq!(col.get(0), Some(Value::BigInt(100)));
        assert_eq!(col.get(1), Some(Value::BigInt(200)));
        assert_eq!(col.len(), 2);

        let mut col2 = Column::new("flag".to_string(), 1, DataType::Bool, true);
        col2.set(0, Some(&Value::Bool(true))).unwrap();
        col2.set(1, Some(&Value::Bool(false))).unwrap();
        col2.set(2, None).unwrap();
        assert_eq!(col2.get(0), Some(Value::Bool(true)));
        assert_eq!(col2.get(1), Some(Value::Bool(false)));
        assert!(col2.is_null(2));
    }

    #[test]
    fn test_flush_and_reload_fixed() {
        let mut col = Column::new("val".to_string(), 0, DataType::Int, true);
        col.set(0, Some(&Value::Int(10))).unwrap();
        col.set(1, Some(&Value::Int(20))).unwrap();
        col.set(2, None).unwrap();

        let (data, offsets, bitmap) = col.get_flush_data();
        assert!(offsets.is_empty());

        let mut restored = Column::new("val".to_string(), 0, DataType::Int, true);
        restored.load_data(data, None, bitmap);

        assert_eq!(restored.get(0), Some(Value::Int(10)));
        assert_eq!(restored.get(1), Some(Value::Int(20)));
        assert!(restored.is_null(2));
        assert_eq!(restored.len(), 3);
    }

    #[test]
    fn test_flush_and_reload_variable() {
        let mut col = Column::new("name".to_string(), 0, DataType::String, true);
        col.set(0, Some(&Value::String("Hello".to_string())))
            .unwrap();
        col.set(1, Some(&Value::String("World".to_string())))
            .unwrap();
        col.set(2, None).unwrap();

        let (data, offsets, bitmap) = col.get_flush_data();
        assert!(!offsets.is_empty());

        let mut restored = Column::new("name".to_string(), 0, DataType::String, true);
        restored.load_data_from_raw(data, offsets, bitmap.map(|b| b.into_vec()), 3);

        assert_eq!(
            restored.get(0),
            Some(Value::String("Hello".to_string()))
        );
        assert_eq!(
            restored.get(1),
            Some(Value::String("World".to_string()))
        );
        assert!(restored.is_null(2));
        assert_eq!(restored.len(), 3);
    }
}
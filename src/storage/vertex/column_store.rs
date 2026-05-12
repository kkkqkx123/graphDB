//! Column Store
//!
//! Columnar storage for vertex properties.
//! Each column stores values of a single property type.

use bitvec::prelude::*;
use crate::core::value::DateValue;
use crate::core::{DataType, StorageError, StorageResult, Value};
use super::encoding::{ColumnEncoding, EncodingType, FsstColumn, FsstEncoder, CompressionSelector, ColumnStats};
use crate::utils::NullBitmap;

#[derive(Debug, Clone)]
pub struct Column {
    pub name: String,
    pub col_id: i32,
    pub data_type: DataType,
    pub nullable: bool,
    data: Vec<u8>,
    offsets: Vec<usize>,
    null_bitmap: Option<BitVec<u8, Lsb0>>,
    row_count: usize,
    encoding: ColumnEncoding,
}

impl Column {
    pub fn new(name: String, col_id: i32, data_type: DataType, nullable: bool) -> Self {
        Self {
            name,
            col_id,
            data_type,
            nullable,
            data: Vec::new(),
            offsets: Vec::new(),
            null_bitmap: if nullable { Some(BitVec::new()) } else { None },
            row_count: 0,
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
        let element_size = Self::element_size(&data_type);
        Self {
            name,
            col_id,
            data_type,
            nullable,
            data: Vec::with_capacity(capacity * element_size),
            offsets: Vec::with_capacity(capacity),
            null_bitmap: if nullable {
                Some(BitVec::with_capacity(capacity))
            } else {
                None
            },
            row_count: 0,
            encoding: ColumnEncoding::None,
        }
    }

    fn element_size(data_type: &DataType) -> usize {
        match data_type {
            DataType::Bool => 1,
            DataType::SmallInt => 2,
            DataType::Int => 4,
            DataType::BigInt => 8,
            DataType::Float => 4,
            DataType::Double => 8,
            DataType::Date => 12,
            DataType::Time => 8,
            DataType::Uuid => 16,
            _ => 0,
        }
    }

    fn is_variable_length(&self) -> bool {
        matches!(self.data_type, DataType::String)
    }

    pub fn set(&mut self, row_idx: usize, value: Option<&Value>) -> StorageResult<()> {
        if self.is_variable_length() {
            while self.offsets.len() <= row_idx {
                self.offsets.push(self.data.len());
            }

            match value {
                Some(v) => {
                    let start = self.data.len();
                    self.write_variable_value(v)?;
                    self.offsets[row_idx] = start;

                    if let Some(ref mut bitmap) = self.null_bitmap {
                        Self::ensure_bitmap_len(bitmap, row_idx + 1);
                        bitmap.set(row_idx, false);
                    }
                }
                None => {
                    if !self.nullable {
                        return Err(StorageError::null_value_not_allowed(self.name.clone()));
                    }
                    self.offsets[row_idx] = usize::MAX;

                    if let Some(ref mut bitmap) = self.null_bitmap {
                        Self::ensure_bitmap_len(bitmap, row_idx + 1);
                        bitmap.set(row_idx, true);
                    }
                }
            }
        } else {
            let element_size = Self::element_size(&self.data_type);
            let offset = row_idx * element_size;

            if offset >= self.data.len() {
                self.data.resize(offset + element_size, 0);
            }

            match value {
                Some(v) => {
                    self.write_fixed_value(offset, v)?;
                    if let Some(ref mut bitmap) = self.null_bitmap {
                        Self::ensure_bitmap_len(bitmap, row_idx + 1);
                        bitmap.set(row_idx, false);
                    }
                }
                None => {
                    if !self.nullable {
                        return Err(StorageError::null_value_not_allowed(self.name.clone()));
                    }
                    if let Some(ref mut bitmap) = self.null_bitmap {
                        Self::ensure_bitmap_len(bitmap, row_idx + 1);
                        bitmap.set(row_idx, true);
                    }
                }
            }
        }

        if row_idx >= self.row_count {
            self.row_count = row_idx + 1;
        }

        Ok(())
    }

    fn ensure_bitmap_len(bitmap: &mut BitVec<u8, Lsb0>, min_len: usize) {
        if bitmap.len() < min_len {
            bitmap.resize(min_len, false);
        }
    }

    pub fn get(&self, row_idx: usize) -> Option<Value> {
        if self.encoding.is_encoded() {
            return self.encoding.get(row_idx);
        }

        if self.is_null(row_idx) {
            return None;
        }

        if self.is_variable_length() {
            if row_idx >= self.offsets.len() {
                return None;
            }
            let start = self.offsets[row_idx];
            if start == usize::MAX {
                return None;
            }
            self.read_variable_value(start)
        } else {
            let element_size = Self::element_size(&self.data_type);
            let offset = row_idx * element_size;

            if offset + element_size > self.data.len() {
                return None;
            }

            self.read_fixed_value(offset)
        }
    }

    fn write_fixed_value(&mut self, offset: usize, value: &Value) -> StorageResult<()> {
        match (&self.data_type, value) {
            (DataType::Bool, Value::Bool(b)) => {
                self.data[offset] = if *b { 1 } else { 0 };
            }
            (DataType::SmallInt, Value::SmallInt(i)) => {
                self.data[offset..offset + 2].copy_from_slice(&i.to_le_bytes());
            }
            (DataType::Int, Value::Int(i)) => {
                self.data[offset..offset + 4].copy_from_slice(&i.to_le_bytes());
            }
            (DataType::BigInt, Value::BigInt(i)) => {
                self.data[offset..offset + 8].copy_from_slice(&i.to_le_bytes());
            }
            (DataType::Float, Value::Float(f)) => {
                self.data[offset..offset + 4].copy_from_slice(&f.to_le_bytes());
            }
            (DataType::Double, Value::Double(d)) => {
                self.data[offset..offset + 8].copy_from_slice(&d.to_le_bytes());
            }
            (DataType::Date, Value::Date(d)) => {
                self.data[offset..offset + 4].copy_from_slice(&d.year.to_le_bytes());
                self.data[offset + 4..offset + 8].copy_from_slice(&d.month.to_le_bytes());
                self.data[offset + 8..offset + 12].copy_from_slice(&d.day.to_le_bytes());
            }
            _ => {
                return Err(StorageError::type_mismatch(self.data_type.clone(), value.data_type(),
                ));
            }
        }
        Ok(())
    }

    fn write_variable_value(&mut self, value: &Value) -> StorageResult<()> {
        match (&self.data_type, value) {
            (DataType::String, Value::String(s)) => {
                let bytes = s.as_bytes();
                let len = bytes.len() as u64;
                self.data.extend_from_slice(&len.to_le_bytes());
                self.data.extend_from_slice(bytes);
            }
            _ => {
                return Err(StorageError::type_mismatch(self.data_type.clone(), value.data_type(),
                ));
            }
        }
        Ok(())
    }

    fn read_fixed_value(&self, offset: usize) -> Option<Value> {
        let element_size = Self::element_size(&self.data_type);
        if offset + element_size > self.data.len() {
            return None;
        }

        match &self.data_type {
            DataType::Bool => Some(Value::Bool(self.data[offset] != 0)),
            DataType::SmallInt => {
                let bytes: [u8; 2] = self.data[offset..offset + 2].try_into().ok()?;
                Some(Value::SmallInt(i16::from_le_bytes(bytes)))
            }
            DataType::Int => {
                let bytes: [u8; 4] = self.data[offset..offset + 4].try_into().ok()?;
                Some(Value::Int(i32::from_le_bytes(bytes)))
            }
            DataType::BigInt => {
                let bytes: [u8; 8] = self.data[offset..offset + 8].try_into().ok()?;
                Some(Value::BigInt(i64::from_le_bytes(bytes)))
            }
            DataType::Float => {
                let bytes: [u8; 4] = self.data[offset..offset + 4].try_into().ok()?;
                Some(Value::Float(f32::from_le_bytes(bytes)))
            }
            DataType::Double => {
                let bytes: [u8; 8] = self.data[offset..offset + 8].try_into().ok()?;
                Some(Value::Double(f64::from_le_bytes(bytes)))
            }
            DataType::Date => {
                let year_bytes: [u8; 4] = self.data[offset..offset + 4].try_into().ok()?;
                let month_bytes: [u8; 4] = self.data[offset + 4..offset + 8].try_into().ok()?;
                let day_bytes: [u8; 4] = self.data[offset + 8..offset + 12].try_into().ok()?;
                Some(Value::Date(DateValue {
                    year: i32::from_le_bytes(year_bytes),
                    month: u32::from_le_bytes(month_bytes),
                    day: u32::from_le_bytes(day_bytes),
                }))
            }
            _ => None,
        }
    }

    fn read_variable_value(&self, start: usize) -> Option<Value> {
        if start + 8 > self.data.len() {
            return None;
        }

        let len_bytes: [u8; 8] = self.data[start..start + 8].try_into().ok()?;
        let len = u64::from_le_bytes(len_bytes) as usize;

        if start + 8 + len > self.data.len() {
            return None;
        }

        match &self.data_type {
            DataType::String => {
                let bytes = &self.data[start + 8..start + 8 + len];
                String::from_utf8(bytes.to_vec()).ok().map(Value::String)
            }
            _ => None,
        }
    }

    pub fn is_null(&self, row_idx: usize) -> bool {
        self.null_bitmap
            .as_ref()
            .map(|b| row_idx < b.len() && b[row_idx])
            .unwrap_or(false)
    }

    pub fn null_count(&self) -> usize {
        self.null_bitmap
            .as_ref()
            .map(|b| b.count_ones())
            .unwrap_or(0)
    }

    pub fn memory_usage(&self) -> usize {
        let data_size = self.data.len();
        let offsets_size = self.offsets.len() * std::mem::size_of::<usize>();
        let bitmap_size = self.null_bitmap.as_ref().map(|b| b.as_raw_slice().len()).unwrap_or(0);
        data_size + offsets_size + bitmap_size
    }

    pub fn len(&self) -> usize {
        self.row_count
    }

    pub fn is_empty(&self) -> bool {
        self.row_count == 0
    }

    pub fn data_size(&self) -> usize {
        self.data.len()
    }

    pub fn clear(&mut self) {
        self.data.clear();
        self.offsets.clear();
        if let Some(ref mut bitmap) = self.null_bitmap {
            bitmap.clear();
        }
        self.row_count = 0;
        self.encoding = ColumnEncoding::None;
    }

    pub fn resize(&mut self, new_count: usize) {
        let element_size = Self::element_size(&self.data_type);
        self.data.resize(new_count * element_size, 0);
        if let Some(ref mut bitmap) = self.null_bitmap {
            bitmap.resize(new_count, false);
        }
        self.row_count = new_count;
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn null_bitmap(&self) -> Option<&BitVec<u8, Lsb0>> {
        self.null_bitmap.as_ref()
    }

    pub fn null_bitmap_raw(&self) -> Option<&[u8]> {
        self.null_bitmap.as_ref().map(|b| b.as_raw_slice())
    }

    pub fn load_data(&mut self, data: Vec<u8>, null_bitmap: Option<BitVec<u8, Lsb0>>) {
        self.data = data;
        self.null_bitmap = null_bitmap;
        let element_size = Self::element_size(&self.data_type);
        self.row_count = self.data.len() / element_size.max(1);
    }

    pub fn load_data_from_raw(&mut self, data: Vec<u8>, null_bitmap_raw: Option<Vec<u8>>, bitmap_bit_len: usize) {
        self.data = data;
        self.null_bitmap = null_bitmap_raw.map(|raw| {
            let mut bv = BitVec::from_vec(raw);
            bv.resize(bitmap_bit_len, false);
            bv
        });
        let element_size = Self::element_size(&self.data_type);
        self.row_count = self.data.len() / element_size.max(1);
    }

    pub fn encoding_type(&self) -> EncodingType {
        self.encoding.encoding_type()
    }
    
    pub fn encoding(&self) -> &ColumnEncoding {
        &self.encoding
    }

    pub fn apply_fsst_encoding(&mut self, max_symbols: usize) -> StorageResult<()> {
        if self.data_type != DataType::String {
            return Err(StorageError::not_supported(
                "FSST encoding only supports String type".to_string(),
            ));
        }

        let mut strings: Vec<Option<String>> = Vec::with_capacity(self.row_count);
        for i in 0..self.row_count {
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

        let mut encoded_data = Vec::with_capacity(self.row_count);
        let mut null_bitmap = NullBitmap::with_capacity(self.row_count);

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
            let symbol_count = u32::from_le_bytes(symbol_table_bytes[0..4].try_into().unwrap()) as usize;
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
                self.row_count = fsst_col.len();
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

    pub fn memory_size(&self) -> usize {
        self.memory_usage() + std::mem::size_of::<Self>()
    }

    pub fn used_memory_size(&self) -> usize {
        let non_null_count = self.row_count - self.null_count();
        let element_size = Self::element_size(&self.data_type);
        non_null_count * element_size + std::mem::size_of::<Self>()
    }
    
    pub fn collect_stats(&self) -> ColumnStats {
        let mut stats = ColumnStats::new(self.data_type.clone());
        stats.row_count = self.row_count;
        stats.null_count = self.null_count();
        
        let mut distinct_values: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut total_length = 0usize;
        let mut run_count = 1usize;
        let mut prev_value: Option<Value> = None;
        
        for i in 0..self.row_count {
            if let Some(value) = self.get(i) {
                match &value {
                    Value::String(s) => {
                        distinct_values.insert(s.clone());
                        total_length += s.len();
                    }
                    Value::SmallInt(v) => {
                        let int_val = *v as i64;
                        Self::update_int_stats(&mut stats, int_val);
                    }
                    Value::Int(v) => {
                        let int_val = *v as i64;
                        Self::update_int_stats(&mut stats, int_val);
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
        stats.avg_length = if self.row_count > 0 {
            total_length as f64 / self.row_count as f64
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
        for i in 0..self.row_count {
            let value = self.get(i);
            dict_col.set(i, value.as_ref())?;
        }
        
        self.encoding = ColumnEncoding::Dictionary(dict_col);
        
        Ok(())
    }
    
    pub fn apply_rle_encoding(&mut self) -> StorageResult<()> {
        use super::encoding::{RleIntColumn, RleBoolColumn};
        
        match self.data_type {
            DataType::Bool => {
                let mut rle_col = RleBoolColumn::new();
                for i in 0..self.row_count {
                    let value = self.get(i);
                    rle_col.append(value.as_ref())?;
                }
                self.encoding = ColumnEncoding::RleBool(rle_col);
            }
            DataType::SmallInt | DataType::Int | DataType::BigInt => {
                let mut rle_col = RleIntColumn::new();
                for i in 0..self.row_count {
                    let value = self.get(i);
                    rle_col.append(value.as_ref())?;
                }
                self.encoding = ColumnEncoding::RleInt(rle_col);
            }
            _ => {
                return Err(StorageError::not_supported(
                    format!("RLE encoding not supported for {:?}", self.data_type),
                ));
            }
        }
        
        Ok(())
    }
    
    pub fn apply_bitpacking_encoding(&mut self) -> StorageResult<()> {
        use super::encoding::BitPackedIntColumn;
        
        match self.data_type {
            DataType::SmallInt | DataType::Int | DataType::BigInt => {
                let mut values: Vec<Option<Value>> = Vec::with_capacity(self.row_count);
                for i in 0..self.row_count {
                    values.push(self.get(i));
                }
                let bp_col = BitPackedIntColumn::analyze(&values, self.data_type.clone())?;
                self.encoding = ColumnEncoding::BitPacked(bp_col);
            }
            _ => {
                return Err(StorageError::not_supported(
                    format!("BitPacking encoding not supported for {:?}", self.data_type),
                ));
            }
        }
        
        Ok(())
    }
    
    pub fn apply_alp_encoding(&mut self) -> StorageResult<()> {
        use super::encoding::AlpColumn;
        
        match self.data_type {
            DataType::Float | DataType::Double => {
                let mut values: Vec<Option<Value>> = Vec::with_capacity(self.row_count);
                for i in 0..self.row_count {
                    values.push(self.get(i));
                }
                let alp_col = AlpColumn::analyze_values(&values, self.data_type.clone())?;
                self.encoding = ColumnEncoding::Alp(alp_col);
            }
            _ => {
                return Err(StorageError::not_supported(
                    format!("ALP encoding not supported for {:?}", self.data_type),
                ));
            }
        }
        
        Ok(())
    }
}

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
        null_bitmap: Option<BitVec<u8, Lsb0>>,
    ) -> StorageResult<()> {
        if let Some(col) = self.get_column_mut(name) {
            col.load_data(data, null_bitmap);
            Ok(())
        } else {
            Err(StorageError::column_not_found(name.to_string()))
        }
    }

    pub fn load_column_from_raw(
        &mut self,
        name: &str,
        data: Vec<u8>,
        null_bitmap_raw: Option<Vec<u8>>,
        bitmap_bit_len: usize,
    ) -> StorageResult<()> {
        if let Some(col) = self.get_column_mut(name) {
            col.load_data_from_raw(data, null_bitmap_raw, bitmap_bit_len);
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

    pub fn memory_size(&self) -> usize {
        let mut total = std::mem::size_of::<Self>();

        for col in &self.columns {
            total += col.memory_size();
        }

        total += self.name_to_index.len() * (std::mem::size_of::<String>() + std::mem::size_of::<usize>());

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
}

//! Edge Table
//!
//! Combines out/in CSRs and property storage for edge management.
//! Uses EdgeOffset (CSR-native offset) instead of global EdgeId for edge identification.

use super::{
    Csr, CsrBase, EdgeRecord, EdgeSchema, EdgeStrategy, LabelId, MutableCsrTrait,
    MutableCsrVariant, Nbr, PropertyTable, Timestamp, VertexId,
};
use crate::core::types::EdgeId;
use crate::core::{DataType, StorageError, StorageResult, Value};
use crate::storage::persistence::{read_header, section, write_header_to, HEADER_SIZE};
use crate::storage::types::{PropertyId, StoragePropertyDef};
use std::collections::{HashMap, HashSet};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct EdgeTableConfig {
    pub initial_vertex_capacity: usize,
    pub initial_edge_capacity: usize,
}

impl Default for EdgeTableConfig {
    fn default() -> Self {
        Self {
            initial_vertex_capacity: 4096,
            initial_edge_capacity: 4096,
        }
    }
}

/// Parameters for update_edge_property_by_offset operation
pub struct UpdateEdgePropertyByOffsetParams {
    pub src: u32,
    pub dst: u32,
    pub rank: i64,
    pub prop_id: u16,
    pub value: Value,
    pub ts: Timestamp,
}

#[derive(Debug)]
struct CsrSegment {
    csr: Csr,
    min_ts: Timestamp,
    max_ts: Timestamp,
}

impl CsrSegment {
    fn new(csr: Csr, min_ts: Timestamp, max_ts: Timestamp) -> Self {
        Self {
            csr,
            min_ts,
            max_ts,
        }
    }
}

#[derive(Debug)]
pub struct EdgeTable {
    label: LabelId,
    label_name: String,
    src_label: LabelId,
    dst_label: LabelId,
    schema: EdgeSchema,
    out_csr: MutableCsrVariant,
    in_csr: MutableCsrVariant,
    out_segments: Vec<CsrSegment>,
    in_segments: Vec<CsrSegment>,
    tombstones: HashMap<EdgeId, Timestamp>,
    properties: PropertyTable,
    is_open: bool,
    next_edge_id: EdgeId,
}

impl EdgeTable {
    pub fn new(schema: EdgeSchema) -> StorageResult<Self> {
        Self::with_config(schema, EdgeTableConfig::default())
    }

    pub fn with_config(schema: EdgeSchema, config: EdgeTableConfig) -> StorageResult<Self> {
        let out_csr = MutableCsrVariant::from_strategy(
            schema.oe_strategy,
            config.initial_vertex_capacity,
            config.initial_edge_capacity,
        )?;
        let in_csr = MutableCsrVariant::from_strategy(
            schema.ie_strategy,
            config.initial_vertex_capacity,
            config.initial_edge_capacity,
        )?;

        let mut properties = PropertyTable::with_capacity(config.initial_edge_capacity);
        for prop in &schema.properties {
            properties.add_property(prop.name.clone(), prop.data_type.clone(), prop.nullable);
        }

        Ok(Self {
            label: schema.label_id,
            label_name: schema.label_name.clone(),
            src_label: schema.src_label,
            dst_label: schema.dst_label,
            schema,
            out_csr,
            in_csr,
            out_segments: Vec::new(),
            in_segments: Vec::new(),
            tombstones: HashMap::new(),
            properties,
            is_open: true,
            next_edge_id: EdgeId(0),
        })
    }

    fn edge_endpoint_key(endpoint: u32, rank: i64) -> VertexId {
        let mut data = Vec::with_capacity(16);
        data.extend_from_slice(&(endpoint as i64).to_be_bytes());
        data.extend_from_slice(&rank.to_be_bytes());
        VertexId::from_bytes(data)
    }

    fn decode_edge_endpoint(key: VertexId) -> (VertexId, i64) {
        let bytes = key.as_bytes();
        if bytes.len() != 16 {
            return (key, 0);
        }

        let mut endpoint_bytes = [0u8; 8];
        endpoint_bytes.copy_from_slice(&bytes[..8]);
        let mut rank_bytes = [0u8; 8];
        rank_bytes.copy_from_slice(&bytes[8..16]);

        (
            VertexId::from_int64(i64::from_be_bytes(endpoint_bytes)),
            i64::from_be_bytes(rank_bytes),
        )
    }

    fn is_tombstoned(&self, edge_id: EdgeId, ts: Timestamp) -> bool {
        self.tombstones
            .get(&edge_id)
            .is_some_and(|delete_ts| *delete_ts <= ts)
    }

    fn base_get_edge(
        &self,
        segments: &[CsrSegment],
        src: u32,
        dst: VertexId,
        ts: Timestamp,
    ) -> Option<Nbr> {
        for segment in segments.iter().rev() {
            if segment.min_ts > ts {
                continue;
            }

            let Some(edge) = segment.csr.get_edge(src, dst) else {
                continue;
            };

            if edge.timestamp <= ts && !self.is_tombstoned(edge.edge_id, ts) {
                return Some(Nbr::new(
                    edge.neighbor,
                    edge.edge_id,
                    edge.prop_offset,
                    edge.timestamp,
                ));
            }
        }

        None
    }

    fn base_edges_of(&self, segments: &[CsrSegment], src: u32, ts: Timestamp) -> Vec<Nbr> {
        let mut edges = Vec::new();
        for segment in segments.iter().rev() {
            if segment.min_ts > ts {
                continue;
            }

            for edge in segment.csr.edges_of(src) {
                if edge.timestamp <= ts && !self.is_tombstoned(edge.edge_id, ts) {
                    edges.push(Nbr::new(
                        edge.neighbor,
                        edge.edge_id,
                        edge.prop_offset,
                        edge.timestamp,
                    ));
                }
            }
        }

        edges
    }

    fn merged_edges_of(
        &self,
        delta: &MutableCsrVariant,
        segments: &[CsrSegment],
        src: u32,
        ts: Timestamp,
    ) -> Vec<Nbr> {
        let mut seen = HashSet::new();
        let mut result = Vec::new();

        for nbr in delta.edges_of(src, ts) {
            if !self.is_tombstoned(nbr.edge_id, ts) && seen.insert(nbr.edge_id) {
                result.push(nbr);
            }
        }

        for nbr in self.base_edges_of(segments, src, ts) {
            if seen.insert(nbr.edge_id) {
                result.push(nbr);
            }
        }

        result
    }

    fn merged_get_edge(
        &self,
        delta: &MutableCsrVariant,
        segments: &[CsrSegment],
        src: u32,
        dst: VertexId,
        ts: Timestamp,
    ) -> Option<Nbr> {
        if let Some(nbr) = delta.get_edge(src, dst, ts) {
            if !self.is_tombstoned(nbr.edge_id, ts) {
                return Some(nbr);
            }
        }

        self.base_get_edge(segments, src, dst, ts)
    }

    fn edge_record_from_nbr(&self, src: u32, nbr: Nbr) -> EdgeRecord {
        let (dst_vid, rank) = Self::decode_edge_endpoint(nbr.neighbor);
        EdgeRecord {
            src_vid: VertexId::from_int64(src as i64),
            dst_vid,
            rank,
            properties: self.properties_for_offset(nbr.prop_offset),
        }
    }

    fn properties_for_offset(&self, prop_offset: u32) -> Vec<(String, Value)> {
        if prop_offset == 0 {
            return Vec::new();
        }

        self.properties
            .get(prop_offset)
            .map(|props| {
                props
                    .into_iter()
                    .filter_map(|(k, v)| v.map(|v| (k, v)))
                    .collect()
            })
            .unwrap_or_default()
    }
}

impl EdgeTable {
    pub fn insert_edge(
        &mut self,
        src: u32,
        dst: u32,
        rank: i64,
        property_values: &[(String, Value)],
        ts: Timestamp,
    ) -> StorageResult<()> {
        if !self.is_open {
            return Err(StorageError::storage_not_open());
        }

        if self.schema.oe_strategy == EdgeStrategy::None {
            return Err(StorageError::invalid_operation(
                "Cannot insert edge: out-edge strategy is None".to_string(),
            ));
        }

        let mut converted_values: Vec<(String, Value)> = Vec::with_capacity(property_values.len());
        for (name, value) in property_values {
            let prop_def = self
                .schema
                .properties
                .iter()
                .find(|p| p.name == *name)
                .ok_or_else(|| StorageError::column_not_found(name.clone()))?;

            if value.data_type() != prop_def.data_type {
                let converted = value.try_cast_to(&prop_def.data_type)?;
                converted_values.push((name.clone(), converted));
            } else {
                converted_values.push((name.clone(), value.clone()));
            }
        }

        let prop_offset = if !converted_values.is_empty() {
            self.properties.insert(&converted_values)?
        } else {
            0
        };

        if self.has_edge(src, dst, rank, ts) {
            self.properties.delete(prop_offset);
            return Err(StorageError::edge_already_exists(format!(
                "{} -> {}@{}",
                src, dst, rank
            )));
        }

        let dst_key = Self::edge_endpoint_key(dst, rank);
        let src_key = Self::edge_endpoint_key(src, rank);

        let edge_id = self.next_edge_id.fetch_add();
        if !self
            .out_csr
            .insert_edge(src, dst_key, edge_id, prop_offset, ts)
        {
            self.properties.delete(prop_offset);
            return Err(StorageError::edge_already_exists(format!(
                "{} -> {}@{}",
                src, dst, rank
            )));
        }

        // in_csr.insert_edge safely returns false if strategy is None
        if !self
            .in_csr
            .insert_edge(dst, src_key, edge_id, prop_offset, ts)
        {
            self.out_csr.delete_edge(src, edge_id, ts);
            self.properties.delete(prop_offset);
            return Err(StorageError::edge_already_exists(format!(
                "{} -> {}@{}",
                dst, src, rank
            )));
        }

        Ok(())
    }

    pub fn delete_edge(
        &mut self,
        src: u32,
        dst: u32,
        rank: i64,
        ts: Timestamp,
    ) -> StorageResult<bool> {
        if !self.is_open {
            return Err(StorageError::storage_not_open());
        }

        let dst_key = Self::edge_endpoint_key(dst, rank);
        let src_key = Self::edge_endpoint_key(src, rank);

        if let Some(nbr) = self.out_csr.get_edge(src, dst_key, ts) {
            let edge_id = nbr.edge_id;

            self.out_csr.delete_edge(src, edge_id, ts);
            self.in_csr.delete_edge_by_dst(dst, src_key, ts);

            return Ok(true);
        }

        if let Some(nbr) = self.base_get_edge(&self.out_segments, src, dst_key, ts) {
            self.tombstones.insert(nbr.edge_id, ts);
            return Ok(true);
        }

        Ok(false)
    }

    pub fn delete_edge_by_offset(
        &mut self,
        src: u32,
        dst: u32,
        rank: i64,
        oe_offset: i32,
        ie_offset: i32,
        ts: Timestamp,
    ) -> StorageResult<bool> {
        if !self.is_open {
            return Err(StorageError::storage_not_open());
        }

        let dst_key = Self::edge_endpoint_key(dst, rank);
        if self.out_csr.get_edge(src, dst_key, ts).is_some() {
            self.out_csr.delete_edge_by_offset(src, oe_offset, ts);
            self.in_csr.delete_edge_by_offset(dst, ie_offset, ts);

            return Ok(true);
        }

        Ok(false)
    }

    pub fn revert_delete_edge_by_offset(
        &mut self,
        src: u32,
        dst: u32,
        _rank: i64,
        oe_offset: i32,
        ie_offset: i32,
        ts: Timestamp,
    ) -> StorageResult<bool> {
        if !self.is_open {
            return Err(StorageError::storage_not_open());
        }

        let reverted = self.out_csr.revert_delete_by_offset(src, oe_offset, ts);

        if reverted {
            self.in_csr.revert_delete_by_offset(dst, ie_offset, ts);
        }

        Ok(reverted)
    }

    pub fn get_edge(&self, src: u32, dst: u32, rank: i64, ts: Timestamp) -> Option<EdgeRecord> {
        if !self.is_open {
            return None;
        }

        let dst_key = Self::edge_endpoint_key(dst, rank);
        let nbr = self.merged_get_edge(&self.out_csr, &self.out_segments, src, dst_key, ts)?;
        let properties = self.properties_for_offset(nbr.prop_offset);

        Some(EdgeRecord {
            src_vid: VertexId::from_int64(src as i64),
            dst_vid: VertexId::from_int64(dst as i64),
            rank,
            properties,
        })
    }

    pub fn out_edges(&self, src: u32, ts: Timestamp) -> Vec<EdgeRecord> {
        if !self.is_open {
            return Vec::new();
        }

        self.merged_edges_of(&self.out_csr, &self.out_segments, src, ts)
            .into_iter()
            .map(|nbr| {
                let (dst_vid, rank) = Self::decode_edge_endpoint(nbr.neighbor);
                let properties = self.properties_for_offset(nbr.prop_offset);

                EdgeRecord {
                    src_vid: VertexId::from_int64(src as i64),
                    dst_vid,
                    rank,
                    properties,
                }
            })
            .collect()
    }

    pub fn in_edges(&self, dst: u32, ts: Timestamp) -> Vec<EdgeRecord> {
        if !self.is_open {
            return Vec::new();
        }

        self.merged_edges_of(&self.in_csr, &self.in_segments, dst, ts)
            .into_iter()
            .map(|nbr| {
                let (src_vid, rank) = Self::decode_edge_endpoint(nbr.neighbor);
                let properties = self.properties_for_offset(nbr.prop_offset);

                EdgeRecord {
                    src_vid,
                    dst_vid: VertexId::from_int64(dst as i64),
                    rank,
                    properties,
                }
            })
            .collect()
    }

    pub fn has_edge(&self, src: u32, dst: u32, rank: i64, ts: Timestamp) -> bool {
        if !self.is_open {
            return false;
        }
        let dst_key = Self::edge_endpoint_key(dst, rank);
        self.merged_get_edge(&self.out_csr, &self.out_segments, src, dst_key, ts)
            .is_some()
    }

    pub fn edge_count(&self) -> u64 {
        self.out_csr.edge_count()
            + self
                .out_segments
                .iter()
                .map(|segment| {
                    segment
                        .csr
                        .iter()
                        .filter(|(_, edge)| !self.is_tombstoned(edge.edge_id, u32::MAX))
                        .count() as u64
                })
                .sum::<u64>()
    }

    pub fn scan(&self, ts: Timestamp) -> Vec<EdgeRecord> {
        if !self.is_open {
            return Vec::new();
        }

        self.iter(ts).collect()
    }

    pub fn add_property(
        &mut self,
        name: String,
        data_type: DataType,
        nullable: bool,
    ) -> StorageResult<()> {
        if !self.is_open {
            return Err(StorageError::storage_not_open());
        }

        if self.properties.has_property(&name) {
            return Err(StorageError::column_already_exists(name));
        }

        self.properties
            .add_property(name.clone(), data_type.clone(), nullable);
        self.schema
            .properties
            .push(StoragePropertyDef::new(name, data_type));
        Ok(())
    }

    pub fn remove_property(&mut self, name: &str) -> StorageResult<()> {
        if !self.is_open {
            return Err(StorageError::storage_not_open());
        }

        let index = self
            .schema
            .properties
            .iter()
            .position(|prop| prop.name == name)
            .ok_or_else(|| StorageError::column_not_found(name.to_string()))?;

        self.schema.properties.remove(index);
        self.properties.remove_property(name)?;
        Ok(())
    }

    pub fn rename_property(&mut self, old_name: &str, new_name: &str) -> StorageResult<()> {
        if !self.is_open {
            return Err(StorageError::storage_not_open());
        }

        if self
            .schema
            .properties
            .iter()
            .any(|prop| prop.name == new_name)
        {
            return Err(StorageError::column_already_exists(new_name.to_string()));
        }

        let index = self
            .schema
            .properties
            .iter()
            .position(|prop| prop.name == old_name)
            .ok_or_else(|| StorageError::column_not_found(old_name.to_string()))?;

        self.schema.properties[index].name = new_name.to_string();
        self.properties.rename_property(old_name, new_name)?;
        Ok(())
    }

    pub fn update_edge_property(
        &mut self,
        src: u32,
        dst: u32,
        rank: i64,
        prop_name: &str,
        value: &Value,
        ts: Timestamp,
    ) -> StorageResult<bool> {
        if !self.is_open {
            return Err(StorageError::storage_not_open());
        }

        let dst_key = Self::edge_endpoint_key(dst, rank);
        if let Some(nbr) = self.merged_get_edge(&self.out_csr, &self.out_segments, src, dst_key, ts)
        {
            self.properties
                .set_property(nbr.prop_offset, prop_name, Some(value.clone()))?;
            return Ok(true);
        }

        Ok(false)
    }

    pub fn update_edge_property_by_offset(
        &mut self,
        params: UpdateEdgePropertyByOffsetParams,
    ) -> StorageResult<bool> {
        if !self.is_open {
            return Err(StorageError::storage_not_open());
        }

        let dst_key = Self::edge_endpoint_key(params.dst, params.rank);
        if let Some(nbr) = self.merged_get_edge(
            &self.out_csr,
            &self.out_segments,
            params.src,
            dst_key,
            params.ts,
        ) {
            self.properties.set_property_by_id(
                nbr.prop_offset,
                PropertyId(params.prop_id),
                Some(params.value.clone()),
            )?;

            let src_key = Self::edge_endpoint_key(params.src, params.rank);
            if let Some(ie_nbr) = self.merged_get_edge(
                &self.in_csr,
                &self.in_segments,
                params.dst,
                src_key,
                params.ts,
            ) {
                assert_eq!(
                    nbr.prop_offset, ie_nbr.prop_offset,
                    "out_csr and in_csr should share the same prop_offset"
                );
            }
            return Ok(true);
        }

        Ok(false)
    }

    pub fn label(&self) -> LabelId {
        self.label
    }

    pub fn src_label(&self) -> LabelId {
        self.src_label
    }

    pub fn dst_label(&self) -> LabelId {
        self.dst_label
    }

    pub fn label_name(&self) -> &str {
        &self.label_name
    }

    pub fn schema(&self) -> &EdgeSchema {
        &self.schema
    }

    pub fn set_schema(&mut self, schema: EdgeSchema) {
        self.schema = schema;
    }

    pub fn iter(&self, ts: Timestamp) -> EdgeTableScanIterator<'_> {
        EdgeTableScanIterator::new(self, ts)
    }

    pub fn flush<P: AsRef<Path>>(
        &self,
        path: P,
        compression: crate::storage::compression::CompressionType,
    ) -> StorageResult<()> {
        use std::fs::{self, File};
        use std::io::Write;

        let path = path.as_ref();
        fs::create_dir_all(path)?;

        let meta_path = path.join("meta.bin");
        let mut meta_file = File::create(&meta_path)?;
        write_header_to(&mut meta_file, section::EDGE_META).map_err(|e| {
            StorageError::io_error(format!("Failed to write edge meta header: {}", e))
        })?;

        meta_file.write_all(&self.label.to_le_bytes())?;
        meta_file.write_all(&self.src_label.to_le_bytes())?;
        meta_file.write_all(&self.dst_label.to_le_bytes())?;

        let label_name_bytes = self.label_name.as_bytes();
        meta_file.write_all(&(label_name_bytes.len() as u32).to_le_bytes())?;
        meta_file.write_all(label_name_bytes)?;

        let is_open_flag: u8 = if self.is_open { 1 } else { 0 };
        meta_file.write_all(&is_open_flag.to_le_bytes())?;

        let schema_json = serde_json::to_string(&self.schema)
            .map_err(|e| StorageError::serialize_error(e.to_string()))?;
        let schema_bytes = schema_json.as_bytes();
        meta_file.write_all(&(schema_bytes.len() as u32).to_le_bytes())?;
        meta_file.write_all(schema_bytes)?;

        meta_file.write_all(&self.next_edge_id.0.to_le_bytes())?;
        meta_file.write_all(&(self.tombstones.len() as u64).to_le_bytes())?;
        for (edge_id, delete_ts) in &self.tombstones {
            meta_file.write_all(&edge_id.0.to_le_bytes())?;
            meta_file.write_all(&delete_ts.to_le_bytes())?;
        }

        drop(meta_file);
        crate::storage::compression::compress_file_inplace(&meta_path, compression)?;

        let out_csr_path = path.join("out_csr.bin");
        self.flush_csr(
            &self.out_csr,
            &self.out_segments,
            &out_csr_path,
            section::EDGE_OUT_CSR,
        )?;
        crate::storage::compression::compress_file_inplace(&out_csr_path, compression)?;

        let in_csr_path = path.join("in_csr.bin");
        self.flush_csr(
            &self.in_csr,
            &self.in_segments,
            &in_csr_path,
            section::EDGE_IN_CSR,
        )?;
        crate::storage::compression::compress_file_inplace(&in_csr_path, compression)?;

        let props_path = path.join("properties.bin");
        self.flush_properties(&props_path)?;
        crate::storage::compression::compress_file_inplace(&props_path, compression)?;

        Ok(())
    }

    fn flush_csr(
        &self,
        csr: &MutableCsrVariant,
        segments: &[CsrSegment],
        path: &Path,
        section_id: u32,
    ) -> StorageResult<()> {
        use std::fs::File;
        use std::io::Write;

        let mut file = File::create(path)?;
        write_header_to(&mut file, section_id)
            .map_err(|e| StorageError::io_error(format!("Failed to write CSR header: {}", e)))?;

        let data = csr.dump();
        file.write_all(&(data.len() as u64).to_le_bytes())?;
        file.write_all(&data)?;
        file.write_all(&(segments.len() as u64).to_le_bytes())?;
        for segment in segments {
            file.write_all(&segment.min_ts.to_le_bytes())?;
            file.write_all(&segment.max_ts.to_le_bytes())?;
            let data = segment.csr.dump();
            file.write_all(&(data.len() as u64).to_le_bytes())?;
            file.write_all(&data)?;
        }

        Ok(())
    }

    fn flush_properties(&self, path: &Path) -> StorageResult<()> {
        use std::fs::File;
        use std::io::Write;

        let mut file = File::create(path)?;
        write_header_to(&mut file, section::EDGE_PROPERTIES).map_err(|e| {
            StorageError::io_error(format!("Failed to write properties header: {}", e))
        })?;

        let data = self.properties.dump();
        file.write_all(&(data.len() as u64).to_le_bytes())?;
        file.write_all(&data)?;

        Ok(())
    }

    pub fn load<P: AsRef<Path>>(&mut self, path: P) -> StorageResult<()> {
        use std::io::Read;

        let path = path.as_ref();

        let meta_path = path.join("meta.bin");
        let meta_data = crate::storage::compression::read_decompressed(&meta_path)?;
        let mut meta_cursor = &meta_data[..];
        let mut header_buf = [0u8; HEADER_SIZE];
        meta_cursor.read_exact(&mut header_buf)?;
        {
            let mut slice = &header_buf[..];
            let (_version, sid) = read_header(&mut slice)?;
            if sid != section::EDGE_META {
                return Err(StorageError::deserialize_error(format!(
                    "unexpected section id in edge meta: expected {:#06x}, got {:#06x}",
                    section::EDGE_META,
                    sid
                )));
            }
        }

        let mut label_bytes = [0u8; 4];
        meta_cursor.read_exact(&mut label_bytes)?;
        self.label = u32::from_le_bytes(label_bytes);

        let mut src_label_bytes = [0u8; 4];
        meta_cursor.read_exact(&mut src_label_bytes)?;
        self.src_label = u32::from_le_bytes(src_label_bytes);

        let mut dst_label_bytes = [0u8; 4];
        meta_cursor.read_exact(&mut dst_label_bytes)?;
        self.dst_label = u32::from_le_bytes(dst_label_bytes);

        let mut label_name_len_bytes = [0u8; 4];
        meta_cursor.read_exact(&mut label_name_len_bytes)?;
        let label_name_len = u32::from_le_bytes(label_name_len_bytes) as usize;

        let mut label_name_bytes = vec![0u8; label_name_len];
        meta_cursor.read_exact(&mut label_name_bytes)?;
        self.label_name = String::from_utf8(label_name_bytes)
            .map_err(|e| StorageError::deserialize_error(e.to_string()))?;

        let mut is_open_bytes = [0u8; 1];
        meta_cursor.read_exact(&mut is_open_bytes)?;
        self.is_open = is_open_bytes[0] != 0;

        let mut schema_len_bytes = [0u8; 4];
        meta_cursor.read_exact(&mut schema_len_bytes)?;
        let schema_len = u32::from_le_bytes(schema_len_bytes) as usize;
        let mut schema_bytes = vec![0u8; schema_len];
        meta_cursor.read_exact(&mut schema_bytes)?;
        let schema_json = String::from_utf8(schema_bytes)
            .map_err(|e| StorageError::deserialize_error(e.to_string()))?;
        self.schema = serde_json::from_str(&schema_json)
            .map_err(|e| StorageError::deserialize_error(e.to_string()))?;

        let mut next_edge_id_bytes = [0u8; 8];
        meta_cursor.read_exact(&mut next_edge_id_bytes)?;
        self.next_edge_id = EdgeId(u64::from_le_bytes(next_edge_id_bytes));

        let mut tombstone_count_bytes = [0u8; 8];
        meta_cursor.read_exact(&mut tombstone_count_bytes)?;
        let tombstone_count = u64::from_le_bytes(tombstone_count_bytes) as usize;
        self.tombstones.clear();
        for _ in 0..tombstone_count {
            let mut edge_id_bytes = [0u8; 8];
            meta_cursor.read_exact(&mut edge_id_bytes)?;
            let mut delete_ts_bytes = [0u8; 4];
            meta_cursor.read_exact(&mut delete_ts_bytes)?;
            self.tombstones.insert(
                EdgeId(u64::from_le_bytes(edge_id_bytes)),
                u32::from_le_bytes(delete_ts_bytes),
            );
        }

        let out_csr_path = path.join("out_csr.bin");
        Self::load_csr_static(&mut self.out_csr, &mut self.out_segments, &out_csr_path)?;

        let in_csr_path = path.join("in_csr.bin");
        Self::load_csr_static(&mut self.in_csr, &mut self.in_segments, &in_csr_path)?;

        let props_path = path.join("properties.bin");
        self.load_properties(&props_path)?;

        if self.next_edge_id.0 == 0 {
            let ts = u32::MAX;
            let max_id = self
                .out_csr
                .iter(ts)
                .map(|(_, nbr)| nbr.edge_id.0 + 1)
                .chain(
                    self.out_segments
                        .iter()
                        .flat_map(|segment| segment.csr.iter().map(|(_, nbr)| nbr.edge_id.0 + 1)),
                )
                .max()
                .unwrap_or(0);
            self.next_edge_id = EdgeId(max_id);
        }
        self.is_open = true;
        Ok(())
    }

    fn load_csr_static(
        csr: &mut MutableCsrVariant,
        segments: &mut Vec<CsrSegment>,
        path: &Path,
    ) -> StorageResult<()> {
        use std::io::Read;

        let raw_data = crate::storage::compression::read_decompressed(path)?;
        let mut cursor = &raw_data[..];
        let mut header_buf = [0u8; HEADER_SIZE];
        cursor.read_exact(&mut header_buf)?;
        {
            let mut slice = &header_buf[..];
            let (_version, sid) = read_header(&mut slice)?;
            if sid != section::EDGE_OUT_CSR && sid != section::EDGE_IN_CSR {
                return Err(StorageError::deserialize_error(format!(
                    "unexpected section id in edge CSR: expected {:#06x} or {:#06x}, got {:#06x}",
                    section::EDGE_OUT_CSR,
                    section::EDGE_IN_CSR,
                    sid
                )));
            }
        }

        let mut len_bytes = [0u8; 8];
        cursor.read_exact(&mut len_bytes)?;
        let len = u64::from_le_bytes(len_bytes) as usize;

        let mut data = vec![0u8; len];
        cursor.read_exact(&mut data)?;

        csr.load(&data)?;
        segments.clear();

        let mut segment_count_bytes = [0u8; 8];
        cursor.read_exact(&mut segment_count_bytes)?;
        let segment_count = u64::from_le_bytes(segment_count_bytes) as usize;
        for _ in 0..segment_count {
            let mut min_ts_bytes = [0u8; 4];
            cursor.read_exact(&mut min_ts_bytes)?;
            let min_ts = u32::from_le_bytes(min_ts_bytes);

            let mut max_ts_bytes = [0u8; 4];
            cursor.read_exact(&mut max_ts_bytes)?;
            let max_ts = u32::from_le_bytes(max_ts_bytes);

            let mut segment_len_bytes = [0u8; 8];
            cursor.read_exact(&mut segment_len_bytes)?;
            let segment_len = u64::from_le_bytes(segment_len_bytes) as usize;

            let mut segment_data = vec![0u8; segment_len];
            cursor.read_exact(&mut segment_data)?;

            let mut segment_csr = Csr::new();
            segment_csr.load(&segment_data)?;
            segments.push(CsrSegment::new(segment_csr, min_ts, max_ts));
        }

        Ok(())
    }

    fn load_properties(&mut self, path: &Path) -> StorageResult<()> {
        use std::io::Read;

        let raw_data = crate::storage::compression::read_decompressed(path)?;
        let mut cursor = &raw_data[..];
        let mut header_buf = [0u8; HEADER_SIZE];
        cursor.read_exact(&mut header_buf)?;
        {
            let mut slice = &header_buf[..];
            let (_version, sid) = read_header(&mut slice)?;
            if sid != section::EDGE_PROPERTIES {
                return Err(StorageError::deserialize_error(format!(
                    "unexpected section id in edge properties: expected {:#06x}, got {:#06x}",
                    section::EDGE_PROPERTIES,
                    sid
                )));
            }
        }

        let mut len_bytes = [0u8; 8];
        cursor.read_exact(&mut len_bytes)?;
        let len = u64::from_le_bytes(len_bytes) as usize;

        let mut data = vec![0u8; len];
        cursor.read_exact(&mut data)?;

        self.properties.load(&data)?;

        Ok(())
    }

    pub fn compact_csr(&mut self, ts: Timestamp, reserve_ratio: f32) -> usize {
        let removed = self.out_csr.compact_with_ts(ts, reserve_ratio)
            + self.in_csr.compact_with_ts(ts, reserve_ratio);
        self.freeze_csr(ts);
        removed
    }

    pub fn freeze_csr(&mut self, ts: Timestamp) -> usize {
        let out_frozen = Self::freeze_delta(&mut self.out_csr, &mut self.out_segments, ts);
        let in_frozen = Self::freeze_delta(&mut self.in_csr, &mut self.in_segments, ts);
        out_frozen + in_frozen
    }

    fn freeze_delta(
        delta: &mut MutableCsrVariant,
        segments: &mut Vec<CsrSegment>,
        ts: Timestamp,
    ) -> usize {
        let entries: Vec<_> = delta
            .iter(ts)
            .map(|(src, nbr)| {
                let src_u32 = src.as_int64().unwrap_or(0) as u32;
                (src_u32, nbr)
            })
            .collect();
        if entries.is_empty() {
            delta.clear();
            return 0;
        }

        let vertex_capacity = entries
            .iter()
            .map(|(src, _)| *src as usize + 1)
            .max()
            .unwrap_or_else(|| delta.vertex_capacity())
            .max(delta.vertex_capacity());
        let min_ts = entries
            .iter()
            .map(|(_, nbr)| nbr.timestamp)
            .min()
            .unwrap_or(ts);
        let max_ts = entries
            .iter()
            .map(|(_, nbr)| nbr.timestamp)
            .max()
            .unwrap_or(ts);
        let csr = Csr::from_nbr_entries(&entries, vertex_capacity);
        let frozen = entries.len();

        segments.push(CsrSegment::new(csr, min_ts, max_ts));
        delta.clear();
        frozen
    }

    pub fn compact_properties(&mut self, ts: Timestamp) {
        let mut valid_offsets = std::collections::HashSet::new();

        for (_, nbr) in self.out_csr.iter(ts) {
            if nbr.prop_offset > 0 {
                valid_offsets.insert(nbr.prop_offset);
            }
        }

        for segment in &self.out_segments {
            for (_, nbr) in segment.csr.iter() {
                if nbr.timestamp <= ts
                    && !self.is_tombstoned(nbr.edge_id, ts)
                    && nbr.prop_offset > 0
                {
                    valid_offsets.insert(nbr.prop_offset);
                }
            }
        }

        for (_, nbr) in self.in_csr.iter(ts) {
            if nbr.prop_offset > 0 {
                valid_offsets.insert(nbr.prop_offset);
            }
        }

        for segment in &self.in_segments {
            for (_, nbr) in segment.csr.iter() {
                if nbr.timestamp <= ts
                    && !self.is_tombstoned(nbr.edge_id, ts)
                    && nbr.prop_offset > 0
                {
                    valid_offsets.insert(nbr.prop_offset);
                }
            }
        }

        self.properties.compact(&valid_offsets);
    }

    pub fn memory_size(&self) -> usize {
        self.used_memory_size()
    }

    pub fn used_memory_size(&self) -> usize {
        let mut total = 0;

        total += self.out_csr.used_memory_size();
        total += self.in_csr.used_memory_size();
        total += self
            .out_segments
            .iter()
            .map(|segment| segment.csr.used_memory_size())
            .sum::<usize>();
        total += self
            .in_segments
            .iter()
            .map(|segment| segment.csr.used_memory_size())
            .sum::<usize>();
        total += self.tombstones.len() * std::mem::size_of::<(EdgeId, Timestamp)>();
        total += self.properties.used_memory_size();

        total
    }
}

pub struct EdgeTableScanIterator<'a> {
    _table: &'a EdgeTable,
    records: std::vec::IntoIter<EdgeRecord>,
}

impl<'a> EdgeTableScanIterator<'a> {
    pub fn new(table: &'a EdgeTable, ts: Timestamp) -> Self {
        let mut seen = HashSet::new();
        let mut records = Vec::new();

        for (src_vid, nbr) in table.out_csr.iter(ts) {
            if !table.is_tombstoned(nbr.edge_id, ts) && seen.insert(nbr.edge_id) {
                records
                    .push(table.edge_record_from_nbr(src_vid.as_int64().unwrap_or(0) as u32, nbr));
            }
        }

        for segment in table.out_segments.iter().rev() {
            if segment.min_ts > ts {
                continue;
            }

            for (src_vid, edge) in segment.csr.iter() {
                if edge.timestamp <= ts
                    && !table.is_tombstoned(edge.edge_id, ts)
                    && seen.insert(edge.edge_id)
                {
                    records.push(table.edge_record_from_nbr(
                        src_vid.as_int64().unwrap_or(0) as u32,
                        Nbr::new(
                            edge.neighbor,
                            edge.edge_id,
                            edge.prop_offset,
                            edge.timestamp,
                        ),
                    ));
                }
            }
        }

        Self {
            _table: table,
            records: records.into_iter(),
        }
    }
}

impl<'a> Iterator for EdgeTableScanIterator<'a> {
    type Item = EdgeRecord;

    fn next(&mut self) -> Option<Self::Item> {
        self.records.next()
    }
}

#[cfg(test)]
#[path = "edge_table_tests.rs"]
mod tests;

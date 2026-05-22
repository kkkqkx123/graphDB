//! Edge Table
//!
//! Combines out/in CSRs and property storage for edge management.
//! Uses EdgeOffset (CSR-native offset) instead of global EdgeId for edge identification.

use std::path::Path;

use super::{
    CsrBase, CsrEdgeIterator, CsrIterator, EdgeRecord, EdgeSchema, EdgeStrategy, LabelId,
    MutableCsrTrait, MutableCsrVariant, PropertyTable, Timestamp, VertexId,
};
use crate::core::{DataType, StorageError, StorageResult, Value};
use crate::storage::storage_types::{EdgeOffset, PropertyId, StoragePropertyDef};
use crate::storage::utils::persistence_format::{read_header, section, write_header_to, HEADER_SIZE};

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
    pub src: VertexId,
    pub dst: VertexId,
    pub oe_offset: EdgeOffset,
    pub ie_offset: EdgeOffset,
    pub prop_id: u16,
    pub value: Value,
    pub ts: Timestamp,
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
    properties: PropertyTable,
    is_open: bool,
    next_edge_id: u64,
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
            properties,
            is_open: true,
            next_edge_id: 0,
        })
    }
}

impl EdgeTable {
    pub fn open<P: AsRef<Path>>(&mut self, _path: P) -> StorageResult<()> {
        self.is_open = true;
        Ok(())
    }

    pub fn close(&mut self) {
        self.is_open = false;
    }

    pub fn ensure_capacity(&mut self, vertex_capacity: usize, _edge_capacity: usize) {
        self.out_csr.resize(vertex_capacity);
        self.in_csr.resize(vertex_capacity);
    }

    pub fn insert_edge(
        &mut self,
        src: VertexId,
        dst: VertexId,
        property_values: &[(String, Value)],
        ts: Timestamp,
    ) -> StorageResult<EdgeOffset> {
        if !self.is_open {
            return Err(StorageError::storage_not_open());
        }

        if self.schema.oe_strategy == EdgeStrategy::None {
            return Err(StorageError::invalid_operation(
                "Edge strategy is None".to_string(),
            ));
        }

        let mut converted_values: Vec<(String, Value)> = Vec::with_capacity(property_values.len());
        for (name, value) in property_values {
            let prop_def = self
                .schema
                .properties
                .iter()
                .find(|p| &p.name == name)
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

        if self.out_csr.has_edge(src, dst, ts) {
            self.properties.delete(prop_offset);
            return Err(StorageError::edge_already_exists(format!(
                "{} -> {}",
                src, dst
            )));
        }

        let edge_id = self.next_edge_id;
        self.next_edge_id += 1;
        if !self.out_csr.insert_edge(src, dst, edge_id, prop_offset, ts) {
            self.properties.delete(prop_offset);
            return Err(StorageError::edge_already_exists(format!(
                "{} -> {}",
                src, dst
            )));
        }

        if self.schema.ie_strategy != EdgeStrategy::None
            && !self.in_csr.insert_edge(dst, src, edge_id, prop_offset, ts)
        {
            self.out_csr.delete_edge(src, edge_id, ts);
            self.properties.delete(prop_offset);
            return Err(StorageError::edge_already_exists(format!(
                "{} -> {}",
                dst, src
            )));
        }

        Ok(EdgeOffset::new(prop_offset as i32))
    }

    pub fn delete_edge(
        &mut self,
        src: VertexId,
        dst: VertexId,
        ts: Timestamp,
    ) -> StorageResult<bool> {
        if !self.is_open {
            return Err(StorageError::storage_not_open());
        }

        if let Some(nbr) = self.out_csr.get_edge(src, dst, ts) {
            let edge_id = nbr.edge_id;

            self.out_csr.delete_edge(src, edge_id, ts);

            if self.schema.ie_strategy != EdgeStrategy::None {
                self.in_csr.delete_edge_by_dst(dst, src, ts);
            }

            return Ok(true);
        }

        Ok(false)
    }

    pub fn delete_edge_by_offset(
        &mut self,
        src: VertexId,
        dst: VertexId,
        oe_offset: i32,
        ie_offset: i32,
        ts: Timestamp,
    ) -> StorageResult<bool> {
        if !self.is_open {
            return Err(StorageError::storage_not_open());
        }

        if self.out_csr.get_edge(src, dst, ts).is_some() {
            self.out_csr.delete_edge_by_offset(src, oe_offset, ts);

            if self.schema.ie_strategy != EdgeStrategy::None {
                self.in_csr.delete_edge_by_offset(dst, ie_offset, ts);
            }

            return Ok(true);
        }

        Ok(false)
    }

    pub fn revert_delete_edge_by_offset(
        &mut self,
        src: VertexId,
        dst: VertexId,
        oe_offset: EdgeOffset,
        ie_offset: EdgeOffset,
        ts: Timestamp,
    ) -> StorageResult<bool> {
        if !self.is_open {
            return Err(StorageError::storage_not_open());
        }

        let _edge_id = self.out_csr.find_deleted_edge(src, dst);
        let reverted = self
            .out_csr
            .revert_delete_by_offset(src, oe_offset.as_i32(), ts);

        if reverted && self.schema.ie_strategy != EdgeStrategy::None {
            self.in_csr
                .revert_delete_by_offset(dst, ie_offset.as_i32(), ts);
        }

        Ok(reverted)
    }

    pub fn delete_edge_by_id(&mut self, _edge_id: u64, _ts: Timestamp) -> StorageResult<bool> {
        Err(StorageError::invalid_operation(
            "delete_edge_by_id is not supported. Use delete_edge or delete_edge_by_offset instead."
                .to_string(),
        ))
    }

    pub fn get_edge(&self, src: VertexId, dst: VertexId, ts: Timestamp) -> Option<EdgeRecord> {
        if !self.is_open {
            return None;
        }

        let nbr = self.out_csr.get_edge(src, dst, ts)?;

        let properties = if nbr.prop_offset > 0 {
            self.properties
                .get(nbr.prop_offset)
                .map(|props| {
                    props
                        .into_iter()
                        .filter_map(|(k, v)| v.map(|v| (k, v)))
                        .collect()
                })
                .unwrap_or_default()
        } else {
            Vec::new()
        };

        Some(EdgeRecord {
            edge_id: nbr.edge_id,
            src_vid: src,
            dst_vid: dst,
            properties,
        })
    }

    pub fn get_edge_nbr(&self, src: VertexId, dst: VertexId, ts: Timestamp) -> Option<super::Nbr> {
        if !self.is_open {
            return None;
        }
        self.out_csr.get_edge(src, dst, ts)
    }

    pub fn get_edge_by_offset(
        &self,
        src: VertexId,
        dst: VertexId,
        ts: Timestamp,
    ) -> Option<EdgeRecord> {
        if !self.is_open {
            return None;
        }

        let nbr = self.out_csr.get_edge(src, dst, ts)?;

        let properties = if nbr.prop_offset > 0 {
            self.properties
                .get(nbr.prop_offset)
                .map(|props| {
                    props
                        .into_iter()
                        .filter_map(|(k, v)| v.map(|v| (k, v)))
                        .collect()
                })
                .unwrap_or_default()
        } else {
            Vec::new()
        };

        Some(EdgeRecord {
            edge_id: nbr.edge_id,
            src_vid: src,
            dst_vid: dst,
            properties,
        })
    }

    pub fn update_properties(
        &mut self,
        src: VertexId,
        dst: VertexId,
        values: &[(String, Value)],
        ts: Timestamp,
    ) -> StorageResult<bool> {
        if !self.is_open {
            return Err(StorageError::storage_not_open());
        }

        if let Some(nbr) = self.out_csr.get_edge(src, dst, ts) {
            self.properties.update(nbr.prop_offset, values)?;

            return Ok(true);
        }

        Ok(false)
    }

    pub fn update_properties_by_id(
        &mut self,
        src: VertexId,
        dst: VertexId,
        values: &[(u16, Value)],
        ts: Timestamp,
    ) -> StorageResult<bool> {
        if !self.is_open {
            return Err(StorageError::storage_not_open());
        }

        if let Some(nbr) = self.out_csr.get_edge(src, dst, ts) {
            for (prop_id, value) in values {
                let prop_id = PropertyId::new(*prop_id);
                self.properties.set_property_by_id(
                    nbr.prop_offset,
                    prop_id,
                    Some(value.clone()),
                )?;
            }

            return Ok(true);
        }

        Ok(false)
    }

    pub fn out_edges(&self, src: VertexId, ts: Timestamp) -> Vec<EdgeRecord> {
        if !self.is_open {
            return Vec::new();
        }

        self.out_csr
            .edges_of(src, ts)
            .into_iter()
            .map(|nbr| {
                let properties = if nbr.prop_offset > 0 {
                    self.properties
                        .get(nbr.prop_offset)
                        .map(|props| {
                            props
                                .into_iter()
                                .filter_map(|(k, v)| v.map(|v| (k, v)))
                                .collect()
                        })
                        .unwrap_or_default()
                } else {
                    Vec::new()
                };

                EdgeRecord {
                    edge_id: nbr.edge_id,
                    src_vid: src,
                    dst_vid: nbr.neighbor,
                    properties,
                }
            })
            .collect()
    }

    pub fn in_edges(&self, dst: VertexId, ts: Timestamp) -> Vec<EdgeRecord> {
        if !self.is_open {
            return Vec::new();
        }

        self.in_csr
            .edges_of(dst, ts)
            .into_iter()
            .map(|nbr| {
                let properties = if nbr.prop_offset > 0 {
                    self.properties
                        .get(nbr.prop_offset)
                        .map(|props| {
                            props
                                .into_iter()
                                .filter_map(|(k, v)| v.map(|v| (k, v)))
                                .collect()
                        })
                        .unwrap_or_default()
                } else {
                    Vec::new()
                };

                EdgeRecord {
                    edge_id: nbr.edge_id,
                    src_vid: nbr.neighbor,
                    dst_vid: dst,
                    properties,
                }
            })
            .collect()
    }

    pub fn out_degree(&self, src: VertexId, ts: Timestamp) -> usize {
        if !self.is_open {
            return 0;
        }
        self.out_csr.degree(src, ts)
    }

    pub fn in_degree(&self, dst: VertexId, ts: Timestamp) -> usize {
        if !self.is_open {
            return 0;
        }
        self.in_csr.degree(dst, ts)
    }

    pub fn has_edge(&self, src: VertexId, dst: VertexId, ts: Timestamp) -> bool {
        if !self.is_open {
            return false;
        }
        self.out_csr.has_edge(src, dst, ts)
    }

    pub fn edge_count(&self) -> u64 {
        self.out_csr.edge_count()
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

        self.properties.add_property(name.clone(), data_type.clone(), nullable);
        self.schema.properties.push(StoragePropertyDef::new(name, data_type));
        Ok(())
    }

    pub fn update_edge_property(
        &mut self,
        src: VertexId,
        dst: VertexId,
        prop_name: &str,
        value: &Value,
        ts: Timestamp,
    ) -> StorageResult<bool> {
        if !self.is_open {
            return Err(StorageError::storage_not_open());
        }

        if let Some(nbr) = self.out_csr.get_edge(src, dst, ts) {
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

        if let Some(nbr) = self.out_csr.get_edge(params.src, params.dst, params.ts) {
            self.properties.set_property_by_id(
                nbr.prop_offset,
                PropertyId(params.prop_id),
                Some(params.value.clone()),
            )?;

            if self.schema.ie_strategy != EdgeStrategy::None {
                if let Some(ie_nbr) = self.in_csr.get_edge(params.dst, params.src, params.ts) {
                    assert_eq!(
                        nbr.prop_offset, ie_nbr.prop_offset,
                        "out_csr and in_csr should share the same prop_offset"
                    );
                }
            }
            return Ok(true);
        }

        Ok(false)
    }

    pub fn revert_delete_edge(
        &mut self,
        src: VertexId,
        dst: VertexId,
        ts: Timestamp,
    ) -> StorageResult<bool> {
        if !self.is_open {
            return Err(StorageError::storage_not_open());
        }

        let eid = self.out_csr.find_deleted_edge(src, dst);
        let eid = match eid {
            Some(id) => id,
            None => return Ok(false),
        };

        let reverted = self.out_csr.revert_delete(src, eid, ts);

        if reverted && self.schema.ie_strategy != EdgeStrategy::None {
            self.in_csr.revert_delete(dst, eid, ts);
        }

        Ok(reverted)
    }

    pub fn label(&self) -> LabelId {
        self.label
    }

    pub fn label_name(&self) -> &str {
        &self.label_name
    }

    pub fn src_label(&self) -> LabelId {
        self.src_label
    }

    pub fn dst_label(&self) -> LabelId {
        self.dst_label
    }

    pub fn schema(&self) -> &EdgeSchema {
        &self.schema
    }

    pub fn set_schema(&mut self, schema: EdgeSchema) {
        self.schema = schema;
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }

    pub fn vertex_capacity(&self) -> usize {
        self.out_csr.vertex_capacity()
    }

    pub fn edges_of(&self, src: VertexId, ts: Timestamp) -> Vec<super::Nbr> {
        if !self.is_open {
            return Vec::new();
        }
        self.out_csr.edges_of(src, ts)
    }

    pub fn iter(&self, ts: Timestamp) -> EdgeTableScanIterator<'_> {
        EdgeTableScanIterator::new(self, ts)
    }

    pub fn iter_edges(&self, src: VertexId, ts: Timestamp) -> EdgeVertexIterator<'_> {
        EdgeVertexIterator::new(self, src, ts)
    }

    pub fn get_properties(&self, prop_offset: u32) -> Option<Vec<(String, Value)>> {
        self.properties.get(prop_offset).map(|props| {
            props
                .into_iter()
                .filter_map(|(k, v)| v.map(|v| (k, v)))
                .collect()
        })
    }

    pub fn compact(&mut self) {
        self.out_csr.compact();
        self.in_csr.compact();
    }

    pub fn clear(&mut self) {
        self.out_csr.clear();
        self.in_csr.clear();
        self.properties.clear();
    }

    pub fn flush<P: AsRef<Path>>(&self, path: P) -> StorageResult<()> {
        use std::fs::{self, File};
        use std::io::Write;

        let path = path.as_ref();
        fs::create_dir_all(path)?;

        let meta_path = path.join("meta.bin");
        let mut meta_file = File::create(&meta_path)?;
        write_header_to(&mut meta_file, section::EDGE_META)
            .map_err(|e| StorageError::io_error(format!("Failed to write edge meta header: {}", e)))?;

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

        meta_file.write_all(&self.next_edge_id.to_le_bytes())?;

        let out_csr_path = path.join("out_csr.bin");
        self.flush_csr(&self.out_csr, &out_csr_path, section::EDGE_OUT_CSR)?;

        let in_csr_path = path.join("in_csr.bin");
        self.flush_csr(&self.in_csr, &in_csr_path, section::EDGE_IN_CSR)?;

        let props_path = path.join("properties.bin");
        self.flush_properties(&props_path)?;

        Ok(())
    }

    fn flush_csr(&self, csr: &MutableCsrVariant, path: &Path, section_id: u32) -> StorageResult<()> {
        use std::fs::File;
        use std::io::Write;

        let mut file = File::create(path)?;
        write_header_to(&mut file, section_id)
            .map_err(|e| StorageError::io_error(format!("Failed to write CSR header: {}", e)))?;

        let data = csr.dump();
        file.write_all(&(data.len() as u64).to_le_bytes())?;
        file.write_all(&data)?;

        Ok(())
    }

    fn flush_properties(&self, path: &Path) -> StorageResult<()> {
        use std::fs::File;
        use std::io::Write;

        let mut file = File::create(path)?;
        write_header_to(&mut file, section::EDGE_PROPERTIES)
            .map_err(|e| StorageError::io_error(format!("Failed to write properties header: {}", e)))?;

        let data = self.properties.dump();
        file.write_all(&(data.len() as u64).to_le_bytes())?;
        file.write_all(&data)?;

        Ok(())
    }

    pub fn load<P: AsRef<Path>>(&mut self, path: P) -> StorageResult<()> {
        use std::fs::File;
        use std::io::Read;

        let path = path.as_ref();
        let mut header_buf = [0u8; HEADER_SIZE];

        let meta_path = path.join("meta.bin");
        let mut meta_file = File::open(&meta_path)?;
        meta_file.read_exact(&mut header_buf)?;
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
        meta_file.read_exact(&mut label_bytes)?;
        self.label = u32::from_le_bytes(label_bytes);

        let mut src_label_bytes = [0u8; 4];
        meta_file.read_exact(&mut src_label_bytes)?;
        self.src_label = u32::from_le_bytes(src_label_bytes);

        let mut dst_label_bytes = [0u8; 4];
        meta_file.read_exact(&mut dst_label_bytes)?;
        self.dst_label = u32::from_le_bytes(dst_label_bytes);

        let mut label_name_len_bytes = [0u8; 4];
        meta_file.read_exact(&mut label_name_len_bytes)?;
        let label_name_len = u32::from_le_bytes(label_name_len_bytes) as usize;

        let mut label_name_bytes = vec![0u8; label_name_len];
        meta_file.read_exact(&mut label_name_bytes)?;
        self.label_name = String::from_utf8(label_name_bytes)
            .map_err(|e| StorageError::deserialize_error(e.to_string()))?;

        let mut is_open_bytes = [0u8; 1];
        if meta_file.read_exact(&mut is_open_bytes).is_ok() {
            self.is_open = is_open_bytes[0] != 0;
        }

        // Read schema JSON (backward compatible: schema may not be present in older files)
        let mut schema_len_bytes = [0u8; 4];
        if meta_file.read_exact(&mut schema_len_bytes).is_ok() {
            let schema_len = u32::from_le_bytes(schema_len_bytes) as usize;
            let mut schema_bytes = vec![0u8; schema_len];
            if meta_file.read_exact(&mut schema_bytes).is_ok() {
                let schema_json = String::from_utf8(schema_bytes)
                    .map_err(|e| StorageError::deserialize_error(e.to_string()))?;
                self.schema = serde_json::from_str(&schema_json)
                    .map_err(|e| StorageError::deserialize_error(e.to_string()))?;
            }
        }

        // Read next_edge_id (backward compatible: may not be present in older files)
        let mut next_edge_id_bytes = [0u8; 8];
        if meta_file.read_exact(&mut next_edge_id_bytes).is_ok() {
            self.next_edge_id = u64::from_le_bytes(next_edge_id_bytes);
        }

        let out_csr_path = path.join("out_csr.bin");
        Self::load_csr_static(&mut self.out_csr, &out_csr_path)?;

        let in_csr_path = path.join("in_csr.bin");
        Self::load_csr_static(&mut self.in_csr, &in_csr_path)?;

        let props_path = path.join("properties.bin");
        self.load_properties(&props_path)?;

        if self.next_edge_id == 0 {
            let ts = u32::MAX;
            self.next_edge_id = self.out_csr.iter(ts).map(|(_, nbr)| nbr.edge_id + 1).max().unwrap_or(0);
        }
        self.is_open = true;
        Ok(())
    }

    fn load_csr_static(csr: &mut MutableCsrVariant, path: &Path) -> StorageResult<()> {
        use std::fs::File;
        use std::io::Read;

        let mut file = File::open(path)?;
        let mut header_buf = [0u8; HEADER_SIZE];
        file.read_exact(&mut header_buf)?;
        {
            let mut slice = &header_buf[..];
            read_header(&mut slice)?;
        }

        let mut len_bytes = [0u8; 8];
        file.read_exact(&mut len_bytes)?;
        let len = u64::from_le_bytes(len_bytes) as usize;

        let mut data = vec![0u8; len];
        file.read_exact(&mut data)?;

        csr.load(&data)?;

        Ok(())
    }

    fn load_properties(&mut self, path: &Path) -> StorageResult<()> {
        use std::fs::File;
        use std::io::Read;

        let mut file = File::open(path)?;
        let mut header_buf = [0u8; HEADER_SIZE];
        file.read_exact(&mut header_buf)?;
        {
            let mut slice = &header_buf[..];
            read_header(&mut slice)?;
        }

        let mut len_bytes = [0u8; 8];
        file.read_exact(&mut len_bytes)?;
        let len = u64::from_le_bytes(len_bytes) as usize;

        let mut data = vec![0u8; len];
        file.read_exact(&mut data)?;

        self.properties.load(&data)?;

        Ok(())
    }

    pub fn compact_csr(&mut self, ts: Timestamp, reserve_ratio: f32) -> usize {
        self.out_csr.compact_with_ts(ts, reserve_ratio)
            + self.in_csr.compact_with_ts(ts, reserve_ratio)
    }

    pub fn compact_properties(&mut self, ts: Timestamp) {
        let mut valid_offsets = std::collections::HashSet::new();

        for (_, nbr) in self.out_csr.iter(ts) {
            if nbr.prop_offset > 0 {
                valid_offsets.insert(nbr.prop_offset);
            }
        }

        if self.schema.ie_strategy != EdgeStrategy::None {
            for (_, nbr) in self.in_csr.iter(ts) {
                if nbr.prop_offset > 0 {
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
        total += self.properties.used_memory_size();

        total
    }
}

pub struct EdgeTableScanIterator<'a> {
    table: &'a EdgeTable,
    csr_iter: CsrIterator<'a>,
}

impl<'a> EdgeTableScanIterator<'a> {
    pub fn new(table: &'a EdgeTable, ts: Timestamp) -> Self {
        let csr_iter = table.out_csr.iter(ts);
        Self { table, csr_iter }
    }
}

impl<'a> Iterator for EdgeTableScanIterator<'a> {
    type Item = EdgeRecord;

    fn next(&mut self) -> Option<Self::Item> {
        self.csr_iter.next().map(|(src_vid, nbr)| {
            let properties = if nbr.prop_offset > 0 {
                self.table
                    .properties
                    .get(nbr.prop_offset)
                    .map(|props| {
                        props
                            .into_iter()
                            .filter_map(|(k, v)| v.map(|v| (k, v)))
                            .collect()
                    })
                    .unwrap_or_default()
            } else {
                Vec::new()
            };

            EdgeRecord {
                edge_id: nbr.edge_id,
                src_vid,
                dst_vid: nbr.neighbor,
                properties,
            }
        })
    }
}

pub struct EdgeVertexIterator<'a> {
    table: &'a EdgeTable,
    csr_iter: CsrEdgeIterator<'a>,
    src_vid: VertexId,
}

impl<'a> EdgeVertexIterator<'a> {
    pub fn new(table: &'a EdgeTable, src: VertexId, ts: Timestamp) -> Self {
        let csr_iter = table.out_csr.iter_edges(src, ts);
        Self {
            table,
            csr_iter,
            src_vid: src,
        }
    }
}

impl<'a> Iterator for EdgeVertexIterator<'a> {
    type Item = EdgeRecord;

    fn next(&mut self) -> Option<Self::Item> {
        self.csr_iter.next().map(|nbr| {
            let properties = if nbr.prop_offset > 0 {
                self.table
                    .properties
                    .get(nbr.prop_offset)
                    .map(|props| {
                        props
                            .into_iter()
                            .filter_map(|(k, v)| v.map(|v| (k, v)))
                            .collect()
                    })
                    .unwrap_or_default()
            } else {
                Vec::new()
            };

            EdgeRecord {
                edge_id: nbr.edge_id,
                src_vid: self.src_vid,
                dst_vid: nbr.neighbor,
                properties,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_schema() -> EdgeSchema {
        EdgeSchema {
            label_id: 0,
            label_name: "knows".to_string(),
            src_label: 0,
            dst_label: 0,
            properties: vec![StoragePropertyDef::new(
                "weight".to_string(),
                DataType::Double,
            )],
            oe_strategy: EdgeStrategy::Multiple,
            ie_strategy: EdgeStrategy::Multiple,
        }
    }

    #[test]
    fn test_insert_and_get() {
        let schema = create_test_schema();
        let mut table = EdgeTable::new(schema).unwrap();

        let _edge_offset = table
            .insert_edge(
                VertexId::from_int64(0),
                VertexId::from_int64(1),
                &[("weight".to_string(), Value::Double(1.5))],
                100,
            )
            .unwrap();

        assert!(table.has_edge(VertexId::from_int64(0), VertexId::from_int64(1), 100));

        let edge = table
            .get_edge(VertexId::from_int64(0), VertexId::from_int64(1), 100)
            .unwrap();
        assert_eq!(edge.src_vid, VertexId::from_int64(0));
        assert_eq!(edge.dst_vid, VertexId::from_int64(1));
        assert_eq!(edge.properties.len(), 1);
    }

    #[test]
    fn test_delete() {
        let schema = create_test_schema();
        let mut table = EdgeTable::new(schema).unwrap();

        table
            .insert_edge(
                VertexId::from_int64(0),
                VertexId::from_int64(1),
                &[("weight".to_string(), Value::Double(1.5))],
                100,
            )
            .unwrap();

        assert!(table
            .delete_edge(VertexId::from_int64(0), VertexId::from_int64(1), 200)
            .unwrap());
        assert!(!table.has_edge(VertexId::from_int64(0), VertexId::from_int64(1), 300));
    }

    #[test]
    fn test_out_in_edges() {
        let schema = create_test_schema();
        let mut table = EdgeTable::new(schema).unwrap();

        table
            .insert_edge(VertexId::from_int64(0), VertexId::from_int64(1), &[], 100)
            .unwrap();
        table
            .insert_edge(VertexId::from_int64(0), VertexId::from_int64(2), &[], 100)
            .unwrap();
        table
            .insert_edge(VertexId::from_int64(1), VertexId::from_int64(0), &[], 100)
            .unwrap();

        assert_eq!(table.out_degree(VertexId::from_int64(0), 100), 2);
        assert_eq!(table.in_degree(VertexId::from_int64(0), 100), 1);
        assert_eq!(table.out_degree(VertexId::from_int64(1), 100), 1);
        assert_eq!(table.in_degree(VertexId::from_int64(1), 100), 1);

        let out_edges = table.out_edges(VertexId::from_int64(0), 100);
        assert_eq!(out_edges.len(), 2);

        let in_edges = table.in_edges(VertexId::from_int64(0), 100);
        assert_eq!(in_edges.len(), 1);
    }

    #[test]
    fn test_update_properties() {
        let schema = create_test_schema();
        let mut table = EdgeTable::new(schema).unwrap();

        table
            .insert_edge(
                VertexId::from_int64(0),
                VertexId::from_int64(1),
                &[("weight".to_string(), Value::Double(1.0))],
                100,
            )
            .unwrap();

        table
            .update_properties(
                VertexId::from_int64(0),
                VertexId::from_int64(1),
                &[("weight".to_string(), Value::Double(2.0))],
                100,
            )
            .unwrap();

        let edge = table
            .get_edge(VertexId::from_int64(0), VertexId::from_int64(1), 100)
            .unwrap();
        assert_eq!(edge.properties.len(), 1);
    }

    #[test]
    fn test_flush_load_roundtrip() {
        use std::fs;

        let schema = create_test_schema();
        let mut table = EdgeTable::new(schema).unwrap();

        let ts = 100u32;
        let _edge_id_1 = table
            .insert_edge(
                VertexId::from_int64(1),
                VertexId::from_int64(2),
                &[("weight".to_string(), Value::Double(1.5))],
                ts,
            )
            .unwrap();

        let _edge_id_2 = table
            .insert_edge(
                VertexId::from_int64(1),
                VertexId::from_int64(3),
                &[("weight".to_string(), Value::Double(2.5))],
                ts,
            )
            .unwrap();

        let _edge_id_3 = table
            .insert_edge(
                VertexId::from_int64(2),
                VertexId::from_int64(3),
                &[("weight".to_string(), Value::Double(3.5))],
                ts,
            )
            .unwrap();

        let temp_dir = std::env::temp_dir().join("edge_table_test_flush_load");
        let _ = fs::remove_dir_all(&temp_dir);

        table.flush(&temp_dir).expect("flush should succeed");

        let mut loaded_table = EdgeTable::new(create_test_schema()).unwrap();
        loaded_table.load(&temp_dir).expect("load should succeed");

        assert_eq!(
            loaded_table.out_degree(VertexId::from_int64(1), ts),
            2,
            "scan should work after load"
        );
        assert_eq!(
            loaded_table.out_degree(VertexId::from_int64(2), ts),
            1,
            "scan should work after load"
        );

        assert!(
            loaded_table.has_edge(VertexId::from_int64(1), VertexId::from_int64(2), ts),
            "get_edge should work after load"
        );

        let deleted = loaded_table
            .delete_edge(VertexId::from_int64(1), VertexId::from_int64(3), ts + 1)
            .expect("delete_edge should work after load");
        assert!(deleted, "delete_edge should find the edge");

        assert!(
            !loaded_table.has_edge(VertexId::from_int64(1), VertexId::from_int64(3), ts + 1),
            "deleted edge should not be visible"
        );

        let _ = fs::remove_dir_all(&temp_dir);
    }
}

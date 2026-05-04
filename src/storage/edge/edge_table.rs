//! Edge Table
//!
//! Combines out/in CSRs and property storage for edge management.

use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use super::{EdgeId, EdgeRecord, EdgeSchema, EdgeStrategy, LabelId, MutableCsr, PropertyTable, Timestamp, VertexId};
use crate::core::{DataType, StorageError, StorageResult, Value};

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

#[derive(Debug)]
pub struct EdgeTable {
    label: LabelId,
    label_name: String,
    src_label: LabelId,
    dst_label: LabelId,
    schema: EdgeSchema,
    out_csr: MutableCsr,
    in_csr: MutableCsr,
    properties: PropertyTable,
    edge_id_counter: AtomicU64,
    config: EdgeTableConfig,
    is_open: bool,
}

impl EdgeTable {
    pub fn new(schema: EdgeSchema) -> Self {
        Self::with_config(schema, EdgeTableConfig::default())
    }

    pub fn with_config(schema: EdgeSchema, config: EdgeTableConfig) -> Self {
        let out_csr = MutableCsr::with_capacity(config.initial_vertex_capacity);
        let in_csr = MutableCsr::with_capacity(config.initial_vertex_capacity);

        let mut properties = PropertyTable::with_capacity(config.initial_edge_capacity);
        for prop in &schema.properties {
            properties.add_property(prop.name.clone(), prop.data_type.clone(), prop.nullable);
        }

        Self {
            label: schema.label_id,
            label_name: schema.label_name.clone(),
            src_label: schema.src_label,
            dst_label: schema.dst_label,
            schema,
            out_csr,
            in_csr,
            properties,
            edge_id_counter: AtomicU64::new(0),
            config,
            is_open: true,
        }
    }

    pub fn open<P: AsRef<Path>>(&mut self, _path: P) -> StorageResult<()> {
        self.is_open = true;
        Ok(())
    }

    pub fn close(&mut self) {
        self.is_open = false;
    }

    pub fn ensure_capacity(&mut self, vertex_capacity: usize, edge_capacity: usize) {
        self.out_csr.resize(vertex_capacity);
        self.in_csr.resize(vertex_capacity);
    }

    pub fn insert_edge(
        &mut self,
        src: VertexId,
        dst: VertexId,
        property_values: &[(String, Value)],
        ts: Timestamp,
    ) -> StorageResult<EdgeId> {
        if !self.is_open {
            return Err(StorageError::StorageNotOpen);
        }

        if self.schema.oe_strategy == EdgeStrategy::None {
            return Err(StorageError::InvalidOperation("Edge strategy is None".to_string()));
        }

        let edge_id = self.edge_id_counter.fetch_add(1, Ordering::Relaxed);

        let prop_offset = if !property_values.is_empty() {
            self.properties.insert(property_values)?
        } else {
            0
        };

        if self.schema.oe_strategy == EdgeStrategy::Single {
            if self.out_csr.has_edge(src, dst, ts) {
                self.properties.delete(prop_offset);
                return Err(StorageError::EdgeAlreadyExists(format!("{} -> {}", src, dst)));
            }
        }

        if !self.out_csr.insert_edge(src, dst, edge_id, prop_offset, ts) {
            self.properties.delete(prop_offset);
            return Err(StorageError::EdgeAlreadyExists(format!("{} -> {}", src, dst)));
        }

        if self.schema.ie_strategy != EdgeStrategy::None {
            self.in_csr.insert_edge(dst, src, edge_id, prop_offset, ts);
        }

        Ok(edge_id)
    }

    pub fn delete_edge(
        &mut self,
        src: VertexId,
        dst: VertexId,
        ts: Timestamp,
    ) -> StorageResult<bool> {
        if !self.is_open {
            return Err(StorageError::StorageNotOpen);
        }

        if let Some(nbr) = self.out_csr.get_edge(src, dst, ts) {
            let edge_id = nbr.edge_id;
            let prop_offset = nbr.prop_offset;

            self.out_csr.delete_edge(src, edge_id, ts);

            if self.schema.ie_strategy != EdgeStrategy::None {
                self.in_csr.delete_edge_by_dst(dst, src, ts);
            }

            if prop_offset > 0 {
                self.properties.delete(prop_offset);
            }

            return Ok(true);
        }

        Ok(false)
    }

    pub fn delete_edge_by_id(&mut self, edge_id: EdgeId, ts: Timestamp) -> StorageResult<bool> {
        if !self.is_open {
            return Err(StorageError::StorageNotOpen);
        }

        let mut found = false;
        for src in 0..self.out_csr.vertex_capacity() {
            let edges: Vec<_> = self.out_csr.edges_of(src as VertexId, ts)
                .into_iter()
                .filter(|nbr| nbr.edge_id == edge_id)
                .cloned()
                .collect();

            for nbr in edges {
                self.out_csr.delete_edge(src as VertexId, edge_id, ts);
                self.in_csr.delete_edge_by_dst(nbr.neighbor, src as VertexId, ts);

                if nbr.prop_offset > 0 {
                    self.properties.delete(nbr.prop_offset);
                }
                found = true;
            }
        }

        Ok(found)
    }

    pub fn get_edge(&self, src: VertexId, dst: VertexId, ts: Timestamp) -> Option<EdgeRecord> {
        if !self.is_open {
            return None;
        }

        let nbr = self.out_csr.get_edge(src, dst, ts)?;

        let properties = if nbr.prop_offset > 0 {
            self.properties.get(nbr.prop_offset)
                .map(|props| props.into_iter().filter_map(|(k, v)| v.map(|v| (k, v))).collect())
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

    pub fn get_edge_by_id(&self, edge_id: EdgeId, ts: Timestamp) -> Option<EdgeRecord> {
        if !self.is_open {
            return None;
        }

        for src in 0..self.out_csr.vertex_capacity() {
            if let Some(nbr) = self.out_csr.edges_of(src as VertexId, ts)
                .iter()
                .find(|nbr| nbr.edge_id == edge_id)
            {
                let properties = if nbr.prop_offset > 0 {
                    self.properties.get(nbr.prop_offset)
                        .map(|props| props.into_iter().filter_map(|(k, v)| v.map(|v| (k, v))).collect())
                        .unwrap_or_default()
                } else {
                    Vec::new()
                };

                return Some(EdgeRecord {
                    edge_id: nbr.edge_id,
                    src_vid: src as VertexId,
                    dst_vid: nbr.neighbor,
                    properties,
                });
            }
        }

        None
    }

    pub fn update_properties(
        &mut self,
        src: VertexId,
        dst: VertexId,
        values: &[(String, Value)],
        ts: Timestamp,
    ) -> StorageResult<bool> {
        if !self.is_open {
            return Err(StorageError::StorageNotOpen);
        }

        if let Some(nbr) = self.out_csr.get_edge(src, dst, ts) {
            self.properties.update(nbr.prop_offset, values)?;
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
                    self.properties.get(nbr.prop_offset)
                        .map(|props| props.into_iter().filter_map(|(k, v)| v.map(|v| (k, v))).collect())
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
                    self.properties.get(nbr.prop_offset)
                        .map(|props| props.into_iter().filter_map(|(k, v)| v.map(|v| (k, v))).collect())
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

        let mut edges = Vec::new();
        for src in 0..self.out_csr.vertex_capacity() {
            for nbr in self.out_csr.edges_of(src as VertexId, ts) {
                let properties = if nbr.prop_offset > 0 {
                    self.properties.get(nbr.prop_offset)
                        .map(|props| props.into_iter().filter_map(|(k, v)| v.map(|v| (k, v))).collect())
                        .unwrap_or_default()
                } else {
                    Vec::new()
                };

                edges.push(EdgeRecord {
                    edge_id: nbr.edge_id,
                    src_vid: src as VertexId,
                    dst_vid: nbr.neighbor,
                    properties,
                });
            }
        }
        edges
    }

    pub fn add_property(&mut self, name: String, data_type: DataType, nullable: bool) -> StorageResult<()> {
        if !self.is_open {
            return Err(StorageError::StorageNotOpen);
        }

        if self.properties.has_property(&name) {
            return Err(StorageError::ColumnAlreadyExists(name));
        }

        self.properties.add_property(name, data_type, nullable);
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
            return Err(StorageError::StorageNotOpen);
        }

        if let Some(nbr) = self.out_csr.get_edge(src, dst, ts) {
            self.properties.set_property(nbr.prop_offset, prop_name, Some(value.clone()))?;
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
            return Err(StorageError::StorageNotOpen);
        }

        let edge_id = self.out_csr.find_deleted_edge(src, dst);

        if let Some(eid) = edge_id {
            let reverted_out = self.out_csr.revert_delete(src, eid, ts);
            let reverted_in = if self.schema.ie_strategy != EdgeStrategy::None {
                self.in_csr.revert_delete(dst, eid, ts)
            } else {
                false
            };
            return Ok(reverted_out || reverted_in);
        }

        Ok(false)
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
        self.out_csr.edges_of(src, ts).into_iter().cloned().collect()
    }

    pub fn get_properties(&self, prop_offset: u32) -> Option<Vec<(String, Value)>> {
        self.properties.get(prop_offset)
            .map(|props| props.into_iter().filter_map(|(k, v)| v.map(|v| (k, v))).collect())
    }

    pub fn compact(&mut self) {
        self.out_csr.compact();
        self.in_csr.compact();
    }

    pub fn clear(&mut self) {
        self.out_csr.clear();
        self.in_csr.clear();
        self.properties.clear();
        self.edge_id_counter.store(0, Ordering::Relaxed);
    }

    pub fn flush<P: AsRef<Path>>(&self, path: P) -> StorageResult<()> {
        use std::fs::{self, File};
        use std::io::Write;

        let path = path.as_ref();
        fs::create_dir_all(path)?;

        let meta_path = path.join("meta.bin");
        let mut meta_file = File::create(&meta_path)?;

        meta_file.write_all(&self.label.to_le_bytes())?;
        meta_file.write_all(&self.src_label.to_le_bytes())?;
        meta_file.write_all(&self.dst_label.to_le_bytes())?;

        let label_name_bytes = self.label_name.as_bytes();
        meta_file.write_all(&(label_name_bytes.len() as u32).to_le_bytes())?;
        meta_file.write_all(label_name_bytes)?;

        let edge_id = self.edge_id_counter.load(Ordering::Relaxed);
        meta_file.write_all(&edge_id.to_le_bytes())?;

        let out_csr_path = path.join("out_csr.bin");
        self.flush_csr(&self.out_csr, &out_csr_path)?;

        let in_csr_path = path.join("in_csr.bin");
        self.flush_csr(&self.in_csr, &in_csr_path)?;

        let props_path = path.join("properties.bin");
        self.flush_properties(&props_path)?;

        Ok(())
    }

    fn flush_csr(&self, csr: &MutableCsr, path: &Path) -> StorageResult<()> {
        use std::fs::File;
        use std::io::Write;

        let mut file = File::create(path)?;

        let data = csr.dump();
        file.write_all(&(data.len() as u64).to_le_bytes())?;
        file.write_all(&data)?;

        Ok(())
    }

    fn flush_properties(&self, path: &Path) -> StorageResult<()> {
        use std::fs::File;
        use std::io::Write;

        let mut file = File::create(path)?;

        let data = self.properties.dump();
        file.write_all(&(data.len() as u64).to_le_bytes())?;
        file.write_all(&data)?;

        Ok(())
    }

    pub fn load<P: AsRef<Path>>(&mut self, path: P) -> StorageResult<()> {
        use std::fs::File;
        use std::io::Read;

        let path = path.as_ref();

        let meta_path = path.join("meta.bin");
        let mut meta_file = File::open(&meta_path)?;

        let mut label_bytes = [0u8; 2];
        meta_file.read_exact(&mut label_bytes)?;
        self.label = u16::from_le_bytes(label_bytes);

        let mut src_label_bytes = [0u8; 2];
        meta_file.read_exact(&mut src_label_bytes)?;
        self.src_label = u16::from_le_bytes(src_label_bytes);

        let mut dst_label_bytes = [0u8; 2];
        meta_file.read_exact(&mut dst_label_bytes)?;
        self.dst_label = u16::from_le_bytes(dst_label_bytes);

        let mut label_name_len_bytes = [0u8; 4];
        meta_file.read_exact(&mut label_name_len_bytes)?;
        let label_name_len = u32::from_le_bytes(label_name_len_bytes) as usize;

        let mut label_name_bytes = vec![0u8; label_name_len];
        meta_file.read_exact(&mut label_name_bytes)?;
        self.label_name = String::from_utf8(label_name_bytes)
            .map_err(|e| StorageError::DeserializeError(e.to_string()))?;

        let mut edge_id_bytes = [0u8; 8];
        meta_file.read_exact(&mut edge_id_bytes)?;
        self.edge_id_counter.store(u64::from_le_bytes(edge_id_bytes), Ordering::Relaxed);

        let out_csr_path = path.join("out_csr.bin");
        Self::load_csr_static(&mut self.out_csr, &out_csr_path)?;

        let in_csr_path = path.join("in_csr.bin");
        Self::load_csr_static(&mut self.in_csr, &in_csr_path)?;

        let props_path = path.join("properties.bin");
        self.load_properties(&props_path)?;

        self.is_open = true;
        Ok(())
    }

    fn load_csr_static(csr: &mut MutableCsr, path: &Path) -> StorageResult<()> {
        use std::fs::File;
        use std::io::Read;

        let mut file = File::open(path)?;

        let mut len_bytes = [0u8; 8];
        file.read_exact(&mut len_bytes)?;
        let len = u64::from_le_bytes(len_bytes) as usize;

        let mut data = vec![0u8; len];
        file.read_exact(&mut data)?;

        csr.load(&data);

        Ok(())
    }

    fn load_properties(&mut self, path: &Path) -> StorageResult<()> {
        use std::fs::File;
        use std::io::Read;

        let mut file = File::open(path)?;

        let mut len_bytes = [0u8; 8];
        file.read_exact(&mut len_bytes)?;
        let len = u64::from_le_bytes(len_bytes) as usize;

        let mut data = vec![0u8; len];
        file.read_exact(&mut data)?;

        self.properties.load(&data);

        Ok(())
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
            properties: vec![
                super::super::PropertyDef::new("weight".to_string(), DataType::Double),
            ],
            oe_strategy: EdgeStrategy::Multiple,
            ie_strategy: EdgeStrategy::Multiple,
        }
    }

    #[test]
    fn test_insert_and_get() {
        let schema = create_test_schema();
        let mut table = EdgeTable::new(schema);

        let edge_id = table.insert_edge(
            0,
            1,
            &[("weight".to_string(), Value::Double(1.5))],
            100,
        ).unwrap();

        assert!(table.has_edge(0, 1, 100));

        let edge = table.get_edge(0, 1, 100).unwrap();
        assert_eq!(edge.edge_id, edge_id);
        assert_eq!(edge.src_vid, 0);
        assert_eq!(edge.dst_vid, 1);
    }

    #[test]
    fn test_delete() {
        let schema = create_test_schema();
        let mut table = EdgeTable::new(schema);

        table.insert_edge(0, 1, &[("weight".to_string(), Value::Double(1.5))], 100).unwrap();

        assert!(table.delete_edge(0, 1, 200).unwrap());
        assert!(!table.has_edge(0, 1, 300));
    }

    #[test]
    fn test_out_in_edges() {
        let schema = create_test_schema();
        let mut table = EdgeTable::new(schema);

        table.insert_edge(0, 1, &[], 100).unwrap();
        table.insert_edge(0, 2, &[], 100).unwrap();
        table.insert_edge(1, 0, &[], 100).unwrap();

        assert_eq!(table.out_degree(0, 100), 2);
        assert_eq!(table.in_degree(0, 100), 1);
        assert_eq!(table.out_degree(1, 100), 1);
        assert_eq!(table.in_degree(1, 100), 1);

        let out_edges = table.out_edges(0, 100);
        assert_eq!(out_edges.len(), 2);

        let in_edges = table.in_edges(0, 100);
        assert_eq!(in_edges.len(), 1);
    }

    #[test]
    fn test_update_properties() {
        let schema = create_test_schema();
        let mut table = EdgeTable::new(schema);

        table.insert_edge(0, 1, &[("weight".to_string(), Value::Double(1.0))], 100).unwrap();

        table.update_properties(0, 1, &[("weight".to_string(), Value::Double(2.0))], 100).unwrap();

        let edge = table.get_edge(0, 1, 100).unwrap();
        assert_eq!(edge.properties.len(), 1);
    }
}

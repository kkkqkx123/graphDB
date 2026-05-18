//! Batch Operations for Vertex and Edge Tables
//!
//! Provides efficient batch read/write interfaces for bulk data operations.
//! Optimizes memory allocation and reduces function call overhead.

use crate::core::types::{Timestamp, VertexId};
use crate::core::Value;
use crate::storage::edge::{EdgeRecord, EdgeTable};
use crate::storage::vertex::{VertexRecord, VertexTable};

pub const DEFAULT_BATCH_SIZE: usize = 1024;

/// Type alias for vertex data with properties
pub type VertexData = (String, Vec<(String, Value)>);

/// Type alias for edge data with properties
pub type EdgeData = (VertexId, VertexId, Vec<(String, Value)>);

pub struct VertexBatchReader<'a> {
    table: &'a VertexTable,
    ts: Timestamp,
    current_idx: u32,
    end_idx: u32,
    batch_size: usize,
}

impl<'a> VertexBatchReader<'a> {
    pub fn new(table: &'a VertexTable, ts: Timestamp, batch_size: usize) -> Self {
        let total = table.total_count() as u32;
        Self {
            table,
            ts,
            current_idx: 0,
            end_idx: total,
            batch_size: if batch_size > 0 {
                batch_size
            } else {
                DEFAULT_BATCH_SIZE
            },
        }
    }

    pub fn from_range(
        table: &'a VertexTable,
        ts: Timestamp,
        start: u32,
        end: u32,
        batch_size: usize,
    ) -> Self {
        Self {
            table,
            ts,
            current_idx: start,
            end_idx: end.min(table.total_count() as u32),
            batch_size: if batch_size > 0 {
                batch_size
            } else {
                DEFAULT_BATCH_SIZE
            },
        }
    }
}

impl<'a> Iterator for VertexBatchReader<'a> {
    type Item = Vec<VertexRecord>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_idx >= self.end_idx {
            return None;
        }

        let mut batch = Vec::with_capacity(self.batch_size);
        let limit = (self.current_idx as usize + self.batch_size).min(self.end_idx as usize);

        while (self.current_idx as usize) < limit {
            if let Some(record) = self.table.get_by_internal_id(self.current_idx, self.ts) {
                batch.push(record);
            }
            self.current_idx += 1;
        }

        if batch.is_empty() {
            None
        } else {
            Some(batch)
        }
    }
}

pub struct VertexBatchWriter<'a> {
    table: &'a mut VertexTable,
    buffer: Vec<(String, Vec<(String, Value)>)>,
    buffer_size: usize,
    ts: Timestamp,
}

impl<'a> VertexBatchWriter<'a> {
    pub fn new(table: &'a mut VertexTable, ts: Timestamp, buffer_size: usize) -> Self {
        Self {
            table,
            buffer: Vec::with_capacity(if buffer_size > 0 {
                buffer_size
            } else {
                DEFAULT_BATCH_SIZE
            }),
            buffer_size: if buffer_size > 0 {
                buffer_size
            } else {
                DEFAULT_BATCH_SIZE
            },
            ts,
        }
    }

    pub fn insert(&mut self, external_id: String, properties: Vec<(String, Value)>) {
        self.buffer.push((external_id, properties));

        if self.buffer.len() >= self.buffer_size {
            self.flush();
        }
    }

    pub fn flush(&mut self) {
        if self.buffer.is_empty() {
            return;
        }

        self.table
            .ensure_capacity(self.table.total_count() + self.buffer.len());

        for (id, props) in self.buffer.drain(..) {
            let _ = self.table.insert(&id, &props, self.ts);
        }
    }

    pub fn remaining(&self) -> usize {
        self.buffer.len()
    }
}

impl<'a> Drop for VertexBatchWriter<'a> {
    fn drop(&mut self) {
        self.flush();
    }
}

pub struct EdgeBatchReader<'a> {
    table: &'a EdgeTable,
    ts: Timestamp,
    current_src: VertexId,
    vertex_capacity: usize,
    batch_size: usize,
}

impl<'a> EdgeBatchReader<'a> {
    pub fn new(table: &'a EdgeTable, ts: Timestamp, batch_size: usize) -> Self {
        let vertex_capacity = table.vertex_capacity();
        Self {
            table,
            ts,
            current_src: VertexId::from_int64(0),
            vertex_capacity,
            batch_size: if batch_size > 0 {
                batch_size
            } else {
                DEFAULT_BATCH_SIZE
            },
        }
    }

    pub fn from_src_vertex(
        table: &'a EdgeTable,
        ts: Timestamp,
        start_src: VertexId,
        batch_size: usize,
    ) -> Self {
        let vertex_capacity = table.vertex_capacity();
        Self {
            table,
            ts,
            current_src: start_src,
            vertex_capacity,
            batch_size: if batch_size > 0 {
                batch_size
            } else {
                DEFAULT_BATCH_SIZE
            },
        }
    }
}

impl<'a> Iterator for EdgeBatchReader<'a> {
    type Item = Vec<EdgeRecord>;

    fn next(&mut self) -> Option<Self::Item> {
        let capacity_vid = VertexId::from_int64(self.vertex_capacity as i64);
        if self.current_src >= capacity_vid {
            return None;
        }

        let mut batch = Vec::with_capacity(self.batch_size);
        let mut collected = 0;

        while self.current_src < capacity_vid && collected < self.batch_size {
            let edges = self.table.out_edges(self.current_src, self.ts);
            if !edges.is_empty() {
                let remaining = self.batch_size - collected;
                let to_take = edges.len().min(remaining);
                batch.extend(edges.into_iter().take(to_take));
                collected += to_take;
            }
            let current = self.current_src.as_int64().unwrap_or(0);
            self.current_src = VertexId::from_int64(current + 1);
        }

        if batch.is_empty() {
            None
        } else {
            Some(batch)
        }
    }
}

pub struct EdgeBatchWriter<'a> {
    table: &'a mut EdgeTable,
    buffer: Vec<EdgeData>,
    buffer_size: usize,
    ts: Timestamp,
}

impl<'a> EdgeBatchWriter<'a> {
    pub fn new(table: &'a mut EdgeTable, ts: Timestamp, buffer_size: usize) -> Self {
        Self {
            table,
            buffer: Vec::with_capacity(if buffer_size > 0 {
                buffer_size
            } else {
                DEFAULT_BATCH_SIZE
            }),
            buffer_size: if buffer_size > 0 {
                buffer_size
            } else {
                DEFAULT_BATCH_SIZE
            },
            ts,
        }
    }

    pub fn insert_edge(&mut self, src: VertexId, dst: VertexId, properties: Vec<(String, Value)>) {
        self.buffer.push((src, dst, properties));

        if self.buffer.len() >= self.buffer_size {
            self.flush();
        }
    }

    pub fn flush(&mut self) {
        if self.buffer.is_empty() {
            return;
        }

        for (src, dst, props) in self.buffer.drain(..) {
            let _ = self.table.insert_edge(src, dst, &props, self.ts);
        }
    }

    pub fn remaining(&self) -> usize {
        self.buffer.len()
    }
}

impl<'a> Drop for EdgeBatchWriter<'a> {
    fn drop(&mut self) {
        self.flush();
    }
}

pub struct BatchImportStats {
    pub total_records: usize,
    pub successful_records: usize,
    pub failed_records: usize,
    pub batches_flushed: usize,
    pub duration_ms: u64,
}

impl BatchImportStats {
    pub fn success_rate(&self) -> f64 {
        if self.total_records > 0 {
            self.successful_records as f64 / self.total_records as f64
        } else {
            0.0
        }
    }
}

pub fn batch_import_vertices(
    table: &mut VertexTable,
    vertices: &[VertexData],
    ts: Timestamp,
    batch_size: usize,
) -> BatchImportStats {
    let start = std::time::Instant::now();
    let mut stats = BatchImportStats {
        total_records: vertices.len(),
        successful_records: 0,
        failed_records: 0,
        batches_flushed: 0,
        duration_ms: 0,
    };

    let mut writer = VertexBatchWriter::new(table, ts, batch_size);

    for (id, props) in vertices {
        writer.insert(id.clone(), props.clone());
        stats.successful_records += 1;
    }

    writer.flush();
    stats.batches_flushed = vertices.len().div_ceil(batch_size);

    stats.duration_ms = start.elapsed().as_millis() as u64;
    stats
}

pub fn batch_import_edges(
    table: &mut EdgeTable,
    edges: &[EdgeData],
    ts: Timestamp,
    batch_size: usize,
) -> BatchImportStats {
    let start = std::time::Instant::now();
    let mut stats = BatchImportStats {
        total_records: edges.len(),
        successful_records: 0,
        failed_records: 0,
        batches_flushed: 0,
        duration_ms: 0,
    };

    let mut writer = EdgeBatchWriter::new(table, ts, batch_size);

    for (src, dst, props) in edges {
        writer.insert_edge(*src, *dst, props.clone());
        stats.successful_records += 1;
    }

    writer.flush();
    stats.batches_flushed = edges.len().div_ceil(batch_size);

    stats.duration_ms = start.elapsed().as_millis() as u64;
    stats
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::DataType;
    use crate::storage::edge::{EdgeSchema, EdgeStrategy};
    use crate::storage::storage_types::StoragePropertyDef;
    use crate::storage::vertex::VertexSchema;

    fn create_test_vertex_schema() -> VertexSchema {
        VertexSchema {
            label_id: 0,
            label_name: "person".to_string(),
            properties: vec![
                StoragePropertyDef::new("name".to_string(), DataType::String),
                StoragePropertyDef::new("age".to_string(), DataType::Int).nullable(true),
            ],
            primary_key_index: 0,
        }
    }

    fn create_test_edge_schema() -> EdgeSchema {
        EdgeSchema {
            label_id: 0,
            label_name: "knows".to_string(),
            src_label: 0,
            dst_label: 0,
            properties: vec![StoragePropertyDef::new("weight".to_string(), DataType::Double)],
            oe_strategy: EdgeStrategy::Multiple,
            ie_strategy: EdgeStrategy::Multiple,
        }
    }

    #[test]
    fn test_vertex_batch_reader() {
        let schema = create_test_vertex_schema();
        let mut table = VertexTable::new(0, "person".to_string(), schema);

        for i in 0..100 {
            table
                .insert(
                    &format!("v{}", i),
                    &[
                        ("name".to_string(), Value::String(format!("Person{}", i))),
                        ("age".to_string(), Value::Int(i)),
                    ],
                    100,
                )
                .unwrap();
        }

        let reader = VertexBatchReader::new(&table, 100, 20);
        let mut total_read = 0;

        for batch in reader {
            total_read += batch.len();
            assert!(batch.len() <= 20);
        }

        assert_eq!(total_read, 100);
    }

    #[test]
    fn test_vertex_batch_writer() {
        let schema = create_test_vertex_schema();
        let mut table = VertexTable::new(0, "person".to_string(), schema);

        {
            let mut writer = VertexBatchWriter::new(&mut table, 100, 10);

            for i in 0..25 {
                writer.insert(
                    format!("v{}", i),
                    vec![
                        ("name".to_string(), Value::String(format!("Person{}", i))),
                        ("age".to_string(), Value::Int(i)),
                    ],
                );
            }
        }

        assert_eq!(table.total_count(), 25);
    }

    #[test]
    fn test_edge_batch_reader() {
        let schema = create_test_edge_schema();
        let mut table = EdgeTable::new(schema).unwrap();

        for i in 0..50u64 {
            let src = i * 2;
            let dst = i * 2 + 1;
            let result = table.insert_edge(
                VertexId::from_u64(src),
                VertexId::from_u64(dst),
                &[("weight".to_string(), Value::Double(i as f64 * 0.1))],
                100,
            );
            assert!(
                result.is_ok(),
                "Failed to insert edge {}->{}: {:?}",
                src,
                dst,
                result
            );
        }

        let reader = EdgeBatchReader::new(&table, 100, 20);
        let mut total_read = 0;

        for batch in reader {
            total_read += batch.len();
            assert!(batch.len() <= 20);
        }

        assert_eq!(total_read, 50);
    }

    #[test]
    fn test_edge_batch_writer() {
        let schema = create_test_edge_schema();
        let mut table = EdgeTable::new(schema).unwrap();

        {
            let mut writer = EdgeBatchWriter::new(&mut table, 100, 10);

            for i in 0..25u64 {
                let src = i * 2 + 1000;
                let dst = i * 2 + 1 + 1000;
                writer.insert_edge(
                    VertexId::from_u64(src),
                    VertexId::from_u64(dst),
                    vec![("weight".to_string(), Value::Double(i as f64 * 0.1))],
                );
            }
        }

        assert_eq!(table.edge_count(), 25);
    }

    #[test]
    fn test_batch_import_vertices() {
        let schema = create_test_vertex_schema();
        let mut table = VertexTable::new(0, "person".to_string(), schema);

        let vertices: Vec<_> = (0..100)
            .map(|i| {
                (
                    format!("v{}", i),
                    vec![
                        ("name".to_string(), Value::String(format!("Person{}", i))),
                        ("age".to_string(), Value::Int(i)),
                    ],
                )
            })
            .collect();

        let stats = batch_import_vertices(&mut table, &vertices, 100, 20);

        assert_eq!(stats.total_records, 100);
        assert_eq!(stats.successful_records, 100);
        assert_eq!(stats.failed_records, 0);
    }

    #[test]
    fn test_batch_import_edges() {
        let schema = create_test_edge_schema();
        let mut table = EdgeTable::new(schema).unwrap();

        let edges: Vec<_> = (0..50)
            .map(|i| {
                (
                    VertexId::from_int64(0),
                    VertexId::from_int64(i as i64 + 1),
                    vec![("weight".to_string(), Value::Double(i as f64 * 0.1))],
                )
            })
            .collect();

        let stats = batch_import_edges(&mut table, &edges, 100, 10);

        assert_eq!(stats.total_records, 50);
        assert_eq!(stats.successful_records, 50);
        assert_eq!(stats.failed_records, 0);
    }
}

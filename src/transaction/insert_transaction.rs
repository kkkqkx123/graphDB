//! Insert Transaction
//!
//! Provides insert-only transaction for MVCC-based graph database.
//! An insert transaction can only add new vertices and edges, not modify
//! or delete existing data. This allows for higher concurrency compared
//! to update transactions.

use std::collections::HashMap;
use std::sync::Arc;

use oxicode::{encode_to_vec, decode_from_slice};

use super::read_transaction::INVALID_TIMESTAMP;
use super::version_manager::{VersionManager, VersionManagerError};
use super::wal::types::{
    CreateEdgeTypeRedo, CreateVertexTypeRedo, EdgeId, InsertEdgeRedo, InsertVertexRedo,
    LabelId, Timestamp, VertexId, WalHeader, WalOpType,
};
use super::wal::writer::WalWriter;

/// Insert transaction error
#[derive(Debug, Clone, thiserror::Error)]
pub enum InsertTransactionError {
    #[error("Version manager error: {0}")]
    VersionManagerError(#[from] VersionManagerError),

    #[error("WAL error: {0}")]
    WalError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Transaction already released")]
    AlreadyReleased,

    #[error("Label not found: {0}")]
    LabelNotFound(LabelId),

    #[error("Vertex already exists: {0}")]
    VertexAlreadyExists(VertexId),

    #[error("Vertex not found: {0}")]
    VertexNotFound(VertexId),

    #[error("Property type mismatch: expected {expected}, got {actual}")]
    PropertyTypeMismatch { expected: String, actual: String },

    #[error("Property count mismatch: expected {expected}, got {actual}")]
    PropertyCountMismatch { expected: usize, actual: usize },

    #[error("Schema error: {0}")]
    SchemaError(String),
}

/// Insert transaction result type
pub type InsertTransactionResult<T> = Result<T, InsertTransactionError>;

/// Insert Transaction
///
/// A transaction that can only insert new data (vertices and edges).
/// Insert transactions can run concurrently with each other and with
/// read transactions, but not with update transactions.
///
/// # Example
///
/// ```rust,ignore
/// let mut txn = InsertTransaction::new(&mut graph, &version_manager, &mut wal_writer)?;
/// txn.add_vertex(label, id, properties)?;
/// txn.commit()?;
/// ```
pub struct InsertTransaction<'a> {
    graph: &'a mut dyn InsertTarget,
    version_manager: &'a VersionManager,
    wal_writer: &'a mut dyn WalWriter,
    timestamp: Timestamp,
    wal_buffer: Vec<u8>,
    added_vertices: HashMap<LabelId, VertexId>,
    vertex_nums: HashMap<LabelId, u64>,
}

/// Target for insert operations (will be PropertyGraph in phase 2)
pub trait InsertTarget: Send + Sync {
    fn add_vertex(
        &mut self,
        label: LabelId,
        oid: &[u8],
        properties: &[(String, Vec<u8>)],
        ts: Timestamp,
    ) -> InsertTransactionResult<VertexId>;

    fn add_edge(
        &mut self,
        src_label: LabelId,
        src_vid: VertexId,
        dst_label: LabelId,
        dst_vid: VertexId,
        edge_label: LabelId,
        properties: &[(String, Vec<u8>)],
        ts: Timestamp,
    ) -> InsertTransactionResult<EdgeId>;

    fn get_vertex_id(
        &self,
        label: LabelId,
        oid: &[u8],
        ts: Timestamp,
    ) -> Option<VertexId>;

    fn get_vertex_oid(
        &self,
        label: LabelId,
        vid: VertexId,
        ts: Timestamp,
    ) -> Option<Vec<u8>>;

    fn get_vertex_property_types(&self, label: LabelId) -> Vec<String>;
    fn get_edge_property_types(&self, src_label: LabelId, dst_label: LabelId, edge_label: LabelId) -> Vec<String>;
    fn vertex_label_num(&self) -> usize;
    fn lid_num(&self, label: LabelId) -> usize;
}

impl<'a> InsertTransaction<'a> {
    /// Create a new insert transaction
    ///
    /// Acquires an insert timestamp from the version manager.
    pub fn new(
        graph: &'a mut dyn InsertTarget,
        version_manager: &'a VersionManager,
        wal_writer: &'a mut dyn WalWriter,
    ) -> InsertTransactionResult<Self> {
        let timestamp = version_manager.acquire_insert_timestamp();
        let mut wal_buffer = Vec::new();
        wal_buffer.resize(WalHeader::SIZE, 0);

        Ok(Self {
            graph,
            version_manager,
            wal_writer,
            timestamp,
            wal_buffer,
            added_vertices: HashMap::new(),
            vertex_nums: HashMap::new(),
        })
    }

    /// Get the transaction's timestamp
    pub fn timestamp(&self) -> Timestamp {
        self.timestamp
    }

    /// Get vertex index by label and object ID
    pub fn get_vertex_index(&self, label: LabelId, oid: &[u8]) -> Option<VertexId> {
        if let Some(vid) = self.graph.get_vertex_id(label, oid, self.timestamp) {
            return Some(vid);
        }

        if let Some(&base) = self.added_vertices.get(&label) {
            let added = self.vertex_nums.get(&label).copied().unwrap_or(0);
            if added > 0 {
                return Some(base + added);
            }
        }

        None
    }

    /// Get vertex object ID by label and internal ID
    pub fn get_vertex_id(&self, label: LabelId, vid: VertexId) -> Option<Vec<u8>> {
        if let Some(&base) = self.added_vertices.get(&label) {
            if vid >= base {
                return None;
            }
        }
        self.graph.get_vertex_oid(label, vid, self.timestamp)
    }

    /// Add a new vertex
    ///
    /// # Arguments
    /// * `label` - Vertex label ID
    /// * `oid` - Object ID (external ID)
    /// * `properties` - Vertex properties as (name, value) pairs
    ///
    /// # Returns
    /// The internal vertex ID if successful
    pub fn add_vertex(
        &mut self,
        label: LabelId,
        oid: &[u8],
        properties: &[(String, Vec<u8>)],
    ) -> InsertTransactionResult<VertexId> {
        let expected_types = self.graph.get_vertex_property_types(label);
        if expected_types.len() != properties.len() {
            return Err(InsertTransactionError::PropertyCountMismatch {
                expected: expected_types.len(),
                actual: properties.len(),
            });
        }

        if self.get_vertex_index(label, oid).is_some() {
            return Err(InsertTransactionError::VertexAlreadyExists(0));
        }

        let base = self.added_vertices.entry(label).or_insert_with(|| {
            self.graph.lid_num(label) as VertexId
        });
        let num = self.vertex_nums.entry(label).or_insert(0u64);
        let vid = *base + *num;
        *num += 1;

        let redo = InsertVertexRedo {
            label,
            oid: oid.to_vec(),
            properties: properties.to_vec(),
        };
        self.serialize_redo(WalOpType::InsertVertex, &redo)?;

        Ok(vid)
    }

    /// Add a new edge
    ///
    /// # Arguments
    /// * `src_label` - Source vertex label ID
    /// * `src_vid` - Source vertex internal ID
    /// * `dst_label` - Destination vertex label ID
    /// * `dst_vid` - Destination vertex internal ID
    /// * `edge_label` - Edge label ID
    /// * `properties` - Edge properties as (name, value) pairs
    pub fn add_edge(
        &mut self,
        src_label: LabelId,
        src_vid: VertexId,
        dst_label: LabelId,
        dst_vid: VertexId,
        edge_label: LabelId,
        properties: &[(String, Vec<u8>)],
    ) -> InsertTransactionResult<()> {
        let expected_types = self.graph.get_edge_property_types(src_label, dst_label, edge_label);
        if expected_types.len() != properties.len() {
            return Err(InsertTransactionError::PropertyCountMismatch {
                expected: expected_types.len(),
                actual: properties.len(),
            });
        }

        let src_oid = self.graph.get_vertex_oid(src_label, src_vid, self.timestamp)
            .ok_or(InsertTransactionError::VertexNotFound(src_vid))?;
        let dst_oid = self.graph.get_vertex_oid(dst_label, dst_vid, self.timestamp)
            .ok_or(InsertTransactionError::VertexNotFound(dst_vid))?;

        let redo = InsertEdgeRedo {
            src_label,
            src_oid,
            dst_label,
            dst_oid,
            edge_label,
            properties: properties.to_vec(),
        };
        self.serialize_redo(WalOpType::InsertEdge, &redo)?;

        Ok(())
    }

    /// Commit the insert transaction
    ///
    /// Writes the WAL and releases the timestamp.
    pub fn commit(mut self) -> InsertTransactionResult<()> {
        if self.timestamp == INVALID_TIMESTAMP {
            return Ok(());
        }

        if self.wal_buffer.len() == WalHeader::SIZE {
            self.version_manager.release_insert_timestamp(self.timestamp);
            self.clear();
            return Ok(());
        }

        self.write_wal_header();

        self.wal_writer
            .append(&self.wal_buffer)
            .map_err(|e| InsertTransactionError::WalError(e.to_string()))?;

        self.ingest_wal()?;

        self.version_manager.release_insert_timestamp(self.timestamp);
        self.clear();

        Ok(())
    }

    /// Abort the insert transaction
    ///
    /// Simply releases the timestamp without writing WAL.
    pub fn abort(mut self) -> InsertTransactionResult<()> {
        if self.timestamp != INVALID_TIMESTAMP {
            self.version_manager.release_insert_timestamp(self.timestamp);
            self.clear();
        }
        Ok(())
    }

    /// Serialize a redo log entry
    fn serialize_redo<T: serde::Serialize + oxicode::Encode>(&mut self, op_type: WalOpType, redo: &T) -> InsertTransactionResult<()> {
        let op_byte = op_type as u8;
        self.wal_buffer.push(op_byte);

        let encoded = encode_to_vec(redo)
            .map_err(|e| InsertTransactionError::SerializationError(e.to_string()))?;

        let len = encoded.len() as u32;
        self.wal_buffer.extend_from_slice(&len.to_le_bytes());
        self.wal_buffer.extend_from_slice(&encoded);

        Ok(())
    }

    /// Write the WAL header
    fn write_wal_header(&mut self) {
        let header = WalHeader::new(WalOpType::InsertVertex, self.timestamp, 0);
        let header_bytes = header.as_bytes();
        self.wal_buffer[..WalHeader::SIZE].copy_from_slice(header_bytes);
    }

    /// Ingest WAL entries into the graph
    fn ingest_wal(&mut self) -> InsertTransactionResult<()> {
        let data = &self.wal_buffer[WalHeader::SIZE..];
        let mut offset = 0;

        while offset < data.len() {
            let op_type = WalOpType::try_from(data[offset])
                .map_err(|e| InsertTransactionError::WalError(e.to_string()))?;
            offset += 1;

            let len = u32::from_le_bytes([data[offset], data[offset + 1], data[offset + 2], data[offset + 3]]) as usize;
            offset += 4;

            let payload = &data[offset..offset + len];
            offset += len;

            match op_type {
                WalOpType::InsertVertex => {
                    let redo: InsertVertexRedo = decode_from_slice(payload)
                        .map_err(|e| InsertTransactionError::SerializationError(e.to_string()))?
                        .0;
                    self.graph.add_vertex(redo.label, &redo.oid, &redo.properties, self.timestamp)?;
                }
                WalOpType::InsertEdge => {
                    let redo: InsertEdgeRedo = decode_from_slice(payload)
                        .map_err(|e| InsertTransactionError::SerializationError(e.to_string()))?
                        .0;
                    let src_vid = self.graph.get_vertex_id(redo.src_label, &redo.src_oid, self.timestamp)
                        .ok_or(InsertTransactionError::VertexNotFound(0))?;
                    let dst_vid = self.graph.get_vertex_id(redo.dst_label, &redo.dst_oid, self.timestamp)
                        .ok_or(InsertTransactionError::VertexNotFound(0))?;
                    self.graph.add_edge(
                        redo.src_label, src_vid,
                        redo.dst_label, dst_vid,
                        redo.edge_label, &redo.properties, self.timestamp
                    )?;
                }
                _ => {
                    return Err(InsertTransactionError::WalError(format!(
                        "Unexpected op type: {:?}",
                        op_type
                    )));
                }
            }
        }

        Ok(())
    }

    /// Clear internal state
    fn clear(&mut self) {
        self.wal_buffer.clear();
        self.wal_buffer.resize(WalHeader::SIZE, 0);
        self.added_vertices.clear();
        self.vertex_nums.clear();
        self.timestamp = INVALID_TIMESTAMP;
    }
}

impl<'a> Drop for InsertTransaction<'a> {
    fn drop(&mut self) {
        if self.timestamp != INVALID_TIMESTAMP {
            self.version_manager.release_insert_timestamp(self.timestamp);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::wal::writer::DummyWalWriter;

    struct MockInsertTarget;

    impl InsertTarget for MockInsertTarget {
        fn add_vertex(
            &mut self,
            _label: LabelId,
            _oid: &[u8],
            _properties: &[(String, Vec<u8>)],
            _ts: Timestamp,
        ) -> InsertTransactionResult<VertexId> {
            Ok(1)
        }

        fn add_edge(
            &mut self,
            _src_label: LabelId,
            _src_vid: VertexId,
            _dst_label: LabelId,
            _dst_vid: VertexId,
            _edge_label: LabelId,
            _properties: &[(String, Vec<u8>)],
            _ts: Timestamp,
        ) -> InsertTransactionResult<EdgeId> {
            Ok(1)
        }

        fn get_vertex_id(&self, _label: LabelId, _oid: &[u8], _ts: Timestamp) -> Option<VertexId> {
            None
        }

        fn get_vertex_oid(&self, _label: LabelId, _vid: VertexId, _ts: Timestamp) -> Option<Vec<u8>> {
            Some(vec![])
        }

        fn get_vertex_property_types(&self, _label: LabelId) -> Vec<String> {
            vec![]
        }

        fn get_edge_property_types(&self, _src_label: LabelId, _dst_label: LabelId, _edge_label: LabelId) -> Vec<String> {
            vec![]
        }

        fn vertex_label_num(&self) -> usize {
            0
        }

        fn lid_num(&self, _label: LabelId) -> usize {
            0
        }
    }

    #[test]
    fn test_insert_transaction_basic() {
        let vm = VersionManager::new();
        let mut target = MockInsertTarget;
        let mut wal = DummyWalWriter::new();

        let txn = InsertTransaction::new(&mut target, &vm, &mut wal)
            .expect("Failed to create insert transaction");

        assert!(txn.timestamp() >= 1);
    }

    #[test]
    fn test_insert_transaction_commit() {
        let vm = VersionManager::new();
        let mut target = MockInsertTarget;
        let mut wal = DummyWalWriter::new();

        let txn = InsertTransaction::new(&mut target, &vm, &mut wal)
            .expect("Failed to create insert transaction");

        txn.commit().expect("Commit failed");

        assert_eq!(vm.pending_count(), 0);
    }

    #[test]
    fn test_insert_transaction_abort() {
        let vm = VersionManager::new();
        let mut target = MockInsertTarget;
        let mut wal = DummyWalWriter::new();

        let txn = InsertTransaction::new(&mut target, &vm, &mut wal)
            .expect("Failed to create insert transaction");

        txn.abort().expect("Abort failed");

        assert_eq!(vm.pending_count(), 0);
    }
}

//! Core EdgeTable operations: CRUD, properties, queries, and compaction.
//!
//! Provides fundamental edge table functionality including insertion, deletion,
//! querying, property management, and basic maintenance operations.

use super::segment::{CsrSegment, DeletionInfo, SegmentVersion};
use super::mvcc::MVCCManager;
use super::stats::DeletionStats;
use super::super::{CsrVariant, EdgeSchema, EdgeStrategy, Nbr, EdgeRecord, CsrBase, MutableCsrTrait};
use crate::core::types::{EdgeId, CompactConfig, LabelId, VertexId, Timestamp};
use crate::core::{DataType, StorageError, StorageResult, Value};
use crate::storage::types::{PropertyId, StoragePropertyDef};
use crate::storage::edge::PropertyTable;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct EdgeTableConfig {
    pub initial_vertex_capacity: usize,
    pub initial_edge_capacity: usize,
    pub max_segments_per_direction: usize,
}

impl Default for EdgeTableConfig {
    fn default() -> Self {
        Self {
            initial_vertex_capacity: 4096,
            initial_edge_capacity: 4096,
            max_segments_per_direction: 100,
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

pub struct EdgeTableCore {
    pub label: LabelId,
    pub label_name: String,
    pub src_label: LabelId,
    pub dst_label: LabelId,
    pub schema: EdgeSchema,
    pub out_csr: CsrVariant,
    pub in_csr: CsrVariant,
    pub out_segments: Vec<CsrSegment>,
    pub in_segments: Vec<CsrSegment>,
    pub mvcc: MVCCManager,
    pub properties: PropertyTable,
    pub is_open: bool,
    pub next_edge_id: EdgeId,
    pub config: EdgeTableConfig,
    pub stats_manager: Option<std::sync::Arc<crate::core::stats::StatsManager>>,
}

impl EdgeTableCore {
    pub fn new(schema: EdgeSchema) -> StorageResult<Self> {
        Self::with_config(schema, EdgeTableConfig::default())
    }

    pub fn with_config(schema: EdgeSchema, config: EdgeTableConfig) -> StorageResult<Self> {
        schema.validate()?;

        let out_csr = CsrVariant::from_strategy(
            schema.oe_strategy,
            config.initial_vertex_capacity,
            config.initial_edge_capacity,
        )?;
        let in_csr = CsrVariant::from_strategy(
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
            mvcc: MVCCManager::new(),
            properties,
            is_open: true,
            next_edge_id: EdgeId(0),
            config,
            stats_manager: None,
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

    pub fn set_stats_manager(&mut self, stats: std::sync::Arc<crate::core::stats::StatsManager>) {
        self.stats_manager = Some(stats);
    }

    fn base_get_edge(
        &self,
        segments: &[CsrSegment],
        src: u32,
        dst: VertexId,
        ts: Timestamp,
    ) -> Option<Nbr> {
        for segment in segments.iter().rev() {
            if segment.create_ts_min > ts {
                continue;
            }

            if segment.deletion_info.all_deleted_before(ts) {
                continue;
            }

            let positioned_edges = segment.csr.edges_of_with_position(src);
            for (position, edge) in positioned_edges {
                if edge.neighbor == dst && edge.timestamp <= ts {
                    let edge_id = segment.recover_edge_id(edge, position);
                    if !self.mvcc.is_tombstoned(edge_id, ts) {
                        return Some(Nbr::new(
                            edge.neighbor,
                            edge_id,
                            edge.prop_offset,
                            edge.timestamp,
                        ));
                    }
                }
            }
        }

        None
    }

    fn base_edges_of(&self, segments: &[CsrSegment], src: u32, ts: Timestamp) -> Vec<Nbr> {
        let mut edges = Vec::new();

        for segment in segments.iter().rev() {
            if segment.create_ts_min > ts {
                continue;
            }

            if segment.deletion_info.all_deleted_before(ts) {
                continue;
            }

            for (position, edge) in segment.csr.edges_of_with_position(src) {
                if edge.timestamp <= ts {
                    let edge_id = segment.recover_edge_id(edge, position);
                    if !self.mvcc.is_tombstoned(edge_id, ts) {
                        edges.push(Nbr::new(
                            edge.neighbor,
                            edge_id,
                            edge.prop_offset,
                            edge.timestamp,
                        ));
                    }
                }
            }
        }

        edges
    }

    fn merged_edges_of(
        &self,
        delta: &CsrVariant,
        segments: &[CsrSegment],
        src: u32,
        ts: Timestamp,
    ) -> Vec<Nbr> {
        let mut seen = HashSet::new();
        let mut result = Vec::new();

        for nbr in delta.edges_of(src, ts) {
            if !self.mvcc.is_tombstoned(nbr.edge_id, ts) && seen.insert(nbr.edge_id) {
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
        delta: &CsrVariant,
        segments: &[CsrSegment],
        src: u32,
        dst: VertexId,
        ts: Timestamp,
    ) -> Option<Nbr> {
        if let Some(nbr) = delta.get_edge(src, dst, ts) {
            if !self.mvcc.is_tombstoned(nbr.edge_id, ts) {
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

    pub fn validate_segment_integrity(&self) -> usize {
        let mut valid_count = 0;

        for segment in &self.out_segments {
            if segment.version.validate(segment) {
                valid_count += 1;
            }
        }

        for segment in &self.in_segments {
            if segment.version.validate(segment) {
                valid_count += 1;
            }
        }

        valid_count
    }

    pub fn segment_versions(&self) -> Vec<(usize, u32, u32)> {
        let mut versions = Vec::new();

        for (idx, seg) in self.out_segments.iter().enumerate() {
            versions.push((idx, seg.version.version, seg.version.checksum));
        }

        for (idx, seg) in self.in_segments.iter().enumerate() {
            versions.push((idx + 1000, seg.version.version, seg.version.checksum));
        }

        versions
    }

    pub fn update_segment_checksums(&mut self) {
        for segment in &mut self.out_segments {
            segment.version.checksum = SegmentVersion::compute_checksum(segment);
            segment.version.increment();
        }

        for segment in &mut self.in_segments {
            segment.version.checksum = SegmentVersion::compute_checksum(segment);
            segment.version.increment();
        }
    }

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

        if self.schema.oe_strategy == super::super::EdgeStrategy::None {
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
            let edge_id = nbr.edge_id;
            self.mvcc.pending_segment_deletions.insert(edge_id, ts);
            self.mvcc.tombstones.insert(edge_id, ts);
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
                        .filter(|(_, edge)| !self.mvcc.is_tombstoned(edge.edge_id, u32::MAX))
                        .count() as u64
                })
                .sum::<u64>()
    }

    pub fn delta_edge_count(&self) -> u64 {
        self.out_csr.edge_count() + self.in_csr.edge_count()
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
                if nbr.prop_offset != ie_nbr.prop_offset {
                    return Err(StorageError::data_corruption(
                        format!(
                            "property offset mismatch: out_csr={}, in_csr={} at edge ({}, {})",
                            nbr.prop_offset, ie_nbr.prop_offset, params.src, params.dst
                        ),
                    ));
                }
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

    pub fn maybe_compact_for_flush(&mut self, ts: Timestamp, threshold: f32) {
        const RESERVE_RATIO: f32 = 0.25;
        if self.out_csr.fragmentation_ratio() > threshold {
            self.out_csr.compact_with_ts(ts, RESERVE_RATIO);
        }
        if self.in_csr.fragmentation_ratio() > threshold {
            self.in_csr.compact_with_ts(ts, RESERVE_RATIO);
        }
    }

    pub fn compact_csr_only(&mut self, ts: Timestamp, reserve_ratio: f32) -> usize {
        self.out_csr.compact_with_ts(ts, reserve_ratio)
            + self.in_csr.compact_with_ts(ts, reserve_ratio)
    }

    pub fn deletion_stats(&self) -> DeletionStats {
        let mut stats = DeletionStats::default();

        let mut total_edge_count = 0u64;
        let mut total_deleted_count = 0u64;

        for segment in self.out_segments.iter().chain(self.in_segments.iter()) {
            let edge_count = segment.csr.edge_count();
            total_edge_count += edge_count;

            match segment.deletion_info {
                DeletionInfo::NoDeletes => {}
                DeletionInfo::HasDeletes { min_ts, max_ts, deleted_count } => {
                    total_deleted_count += deleted_count as u64;
                    stats.segments_with_deletions += 1;

                    if (deleted_count as u64) == edge_count {
                        stats.completely_deleted_segments += 1;
                    }

                    if let Some(ref mut oldest) = stats.oldest_deletion_ts {
                        *oldest = (*oldest).min(min_ts);
                    } else {
                        stats.oldest_deletion_ts = Some(min_ts);
                    }

                    if let Some(ref mut newest) = stats.newest_deletion_ts {
                        *newest = (*newest).max(max_ts);
                    } else {
                        stats.newest_deletion_ts = Some(max_ts);
                    }
                }
            }
        }

        stats.total_frozen_edges = total_edge_count;
        stats.total_deleted_edges = total_deleted_count;

        stats
    }

    pub fn segments_total_bytes(&self) -> usize {
        self.out_segments.iter().map(|s| s.estimated_bytes()).sum::<usize>()
            + self.in_segments.iter().map(|s| s.estimated_bytes()).sum::<usize>()
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
                    && !self.mvcc.is_tombstoned(nbr.edge_id, ts)
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
                    && !self.mvcc.is_tombstoned(nbr.edge_id, ts)
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
        total += self.mvcc.tombstones.len() * std::mem::size_of::<(EdgeId, Timestamp)>();
        total += self.properties.used_memory_size();

        total
    }
}

pub struct EdgeTableScanIterator<'a> {
    _table: &'a EdgeTableCore,
    records: std::vec::IntoIter<EdgeRecord>,
}

impl<'a> EdgeTableScanIterator<'a> {
    pub fn new(table: &'a EdgeTableCore, ts: Timestamp) -> Self {
        let mut seen = HashSet::new();
        let mut records = Vec::new();

        for (src_vid, nbr) in table.out_csr.iter(ts) {
            if !table.mvcc.is_tombstoned(nbr.edge_id, ts) && seen.insert(nbr.edge_id) {
                records
                    .push(table.edge_record_from_nbr(src_vid.as_int64().unwrap_or(0) as u32, nbr));
            }
        }

        for segment in table.out_segments.iter().rev() {
            if segment.create_ts_min > ts {
                continue;
            }

            for (src_vid, edge) in segment.csr.iter() {
                if edge.timestamp <= ts
                    && !table.mvcc.is_tombstoned(edge.edge_id, ts)
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



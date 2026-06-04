use crate::core::types::{DataType, EdgeTypeInfo, LabelId, PropertyDef, SpaceInfo, TagInfo, Timestamp, VertexId};
use crate::core::wal::traits::RecoveryApplier;
use crate::core::{StorageError, StorageResult};
use crate::storage::edge::EdgeStrategy;
use crate::storage::engine::graph_storage::GraphStorageContext;
use crate::storage::engine::params::EdgeOperationParams;
use crate::storage::engine::transaction::{AddEdgeParams, DeleteEdgeParams, TransactionOps};
use crate::storage::types::StoragePropertyDef;
use crate::transaction::codec::bytes_to_value;
use crate::transaction::wal::{
    AddEdgePropRedo, AddVertexPropRedo, CreateEdgeTypeRedo, CreateVertexTypeRedo,
    DeleteEdgePropRedo, DeleteEdgeRedo, DeleteEdgeTypeRedo, DeleteVertexPropRedo,
    DeleteVertexTypeRedo, InsertEdgeRedo, RenameEdgePropRedo, RenameVertexPropRedo,
    UpdateEdgePropRedo,
};

impl RecoveryApplier for GraphStorageContext {
    // ========================================================================
    // Data Operations
    // ========================================================================

    fn replay_insert_vertex(
        &self,
        label: LabelId,
        vid: VertexId,
        properties: &[(String, Vec<u8>)],
        ts: Timestamp,
    ) -> StorageResult<()> {
        {
            let mut vertex_tables = self.data_store().vertex_tables().write();
            TransactionOps::add_vertex(&mut vertex_tables, label, vid, properties, ts)?;
        }
        self.mark_vertex_modified(label);
        Ok(())
    }

    fn replay_insert_edge(&self, redo: &InsertEdgeRedo, ts: Timestamp) -> StorageResult<()> {
        let (src_internal, dst_internal) = {
            let vertex_tables = self.data_store().vertex_tables().read();
            let src_table = vertex_tables.get(&redo.src_label).ok_or_else(|| {
                StorageError::db_error(format!(
                    "Source vertex label not found during recovery: label={}",
                    redo.src_label
                ))
            })?;
            let dst_table = vertex_tables.get(&redo.dst_label).ok_or_else(|| {
                StorageError::db_error(format!(
                    "Destination vertex label not found during recovery: label={}",
                    redo.dst_label
                ))
            })?;

            let src_internal = TransactionOps::resolve_vertex_id(src_table, redo.src_vid, ts)
                .ok_or_else(|| {
                    StorageError::db_error(format!(
                        "Source vertex not found during recovery: label={}, vid={:?}",
                        redo.src_label, redo.src_vid
                    ))
                })?;
            let dst_internal = TransactionOps::resolve_vertex_id(dst_table, redo.dst_vid, ts)
                .ok_or_else(|| {
                    StorageError::db_error(format!(
                        "Destination vertex not found during recovery: label={}, vid={:?}",
                        redo.dst_label, redo.dst_vid
                    ))
                })?;
            (src_internal, dst_internal)
        };

        let params = AddEdgeParams {
            src_label: redo.src_label,
            src_vid: VertexId::from_u64(src_internal as u64),
            dst_label: redo.dst_label,
            dst_vid: VertexId::from_u64(dst_internal as u64),
            edge_label: redo.edge_label,
            rank: redo.rank,
        };

        {
            let vertex_tables = self.data_store().vertex_tables().read();
            let mut edge_tables = self.data_store().edge_tables().write();
            TransactionOps::add_edge(
                &mut edge_tables,
                &vertex_tables,
                params,
                &redo.properties,
                ts,
            )
            .map_err(|e| StorageError::db_error(format!("Failed to replay insert edge: {}", e)))?;
        }

        self.mark_edge_modified(redo.edge_label);
        Ok(())
    }

    fn replay_update_vertex_prop(
        &self,
        label: LabelId,
        vid: VertexId,
        prop_name: &str,
        value: &[u8],
        ts: Timestamp,
    ) -> StorageResult<()> {
        let prop_value = bytes_to_value(value).ok_or_else(|| {
            StorageError::deserialize_error(
                "Failed to decode property value in WAL recovery".to_string(),
            )
        })?;

        {
            let mut vertex_tables = self.data_store().vertex_tables().write();
            TransactionOps::update_vertex_property_by_vid(
                &mut vertex_tables,
                label,
                vid,
                prop_name,
                &prop_value,
                ts,
            )?;
        }

        self.mark_vertex_modified(label);
        Ok(())
    }

    fn replay_update_edge_prop(
        &self,
        redo: &UpdateEdgePropRedo,
        ts: Timestamp,
    ) -> StorageResult<()> {
        let prop_value = bytes_to_value(&redo.value).ok_or_else(|| {
            StorageError::deserialize_error(
                "Failed to decode property value in WAL recovery".to_string(),
            )
        })?;

        let params = EdgeOperationParams {
            src_label: redo.src_label,
            src_id: redo.src_vid,
            dst_label: redo.dst_label,
            dst_id: redo.dst_vid,
            edge_label: redo.edge_label,
            rank: redo.rank,
        };

        {
            let vertex_tables = self.data_store().vertex_tables().read();
            let mut edge_tables = self.data_store().edge_tables().write();
            TransactionOps::update_edge_property(
                &mut edge_tables,
                &vertex_tables,
                params,
                &redo.prop_name,
                &prop_value,
                ts,
            )?;
        }
        self.mark_edge_modified(redo.edge_label);

        Ok(())
    }

    fn replay_delete_vertex(
        &self,
        label: LabelId,
        vid: VertexId,
        ts: Timestamp,
    ) -> StorageResult<()> {
        {
            let mut vertex_tables = self.data_store().vertex_tables().write();
            TransactionOps::delete_vertex_by_external_vid(&mut vertex_tables, label, vid, ts)
                .map_err(|e| {
                    StorageError::db_error(format!("Failed to replay delete vertex: {}", e))
                })?;
        }
        self.mark_vertex_modified(label);
        Ok(())
    }

    fn replay_delete_edge(&self, redo: &DeleteEdgeRedo, ts: Timestamp) -> StorageResult<()> {
        let (src_internal, dst_internal) = {
            let vertex_tables = self.data_store().vertex_tables().read();
            let src_table = vertex_tables.get(&redo.src_label).ok_or_else(|| {
                StorageError::db_error(format!(
                    "Source vertex label not found during recovery: label={}",
                    redo.src_label
                ))
            })?;
            let dst_table = vertex_tables.get(&redo.dst_label).ok_or_else(|| {
                StorageError::db_error(format!(
                    "Destination vertex label not found during recovery: label={}",
                    redo.dst_label
                ))
            })?;

            let src_internal = TransactionOps::resolve_vertex_id(src_table, redo.src_vid, ts)
                .ok_or_else(|| {
                    StorageError::db_error(format!(
                        "Source vertex not found during recovery: label={}, vid={:?}",
                        redo.src_label, redo.src_vid
                    ))
                })?;
            let dst_internal = TransactionOps::resolve_vertex_id(dst_table, redo.dst_vid, ts)
                .ok_or_else(|| {
                    StorageError::db_error(format!(
                        "Destination vertex not found during recovery: label={}, vid={:?}",
                        redo.dst_label, redo.dst_vid
                    ))
                })?;
            (src_internal, dst_internal)
        };

        let params = DeleteEdgeParams {
            src_label: redo.src_label,
            src_vid: VertexId::from_u64(src_internal as u64),
            dst_label: redo.dst_label,
            dst_vid: VertexId::from_u64(dst_internal as u64),
            edge_label: redo.edge_label,
            rank: redo.rank,
        };

        {
            let mut edge_tables = self.data_store().edge_tables().write();
            TransactionOps::delete_edge(&mut edge_tables, params, 0, 0, ts).map_err(|e| {
                StorageError::db_error(format!("Failed to replay delete edge: {}", e))
            })?;
        }
        self.mark_edge_modified(redo.edge_label);
        Ok(())
    }

    // ========================================================================
    // Schema Operations
    // ========================================================================

    fn replay_create_vertex_type(
        &self,
        redo: &CreateVertexTypeRedo,
        _ts: Timestamp,
    ) -> StorageResult<()> {
        let mut properties = Vec::with_capacity(redo.schema.len());
        for (name, type_name) in &redo.schema {
            properties.push(StoragePropertyDef::new(
                name.clone(),
                parse_data_type(type_name)?,
            ));
        }

        if properties.is_empty() {
            log::warn!(
                "replay_create_vertex_type skipped because schema is empty: {}",
                redo.label_name
            );
            return Ok(());
        }

        let primary_key = properties
            .first()
            .map(|prop| prop.name.clone())
            .unwrap_or_else(|| redo.label_name.clone());

        let label_id = self.create_vertex_type(&redo.label_name, properties.clone(), &primary_key)?;
        let space_name = recovery_space_name(self)?;
        let tag = TagInfo::new(redo.label_name.clone()).with_properties(
            redo.schema
                .iter()
                .map(|(name, type_name)| {
                    parse_data_type(type_name).map(|data_type| {
                        PropertyDef::new(name.clone(), data_type).with_nullable(false)
                    })
                })
                .collect::<StorageResult<Vec<_>>>()?,
        );
        self.schema_manager()
            .create_tag_with_id(&space_name, &tag, label_id)?;
        Ok(())
    }

    fn replay_create_edge_type(
        &self,
        redo: &CreateEdgeTypeRedo,
        _ts: Timestamp,
    ) -> StorageResult<()> {
        let src_label = self.get_vertex_label_id(&redo.src_label).ok_or_else(|| {
            StorageError::db_error(format!(
                "Source vertex label not found during recovery: {}",
                redo.src_label
            ))
        })?;
        let dst_label = self.get_vertex_label_id(&redo.dst_label).ok_or_else(|| {
            StorageError::db_error(format!(
                "Destination vertex label not found during recovery: {}",
                redo.dst_label
            ))
        })?;

        let mut properties = Vec::with_capacity(redo.schema.len());
        for (name, type_name) in &redo.schema {
            properties.push(StoragePropertyDef::new(
                name.clone(),
                parse_data_type(type_name)?,
            ));
        }

        let label_id = self.create_edge_type(
            &redo.edge_label,
            src_label,
            dst_label,
            properties,
            EdgeStrategy::Multiple,
            EdgeStrategy::Multiple,
        )?;
        let space_name = recovery_space_name(self)?;
        let edge_type = EdgeTypeInfo::new(redo.edge_label.clone())
            .with_src_tag(redo.src_label.clone())
            .with_dst_tag(redo.dst_label.clone())
            .with_properties(
                redo.schema
                    .iter()
                    .map(|(name, type_name)| {
                        parse_data_type(type_name).map(|data_type| {
                            PropertyDef::new(name.clone(), data_type).with_nullable(false)
                        })
                    })
                    .collect::<StorageResult<Vec<_>>>()?,
            );
        self.schema_manager()
            .create_edge_type_with_id(&space_name, &edge_type, label_id)?;
        Ok(())
    }

    fn replay_delete_vertex_type(
        &self,
        redo: &DeleteVertexTypeRedo,
        _ts: Timestamp,
    ) -> StorageResult<()> {
        self.drop_vertex_type(&redo.label_name)?;
        Ok(())
    }

    fn replay_delete_edge_type(
        &self,
        redo: &DeleteEdgeTypeRedo,
        _ts: Timestamp,
    ) -> StorageResult<()> {
        self.drop_edge_type(&redo.edge_label)?;
        Ok(())
    }

    fn replay_add_vertex_prop(
        &self,
        redo: &AddVertexPropRedo,
        _ts: Timestamp,
    ) -> StorageResult<()> {
        let mut props = Vec::with_capacity(redo.properties.len());
        for (name, type_name) in &redo.properties {
            props.push(StoragePropertyDef::new(
                name.clone(),
                parse_data_type(type_name)?,
            ));
        }

        for prop in props {
            self.add_vertex_property(redo.label, prop)?;
        }

        if let Some((space_name, mut tag)) = self.schema_manager().find_tag_by_id(redo.label) {
            for (name, type_name) in &redo.properties {
                let prop = PropertyDef::new(
                    name.clone(),
                    parse_data_type(type_name)?,
                )
                .with_nullable(false);
                if !tag.properties.iter().any(|existing| existing.name == prop.name) {
                    tag.properties.push(prop);
                }
            }
            self.schema_manager().update_tag(&space_name, &tag)?;
        }
        Ok(())
    }

    fn replay_add_edge_prop(&self, redo: &AddEdgePropRedo, _ts: Timestamp) -> StorageResult<()> {
        let mut props = Vec::with_capacity(redo.properties.len());
        for (name, type_name) in &redo.properties {
            props.push(StoragePropertyDef::new(
                name.clone(),
                parse_data_type(type_name)?,
            ));
        }

        for prop in props {
            self.add_edge_property(redo.edge_label, prop)?;
        }

        if let Some((space_name, mut edge_type)) = self
            .schema_manager()
            .find_edge_type_by_id(redo.edge_label)
        {
            for (name, type_name) in &redo.properties {
                let prop = PropertyDef::new(
                    name.clone(),
                    parse_data_type(type_name)?,
                )
                .with_nullable(false);
                if !edge_type.properties.iter().any(|existing| existing.name == prop.name) {
                    edge_type.properties.push(prop);
                }
            }
            self.schema_manager().update_edge_type(&space_name, &edge_type)?;
        }
        Ok(())
    }

    fn replay_delete_vertex_prop(
        &self,
        redo: &DeleteVertexPropRedo,
        _ts: Timestamp,
    ) -> StorageResult<()> {
        let (space_name, mut tag) = self
            .schema_manager()
            .find_tag_by_id(redo.label)
            .ok_or_else(|| {
                StorageError::label_not_found(format!("vertex label {}", redo.label))
            })?;

        tag.properties
            .retain(|prop| !redo.prop_names.iter().any(|name| name == &prop.name));
        self.schema_manager().update_tag(&space_name, &tag)?;

        for prop_name in &redo.prop_names {
            self.delete_vertex_property(redo.label, prop_name)?;
        }
        Ok(())
    }

    fn replay_delete_edge_prop(
        &self,
        redo: &DeleteEdgePropRedo,
        _ts: Timestamp,
    ) -> StorageResult<()> {
        let (space_name, mut edge_type) = self
            .schema_manager()
            .find_edge_type_by_id(redo.edge_label)
            .ok_or_else(|| {
                StorageError::label_not_found(format!("edge label {}", redo.edge_label))
            })?;

        edge_type
            .properties
            .retain(|prop| !redo.prop_names.iter().any(|name| name == &prop.name));
        self.schema_manager().update_edge_type(&space_name, &edge_type)?;

        for prop_name in &redo.prop_names {
            self.delete_edge_property(redo.edge_label, prop_name)?;
        }
        Ok(())
    }

    fn replay_rename_vertex_prop(
        &self,
        redo: &RenameVertexPropRedo,
        _ts: Timestamp,
    ) -> StorageResult<()> {
        let (space_name, mut tag) = self
            .schema_manager()
            .find_tag_by_id(redo.label)
            .ok_or_else(|| {
                StorageError::label_not_found(format!("vertex label {}", redo.label))
            })?;

        let prop = tag
            .properties
            .iter_mut()
            .find(|prop| prop.name == redo.old_name)
            .ok_or_else(|| StorageError::column_not_found(redo.old_name.clone()))?;
        prop.name = redo.new_name.clone();

        self.schema_manager().update_tag(&space_name, &tag)?;
        self.rename_vertex_property(redo.label, &redo.old_name, &redo.new_name)?;
        Ok(())
    }

    fn replay_rename_edge_prop(
        &self,
        redo: &RenameEdgePropRedo,
        _ts: Timestamp,
    ) -> StorageResult<()> {
        let (space_name, mut edge_type) = self
            .schema_manager()
            .find_edge_type_by_id(redo.edge_label)
            .ok_or_else(|| StorageError::label_not_found(format!("edge label {}", redo.edge_label)))?;

        let prop = edge_type
            .properties
            .iter_mut()
            .find(|prop| prop.name == redo.old_name)
            .ok_or_else(|| StorageError::column_not_found(redo.old_name.clone()))?;
        prop.name = redo.new_name.clone();

        self.schema_manager().update_edge_type(&space_name, &edge_type)?;
        self.rename_edge_property(redo.edge_label, &redo.old_name, &redo.new_name)?;
        Ok(())
    }
}

fn recovery_space_name(ctx: &GraphStorageContext) -> StorageResult<String> {
    if let Some(space) = ctx.schema_manager().list_spaces()?.into_iter().next() {
        return Ok(space.space_name);
    }

    let mut space = SpaceInfo::new("__graphdb_recovery__".to_string());
    ctx.schema_manager().create_space(&mut space)?;
    Ok(space.space_name)
}

fn parse_data_type(raw: &str) -> StorageResult<DataType> {
    let upper = raw.trim().to_ascii_uppercase();

    let ty = match upper.as_str() {
        "EMPTY" => DataType::Empty,
        "NULL" => DataType::Null,
        "BOOL" => DataType::Bool,
        "SMALLINT" => DataType::SmallInt,
        "INT" => DataType::Int,
        "BIGINT" => DataType::BigInt,
        "FLOAT" => DataType::Float,
        "DOUBLE" => DataType::Double,
        "DECIMAL128" => DataType::Decimal128,
        "STRING" => DataType::String,
        "DATE" => DataType::Date,
        "TIME" => DataType::Time,
        "DATETIME" => DataType::DateTime,
        "VERTEX" => DataType::Vertex,
        "EDGE" => DataType::Edge,
        "PATH" => DataType::Path,
        "LIST" => DataType::List,
        "MAP" => DataType::Map,
        "SET" => DataType::Set,
        "GEOGRAPHY" => DataType::Geography,
        "DATASET" => DataType::DataSet,
        "VID" => DataType::VID,
        "BLOB" => DataType::Blob,
        "TIMESTAMP" => DataType::Timestamp,
        "VECTOR" => DataType::Vector,
        "JSON" => DataType::Json,
        "JSONB" => DataType::JsonB,
        "UUID" => DataType::Uuid,
        "INTERVAL" => DataType::Interval,
        value if value.starts_with("FIXEDSTRING(") && value.ends_with(')') => {
            let inner = &value["FIXEDSTRING(".len()..value.len() - 1];
            let size = inner.trim().parse::<usize>().map_err(|e| {
                StorageError::deserialize_error(format!(
                    "Invalid FIXEDSTRING size in WAL recovery: {}",
                    e
                ))
            })?;
            DataType::FixedString(size)
        }
        value if value.starts_with("VECTOR_DENSE(") && value.ends_with(')') => {
            let inner = &value["VECTOR_DENSE(".len()..value.len() - 1];
            let size = inner.trim().parse::<usize>().map_err(|e| {
                StorageError::deserialize_error(format!(
                    "Invalid VECTOR_DENSE size in WAL recovery: {}",
                    e
                ))
            })?;
            DataType::VectorDense(size)
        }
        value if value.starts_with("VECTOR_SPARSE(") && value.ends_with(')') => {
            let inner = &value["VECTOR_SPARSE(".len()..value.len() - 1];
            let size = inner.trim().parse::<usize>().map_err(|e| {
                StorageError::deserialize_error(format!(
                    "Invalid VECTOR_SPARSE size in WAL recovery: {}",
                    e
                ))
            })?;
            DataType::VectorSparse(size)
        }
        other => {
            return Err(StorageError::deserialize_error(format!(
                "Unsupported data type in WAL recovery: {}",
                other
            )));
        }
    };

    Ok(ty)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Value;
    use crate::core::wal::traits::RecoveryApplier;
    use crate::storage::engine::{EdgeOperationParams, InsertEdgeParams};

    #[test]
    fn test_schema_replay_roundtrip() {
        let ctx = GraphStorageContext::new();

        ctx.replay_create_vertex_type(
            &CreateVertexTypeRedo {
                label_name: "Person".to_string(),
                schema: vec![
                    ("id".to_string(), "BIGINT".to_string()),
                    ("name".to_string(), "STRING".to_string()),
                ],
            },
            1,
        )
        .expect("Vertex type replay should succeed");

        ctx.replay_create_vertex_type(
            &CreateVertexTypeRedo {
                label_name: "City".to_string(),
                schema: vec![
                    ("id".to_string(), "BIGINT".to_string()),
                    ("name".to_string(), "STRING".to_string()),
                ],
            },
            1,
        )
        .expect("Second vertex type replay should succeed");

        let person_label = ctx
            .get_vertex_label_id("Person")
            .expect("Person label should exist");
        let city_label = ctx
            .get_vertex_label_id("City")
            .expect("City label should exist");

        ctx.replay_add_vertex_prop(
            &AddVertexPropRedo {
                label: person_label,
                properties: vec![("age".to_string(), "INT".to_string())],
            },
            2,
        )
        .expect("Vertex property replay should succeed");

        ctx.replay_rename_vertex_prop(
            &RenameVertexPropRedo {
                label: person_label,
                old_name: "name".to_string(),
                new_name: "full_name".to_string(),
            },
            2,
        )
        .expect("Vertex rename replay should succeed");

        ctx.replay_delete_vertex_prop(
            &DeleteVertexPropRedo {
                label: person_label,
                prop_names: vec!["age".to_string()],
            },
            2,
        )
        .expect("Vertex delete replay should succeed");

        ctx.replay_create_edge_type(
            &CreateEdgeTypeRedo {
                src_label: "Person".to_string(),
                dst_label: "City".to_string(),
                edge_label: "LIVES_IN".to_string(),
                schema: vec![("since".to_string(), "INT".to_string())],
            },
            3,
        )
        .expect("Edge type replay should succeed");

        let lives_in_label = ctx
            .get_edge_label_id("LIVES_IN")
            .expect("Edge label should exist");

        ctx.replay_add_edge_prop(
            &AddEdgePropRedo {
                src_label: person_label,
                dst_label: city_label,
                edge_label: lives_in_label,
                properties: vec![("cost".to_string(), "INT".to_string())],
            },
            3,
        )
        .expect("Edge property replay should succeed");

        ctx.replay_rename_edge_prop(
            &RenameEdgePropRedo {
                src_label: person_label,
                dst_label: city_label,
                edge_label: lives_in_label,
                old_name: "since".to_string(),
                new_name: "started".to_string(),
            },
            3,
        )
        .expect("Edge rename replay should succeed");

        ctx.replay_delete_edge_prop(
            &DeleteEdgePropRedo {
                src_label: person_label,
                dst_label: city_label,
                edge_label: lives_in_label,
                prop_names: vec!["cost".to_string()],
            },
            3,
        )
        .expect("Edge delete replay should succeed");

        let person_tag = ctx
            .schema_manager()
            .find_tag_by_id(person_label)
            .expect("Person tag should exist")
            .1;
        assert_eq!(
            person_tag
                .properties
                .iter()
                .map(|prop| prop.name.as_str())
                .collect::<Vec<_>>(),
            vec!["id", "full_name"]
        );

        let lives_in_type = ctx
            .schema_manager()
            .find_edge_type_by_id(lives_in_label)
            .expect("Edge type should exist")
            .1;
        assert_eq!(
            lives_in_type
                .properties
                .iter()
                .map(|prop| prop.name.as_str())
                .collect::<Vec<_>>(),
            vec!["started"]
        );

        ctx.insert_vertex_by_i64(
            person_label,
            1001,
            &[
                ("id".to_string(), Value::BigInt(1001)),
                ("full_name".to_string(), Value::String("Alice".to_string())),
            ],
            4,
        )
        .expect("Vertex insert should succeed after property replay");

        ctx.insert_vertex_by_i64(
            city_label,
            2001,
            &[
                ("id".to_string(), Value::BigInt(2001)),
                ("name".to_string(), Value::String("Shanghai".to_string())),
            ],
            4,
        )
        .expect("City vertex insert should succeed");

        let vertex = ctx
            .get_vertex_by_i64(person_label, 1001, 5)
            .expect("Inserted vertex should be visible");
        assert_eq!(
            vertex
                .properties
                .iter()
                .find(|(name, _)| name == "full_name")
                .map(|(_, value)| value),
            Some(&Value::String("Alice".to_string()))
        );
        assert!(vertex.properties.iter().all(|(name, _)| name != "age"));

        let edge_offset = ctx
            .insert_edge(InsertEdgeParams {
                edge_label: lives_in_label,
                src_label: person_label,
                src_id: VertexId::from_int64(1001),
                dst_label: city_label,
                dst_id: VertexId::from_int64(2001),
                rank: 0,
                properties: &[("started".to_string(), Value::Int(2012))],
                ts: 5,
            })
            .expect("Edge insert should succeed after property replay");

        let edge = ctx
            .get_edge(
                &EdgeOperationParams {
                    edge_label: lives_in_label,
                    src_label: person_label,
                    src_id: VertexId::from_int64(1001),
                    dst_label: city_label,
                    dst_id: VertexId::from_int64(2001),
                    rank: 0,
                },
                5,
            )
            .expect("Inserted edge should be visible");
        assert!(!edge_offset.is_none());
        assert_eq!(
            edge.properties
                .iter()
                .find(|(name, _)| name == "started")
                .map(|(_, value)| value),
            Some(&Value::Int(2012))
        );
        assert!(edge.properties.iter().all(|(name, _)| name != "cost"));
        assert!(edge.properties.iter().all(|(name, _)| name != "since"));
    }
}

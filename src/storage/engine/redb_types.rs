//! Redb storage sharing type definition
//!
//! Provide the shared types required for operating on the Redb database, including ByteKey and table definitions.

use redb::{TableDefinition, TypeName};
use std::cmp::Ordering as CmpOrdering;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ByteKey(pub Vec<u8>);

impl redb::Key for ByteKey {
    fn compare(data1: &[u8], data2: &[u8]) -> CmpOrdering {
        data1.cmp(data2)
    }
}

impl redb::Value for ByteKey {
    type SelfType<'a>
        = ByteKey
    where
        Self: 'a;
    type AsBytes<'a>
        = Vec<u8>
    where
        Self: 'a;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> ByteKey
    where
        Self: 'a,
    {
        ByteKey(data.to_vec())
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Vec<u8>
    where
        Self: 'b,
    {
        value.0.clone()
    }

    fn type_name() -> TypeName {
        TypeName::new("graphdb::ByteKey")
    }
}

pub const NODES_TABLE: TableDefinition<ByteKey, ByteKey> = TableDefinition::new("nodes");
pub const EDGES_TABLE: TableDefinition<ByteKey, ByteKey> = TableDefinition::new("edges");
pub const INDEXES_TABLE: TableDefinition<ByteKey, ByteKey> = TableDefinition::new("indexes");
pub const SPACES_TABLE: TableDefinition<ByteKey, ByteKey> = TableDefinition::new("spaces");
pub const TAGS_TABLE: TableDefinition<ByteKey, ByteKey> = TableDefinition::new("tags");
pub const EDGE_TYPES_TABLE: TableDefinition<ByteKey, ByteKey> = TableDefinition::new("edge_types");
pub const TAG_INDEXES_TABLE: TableDefinition<ByteKey, ByteKey> =
    TableDefinition::new("tag_indexes");
pub const EDGE_INDEXES_TABLE: TableDefinition<ByteKey, ByteKey> =
    TableDefinition::new("edge_indexes");
pub const INDEX_DATA_TABLE: TableDefinition<ByteKey, ByteKey> = TableDefinition::new("index_data");
pub const VERTEX_DATA_TABLE: TableDefinition<ByteKey, ByteKey> =
    TableDefinition::new("vertex_data");
pub const EDGE_DATA_TABLE: TableDefinition<ByteKey, ByteKey> = TableDefinition::new("edge_data");
pub const PASSWORDS_TABLE: TableDefinition<ByteKey, ByteKey> = TableDefinition::new("passwords");
pub const INDEX_COUNTER_TABLE: TableDefinition<ByteKey, ByteKey> =
    TableDefinition::new("index_counter");
pub const SCHEMA_VERSIONS_TABLE: TableDefinition<ByteKey, ByteKey> =
    TableDefinition::new("schema_versions");
pub const SCHEMA_CHANGES_TABLE: TableDefinition<ByteKey, ByteKey> =
    TableDefinition::new("schema_changes");
pub const CURRENT_VERSIONS_TABLE: TableDefinition<ByteKey, ByteKey> =
    TableDefinition::new("current_versions");

// Tag/Edge ID Generator Table – Used to automatically generate incremental IDs for each Space.
pub const TAG_ID_COUNTER_TABLE: TableDefinition<ByteKey, ByteKey> =
    TableDefinition::new("tag_id_counters");
pub const EDGE_TYPE_ID_COUNTER_TABLE: TableDefinition<ByteKey, ByteKey> =
    TableDefinition::new("edge_type_id_counters");

// Space/Tag/Edge Name Index Table – Used for the mapping from names to IDs
pub const SPACE_NAME_INDEX_TABLE: TableDefinition<ByteKey, ByteKey> =
    TableDefinition::new("space_name_index");
pub const TAG_NAME_INDEX_TABLE: TableDefinition<ByteKey, ByteKey> =
    TableDefinition::new("tag_name_index");
pub const EDGE_TYPE_NAME_INDEX_TABLE: TableDefinition<ByteKey, ByteKey> =
    TableDefinition::new("edge_type_name_index");

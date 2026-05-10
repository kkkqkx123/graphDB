//! Edge Iterator - provides lazy iteration over edge records
//!
//! Offers:
//! - EdgeScanIterator: Lazy iterator over all edges in an EdgeTable
//! - EdgeRangeIterator: Iterator over edges of specified vertices
//! - EdgeFilterIterator: Iterator with predicate pushdown support

use crate::storage::edge::{EdgeRecord, EdgeTable, EdgeTableScanIterator, EdgeVertexIterator, Timestamp, VertexId};
use crate::storage::iterator::predicate::PredicateEnum;

pub type EdgeScanIterator<'a> = EdgeTableScanIterator<'a>;
pub type EdgeVertexScanIterator<'a> = EdgeVertexIterator<'a>;

pub struct EdgeRangeIterator<'a> {
    iter: EdgeRangeIteratorInner<'a>,
}

enum EdgeRangeIteratorInner<'a> {
    Single {
        iter: EdgeVertexIterator<'a>,
    },
    Multi {
        table: &'a EdgeTable,
        ts: Timestamp,
        src_vertices: Vec<VertexId>,
        current_idx: usize,
        current_iter: Option<EdgeVertexIterator<'a>>,
    },
}

impl<'a> EdgeRangeIterator<'a> {
    pub fn new(table: &'a EdgeTable, src_vertices: Vec<VertexId>, ts: Timestamp) -> Self {
        if src_vertices.len() == 1 {
            Self {
                iter: EdgeRangeIteratorInner::Single {
                    iter: table.iter_edges(src_vertices[0], ts),
                },
            }
        } else {
            Self {
                iter: EdgeRangeIteratorInner::Multi {
                    table,
                    ts,
                    src_vertices,
                    current_idx: 0,
                    current_iter: None,
                },
            }
        }
    }
}

impl<'a> Iterator for EdgeRangeIterator<'a> {
    type Item = EdgeRecord;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.iter {
            EdgeRangeIteratorInner::Single { iter } => iter.next(),
            EdgeRangeIteratorInner::Multi {
                table,
                ts,
                src_vertices,
                current_idx,
                current_iter,
            } => {
                loop {
                    if let Some(ref mut iter) = current_iter {
                        if let Some(record) = iter.next() {
                            return Some(record);
                        }
                    }

                    if *current_idx >= src_vertices.len() {
                        return None;
                    }

                    let src = src_vertices[*current_idx];
                    *current_idx += 1;
                    *current_iter = Some(table.iter_edges(src, *ts));
                }
            }
        }
    }
}

pub struct EdgeFilterIterator<'a> {
    iter: EdgeTableScanIterator<'a>,
    predicate: PredicateEnum,
}

impl<'a> EdgeFilterIterator<'a> {
    pub fn new(table: &'a EdgeTable, ts: Timestamp, predicate: PredicateEnum) -> Self {
        Self {
            iter: table.iter(ts),
            predicate,
        }
    }
}

impl<'a> Iterator for EdgeFilterIterator<'a> {
    type Item = EdgeRecord;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let record = self.iter.next()?;
            let row: Vec<crate::core::Value> = record
                .properties
                .iter()
                .map(|(_, v)| v.clone())
                .collect();

            if self.predicate.evaluate(&row) {
                return Some(record);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::DataType;
    use crate::core::Value;
    use crate::storage::edge::{EdgeSchema, EdgeStrategy, PropertyDef};

    fn create_test_schema() -> EdgeSchema {
        EdgeSchema {
            label_id: 0,
            label_name: "knows".to_string(),
            src_label: 1,
            dst_label: 2,
            properties: vec![PropertyDef::new("weight".to_string(), DataType::Double)],
            oe_strategy: EdgeStrategy::Multiple,
            ie_strategy: EdgeStrategy::Multiple,
        }
    }

    fn create_test_table() -> EdgeTable {
        let schema = create_test_schema();
        let mut table = EdgeTable::new(schema);

        let ts = 100u32;
        table.insert_edge(1, 2, &[("weight".to_string(), Value::Double(1.0))], ts).unwrap();
        table.insert_edge(1, 3, &[("weight".to_string(), Value::Double(2.0))], ts).unwrap();
        table.insert_edge(2, 3, &[("weight".to_string(), Value::Double(3.0))], ts).unwrap();

        table
    }

    #[test]
    fn test_edge_scan_iterator() {
        let table = create_test_table();
        let iter = table.iter(100);
        let edges: Vec<_> = iter.collect();

        assert_eq!(edges.len(), 3);
    }

    #[test]
    fn test_edge_range_iterator() {
        let table = create_test_table();
        let iter = EdgeRangeIterator::new(&table, vec![1, 2], 100);
        let edges: Vec<_> = iter.collect();

        assert_eq!(edges.len(), 3);
    }

    #[test]
    fn test_edge_filter_iterator() {
        use crate::storage::iterator::predicate::CompareOp;

        let table = create_test_table();
        
        // Debug: print all edges and their properties
        for edge in table.iter(100) {
            println!("Edge {} -> {}: {:?}", edge.src_vid, edge.dst_vid, edge.properties);
        }

        let predicate = PredicateEnum::simple("0", CompareOp::Greater, Value::Double(1.5));
        let iter = EdgeFilterIterator::new(&table, 100, predicate);
        let edges: Vec<_> = iter.collect();

        assert_eq!(edges.len(), 2);
    }
}

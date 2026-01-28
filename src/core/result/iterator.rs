use crate::core::value::Value;
use crate::core::DBResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IteratorType {
    Default,
    GetNeighbors,
    Prop,
}

#[deprecated(since = "0.1.0", note = "请使用 ResultIterator trait")]
pub trait Iterator: Send + Sync + std::fmt::Debug {
    fn iterator_type(&self) -> IteratorType;

    fn next(&mut self) -> DBResult<Option<Vec<Value>>>;

    fn reset(&mut self) -> DBResult<()>;

    fn size(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.size() == 0
    }
}

#[derive(Debug)]
pub struct DefaultIterator {
    rows: Vec<Vec<Value>>,
    index: usize,
}

impl DefaultIterator {
    pub fn new(rows: Vec<Vec<Value>>) -> Self {
        Self { rows, index: 0 }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            rows: Vec::with_capacity(capacity),
            index: 0,
        }
    }

    pub fn add_row(&mut self, row: Vec<Value>) {
        self.rows.push(row);
    }

    pub fn rows(&self) -> &[Vec<Value>] {
        &self.rows
    }

    pub fn rows_mut(&mut self) -> &mut Vec<Vec<Value>> {
        &mut self.rows
    }
}

impl r#Iterator for DefaultIterator {
    fn iterator_type(&self) -> IteratorType {
        IteratorType::Default
    }

    fn next(&mut self) -> DBResult<Option<Vec<Value>>> {
        if self.index < self.rows.len() {
            let row = self.rows[self.index].clone();
            self.index += 1;
            Ok(Some(row))
        } else {
            Ok(None)
        }
    }

    fn reset(&mut self) -> DBResult<()> {
        self.index = 0;
        Ok(())
    }

    fn size(&self) -> usize {
        self.rows.len()
    }
}

#[derive(Debug)]
pub struct GetNeighborsIterator {
    vertices: Vec<Value>,
    edges: Vec<Vec<Value>>,
    vertex_index: usize,
    edge_index: usize,
}

impl GetNeighborsIterator {
    pub fn new(vertices: Vec<Value>, edges: Vec<Vec<Value>>) -> Self {
        Self {
            vertices,
            edges,
            vertex_index: 0,
            edge_index: 0,
        }
    }

    pub fn with_capacity(vertex_capacity: usize, edge_capacity: usize) -> Self {
        Self {
            vertices: Vec::with_capacity(vertex_capacity),
            edges: Vec::with_capacity(edge_capacity),
            vertex_index: 0,
            edge_index: 0,
        }
    }

    pub fn add_vertex(&mut self, vertex: Value) {
        self.vertices.push(vertex);
    }

    pub fn add_edge(&mut self, edge: Vec<Value>) {
        self.edges.push(edge);
    }

    pub fn vertices(&self) -> &[Value] {
        &self.vertices
    }

    pub fn edges(&self) -> &[Vec<Value>] {
        &self.edges
    }

    pub fn vertices_mut(&mut self) -> &mut Vec<Value> {
        &mut self.vertices
    }

    pub fn edges_mut(&mut self) -> &mut Vec<Vec<Value>> {
        &mut self.edges
    }
}

impl r#Iterator for GetNeighborsIterator {
    fn iterator_type(&self) -> IteratorType {
        IteratorType::GetNeighbors
    }

    fn next(&mut self) -> DBResult<Option<Vec<Value>>> {
        if self.vertex_index < self.vertices.len() {
            let vertex = self.vertices[self.vertex_index].clone();
            self.vertex_index += 1;

            let mut row = vec![vertex];

            if self.edge_index < self.edges.len() {
                let edge = self.edges[self.edge_index].clone();
                row.extend(edge);
                self.edge_index += 1;
            }

            Ok(Some(row))
        } else {
            Ok(None)
        }
    }

    fn reset(&mut self) -> DBResult<()> {
        self.vertex_index = 0;
        self.edge_index = 0;
        Ok(())
    }

    fn size(&self) -> usize {
        self.vertices.len()
    }
}

#[derive(Debug)]
pub struct PropIterator {
    props: Vec<Vec<Value>>,
    index: usize,
}

impl PropIterator {
    pub fn new(props: Vec<Vec<Value>>) -> Self {
        Self { props, index: 0 }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            props: Vec::with_capacity(capacity),
            index: 0,
        }
    }

    pub fn add_prop(&mut self, prop: Vec<Value>) {
        self.props.push(prop);
    }

    pub fn props(&self) -> &[Vec<Value>] {
        &self.props
    }

    pub fn props_mut(&mut self) -> &mut Vec<Vec<Value>> {
        &mut self.props
    }
}

impl r#Iterator for PropIterator {
    fn iterator_type(&self) -> IteratorType {
        IteratorType::Prop
    }

    fn next(&mut self) -> DBResult<Option<Vec<Value>>> {
        if self.index < self.props.len() {
            let prop = self.props[self.index].clone();
            self.index += 1;
            Ok(Some(prop))
        } else {
            Ok(None)
        }
    }

    fn reset(&mut self) -> DBResult<()> {
        self.index = 0;
        Ok(())
    }

    fn size(&self) -> usize {
        self.props.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_iterator() {
        let rows = vec![
            vec![Value::Int(1), Value::String("Alice".to_string())],
            vec![Value::Int(2), Value::String("Bob".to_string())],
        ];

        let mut iter = DefaultIterator::new(rows);

        assert_eq!(iter.iterator_type(), IteratorType::Default);
        assert_eq!(iter.size(), 2);

        let row1 = iter.next().unwrap();
        assert!(row1.is_some());
        assert_eq!(row1.unwrap()[0], Value::Int(1));

        let row2 = iter.next().unwrap();
        assert!(row2.is_some());
        assert_eq!(row2.unwrap()[0], Value::Int(2));

        let row3 = iter.next().unwrap();
        assert!(row3.is_none());
    }

    #[test]
    fn test_default_iterator_reset() {
        let rows = vec![vec![Value::Int(1)]];
        let mut iter = DefaultIterator::new(rows);

        iter.next().unwrap();
        assert_eq!(iter.next().unwrap(), None);

        iter.reset().unwrap();
        assert_eq!(iter.next().unwrap().unwrap()[0], Value::Int(1));
    }

    #[test]
    fn test_get_neighbors_iterator() {
        let vertices = vec![Value::Int(1), Value::Int(2)];
        let edges = vec![
            vec![Value::String("edge1".to_string())],
            vec![Value::String("edge2".to_string())],
        ];

        let mut iter = GetNeighborsIterator::new(vertices, edges);

        assert_eq!(iter.iterator_type(), IteratorType::GetNeighbors);
        assert_eq!(iter.size(), 2);

        let row1 = iter.next().unwrap().unwrap();
        assert_eq!(row1[0], Value::Int(1));
        assert_eq!(row1[1], Value::String("edge1".to_string()));

        let row2 = iter.next().unwrap().unwrap();
        assert_eq!(row2[0], Value::Int(2));
        assert_eq!(row2[1], Value::String("edge2".to_string()));
    }

    #[test]
    fn test_prop_iterator() {
        let props = vec![
            vec![Value::String("name".to_string()), Value::String("Alice".to_string())],
            vec![Value::String("age".to_string()), Value::Int(25)],
        ];

        let mut iter = PropIterator::new(props);

        assert_eq!(iter.iterator_type(), IteratorType::Prop);
        assert_eq!(iter.size(), 2);

        let prop1 = iter.next().unwrap().unwrap();
        assert_eq!(prop1[0], Value::String("name".to_string()));

        let prop2 = iter.next().unwrap().unwrap();
        assert_eq!(prop2[0], Value::String("age".to_string()));
    }

    #[test]
    fn test_iterator_is_empty() {
        let iter = DefaultIterator::new(vec![]);
        assert!(iter.is_empty());

        let iter = DefaultIterator::new(vec![vec![Value::Int(1)]]);
        assert!(!iter.is_empty());
    }
}

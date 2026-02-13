use crate::query::parser::parser::Parser;
use crate::query::parser::ast::stmt::*;

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_statement(query: &str) -> Result<Stmt, crate::query::parser::core::error::ParseError> {
        let mut parser = Parser::new(query);
        parser.parse()
    }

    #[test]
    fn test_insert_edge_basic() {
        let query = "INSERT EDGE KNOWS(since) VALUES 1 -> 2:('2020-01-01')";
        let result = parse_statement(query);
        assert!(result.is_ok(), "INSERT EDGE 解析应该成功: {:?}", result.err());
        
        let stmt = result.expect("INSERT EDGE解析应该成功");
        assert_eq!(stmt.kind(), "INSERT");
        
        if let Stmt::Insert(insert_stmt) = stmt {
            if let InsertTarget::Edge { edge_name, edges, .. } = insert_stmt.target {
                assert_eq!(edge_name, "KNOWS");
                assert_eq!(edges.len(), 1);
                let (_, _, _, values) = &edges[0];
                assert_eq!(values.len(), 1);
            } else {
                panic!("期望 Edge 目标");
            }
        } else {
            panic!("期望 Insert 语句");
        }
    }

    #[test]
    fn test_insert_edge_with_rank() {
        let query = "INSERT EDGE KNOWS(since) VALUES 1 -> 2 @0:('2020-01-01')";
        let result = parse_statement(query);
        assert!(result.is_ok(), "INSERT EDGE 带 rank 解析应该成功: {:?}", result.err());
        
        let stmt = result.expect("INSERT EDGE带rank解析应该成功");
        assert_eq!(stmt.kind(), "INSERT");
        
        if let Stmt::Insert(insert_stmt) = stmt {
            if let InsertTarget::Edge { edge_name, edges, .. } = insert_stmt.target {
                assert_eq!(edge_name, "KNOWS");
                assert_eq!(edges.len(), 1);
                let (_, _, rank, _) = &edges[0];
                assert!(rank.is_some(), "rank 应该存在");
            } else {
                panic!("期望 Edge 目标");
            }
        } else {
            panic!("期望 Insert 语句");
        }
    }

    #[test]
    fn test_insert_edge_multiple() {
        let query = "INSERT EDGE KNOWS(since) VALUES 1 -> 2:('2020-01-01'), 2 -> 3:('2021-01-01')";
        let result = parse_statement(query);
        assert!(result.is_ok(), "INSERT 多个边解析应该成功: {:?}", result.err());
        
        let stmt = result.expect("INSERT多个边解析应该成功");
        assert_eq!(stmt.kind(), "INSERT");
    }

    #[test]
    fn test_insert_edge_multiple_properties() {
        let query = "INSERT EDGE KNOWS(since, weight) VALUES 1 -> 2:('2020-01-01', 0.9)";
        let result = parse_statement(query);
        assert!(result.is_ok(), "INSERT EDGE 多属性解析应该成功: {:?}", result.err());
        
        let stmt = result.expect("INSERT EDGE多属性解析应该成功");
        assert_eq!(stmt.kind(), "INSERT");
    }

    #[test]
    fn test_insert_vertex_basic() {
        let query = "INSERT VERTEX Person(name, age) VALUES 1:('Alice', 30)";
        let result = parse_statement(query);
        assert!(result.is_ok(), "INSERT VERTEX 解析应该成功: {:?}", result.err());
        
        let stmt = result.expect("INSERT VERTEX解析应该成功");
        assert_eq!(stmt.kind(), "INSERT");
    }

    #[test]
    fn test_insert_vertex_multiple() {
        let query = "INSERT VERTEX Person(name, age) VALUES 1:('Alice', 30), 2:('Bob', 25)";
        let result = parse_statement(query);
        assert!(result.is_ok(), "INSERT 多个顶点解析应该成功: {:?}", result.err());
        
        let stmt = result.expect("INSERT多个顶点解析应该成功");
        assert_eq!(stmt.kind(), "INSERT");
    }

    #[test]
    fn test_delete_edge_basic() {
        let query = "DELETE EDGE KNOWS 1 -> 2";
        let result = parse_statement(query);
        assert!(result.is_ok(), "DELETE EDGE 解析应该成功: {:?}", result.err());
        
        let stmt = result.expect("DELETE EDGE解析应该成功");
        assert_eq!(stmt.kind(), "DELETE");
    }

    #[test]
    fn test_delete_edge_with_rank() {
        let query = "DELETE EDGE KNOWS 1 -> 2 @0";
        let result = parse_statement(query);
        assert!(result.is_ok(), "DELETE EDGE 带 rank 解析应该成功: {:?}", result.err());
        
        let stmt = result.expect("DELETE EDGE带rank解析应该成功");
        assert_eq!(stmt.kind(), "DELETE");
    }

    #[test]
    fn test_delete_edge_multiple() {
        let query = "DELETE EDGE KNOWS 1 -> 2, 2 -> 3";
        let result = parse_statement(query);
        assert!(result.is_ok(), "DELETE 多个边解析应该成功: {:?}", result.err());
        
        let stmt = result.expect("DELETE多个边解析应该成功");
        assert_eq!(stmt.kind(), "DELETE");
    }

    #[test]
    fn test_set_property_basic() {
        let query = "SET p.age = 26";
        let result = parse_statement(query);
        assert!(result.is_ok(), "SET 属性解析应该成功: {:?}", result.err());
        
        let stmt = result.expect("SET属性解析应该成功");
        assert_eq!(stmt.kind(), "SET");
    }

    #[test]
    fn test_set_property_multiple() {
        let query = "SET p.age = 26, p.name = 'Alice'";
        let result = parse_statement(query);
        assert!(result.is_ok(), "SET 多个属性解析应该成功: {:?}", result.err());
        
        let stmt = result.expect("SET多个属性解析应该成功");
        assert_eq!(stmt.kind(), "SET");
    }

    #[test]
    fn test_set_property_with_expression() {
        let query = "SET p.age = p.age + 1";
        let result = parse_statement(query);
        assert!(result.is_ok(), "SET 带表达式解析应该成功: {:?}", result.err());
        
        let stmt = result.expect("SET带表达式解析应该成功");
        assert_eq!(stmt.kind(), "SET");
    }

    #[test]
    fn test_update_vertex_basic() {
        let query = "UPDATE 1 SET age = 26";
        let result = parse_statement(query);
        assert!(result.is_ok(), "UPDATE 顶点解析应该成功: {:?}", result.err());
        
        let stmt = result.expect("UPDATE顶点解析应该成功");
        assert_eq!(stmt.kind(), "UPDATE");
    }

    #[test]
    fn test_delete_vertex_basic() {
        let query = "DELETE VERTEX 1";
        let result = parse_statement(query);
        assert!(result.is_ok(), "DELETE VERTEX 解析应该成功: {:?}", result.err());
        
        let stmt = result.expect("DELETE VERTEX解析应该成功");
        assert_eq!(stmt.kind(), "DELETE");
    }
}

//! Cost-Based Optimization Strategy Tests
//!
//! Test coverage:
//! - Join order optimization
//! - Index selection strategies
//! - Traversal direction optimization
//! - Aggregate strategy selection
//! - Plan enumeration

use crate::common::test_scenario::TestScenario;

// ==================== Join Order Tests ====================

mod join_order {
    use super::*;

    #[test]
    fn test_simple_join_order() {
        TestScenario::new()
            .expect("Failed to create test scenario")
            .setup_space("test_simple_join")
            .exec_ddl("CREATE TAG person(name STRING)")
            .exec_ddl("CREATE TAG company(name STRING)")
            .exec_ddl("CREATE EDGE works_at()")
            .assert_success()
            .query("MATCH (p:person)-[:works_at]->(c:company) RETURN p, c")
            .assert_success();
    }

    #[test]
    fn test_multi_table_join_order() {
        TestScenario::new()
            .expect("Failed to create test scenario")
            .setup_space("test_multi_join")
            .exec_ddl("CREATE TAG person(name STRING)")
            .exec_ddl("CREATE TAG company(name STRING)")
            .exec_ddl("CREATE TAG department(name STRING)")
            .exec_ddl("CREATE EDGE works_at()")
            .exec_ddl("CREATE EDGE belongs_to()")
            .assert_success()
            .query("MATCH (p:person)-[:works_at]->(c:company)-[:belongs_to]->(d:department) RETURN p, d")
            .assert_success();
    }

    #[test]
    fn test_join_with_filter() {
        TestScenario::new()
            .expect("Failed to create test scenario")
            .setup_space("test_join_filter")
            .exec_ddl("CREATE TAG person(name STRING, age INT)")
            .exec_ddl("CREATE TAG company(name STRING)")
            .exec_ddl("CREATE EDGE works_at()")
            .assert_success()
            .query("MATCH (p:person)-[:works_at]->(c:company) WHERE p.age > 30 RETURN p, c")
            .assert_success();
    }
}

// ==================== Index Selection Tests ====================

mod index_selection {
    use super::*;

    #[test]
    fn test_single_index_selection() {
        TestScenario::new()
            .expect("Failed to create test scenario")
            .setup_space("test_single_index")
            .exec_ddl("CREATE TAG person(name STRING, age INT)")
            .exec_ddl("CREATE TAG INDEX idx_person_age ON person(age)")
            .assert_success()
            .query("MATCH (n:person) WHERE n.age = 30 RETURN n")
            .assert_success();
    }

    #[test]
    fn test_composite_index_selection() {
        TestScenario::new()
            .expect("Failed to create test scenario")
            .setup_space("test_composite_index")
            .exec_ddl("CREATE TAG person(name STRING, age INT, city STRING)")
            .exec_ddl("CREATE TAG INDEX idx_person_age_city ON person(age, city)")
            .assert_success()
            .query("MATCH (n:person) WHERE n.age = 30 AND n.city = 'Beijing' RETURN n")
            .assert_success();
    }

    #[test]
    fn test_index_with_range_query() {
        TestScenario::new()
            .expect("Failed to create test scenario")
            .setup_space("test_index_range")
            .exec_ddl("CREATE TAG person(name STRING, age INT)")
            .exec_ddl("CREATE TAG INDEX idx_person_age ON person(age)")
            .assert_success()
            .query("MATCH (n:person) WHERE n.age > 25 AND n.age < 50 RETURN n")
            .assert_success();
    }

    #[test]
    fn test_multiple_index_candidates() {
        TestScenario::new()
            .expect("Failed to create test scenario")
            .setup_space("test_multi_index")
            .exec_ddl("CREATE TAG person(name STRING, age INT, city STRING)")
            .exec_ddl("CREATE TAG INDEX idx_person_age ON person(age)")
            .exec_ddl("CREATE TAG INDEX idx_person_city ON person(city)")
            .assert_success()
            .query("MATCH (n:person) WHERE n.age = 30 AND n.city = 'Beijing' RETURN n")
            .assert_success();
    }
}

// ==================== Traversal Direction Tests ====================

mod traversal_direction {
    use super::*;

    #[test]
    fn test_forward_traversal() {
        TestScenario::new()
            .expect("Failed to create test scenario")
            .setup_space("test_forward")
            .exec_ddl("CREATE TAG person(name STRING)")
            .exec_ddl("CREATE EDGE follows()")
            .assert_success()
            .query("MATCH (a:person)-[:follows]->(b:person) RETURN a, b")
            .assert_success();
    }

    #[test]
    fn test_backward_traversal() {
        TestScenario::new()
            .expect("Failed to create test scenario")
            .setup_space("test_backward")
            .exec_ddl("CREATE TAG person(name STRING)")
            .exec_ddl("CREATE EDGE follows()")
            .assert_success()
            .query("GO FROM 1 OVER follows REVERSELY YIELD $$.person.name AS name")
            .assert_success();
    }

    #[test]
    fn test_bidirectional_traversal() {
        TestScenario::new()
            .expect("Failed to create test scenario")
            .setup_space("test_bidirectional")
            .exec_ddl("CREATE TAG person(name STRING)")
            .exec_ddl("CREATE EDGE follows()")
            .assert_success()
            .query("MATCH (a:person)-[:follows]-(b:person) RETURN a, b")
            .assert_success();
    }

    #[test]
    fn test_multi_hop_traversal() {
        TestScenario::new()
            .expect("Failed to create test scenario")
            .setup_space("test_multi_hop")
            .exec_ddl("CREATE TAG person(name STRING)")
            .exec_ddl("CREATE EDGE follows()")
            .assert_success()
            .query("MATCH (a:person)-[:follows]->(b:person)-[:follows]->(c:person) RETURN a, c")
            .assert_success();
    }
}

// ==================== Aggregate Strategy Tests ====================

mod aggregate_strategy {
    use super::*;

    #[test]
    fn test_simple_aggregate() {
        TestScenario::new()
            .expect("Failed to create test scenario")
            .setup_space("test_simple_agg")
            .exec_ddl("CREATE TAG person(age INT)")
            .assert_success()
            .query("MATCH (n:person) RETURN count(n)")
            .assert_success();
    }

    #[test]
    fn test_grouped_aggregate() {
        TestScenario::new()
            .expect("Failed to create test scenario")
            .setup_space("test_grouped_agg")
            .exec_ddl("CREATE TAG person(city STRING, age INT)")
            .assert_success()
            .query("MATCH (n:person) RETURN n.city, count(n), avg(n.age)")
            .assert_success();
    }

    #[test]
    fn test_aggregate_with_filter() {
        TestScenario::new()
            .expect("Failed to create test scenario")
            .setup_space("test_agg_filter")
            .exec_ddl("CREATE TAG person(age INT)")
            .assert_success()
            .query("MATCH (n:person) WHERE n.age > 18 RETURN count(n)")
            .assert_success();
    }

    #[test]
    fn test_multiple_aggregates() {
        TestScenario::new()
            .expect("Failed to create test scenario")
            .setup_space("test_multi_agg")
            .exec_ddl("CREATE TAG person(age INT, salary INT)")
            .assert_success()
            .query("MATCH (n:person) RETURN count(n), avg(n.age), sum(n.salary), max(n.age), min(n.age)")
            .assert_success();
    }
}

// ==================== Plan Enumeration Tests ====================

mod plan_enumeration {
    use super::*;

    #[test]
    fn test_simple_plan_enumeration() {
        TestScenario::new()
            .expect("Failed to create test scenario")
            .setup_space("test_simple_enum")
            .exec_ddl("CREATE TAG person(name STRING)")
            .assert_success()
            .query("MATCH (n:person) RETURN n")
            .assert_success();
    }

    #[test]
    fn test_join_plan_enumeration() {
        TestScenario::new()
            .expect("Failed to create test scenario")
            .setup_space("test_join_enum")
            .exec_ddl("CREATE TAG person(name STRING)")
            .exec_ddl("CREATE TAG company(name STRING)")
            .exec_ddl("CREATE EDGE works_at()")
            .assert_success()
            .query("MATCH (p:person)-[:works_at]->(c:company) RETURN p, c")
            .assert_success();
    }

    #[test]
    fn test_complex_query_enumeration() {
        TestScenario::new()
            .expect("Failed to create test scenario")
            .setup_space("test_complex_enum")
            .exec_ddl("CREATE TAG person(name STRING, age INT)")
            .exec_ddl("CREATE TAG company(name STRING)")
            .exec_ddl("CREATE EDGE works_at()")
            .assert_success()
            .query("MATCH (p:person)-[:works_at]->(c:company) WHERE p.age > 25 RETURN p.name, c.name ORDER BY p.age LIMIT 10")
            .assert_success();
    }
}

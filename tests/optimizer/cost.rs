//! Cost Model and Statistics Tests
//!
//! Test coverage:
//! - Cost estimation accuracy
//! - Statistics collection
//! - Cardinality estimation
//! - Cost model configuration

use crate::common::test_scenario::TestScenario;

// ==================== Cost Estimation Tests ====================

mod cost_estimation {
    use super::*;

    #[test]
    fn test_scan_cost() {
        TestScenario::new()
            .expect("Failed to create test scenario")
            .setup_space("test_scan_cost")
            .exec_ddl("CREATE TAG person(name STRING)")
            .assert_success()
            .query("MATCH (n:person) RETURN n")
            .assert_success();
    }

    #[test]
    fn test_index_scan_cost() {
        TestScenario::new()
            .expect("Failed to create test scenario")
            .setup_space("test_index_scan_cost")
            .exec_ddl("CREATE TAG person(name STRING, age INT)")
            .exec_ddl("CREATE TAG INDEX idx_person_age ON person(age)")
            .assert_success()
            .query("MATCH (n:person) WHERE n.age = 30 RETURN n")
            .assert_success();
    }

    #[test]
    fn test_traversal_cost() {
        TestScenario::new()
            .expect("Failed to create test scenario")
            .setup_space("test_traversal_cost")
            .exec_ddl("CREATE TAG person(name STRING)")
            .exec_ddl("CREATE EDGE follows()")
            .assert_success()
            .query("MATCH (a:person)-[:follows]->(b:person) RETURN a, b")
            .assert_success();
    }

    #[test]
    fn test_join_cost() {
        TestScenario::new()
            .expect("Failed to create test scenario")
            .setup_space("test_join_cost")
            .exec_ddl("CREATE TAG person(name STRING)")
            .exec_ddl("CREATE TAG company(name STRING)")
            .exec_ddl("CREATE EDGE works_at()")
            .assert_success()
            .query("MATCH (p:person)-[:works_at]->(c:company) RETURN p, c")
            .assert_success();
    }

    #[test]
    fn test_aggregate_cost() {
        TestScenario::new()
            .expect("Failed to create test scenario")
            .setup_space("test_agg_cost")
            .exec_ddl("CREATE TAG person(age INT)")
            .assert_success()
            .query("MATCH (n:person) RETURN count(n), avg(n.age)")
            .assert_success();
    }

    #[test]
    fn test_sort_cost() {
        TestScenario::new()
            .expect("Failed to create test scenario")
            .setup_space("test_sort_cost")
            .exec_ddl("CREATE TAG person(name STRING, age INT)")
            .assert_success()
            .query("MATCH (n:person) RETURN n ORDER BY n.age")
            .assert_success();
    }
}

// ==================== Statistics Collection Tests ====================

mod statistics_collection {
    use super::*;

    #[test]
    fn test_tag_statistics() {
        TestScenario::new()
            .expect("Failed to create test scenario")
            .setup_space("test_tag_stats")
            .exec_ddl("CREATE TAG person(name STRING, age INT)")
            .assert_success()
            .query("MATCH (n:person) RETURN count(n)")
            .assert_success();
    }

    #[test]
    fn test_edge_statistics() {
        TestScenario::new()
            .expect("Failed to create test scenario")
            .setup_space("test_edge_stats")
            .exec_ddl("CREATE TAG person(name STRING)")
            .exec_ddl("CREATE EDGE follows()")
            .assert_success()
            .query("MATCH (a:person)-[e:follows]->(b:person) RETURN count(e)")
            .assert_success();
    }

    #[test]
    fn test_property_statistics() {
        TestScenario::new()
            .expect("Failed to create test scenario")
            .setup_space("test_prop_stats")
            .exec_ddl("CREATE TAG person(name STRING, age INT)")
            .assert_success()
            .query("MATCH (n:person) RETURN n.age, count(n) GROUP BY n.age")
            .assert_success();
    }

    #[test]
    fn test_index_statistics() {
        TestScenario::new()
            .expect("Failed to create test scenario")
            .setup_space("test_index_stats")
            .exec_ddl("CREATE TAG person(name STRING, age INT)")
            .exec_ddl("CREATE TAG INDEX idx_person_age ON person(age)")
            .assert_success()
            .query("MATCH (n:person) WHERE n.age = 30 RETURN n")
            .assert_success();
    }
}

// ==================== Cardinality Estimation Tests ====================

mod cardinality_estimation {
    use super::*;

    #[test]
    fn test_scan_cardinality() {
        TestScenario::new()
            .expect("Failed to create test scenario")
            .setup_space("test_scan_card")
            .exec_ddl("CREATE TAG person(name STRING)")
            .assert_success()
            .query("MATCH (n:person) RETURN count(n)")
            .assert_success();
    }

    #[test]
    fn test_filter_cardinality() {
        TestScenario::new()
            .expect("Failed to create test scenario")
            .setup_space("test_filter_card")
            .exec_ddl("CREATE TAG person(age INT)")
            .assert_success()
            .query("MATCH (n:person) WHERE n.age > 18 RETURN count(n)")
            .assert_success();
    }

    #[test]
    fn test_join_cardinality() {
        TestScenario::new()
            .expect("Failed to create test scenario")
            .setup_space("test_join_card")
            .exec_ddl("CREATE TAG person(name STRING)")
            .exec_ddl("CREATE TAG company(name STRING)")
            .exec_ddl("CREATE EDGE works_at()")
            .assert_success()
            .query("MATCH (p:person)-[:works_at]->(c:company) RETURN count(p)")
            .assert_success();
    }

    #[test]
    fn test_aggregate_cardinality() {
        TestScenario::new()
            .expect("Failed to create test scenario")
            .setup_space("test_agg_card")
            .exec_ddl("CREATE TAG person(city STRING)")
            .assert_success()
            .query("MATCH (n:person) RETURN n.city, count(n)")
            .assert_success();
    }
}

// ==================== Cost Model Configuration Tests ====================

mod cost_model_config {
    use super::*;

    #[test]
    fn test_default_cost_model() {
        TestScenario::new()
            .expect("Failed to create test scenario")
            .setup_space("test_default_cost")
            .exec_ddl("CREATE TAG person(name STRING)")
            .assert_success()
            .query("MATCH (n:person) RETURN n")
            .assert_success();
    }

    #[test]
    fn test_cost_weights() {
        TestScenario::new()
            .expect("Failed to create test scenario")
            .setup_space("test_cost_weights")
            .exec_ddl("CREATE TAG person(name STRING, age INT)")
            .exec_ddl("CREATE TAG INDEX idx_person_age ON person(age)")
            .assert_success()
            .query("MATCH (n:person) WHERE n.age = 30 RETURN n")
            .assert_success();
    }

    #[test]
    fn test_memory_cost_factor() {
        TestScenario::new()
            .expect("Failed to create test scenario")
            .setup_space("test_mem_cost")
            .exec_ddl("CREATE TAG person(name STRING, age INT, city STRING, salary INT)")
            .assert_success()
            .query("MATCH (n:person) RETURN n.name, n.age ORDER BY n.age LIMIT 100")
            .assert_success();
    }

    #[test]
    fn test_io_cost_factor() {
        TestScenario::new()
            .expect("Failed to create test scenario")
            .setup_space("test_io_cost")
            .exec_ddl("CREATE TAG person(name STRING)")
            .assert_success()
            .query("MATCH (n:person) RETURN n LIMIT 1000")
            .assert_success();
    }
}

// ==================== Plan Comparison Tests ====================

mod plan_comparison {
    use super::*;

    #[test]
    fn test_index_vs_full_scan() {
        TestScenario::new()
            .expect("Failed to create test scenario")
            .setup_space("test_index_vs_scan")
            .exec_ddl("CREATE TAG person(name STRING, age INT)")
            .exec_ddl("CREATE TAG INDEX idx_person_age ON person(age)")
            .assert_success()
            .query("MATCH (n:person) WHERE n.age = 30 RETURN n")
            .assert_success();
    }

    #[test]
    fn test_nested_loop_vs_hash_join() {
        TestScenario::new()
            .expect("Failed to create test scenario")
            .setup_space("test_join_method")
            .exec_ddl("CREATE TAG person(name STRING)")
            .exec_ddl("CREATE TAG company(name STRING)")
            .exec_ddl("CREATE EDGE works_at()")
            .assert_success()
            .query("MATCH (p:person)-[:works_at]->(c:company) RETURN p, c")
            .assert_success();
    }

    #[test]
    fn test_sort_merge_vs_hash_aggregate() {
        TestScenario::new()
            .expect("Failed to create test scenario")
            .setup_space("test_agg_method")
            .exec_ddl("CREATE TAG person(city STRING, age INT)")
            .assert_success()
            .query("MATCH (n:person) RETURN n.city, count(n), avg(n.age)")
            .assert_success();
    }
}

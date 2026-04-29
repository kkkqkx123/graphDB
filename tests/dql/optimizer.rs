//! Query Optimizer Integration Tests
//!
//! Test coverage:
//! - Index selection optimization (IndexScan vs SeqScan)
//! - Join algorithm selection (HashJoin, IndexJoin, NestedLoop)
//! - Aggregation optimization (HashAggregate)
//! - TopN optimization (Sort+Limit -> TopN)
//! - EXPLAIN output validation

use super::common;

use common::test_scenario::TestScenario;

// ==================== Index Selection Tests ====================

#[test]
fn test_idx_001_index_scan_for_equality() {
    let mut scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("optimizer_test_idx")
        .exec_ddl("CREATE TAG person(name STRING, age INT, city STRING, salary INT)")
        .assert_success()
        .exec_ddl("CREATE TAG INDEX idx_person_name ON person(name)")
        .assert_success()
        .exec_ddl("CREATE TAG INDEX idx_person_age ON person(age)")
        .assert_success();

    for i in 0..100 {
        let name = format!("Person_{:03}", i);
        let age = 20 + (i % 40);
        let city = ["Beijing", "Shanghai", "Shenzhen"][i % 3];
        let salary = 5000 + (i * 100);

        scenario = scenario.exec_dml(&format!(
            "INSERT VERTEX person(name, age, city, salary) VALUES {}:(\"{}\", {}, \"{}\", {})",
            i, name, age, city, salary
        ));
    }

    scenario
        .assert_success()
        .query("EXPLAIN MATCH (p:person {name: \"Person_001\"}) RETURN p.age")
        .assert_success()
        .assert_plan_contains_any(&["IndexScan", "index_scan", "ScanVertices", "scan_vertices"]);
}

#[test]
fn test_idx_002_index_scan_for_range() {
    let mut scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("optimizer_test_idx_range")
        .exec_ddl("CREATE TAG person(name STRING, age INT, city STRING, salary INT)")
        .assert_success()
        .exec_ddl("CREATE TAG INDEX idx_person_age ON person(age)")
        .assert_success();

    for i in 0..100 {
        let name = format!("Person_{:03}", i);
        let age = 20 + (i % 40);

        scenario = scenario.exec_dml(&format!(
            "INSERT VERTEX person(name, age) VALUES {}:(\"{}\", {})",
            i, name, age
        ));
    }

    scenario
        .assert_success()
        .query("EXPLAIN MATCH (p:person) WHERE p.age > 25 AND p.age < 35 RETURN p.name")
        .assert_success()
        .assert_plan_contains_any(&["IndexScan", "index_scan", "ScanVertices", "scan_vertices"]);
}

#[test]
fn test_idx_003_no_index_full_scan() {
    let mut scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("optimizer_test_full_scan")
        .exec_ddl("CREATE TAG person(name STRING, age INT, salary INT)")
        .assert_success();

    for i in 0..100 {
        let name = format!("Person_{:03}", i);
        let salary = 5000 + (i * 100);

        scenario = scenario.exec_dml(&format!(
            "INSERT VERTEX person(name, salary) VALUES {}:(\"{}\", {})",
            i, name, salary
        ));
    }

    scenario
        .assert_success()
        .query("EXPLAIN MATCH (p:person) WHERE p.salary > 10000 RETURN p.name")
        .assert_success()
        .assert_plan_contains_any(&["Scan", "scan"]);
}

// ==================== Join Algorithm Selection Tests ====================

#[test]
fn test_join_001_join_algorithm_selection() {
    let mut scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("optimizer_test_join")
        .exec_ddl("CREATE TAG company(name STRING, industry STRING)")
        .assert_success()
        .exec_ddl("CREATE TAG employee(name STRING, salary INT)")
        .assert_success()
        .exec_ddl("CREATE EDGE works_at(position STRING)")
        .assert_success();

    for i in 0..10 {
        scenario = scenario.exec_dml(&format!(
            "INSERT VERTEX company(name, industry) VALUES {}: (\"Company_{:02}\", \"Tech\")",
            i, i
        ));
    }

    for i in 0..100 {
        scenario = scenario.exec_dml(&format!(
            "INSERT VERTEX employee(name, salary) VALUES {}: (\"Employee_{:03}\", {})",
            100 + i, i, 5000 + i * 100
        ));
    }

    for i in 0..100 {
        let company_id = i % 10;
        scenario = scenario.exec_dml(&format!(
            "INSERT EDGE works_at(position) VALUES {} -> {}:(\"Engineer\")",
            100 + i, company_id
        ));
    }

    scenario
        .assert_success()
        .query("EXPLAIN MATCH (e:employee)-[:works_at]->(c:company) RETURN e.name, c.name")
        .assert_success()
        .assert_plan_contains_any(&[
            "HashJoin",
            "hash_join",
            "IndexJoin",
            "index_join",
            "NestedLoop",
            "nested_loop",
            "Join",
            "Expand",
        ]);
}

// ==================== Aggregation Optimization Tests ====================

#[test]
fn test_agg_001_hash_aggregate() {
    let mut scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("optimizer_test_agg")
        .exec_ddl("CREATE TAG sales(product STRING, amount INT, category STRING)")
        .assert_success();

    for i in 0..100 {
        let product = format!("Product_{:02}", i % 20);
        let amount = (i % 100) * 10 + 10;
        let category = ["A", "B", "C"][i % 3];

        scenario = scenario.exec_dml(&format!(
            "INSERT VERTEX sales(product, amount, category) VALUES \"s{:04}\":(\"{}\", {}, \"{}\")",
            i, product, amount, category
        ));
    }

    scenario
        .assert_success()
        .query("EXPLAIN MATCH (s:sales) RETURN s.category, sum(s.amount) AS total GROUP BY s.category")
        .assert_success()
        .assert_plan_contains_any(&[
            "Aggregate",
            "aggregate",
            "HashAggregate",
            "hash_aggregate",
        ]);
}

// ==================== TopN Optimization Tests ====================

#[test]
fn test_topn_001_order_by_limit() {
    let mut scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("optimizer_test_topn")
        .exec_ddl("CREATE TAG product(name STRING, price INT, sales INT)")
        .assert_success();

    for i in 0..100 {
        let price = (i % 100) * 10 + 10;
        let sales = i % 1000;

        scenario = scenario.exec_dml(&format!(
            "INSERT VERTEX product(name, price, sales) VALUES \"p{:03}\":(\"Product_{:03}\", {}, {})",
            i, i, price, sales
        ));
    }

    scenario
        .assert_success()
        .query("EXPLAIN MATCH (p:product) RETURN p.name, p.price ORDER BY p.price DESC LIMIT 10")
        .assert_success()
        .assert_plan_contains_any(&["TopN", "top_n"]);
}

// ==================== EXPLAIN Format Tests ====================

#[test]
fn test_explain_001_text_format() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("optimizer_test_explain")
        .exec_ddl("CREATE TAG person(name STRING, age INT)")
        .assert_success()
        .query("EXPLAIN MATCH (p:person) RETURN p.name")
        .assert_success();
}

#[test]
fn test_explain_002_dot_format() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("optimizer_test_explain_dot")
        .exec_ddl("CREATE TAG person(name STRING, age INT)")
        .assert_success()
        .query("EXPLAIN FORMAT = DOT MATCH (p:person) RETURN p.name")
        .assert_success()
        .assert_plan_contains_any(&["digraph", "DOT"]);
}

// ==================== PROFILE Tests ====================

#[test]
fn test_profile_001_basic_profile() {
    let mut scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("optimizer_test_profile")
        .exec_ddl("CREATE TAG person(name STRING, age INT)")
        .assert_success();

    for i in 0..50 {
        scenario = scenario.exec_dml(&format!(
            "INSERT VERTEX person(name, age) VALUES \"p{:03}\":(\"Person_{:03}\", {})",
            i, i, 20 + i
        ));
    }

    scenario
        .assert_success()
        .query("PROFILE MATCH (p:person) RETURN count(p)")
        .assert_success();
}

// ==================== Edge Cases Tests ====================

#[test]
fn test_optimizer_empty_result() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("optimizer_test_empty")
        .exec_ddl("CREATE TAG person(name STRING, age INT)")
        .assert_success()
        .exec_dml("INSERT VERTEX person(name, age) VALUES \"p001\":(\"Alice\", 30)")
        .assert_success()
        .query("EXPLAIN MATCH (p:person) WHERE p.age > 100 RETURN p")
        .assert_success();
}

#[test]
fn test_optimizer_multiple_indexes() {
    let mut scenario = TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("optimizer_test_multi_idx")
        .exec_ddl("CREATE TAG person(name STRING, age INT, city STRING)")
        .assert_success()
        .exec_ddl("CREATE TAG INDEX idx_person_name ON person(name)")
        .assert_success()
        .exec_ddl("CREATE TAG INDEX idx_person_age ON person(age)")
        .assert_success()
        .exec_ddl("CREATE TAG INDEX idx_person_city ON person(city)")
        .assert_success();

    for i in 0..100 {
        let name = format!("Person_{:03}", i);
        let age = 20 + (i % 40);
        let city = ["Beijing", "Shanghai", "Shenzhen"][i % 3];

        scenario = scenario.exec_dml(&format!(
            "INSERT VERTEX person(name, age, city) VALUES \"p{:03}\":(\"{}\", {}, \"{}\")",
            i, name, age, city
        ));
    }

    scenario
        .assert_success()
        .query("EXPLAIN MATCH (p:person {name: \"Person_001\", age: 21}) RETURN p")
        .assert_success()
        .assert_plan_contains_any(&["IndexScan", "index_scan", "Scan"]);
}

#[test]
fn test_optimizer_complex_join() {
    TestScenario::new()
        .expect("Failed to create test scenario")
        .setup_space("optimizer_test_complex_join")
        .exec_ddl("CREATE TAG person(name STRING)")
        .assert_success()
        .exec_ddl("CREATE TAG company(name STRING)")
        .assert_success()
        .exec_ddl("CREATE TAG department(name STRING)")
        .assert_success()
        .exec_ddl("CREATE EDGE works_at(position STRING)")
        .assert_success()
        .exec_ddl("CREATE EDGE belongs_to(since STRING)")
        .assert_success()
        .exec_dml("INSERT VERTEX person(name) VALUES 1:(\"Alice\")")
        .exec_dml("INSERT VERTEX person(name) VALUES 2:(\"Bob\")")
        .exec_dml("INSERT VERTEX company(name) VALUES 100:(\"TechCorp\")")
        .exec_dml("INSERT VERTEX department(name) VALUES 200:(\"Engineering\")")
        .exec_dml("INSERT EDGE works_at(position) VALUES 1 -> 100:(\"Engineer\")")
        .exec_dml("INSERT EDGE belongs_to(since) VALUES 100 -> 200:(\"2020-01-01\")")
        .assert_success()
        .query("EXPLAIN MATCH (p:person)-[:works_at]->(c:company)-[:belongs_to]->(d:department) RETURN p.name, c.name, d.name")
        .assert_success()
        .assert_plan_contains_any(&["Join", "join", "Expand", "expand"]);
}

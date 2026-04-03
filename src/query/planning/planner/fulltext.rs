//! Full-Text Search Planner
//!
//! This module implements the execution plan generator for full-text search queries,
//! converting AST nodes into execution plans.

use std::sync::Arc;

use crate::query::parser::ast::{
    AlterFulltextIndex, CreateFulltextIndex, DescribeFulltextIndex, DropFulltextIndex,
    FulltextMatchCondition, LookupFulltext, MatchFulltext, SearchStatement, ShowFulltextIndex,
    Stmt,
};
use crate::query::planning::plan::{ExecutionPlan, SubPlan};
use crate::query::planning::planner::{Planner, PlannerError};
use crate::query::validator::ValidatedStatement;
use crate::query::QueryContext;

/// Full-text search planner
#[derive(Debug)]
pub struct FulltextSearchPlanner {
    enabled: bool,
}

impl Default for FulltextSearchPlanner {
    fn default() -> Self {
        Self::new()
    }
}

impl FulltextSearchPlanner {
    /// Create a new full-text search planner
    pub fn new() -> Self {
        Self { enabled: true }
    }
}

impl Planner for FulltextSearchPlanner {
    fn transform(
        &mut self,
        validated: &ValidatedStatement,
        qctx: Arc<QueryContext>,
    ) -> Result<SubPlan, PlannerError> {
        let stmt = validated.ast();

        match stmt {
            Stmt::CreateFulltextIndex(create) => {
                self.transform_create_index(create, qctx)
            }
            Stmt::DropFulltextIndex(drop) => {
                self.transform_drop_index(drop, qctx)
            }
            Stmt::AlterFulltextIndex(alter) => {
                self.transform_alter_index(alter, qctx)
            }
            Stmt::ShowFulltextIndex(show) => {
                self.transform_show_index(show, qctx)
            }
            Stmt::DescribeFulltextIndex(describe) => {
                self.transform_describe_index(describe, qctx)
            }
            Stmt::Search(search) => {
                self.transform_search(search, qctx)
            }
            Stmt::LookupFulltext(lookup) => {
                self.transform_lookup(lookup, qctx)
            }
            Stmt::MatchFulltext(match_stmt) => {
                self.transform_match(match_stmt, qctx)
            }
            _ => Err(PlannerError::new(
                crate::query::planning::planner::PlannerErrorType::PlanningFailed,
                "Not a full-text search statement".to_string(),
            )),
        }
    }

    fn match_planner(&self, stmt: &Stmt) -> bool {
        matches!(
            stmt,
            Stmt::CreateFulltextIndex(_)
                | Stmt::DropFulltextIndex(_)
                | Stmt::AlterFulltextIndex(_)
                | Stmt::ShowFulltextIndex(_)
                | Stmt::DescribeFulltextIndex(_)
                | Stmt::Search(_)
                | Stmt::LookupFulltext(_)
                | Stmt::MatchFulltext(_)
        )
    }
}

impl FulltextSearchPlanner {
    /// Transform CREATE FULLTEXT INDEX statement
    fn transform_create_index(
        &self,
        create: &CreateFulltextIndex,
        qctx: Arc<QueryContext>,
    ) -> Result<SubPlan, PlannerError> {
        // Create index execution plan
        let plan = crate::query::planning::plan::Operator::CreateFulltextIndex {
            index_name: create.index_name.clone(),
            schema_name: create.schema_name.clone(),
            fields: create.fields.clone(),
            engine_type: create.engine_type,
            options: create.options.clone(),
            if_not_exists: create.if_not_exists,
        };

        let mut sub_plan = SubPlan::new();
        sub_plan.set_root(plan);
        Ok(sub_plan)
    }

    /// Transform DROP FULLTEXT INDEX statement
    fn transform_drop_index(
        &self,
        drop: &DropFulltextIndex,
        _qctx: Arc<QueryContext>,
    ) -> Result<SubPlan, PlannerError> {
        let plan = crate::query::planning::plan::Operator::DropFulltextIndex {
            index_name: drop.index_name.clone(),
            if_exists: drop.if_exists,
        };

        let mut sub_plan = SubPlan::new();
        sub_plan.set_root(plan);
        Ok(sub_plan)
    }

    /// Transform ALTER FULLTEXT INDEX statement
    fn transform_alter_index(
        &self,
        alter: &AlterFulltextIndex,
        _qctx: Arc<QueryContext>,
    ) -> Result<SubPlan, PlannerError> {
        let plan = crate::query::planning::plan::Operator::AlterFulltextIndex {
            index_name: alter.index_name.clone(),
            actions: alter.actions.clone(),
        };

        let mut sub_plan = SubPlan::new();
        sub_plan.set_root(plan);
        Ok(sub_plan)
    }

    /// Transform SHOW FULLTEXT INDEX statement
    fn transform_show_index(
        &self,
        show: &ShowFulltextIndex,
        _qctx: Arc<QueryContext>,
    ) -> Result<SubPlan, PlannerError> {
        let plan = crate::query::planning::plan::Operator::ShowFulltextIndex {
            pattern: show.pattern.clone(),
            from_schema: show.from_schema.clone(),
        };

        let mut sub_plan = SubPlan::new();
        sub_plan.set_root(plan);
        Ok(sub_plan)
    }

    /// Transform DESCRIBE FULLTEXT INDEX statement
    fn transform_describe_index(
        &self,
        describe: &DescribeFulltextIndex,
        _qctx: Arc<QueryContext>,
    ) -> Result<SubPlan, PlannerError> {
        let plan = crate::query::planning::plan::Operator::DescribeFulltextIndex {
            index_name: describe.index_name.clone(),
        };

        let mut sub_plan = SubPlan::new();
        sub_plan.set_root(plan);
        Ok(sub_plan)
    }

    /// Transform SEARCH statement
    fn transform_search(
        &self,
        search: &SearchStatement,
        qctx: Arc<QueryContext>,
    ) -> Result<SubPlan, PlannerError> {
        // Create full-text search executor plan
        let plan = crate::query::planning::plan::Operator::FulltextSearch {
            index_name: search.index_name.clone(),
            query: search.query.clone(),
            yield_clause: search.yield_clause.clone(),
            where_clause: search.where_clause.clone(),
            order_clause: search.order_clause.clone(),
            limit: search.limit,
            offset: search.offset,
        };

        let mut sub_plan = SubPlan::new();
        sub_plan.set_root(plan);
        Ok(sub_plan)
    }

    /// Transform LOOKUP FULLTEXT statement
    fn transform_lookup(
        &self,
        lookup: &LookupFulltext,
        _qctx: Arc<QueryContext>,
    ) -> Result<SubPlan, PlannerError> {
        let plan = crate::query::planning::plan::Operator::FulltextLookup {
            schema_name: lookup.schema_name.clone(),
            index_name: lookup.index_name.clone(),
            query: lookup.query.clone(),
            yield_clause: lookup.yield_clause.clone(),
            limit: lookup.limit,
        };

        let mut sub_plan = SubPlan::new();
        sub_plan.set_root(plan);
        Ok(sub_plan)
    }

    /// Transform MATCH with full-text statement
    fn transform_match(
        &self,
        match_stmt: &MatchFulltext,
        _qctx: Arc<QueryContext>,
    ) -> Result<SubPlan, PlannerError> {
        // MATCH with full-text requires special handling
        // It combines graph pattern matching with full-text search
        let plan = crate::query::planning::plan::Operator::MatchFulltext {
            pattern: match_stmt.pattern.clone(),
            fulltext_condition: match_stmt.fulltext_condition.clone(),
            yield_clause: match_stmt.yield_clause.clone(),
        };

        let mut sub_plan = SubPlan::new();
        sub_plan.set_root(plan);
        Ok(sub_plan)
    }
}

/// Helper function to register full-text search planner
pub fn register_fulltext_planner(registry: &mut crate::query::planning::planner::PlannerRegistry) {
    registry.register(Box::new(FulltextSearchPlanner::new()));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::parser::ast::{FulltextQueryExpr, IndexFieldDef};

    #[test]
    fn test_match_planner() {
        let planner = FulltextSearchPlanner::new();

        let search_stmt = Stmt::Search(SearchStatement::new(
            "test_index".to_string(),
            FulltextQueryExpr::Simple("test".to_string()),
        ));

        assert!(planner.match_planner(&search_stmt));

        let create_stmt = Stmt::CreateFulltextIndex(CreateFulltextIndex::new(
            "idx_test".to_string(),
            "schema".to_string(),
            vec![IndexFieldDef::new("field".to_string())],
            crate::core::types::FulltextEngineType::Bm25,
        ));

        assert!(planner.match_planner(&create_stmt));
    }
}

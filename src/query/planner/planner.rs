//! Main planner trait and implementation
use super::plan::SubPlan;
use crate::query::context::AstContext;

// Match function type - takes AstContext and returns whether the planner matches
pub type MatchFunc = fn(&AstContext) -> bool;

// Planner instantiation function type - returns a new planner instance
pub type PlannerInstantiateFunc = fn() -> Box<dyn Planner>;

// Structure that combines match function and planner instantiate function
#[derive(Debug, Clone)]
pub struct MatchAndInstantiate {
    pub match_func: MatchFunc,
    pub instantiate_func: PlannerInstantiateFunc,
}

// Main planner trait that all planners implement
pub trait Planner: std::fmt::Debug {
    // The main transformation function that converts an AST context to an execution plan
    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError>;

    // Match function for this planner - whether it can handle the given AST context
    fn match_planner(&self, ast_ctx: &AstContext) -> bool;
}

// Sequential planner that handles sequential execution of statements
#[derive(Debug)]
pub struct SequentialPlanner {
    #[allow(dead_code)]
    planners: Vec<MatchAndInstantiate>,
}

impl SequentialPlanner {
    pub fn new() -> Self {
        Self {
            planners: Vec::new(),
        }
    }

    pub fn make() -> Box<dyn Planner> {
        Box::new(Self::new())
    }

    pub fn match_ast_ctx(_ast_ctx: &AstContext) -> bool {
        // For sequential planner, we generally match any statement
        // In a real implementation, we might check if the statement type is appropriate
        true
    }

    // Converts an AST context to a plan (similar to the original Nebula toPlan method)
    pub fn to_plan(ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        // Select the appropriate planner based on the AST context and call its transform method
        let mut planners_registry = PlannersRegistry::new();
        Self::register_planners(&mut planners_registry);

        planners_registry.create_plan(ast_ctx)
    }

    // Register all available planners in the registry
    pub fn register_planners(registry: &mut PlannersRegistry) {
        // Register match planner
        registry.add_planner(
            "MATCH".to_string(),
            MatchAndInstantiate {
                match_func: crate::query::planner::match_planner::MatchPlanner::match_ast_ctx,
                instantiate_func: crate::query::planner::match_planner::MatchPlanner::make,
            },
        );

        // Register go planner
        registry.add_planner(
            "GO".to_string(),
            MatchAndInstantiate {
                match_func: crate::query::planner::go_planner::GoPlanner::match_ast_ctx,
                instantiate_func: crate::query::planner::go_planner::GoPlanner::make,
            },
        );

        // Register lookup planner
        registry.add_planner(
            "LOOKUP".to_string(),
            MatchAndInstantiate {
                match_func: crate::query::planner::lookup_planner::LookupPlanner::match_ast_ctx,
                instantiate_func: crate::query::planner::lookup_planner::LookupPlanner::make,
            },
        );

        // Register path planner
        registry.add_planner(
            "PATH".to_string(),
            MatchAndInstantiate {
                match_func: crate::query::planner::path_planner::PathPlanner::match_ast_ctx,
                instantiate_func: crate::query::planner::path_planner::PathPlanner::make,
            },
        );

        // Register subgraph planner
        registry.add_planner(
            "SUBGRAPH".to_string(),
            MatchAndInstantiate {
                match_func: crate::query::planner::subgraph_planner::SubgraphPlanner::match_ast_ctx,
                instantiate_func: crate::query::planner::subgraph_planner::SubgraphPlanner::make,
            },
        );
    }
}

impl Planner for SequentialPlanner {
    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        Self::to_plan(ast_ctx)
    }

    fn match_planner(&self, _ast_ctx: &AstContext) -> bool {
        Self::match_ast_ctx(_ast_ctx)
    }
}

// Planner registry that keeps track of all available planners
#[derive(Debug)]
pub struct PlannersRegistry {
    planners: std::collections::HashMap<String, Vec<MatchAndInstantiate>>,
}

impl PlannersRegistry {
    pub fn new() -> Self {
        Self {
            planners: std::collections::HashMap::new(),
        }
    }

    // Add a planner to the registry for a specific statement type
    pub fn add_planner(&mut self, stmt_type: String, match_and_instantiate: MatchAndInstantiate) {
        self.planners
            .entry(stmt_type)
            .or_default()
            .push(match_and_instantiate);
    }

    // Get all planners that match the given AST context
    pub fn get_matching_planners(&self, _ast_ctx: &AstContext) -> Vec<&MatchAndInstantiate> {
        // In a real implementation, this would check the AST context to determine
        // the statement type and return appropriate planners
        self.planners.values().flatten().collect()
    }

    // Create an execution plan using the registry
    pub fn create_plan(&self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        let matching_planners = self.get_matching_planners(ast_ctx);

        for planner_info in matching_planners {
            if (planner_info.match_func)(ast_ctx) {
                let mut planner = (planner_info.instantiate_func)();
                if planner.match_planner(ast_ctx) {
                    return planner.transform(ast_ctx);
                }
            }
        }

        Err(PlannerError::NoSuitablePlanner(
            "No suitable planner found for the given AST context".to_string(),
        ))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PlannerError {
    #[error("No suitable planner found: {0}")]
    NoSuitablePlanner(String),

    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),

    #[error("Plan generation failed: {0}")]
    PlanGenerationFailed(String),

    #[error("Invalid AST context: {0}")]
    InvalidAstContext(String),
}

# Code Analysis: EXPLAIN with LOOKUP - Schema Manager Not Available

## Problem Summary

When executing EXPLAIN with LOOKUP query that uses an index, the query fails with "Schema manager not available" error. This occurs during the validation phase of the LOOKUP statement within EXPLAIN.

## Error Location

**Error Message**: `Schema manager not available`
**Error Source**: `src/query/validator/statements/lookup_validator.rs:185`

```rust
let schema_manager = self.schema_manager.as_ref().ok_or_else(|| {
    ValidationError::new(
        "Schema manager not available".to_string(),
        ValidationErrorType::SemanticError,
    )
})?;
```

## Code Flow Analysis

### 1. EXPLAIN Validator

**File**: `src/query/validator/utility/explain_validator.rs`

The `ExplainValidator` handles EXPLAIN statements by creating an inner validator for the wrapped statement:

```rust
fn validate_impl(&mut self, stmt: &ExplainStmt) -> Result<(), ValidationError> {
    self.format = stmt.format.clone();

    let mut inner_validator = Validator::create_from_stmt(&stmt.statement).ok_or_else(|| {
        ValidationError::new(
            "Failed to create validator for inner statement".to_string(),
            ValidationErrorType::SemanticError,
        )
    })?;

    // Propagate schema_manager to inner validator
    if let Some(ref sm) = self.schema_manager {
        inner_validator.set_schema_manager(sm.clone());
    }

    self.inner_validator = Some(Box::new(inner_validator));
    Ok(())
}
```

**Key Observation**: The `ExplainValidator` has a `set_schema_manager` method and propagates the schema manager to the inner validator. However, the issue may occur during the actual `validate()` call.

### 2. EXPLAIN Validation Process

**File**: `src/query/validator/utility/explain_validator.rs` (lines 140-170)

```rust
fn validate(
    &mut self,
    ast: Arc<Ast>,
    qctx: Arc<QueryContext>,
) -> Result<ValidationResult, ValidationError> {
    let explain_stmt = match &ast.stmt {
        crate::query::parser::ast::Stmt::Explain(explain_stmt) => explain_stmt,
        _ => { /* ... */ }
    };

    // Extract the internal statements
    let inner_stmt = *explain_stmt.statement.clone();

    self.validate_impl(explain_stmt)?;

    // Verify the internal statements
    if let Some(ref mut inner) = self.inner_validator {
        let result = inner.validate(
            Arc::new(Ast::new(inner_stmt, ast.expr_context.clone())),
            qctx,
        );
        // ...
    }
    // ...
}
```

### 3. LOOKUP Validator

**File**: `src/query/validator/statements/lookup_validator.rs`

The `LookupValidator` requires a schema manager to validate the LOOKUP target:

```rust
pub struct LookupValidator {
    // ...
    schema_manager: Option<Arc<RedbSchemaManager>>,
}

impl LookupValidator {
    pub fn new() -> Self {
        Self {
            // ...
            schema_manager: None,
        }
    }

    pub fn set_schema_manager(&mut self, schema_manager: Arc<RedbSchemaManager>) {
        self.schema_manager = Some(schema_manager);
    }
}
```

The schema manager is used in `validate_lookup_target()`:

```rust
fn validate_lookup_target(
    &self,
    space_name: &str,
    label: &str,
    is_edge: bool,
    target_type_specified: bool,
) -> Result<(LookupIndexType, bool), ValidationError> {
    // Check whether schema_manager is available
    let schema_manager = self.schema_manager.as_ref().ok_or_else(|| {
        ValidationError::new(
            "Schema manager not available".to_string(),
            ValidationErrorType::SemanticError,
        )
    })?;
    // ...
}
```

## Root Cause Analysis

The issue appears to be in how the schema manager is propagated through the validation chain:

1. **ExplainValidator** receives the schema manager via `set_schema_manager()`
2. **ExplainValidator.validate_impl()** propagates it to the inner validator (e.g., `LookupValidator`)
3. **However**, there may be a timing issue or the propagation may not work correctly for certain validator types

**Potential Causes**:

1. **Validator Creation Timing**: The `Validator::create_from_stmt()` may create a `LookupValidator` that doesn't properly receive the schema manager

2. **Missing set_schema_manager Call**: The `Validator` enum wrapper may not properly delegate `set_schema_manager` to the inner validator

3. **Validation Order**: The schema manager may not be set before `validate()` is called on the inner validator

## Key Code Locations

| File | Line | Description |
|------|------|-------------|
| `explain_validator.rs` | 85-95 | `validate_impl()` propagates schema_manager |
| `explain_validator.rs` | 140-170 | `validate()` calls inner validator |
| `lookup_validator.rs` | 175-195 | `validate_lookup_target()` requires schema_manager |
| `lookup_validator.rs` | 45-65 | `set_schema_manager()` method |

## Potential Fix

### Option 1: Verify Validator Enum Delegation

Check if the `Validator` enum properly delegates `set_schema_manager` to all inner validator types:

```rust
// In validator_enum.rs
pub enum Validator {
    Lookup(LookupValidator),
    // ...
}

impl Validator {
    pub fn set_schema_manager(&mut self, sm: Arc<RedbSchemaManager>) {
        match self {
            Validator::Lookup(v) => v.set_schema_manager(sm),
            // ... ensure all variants are handled
        }
    }
}
```

### Option 2: Ensure Schema Manager is Set Before Validation

Add checks to ensure the schema manager is always available before validation:

```rust
fn validate_lookup_target(&self, ...) -> Result<...> {
    let schema_manager = self.schema_manager.as_ref().ok_or_else(|| {
        log::error!("Schema manager not set in LookupValidator");
        ValidationError::new(
            "Schema manager not available".to_string(),
            ValidationErrorType::SemanticError,
        )
    })?;
    // ...
}
```

### Option 3: Lazy Schema Manager Initialization

Instead of requiring the schema manager at validation time, consider passing it through the `QueryContext` or making it available globally during the validation phase.

## Related Files

- `src/query/validator/utility/explain_validator.rs` - EXPLAIN validation
- `src/query/validator/statements/lookup_validator.rs` - LOOKUP validation
- `src/query/validator/validator_enum.rs` - Validator enum wrapper

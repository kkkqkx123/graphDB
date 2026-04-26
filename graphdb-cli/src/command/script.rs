use std::collections::HashMap;

use crate::utils::error::{CliError, Result};

#[derive(Debug, Clone)]
pub struct ParsedStatement {
    pub content: String,
    pub start_line: usize,
    pub end_line: usize,
    pub kind: StatementKind,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StatementKind {
    Query,
    MetaCommand,
}

pub struct ScriptParser;

impl ScriptParser {
    pub fn parse(content: &str) -> Vec<ParsedStatement> {
        let mut statements = Vec::new();
        let mut current = String::new();
        let mut line_number = 1;
        let mut start_line = 1;
        let mut parser = StatementBalanceTracker::new();

        for line in content.lines() {
            let trimmed = line.trim();

            if trimmed.is_empty() {
                if !current.is_empty() {
                    current.push('\n');
                }
                line_number += 1;
                continue;
            }

            if trimmed.starts_with("--") || trimmed.starts_with("//") {
                line_number += 1;
                continue;
            }

            if trimmed.starts_with('\\') {
                if !current.trim().is_empty() {
                    statements.push(ParsedStatement {
                        content: current.trim().to_string(),
                        start_line,
                        end_line: line_number - 1,
                        kind: StatementKind::Query,
                    });
                    current.clear();
                    parser = StatementBalanceTracker::new();
                }

                statements.push(ParsedStatement {
                    content: trimmed.to_string(),
                    start_line: line_number,
                    end_line: line_number,
                    kind: StatementKind::MetaCommand,
                });

                line_number += 1;
                start_line = line_number;
                continue;
            }

            if current.is_empty() {
                start_line = line_number;
            } else {
                current.push('\n');
            }
            current.push_str(line);

            for ch in line.chars() {
                parser.feed(ch);
            }

            if parser.is_balanced() && trimmed.ends_with(';') {
                statements.push(ParsedStatement {
                    content: current.trim().to_string(),
                    start_line,
                    end_line: line_number,
                    kind: StatementKind::Query,
                });
                current.clear();
                parser = StatementBalanceTracker::new();
                start_line = line_number + 1;
            }

            line_number += 1;
        }

        if !current.trim().is_empty() {
            statements.push(ParsedStatement {
                content: current.trim().to_string(),
                start_line,
                end_line: line_number - 1,
                kind: StatementKind::Query,
            });
        }

        statements
    }
}

struct StatementBalanceTracker {
    in_single_line_comment: bool,
    in_multi_line_comment: bool,
    in_single_quote: bool,
    in_double_quote: bool,
    paren_depth: i32,
    brace_depth: i32,
    bracket_depth: i32,
    prev_char: Option<char>,
}

impl StatementBalanceTracker {
    fn new() -> Self {
        Self {
            in_single_line_comment: false,
            in_multi_line_comment: false,
            in_single_quote: false,
            in_double_quote: false,
            paren_depth: 0,
            brace_depth: 0,
            bracket_depth: 0,
            prev_char: None,
        }
    }

    fn feed(&mut self, ch: char) {
        if self.in_single_line_comment {
            if ch == '\n' {
                self.in_single_line_comment = false;
            }
            self.prev_char = Some(ch);
            return;
        }

        if self.in_multi_line_comment {
            if ch == '/' && self.prev_char == Some('*') {
                self.in_multi_line_comment = false;
            }
            self.prev_char = Some(ch);
            return;
        }

        if !self.in_single_quote && !self.in_double_quote {
            if ch == '-' && self.prev_char == Some('-') {
                self.in_single_line_comment = true;
                self.prev_char = Some(ch);
                return;
            }
            if ch == '*' && self.prev_char == Some('/') {
                self.in_multi_line_comment = true;
                self.prev_char = Some(ch);
                return;
            }
        }

        match ch {
            '\'' if !self.in_double_quote => {
                self.in_single_quote = !self.in_single_quote;
            }
            '"' if !self.in_single_quote => {
                self.in_double_quote = !self.in_double_quote;
            }
            '(' if !self.in_any_string() => self.paren_depth += 1,
            ')' if !self.in_any_string() => self.paren_depth = (self.paren_depth - 1).max(0),
            '{' if !self.in_any_string() => self.brace_depth += 1,
            '}' if !self.in_any_string() => self.brace_depth = (self.brace_depth - 1).max(0),
            '[' if !self.in_any_string() => self.bracket_depth += 1,
            ']' if !self.in_any_string() => self.bracket_depth = (self.bracket_depth - 1).max(0),
            _ => {}
        }

        self.prev_char = Some(ch);
    }

    fn is_balanced(&self) -> bool {
        !self.in_single_quote
            && !self.in_double_quote
            && !self.in_multi_line_comment
            && self.paren_depth == 0
            && self.brace_depth == 0
            && self.bracket_depth == 0
    }

    fn in_any_string(&self) -> bool {
        self.in_single_quote || self.in_double_quote
    }
}

pub fn is_statement_complete(input: &str) -> bool {
    let trimmed = input.trim();

    if trimmed.is_empty() {
        return true;
    }

    if trimmed.starts_with('\\') {
        return true;
    }

    let mut tracker = StatementBalanceTracker::new();
    for ch in trimmed.chars() {
        tracker.feed(ch);
    }

    if !tracker.is_balanced() {
        return false;
    }

    if trimmed.ends_with(';') {
        return true;
    }

    let upper = trimmed.to_uppercase();
    let auto_complete = [
        "SHOW SPACES",
        "SHOW TAGS",
        "SHOW EDGES",
        "SHOW INDEXES",
        "SHOW USERS",
        "SHOW FUNCTIONS",
    ];
    auto_complete.iter().any(|cmd| upper == *cmd)
}

#[derive(Debug, Clone)]
pub enum ConditionExpr {
    Equals { var: String, value: String },
    NotEquals { var: String, value: String },
    IsSet { var: String },
    IsNotSet { var: String },
}

impl ConditionExpr {
    pub fn parse(expr: &str) -> Result<Self> {
        let expr = expr.trim();

        if let Some(rest) = expr.strip_prefix("!?") {
            let var = rest.trim().to_string();
            return Ok(ConditionExpr::IsNotSet { var });
        }

        if let Some(rest) = expr.strip_prefix('?') {
            let var = rest.trim().to_string();
            return Ok(ConditionExpr::IsSet { var });
        }

        if let Some(pos) = expr.find("==") {
            let var = expr[..pos].trim().to_string();
            let value = expr[pos + 2..].trim().to_string();
            return Ok(ConditionExpr::Equals { var, value });
        }

        if let Some(pos) = expr.find("!=") {
            let var = expr[..pos].trim().to_string();
            let value = expr[pos + 2..].trim().to_string();
            return Ok(ConditionExpr::NotEquals { var, value });
        }

        let var = expr.to_string();
        Ok(ConditionExpr::IsSet { var })
    }

    pub fn evaluate(&self, variables: &HashMap<String, String>) -> bool {
        match self {
            ConditionExpr::Equals { var, value } => {
                variables.get(var).map(|v| v == value).unwrap_or(false)
            }
            ConditionExpr::NotEquals { var, value } => {
                variables.get(var).map(|v| v != value).unwrap_or(true)
            }
            ConditionExpr::IsSet { var } => variables.contains_key(var),
            ConditionExpr::IsNotSet { var } => !variables.contains_key(var),
        }
    }
}

#[derive(Debug, Clone)]
struct ConditionalState {
    condition_met: bool,
    any_branch_taken: bool,
    in_active_branch: bool,
}

#[derive(Debug, Clone)]
pub struct ConditionalStack {
    stack: Vec<ConditionalState>,
}

impl Default for ConditionalStack {
    fn default() -> Self {
        Self::new()
    }
}

impl ConditionalStack {
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    pub fn push_if(&mut self, condition_met: bool) {
        let in_active = self.is_active() && condition_met;
        self.stack.push(ConditionalState {
            condition_met,
            any_branch_taken: condition_met,
            in_active_branch: in_active,
        });
    }

    pub fn push_elif(&mut self, condition_met: bool) {
        let parent_active = self.is_parent_active();
        if let Some(state) = self.stack.last_mut() {
            if state.any_branch_taken {
                state.in_active_branch = false;
            } else if condition_met {
                state.condition_met = true;
                state.any_branch_taken = true;
                state.in_active_branch = parent_active;
            } else {
                state.in_active_branch = false;
            }
        }
    }

    pub fn push_else(&mut self) {
        let parent_active = self.is_parent_active();
        if let Some(state) = self.stack.last_mut() {
            if state.any_branch_taken {
                state.in_active_branch = false;
            } else {
                state.condition_met = true;
                state.any_branch_taken = true;
                state.in_active_branch = parent_active;
            }
        }
    }

    pub fn pop(&mut self) {
        self.stack.pop();
    }

    pub fn is_active(&self) -> bool {
        if self.stack.is_empty() {
            return true;
        }
        self.stack.iter().all(|s| s.in_active_branch)
    }

    fn is_parent_active(&self) -> bool {
        if self.stack.len() <= 1 {
            return true;
        }
        self.stack[..self.stack.len() - 1]
            .iter()
            .all(|s| s.in_active_branch)
    }

    pub fn depth(&self) -> usize {
        self.stack.len()
    }
}

pub struct ScriptExecutionContext {
    pub depth: usize,
    pub call_stack: Vec<String>,
    pub conditional_stack: ConditionalStack,
    pub current_file: Option<String>,
}

impl Default for ScriptExecutionContext {
    fn default() -> Self {
        Self::new()
    }
}

impl ScriptExecutionContext {
    pub fn new() -> Self {
        Self {
            depth: 0,
            call_stack: Vec::new(),
            conditional_stack: ConditionalStack::new(),
            current_file: None,
        }
    }

    pub fn enter_script(&mut self, path: &str) -> Result<()> {
        const MAX_SCRIPT_DEPTH: usize = 16;
        if self.depth >= MAX_SCRIPT_DEPTH {
            return Err(CliError::Other(format!(
                "Script nesting too deep (max {}): {}",
                MAX_SCRIPT_DEPTH, path
            )));
        }

        let canonical = std::path::Path::new(path)
            .canonicalize()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| path.to_string());

        if self.call_stack.contains(&canonical) {
            return Err(CliError::Other(format!(
                "Circular script reference detected: {}",
                path
            )));
        }

        self.depth += 1;
        self.call_stack.push(canonical);
        self.current_file = Some(path.to_string());
        Ok(())
    }

    pub fn exit_script(&mut self) {
        if self.depth > 0 {
            self.depth -= 1;
        }
        self.call_stack.pop();
        self.current_file = self.call_stack.last().cloned();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_parser_empty() {
        let statements = ScriptParser::parse("");
        assert!(statements.is_empty());
    }

    #[test]
    fn test_script_parser_single_query() {
        let statements = ScriptParser::parse("MATCH (v) RETURN v;");
        assert_eq!(statements.len(), 1);
        assert_eq!(statements[0].content, "MATCH (v) RETURN v;");
        assert_eq!(statements[0].start_line, 1);
        assert_eq!(statements[0].end_line, 1);
        assert!(matches!(statements[0].kind, StatementKind::Query));
    }

    #[test]
    fn test_script_parser_multiple_queries() {
        let script = "MATCH (v) RETURN v;\nMATCH (e) RETURN e;";
        let statements = ScriptParser::parse(script);
        assert_eq!(statements.len(), 2);
        assert_eq!(statements[0].content, "MATCH (v) RETURN v;");
        assert_eq!(statements[1].content, "MATCH (e) RETURN e;");
    }

    #[test]
    fn test_script_parser_meta_command() {
        let statements = ScriptParser::parse("\\set VAR value");
        assert_eq!(statements.len(), 1);
        assert_eq!(statements[0].content, "\\set VAR value");
        assert!(matches!(statements[0].kind, StatementKind::MetaCommand));
    }

    #[test]
    fn test_script_parser_mixed() {
        let script = "MATCH (v) RETURN v;\n\\set VAR value\nMATCH (e) RETURN e;";
        let statements = ScriptParser::parse(script);
        assert_eq!(statements.len(), 3);
        assert!(matches!(statements[0].kind, StatementKind::Query));
        assert!(matches!(statements[1].kind, StatementKind::MetaCommand));
        assert!(matches!(statements[2].kind, StatementKind::Query));
    }

    #[test]
    fn test_script_parser_multiline_query() {
        let script = "MATCH (v:Person)\nWHERE v.age > 18\nRETURN v.name;";
        let statements = ScriptParser::parse(script);
        assert_eq!(statements.len(), 1);
        assert_eq!(statements[0].start_line, 1);
        assert_eq!(statements[0].end_line, 3);
        assert!(statements[0].content.contains("MATCH"));
        assert!(statements[0].content.contains("WHERE"));
        assert!(statements[0].content.contains("RETURN"));
    }

    #[test]
    fn test_script_parser_comments() {
        let script = "-- This is a comment\nMATCH (v) RETURN v;\n// Another comment";
        let statements = ScriptParser::parse(script);
        assert_eq!(statements.len(), 1);
        assert_eq!(statements[0].content, "MATCH (v) RETURN v;");
    }

    #[test]
    fn test_script_parser_empty_lines() {
        let script = "\n\nMATCH (v) RETURN v;\n\n";
        let statements = ScriptParser::parse(script);
        assert_eq!(statements.len(), 1);
        assert_eq!(statements[0].content, "MATCH (v) RETURN v;");
    }

    #[test]
    fn test_statement_balance_tracker_new() {
        let tracker = StatementBalanceTracker::new();
        assert!(!tracker.in_single_quote);
        assert!(!tracker.in_double_quote);
        assert_eq!(tracker.paren_depth, 0);
        assert_eq!(tracker.brace_depth, 0);
        assert_eq!(tracker.bracket_depth, 0);
    }

    #[test]
    fn test_statement_balance_tracker_single_quotes() {
        let mut tracker = StatementBalanceTracker::new();
        tracker.feed('\'');
        assert!(tracker.in_single_quote);
        assert!(!tracker.in_double_quote);
        tracker.feed('\'');
        assert!(!tracker.in_single_quote);
    }

    #[test]
    fn test_statement_balance_tracker_double_quotes() {
        let mut tracker = StatementBalanceTracker::new();
        tracker.feed('"');
        assert!(!tracker.in_single_quote);
        assert!(tracker.in_double_quote);
        tracker.feed('"');
        assert!(!tracker.in_double_quote);
    }

    #[test]
    fn test_statement_balance_tracker_parentheses() {
        let mut tracker = StatementBalanceTracker::new();
        tracker.feed('(');
        assert_eq!(tracker.paren_depth, 1);
        tracker.feed(')');
        assert_eq!(tracker.paren_depth, 0);
        assert!(tracker.is_balanced());
    }

    #[test]
    fn test_statement_balance_tracker_braces() {
        let mut tracker = StatementBalanceTracker::new();
        tracker.feed('{');
        assert_eq!(tracker.brace_depth, 1);
        tracker.feed('}');
        assert_eq!(tracker.brace_depth, 0);
        assert!(tracker.is_balanced());
    }

    #[test]
    fn test_statement_balance_tracker_brackets() {
        let mut tracker = StatementBalanceTracker::new();
        tracker.feed('[');
        assert_eq!(tracker.bracket_depth, 1);
        tracker.feed(']');
        assert_eq!(tracker.bracket_depth, 0);
        assert!(tracker.is_balanced());
    }

    #[test]
    fn test_statement_balance_tracker_in_string() {
        let mut tracker = StatementBalanceTracker::new();
        tracker.feed('\'');
        tracker.feed('(');
        tracker.feed(')');
        assert_eq!(tracker.paren_depth, 0);
        tracker.feed('\'');
        assert!(tracker.is_balanced());
    }

    #[test]
    fn test_statement_balance_tracker_not_balanced() {
        let mut tracker = StatementBalanceTracker::new();
        tracker.feed('(');
        assert!(!tracker.is_balanced());
        tracker.feed('{');
        assert!(!tracker.is_balanced());
        tracker.feed('\'');
        assert!(!tracker.is_balanced());
    }

    #[test]
    fn test_is_statement_complete_empty() {
        assert!(is_statement_complete(""));
    }

    #[test]
    fn test_is_statement_complete_meta_command() {
        assert!(is_statement_complete("\\set VAR value"));
        assert!(is_statement_complete("  \\quit  "));
    }

    #[test]
    fn test_is_statement_complete_with_semicolon() {
        assert!(is_statement_complete("MATCH (v) RETURN v;"));
    }

    #[test]
    fn test_is_statement_complete_without_semicolon() {
        assert!(!is_statement_complete("MATCH (v) RETURN v"));
    }

    #[test]
    fn test_is_statement_complete_auto_complete_commands() {
        assert!(is_statement_complete("SHOW SPACES"));
        assert!(is_statement_complete("show spaces"));
        assert!(is_statement_complete("SHOW TAGS"));
        assert!(is_statement_complete("SHOW EDGES"));
        assert!(is_statement_complete("SHOW INDEXES"));
        assert!(is_statement_complete("SHOW USERS"));
        assert!(is_statement_complete("SHOW FUNCTIONS"));
    }

    #[test]
    fn test_is_statement_complete_unbalanced() {
        assert!(!is_statement_complete("MATCH (v WHERE v.age > 0"));
        assert!(!is_statement_complete("MATCH (v) RETURN ['item"));
    }

    #[test]
    fn test_condition_expr_parse_is_set() {
        let expr = ConditionExpr::parse("VAR").unwrap();
        assert!(matches!(expr, ConditionExpr::IsSet { var } if var == "VAR"));

        let expr = ConditionExpr::parse("?VAR").unwrap();
        assert!(matches!(expr, ConditionExpr::IsSet { var } if var == "VAR"));
    }

    #[test]
    fn test_condition_expr_parse_is_not_set() {
        let expr = ConditionExpr::parse("!?VAR").unwrap();
        assert!(matches!(expr, ConditionExpr::IsNotSet { var } if var == "VAR"));
    }

    #[test]
    fn test_condition_expr_parse_equals() {
        let expr = ConditionExpr::parse("VAR == value").unwrap();
        assert!(
            matches!(expr, ConditionExpr::Equals { var, value } if var == "VAR" && value == "value")
        );
    }

    #[test]
    fn test_condition_expr_parse_not_equals() {
        let expr = ConditionExpr::parse("VAR != value").unwrap();
        assert!(
            matches!(expr, ConditionExpr::NotEquals { var, value } if var == "VAR" && value == "value")
        );
    }

    #[test]
    fn test_condition_expr_parse_whitespace() {
        let expr = ConditionExpr::parse("  VAR  ==  value  ").unwrap();
        assert!(
            matches!(expr, ConditionExpr::Equals { var, value } if var == "VAR" && value == "value")
        );
    }

    #[test]
    fn test_condition_expr_evaluate_is_set() {
        let mut vars = HashMap::new();
        let expr = ConditionExpr::IsSet {
            var: "VAR".to_string(),
        };

        assert!(!expr.evaluate(&vars));

        vars.insert("VAR".to_string(), "value".to_string());
        assert!(expr.evaluate(&vars));
    }

    #[test]
    fn test_condition_expr_evaluate_is_not_set() {
        let mut vars = HashMap::new();
        let expr = ConditionExpr::IsNotSet {
            var: "VAR".to_string(),
        };

        assert!(expr.evaluate(&vars));

        vars.insert("VAR".to_string(), "value".to_string());
        assert!(!expr.evaluate(&vars));
    }

    #[test]
    fn test_condition_expr_evaluate_equals() {
        let mut vars = HashMap::new();
        let expr = ConditionExpr::Equals {
            var: "VAR".to_string(),
            value: "test".to_string(),
        };

        assert!(!expr.evaluate(&vars));

        vars.insert("VAR".to_string(), "other".to_string());
        assert!(!expr.evaluate(&vars));

        vars.insert("VAR".to_string(), "test".to_string());
        assert!(expr.evaluate(&vars));
    }

    #[test]
    fn test_condition_expr_evaluate_not_equals() {
        let mut vars = HashMap::new();
        let expr = ConditionExpr::NotEquals {
            var: "VAR".to_string(),
            value: "test".to_string(),
        };

        assert!(expr.evaluate(&vars));

        vars.insert("VAR".to_string(), "other".to_string());
        assert!(expr.evaluate(&vars));

        vars.insert("VAR".to_string(), "test".to_string());
        assert!(!expr.evaluate(&vars));
    }

    #[test]
    fn test_conditional_stack_new() {
        let stack = ConditionalStack::new();
        assert!(stack.is_active());
        assert_eq!(stack.depth(), 0);
    }

    #[test]
    fn test_conditional_stack_push_if_true() {
        let mut stack = ConditionalStack::new();
        stack.push_if(true);
        assert_eq!(stack.depth(), 1);
        assert!(stack.is_active());
    }

    #[test]
    fn test_conditional_stack_push_if_false() {
        let mut stack = ConditionalStack::new();
        stack.push_if(false);
        assert_eq!(stack.depth(), 1);
        assert!(!stack.is_active());
    }

    #[test]
    fn test_conditional_stack_push_elif() {
        let mut stack = ConditionalStack::new();
        stack.push_if(false);
        stack.push_elif(true);
        assert!(stack.is_active());

        let mut stack2 = ConditionalStack::new();
        stack2.push_if(true);
        stack2.push_elif(true);
        assert!(!stack2.is_active());
    }

    #[test]
    fn test_conditional_stack_push_else() {
        let mut stack = ConditionalStack::new();
        stack.push_if(false);
        stack.push_else();
        assert!(stack.is_active());

        let mut stack2 = ConditionalStack::new();
        stack2.push_if(true);
        stack2.push_else();
        assert!(!stack2.is_active());
    }

    #[test]
    fn test_conditional_stack_pop() {
        let mut stack = ConditionalStack::new();
        stack.push_if(true);
        assert_eq!(stack.depth(), 1);
        stack.pop();
        assert_eq!(stack.depth(), 0);
        assert!(stack.is_active());
    }

    #[test]
    fn test_conditional_stack_nested() {
        let mut stack = ConditionalStack::new();
        stack.push_if(true);
        stack.push_if(true);
        assert!(stack.is_active());

        stack.pop();
        stack.push_if(false);
        assert!(!stack.is_active());
    }

    #[test]
    fn test_script_execution_context_new() {
        let ctx = ScriptExecutionContext::new();
        assert_eq!(ctx.depth, 0);
        assert!(ctx.call_stack.is_empty());
        assert!(ctx.current_file.is_none());
    }

    #[test]
    fn test_script_execution_context_enter_exit() {
        let mut ctx = ScriptExecutionContext::new();
        ctx.enter_script("test.sql").unwrap();
        assert_eq!(ctx.depth, 1);
        assert_eq!(ctx.current_file, Some("test.sql".to_string()));

        ctx.exit_script();
        assert_eq!(ctx.depth, 0);
        assert!(ctx.current_file.is_none());
    }

    #[test]
    fn test_script_execution_context_nested() {
        let mut ctx = ScriptExecutionContext::new();
        ctx.enter_script("outer.sql").unwrap();
        ctx.enter_script("inner.sql").unwrap();
        assert_eq!(ctx.depth, 2);
        assert_eq!(ctx.current_file, Some("inner.sql".to_string()));

        ctx.exit_script();
        assert_eq!(ctx.depth, 1);
        assert_eq!(ctx.current_file, Some("outer.sql".to_string()));
    }

    #[test]
    fn test_script_execution_context_max_depth() {
        let mut ctx = ScriptExecutionContext::new();
        for i in 0..16 {
            ctx.enter_script(&format!("script{}.sql", i)).unwrap();
        }
        assert!(ctx.enter_script("too_deep.sql").is_err());
    }

    #[test]
    fn test_script_execution_context_circular() {
        let mut ctx = ScriptExecutionContext::new();
        ctx.enter_script("script.sql").unwrap();
        assert!(ctx.enter_script("script.sql").is_err());
    }

    #[test]
    fn test_parsed_statement_fields() {
        let stmt = ParsedStatement {
            content: "MATCH (v) RETURN v;".to_string(),
            start_line: 1,
            end_line: 1,
            kind: StatementKind::Query,
        };
        assert_eq!(stmt.content, "MATCH (v) RETURN v;");
        assert_eq!(stmt.start_line, 1);
        assert_eq!(stmt.end_line, 1);
        assert!(matches!(stmt.kind, StatementKind::Query));
    }

    #[test]
    fn test_statement_kind_enum() {
        let query = StatementKind::Query;
        let meta = StatementKind::MetaCommand;

        assert!(matches!(query, StatementKind::Query));
        assert!(matches!(meta, StatementKind::MetaCommand));
    }
}

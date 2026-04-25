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

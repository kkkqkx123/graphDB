use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::client::http::{EdgeTypeInfo, SpaceInfo, TagInfo};

#[derive(Debug, Clone)]
pub struct SchemaCache {
    pub spaces: Vec<SpaceInfo>,
    pub tags: Vec<TagInfo>,
    pub edges: Vec<EdgeTypeInfo>,
    pub last_updated: Instant,
    pub ttl: Duration,
}

impl Default for SchemaCache {
    fn default() -> Self {
        Self::new()
    }
}

impl SchemaCache {
    pub fn new() -> Self {
        Self {
            spaces: Vec::new(),
            tags: Vec::new(),
            edges: Vec::new(),
            last_updated: Instant::now(),
            ttl: Duration::from_secs(300),
        }
    }

    pub fn is_stale(&self) -> bool {
        self.last_updated.elapsed() > self.ttl
    }

    pub fn tag_names(&self) -> Vec<String> {
        self.tags.iter().map(|t| t.name.clone()).collect()
    }

    pub fn edge_names(&self) -> Vec<String> {
        self.edges.iter().map(|e| e.name.clone()).collect()
    }

    pub fn space_names(&self) -> Vec<String> {
        self.spaces.iter().map(|s| s.name.clone()).collect()
    }

    pub fn tag_properties(&self, tag_name: &str) -> Vec<String> {
        self.tags
            .iter()
            .find(|t| t.name == tag_name)
            .map(|t| t.fields.iter().map(|f| f.name.clone()).collect())
            .unwrap_or_default()
    }

    pub fn mark_stale(&mut self) {
        self.last_updated = Instant::now() - self.ttl - Duration::from_secs(1);
    }
}

pub type SharedSchemaCache = Arc<Mutex<SchemaCache>>;

pub fn new_shared_cache() -> SharedSchemaCache {
    Arc::new(Mutex::new(SchemaCache::new()))
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CompletionContext {
    Keyword,
    TagName,
    EdgeName,
    PropertyName,
    SpaceName,
    FunctionName,
    VariableName,
    MetaCommandArg,
}

pub fn detect_context(
    line: &str,
    pos: usize,
    variables: &HashMap<String, String>,
) -> CompletionContext {
    let before = &line[..pos];

    if before.starts_with('\\') {
        return detect_meta_context(before);
    }

    let upper = before.to_uppercase();

    if upper.ends_with("USE ") {
        return CompletionContext::SpaceName;
    }

    if let Some(ctx) = detect_tag_context(before) {
        return ctx;
    }

    if let Some(ctx) = detect_edge_context(before) {
        return ctx;
    }

    if let Some(ctx) = detect_property_context(before) {
        return ctx;
    }

    if detect_variable_context(before, variables) {
        return CompletionContext::VariableName;
    }

    let upper_trimmed = upper.trim_end();
    if upper_trimmed.ends_with("RETURN")
        || upper_trimmed.ends_with("WHERE")
        || upper_trimmed.ends_with("SET")
        || upper_trimmed.ends_with("YIELD")
        || upper_trimmed.ends_with("ORDER BY")
        || upper_trimmed.ends_with("GROUP BY")
    {
        return CompletionContext::FunctionName;
    }

    CompletionContext::Keyword
}

fn detect_meta_context(before: &str) -> CompletionContext {
    let trimmed = before.trim_start_matches('\\');
    let parts: Vec<&str> = trimmed.splitn(2, |c: char| c.is_whitespace()).collect();

    if parts.len() >= 2 && !parts[1].trim().is_empty() {
        let cmd = parts[0].to_lowercase();
        match cmd.as_str() {
            "connect" | "c" => return CompletionContext::SpaceName,
            "describe" | "d" => return CompletionContext::TagName,
            "describe_edge" => return CompletionContext::EdgeName,
            "format" => return CompletionContext::MetaCommandArg,
            _ => {}
        }
    }

    CompletionContext::Keyword
}

fn detect_tag_context(before: &str) -> Option<CompletionContext> {
    regex_captures_tag(before)?;
    Some(CompletionContext::TagName)
}

fn regex_captures_tag(before: &str) -> Option<()> {
    let chars: Vec<char> = before.chars().collect();
    let len = chars.len();

    if len < 2 {
        return None;
    }

    let mut i = len - 1;

    while i > 0 && chars[i].is_whitespace() {
        i -= 1;
    }

    if chars[i] != ':' {
        return None;
    }

    if i > 0 && chars[i - 1] == ':' {
        return None;
    }

    let mut j = i;
    j = j.saturating_sub(1);

    while (chars[j] == '_' || chars[j].is_alphanumeric()) && j > 0 {
        j -= 1;
    }

    if j < i && (chars[j].is_alphanumeric() || chars[j] == '_') {
        let ident: String = chars[j..i].iter().collect();
        let upper = ident.to_uppercase();
        if upper == "VERTEX" || upper == "TAG" || upper == "TAGS" || upper == "VT" {
            return Some(());
        }
    }

    let before_colon = &before[..i];
    if before_colon.ends_with('(') || before_colon.ends_with(", ") {
        let trimmed = before_colon
            .trim_end_matches('(')
            .trim_end_matches(", ")
            .trim();
        let upper = trimmed.to_uppercase();
        if upper.ends_with("MATCH")
            || upper.ends_with("CREATE")
            || upper.ends_with("MERGE")
            || upper.ends_with("OPTIONAL MATCH")
        {
            return Some(());
        }
    }

    None
}

fn detect_edge_context(before: &str) -> Option<CompletionContext> {
    let trimmed = before.trim_end();

    if trimmed.ends_with("[:") || trimmed.ends_with("[ :") {
        return Some(CompletionContext::EdgeName);
    }

    let re = trimmed.rfind("-[:");
    let re2 = trimmed.rfind("-[ :");
    if re.is_some() || re2.is_some() {
        return Some(CompletionContext::EdgeName);
    }

    None
}

fn detect_property_context(before: &str) -> Option<CompletionContext> {
    let trimmed = before.trim_end();
    if !trimmed.ends_with('.') {
        return None;
    }

    let before_dot = trimmed.trim_end_matches('.');
    let ident = before_dot
        .rsplit(|c: char| !c.is_alphanumeric() && c != '_')
        .next()?;

    if ident.is_empty() {
        return None;
    }

    let _ = ident;
    Some(CompletionContext::PropertyName)
}

fn detect_variable_context(before: &str, _variables: &HashMap<String, String>) -> bool {
    let trimmed = before.trim_end();
    if !trimmed.ends_with(':') {
        return false;
    }

    let before_colon = trimmed.trim_end_matches(':').trim();
    if before_colon.is_empty() {
        return false;
    }

    let upper = before_colon.to_uppercase();
    if upper.ends_with("LIMIT")
        || upper.ends_with("SKIP")
        || upper.ends_with("WHERE")
        || upper.ends_with("VALUES")
    {
        return true;
    }

    false
}

pub fn get_function_completions() -> Vec<FunctionEntry> {
    vec![
        FunctionEntry::new(
            "count",
            "count(expr)",
            "Count the number of rows",
            FunctionCategory::Aggregate,
        ),
        FunctionEntry::new(
            "sum",
            "sum(expr)",
            "Sum of values",
            FunctionCategory::Aggregate,
        ),
        FunctionEntry::new(
            "avg",
            "avg(expr)",
            "Average of values",
            FunctionCategory::Aggregate,
        ),
        FunctionEntry::new(
            "min",
            "min(expr)",
            "Minimum value",
            FunctionCategory::Aggregate,
        ),
        FunctionEntry::new(
            "max",
            "max(expr)",
            "Maximum value",
            FunctionCategory::Aggregate,
        ),
        FunctionEntry::new(
            "collect",
            "collect(expr)",
            "Collect values into a list",
            FunctionCategory::Aggregate,
        ),
        FunctionEntry::new(
            "length",
            "length(str)",
            "String length",
            FunctionCategory::String,
        ),
        FunctionEntry::new(
            "size",
            "size(list)",
            "List/string size",
            FunctionCategory::String,
        ),
        FunctionEntry::new(
            "trim",
            "trim(str)",
            "Trim whitespace",
            FunctionCategory::String,
        ),
        FunctionEntry::new(
            "lower",
            "lower(str)",
            "Convert to lowercase",
            FunctionCategory::String,
        ),
        FunctionEntry::new(
            "upper",
            "upper(str)",
            "Convert to uppercase",
            FunctionCategory::String,
        ),
        FunctionEntry::new(
            "substring",
            "substring(str, start, len)",
            "Extract substring",
            FunctionCategory::String,
        ),
        FunctionEntry::new(
            "replace",
            "replace(str, old, new)",
            "Replace substring",
            FunctionCategory::String,
        ),
        FunctionEntry::new(
            "abs",
            "abs(num)",
            "Absolute value",
            FunctionCategory::Numeric,
        ),
        FunctionEntry::new("ceil", "ceil(num)", "Round up", FunctionCategory::Numeric),
        FunctionEntry::new(
            "floor",
            "floor(num)",
            "Round down",
            FunctionCategory::Numeric,
        ),
        FunctionEntry::new(
            "round",
            "round(num)",
            "Round to nearest",
            FunctionCategory::Numeric,
        ),
        FunctionEntry::new(
            "sqrt",
            "sqrt(num)",
            "Square root",
            FunctionCategory::Numeric,
        ),
        FunctionEntry::new(
            "head",
            "head(list)",
            "First element",
            FunctionCategory::List,
        ),
        FunctionEntry::new(
            "tail",
            "tail(list)",
            "All but first element",
            FunctionCategory::List,
        ),
        FunctionEntry::new(
            "reverse",
            "reverse(list)",
            "Reverse list",
            FunctionCategory::List,
        ),
        FunctionEntry::new(
            "type",
            "type(edge)",
            "Edge type name",
            FunctionCategory::Type,
        ),
        FunctionEntry::new("id", "id(vertex)", "Vertex ID", FunctionCategory::Type),
        FunctionEntry::new(
            "label",
            "label(vertex)",
            "Vertex labels",
            FunctionCategory::Type,
        ),
        FunctionEntry::new(
            "properties",
            "properties(vertex)",
            "Vertex properties map",
            FunctionCategory::Type,
        ),
        FunctionEntry::new(
            "datetime",
            "datetime()",
            "Current datetime",
            FunctionCategory::Date,
        ),
        FunctionEntry::new(
            "timestamp",
            "timestamp()",
            "Current timestamp",
            FunctionCategory::Date,
        ),
    ]
}

#[derive(Debug, Clone)]
pub enum FunctionCategory {
    Aggregate,
    String,
    Numeric,
    List,
    Type,
    Date,
}

#[derive(Debug, Clone)]
pub struct FunctionEntry {
    pub name: String,
    pub signature: String,
    pub description: String,
    pub category: FunctionCategory,
}

impl FunctionEntry {
    pub fn new(name: &str, signature: &str, description: &str, category: FunctionCategory) -> Self {
        Self {
            name: name.to_string(),
            signature: signature.to_string(),
            description: description.to_string(),
            category,
        }
    }
}

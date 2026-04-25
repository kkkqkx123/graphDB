use rustyline::completion::{Candidate, Completer};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Helper, Result};

const GQL_KEYWORDS: &[&str] = &[
    "MATCH",
    "GO",
    "LOOKUP",
    "FETCH",
    "INSERT",
    "UPDATE",
    "DELETE",
    "CREATE",
    "ALTER",
    "DROP",
    "GRANT",
    "REVOKE",
    "RETURN",
    "WHERE",
    "ORDER",
    "BY",
    "LIMIT",
    "SKIP",
    "AND",
    "OR",
    "NOT",
    "IN",
    "AS",
    "WITH",
    "UNWIND",
    "SET",
    "REMOVE",
    "MERGE",
    "OPTIONAL",
    "DISTINCT",
    "UNION",
    "ALL",
    "EXISTS",
    "CASE",
    "WHEN",
    "THEN",
    "ELSE",
    "END",
    "ASC",
    "DESC",
    "TRUE",
    "FALSE",
    "NULL",
    "IS",
    "LIKE",
    "CONTAINS",
    "STARTS",
    "OVER",
    "STEPS",
    "FROM",
    "TO",
    "YIELD",
    "VERTEX",
    "EDGE",
    "VERTICES",
    "EDGES",
    "TAG",
    "TAGS",
    "SPACE",
    "SPACES",
    "INDEX",
    "INDEXES",
    "SHOW",
    "USE",
    "DESCRIBE",
    "EXPLAIN",
    "PROFILE",
    "REBUILD",
    "SUBGRAPH",
    "GROUP",
    "COUNT",
    "SUM",
    "AVG",
    "MAX",
    "MIN",
    "COLLECT",
    "HEAD",
    "TAIL",
    "SIZE",
    "LENGTH",
    "TYPE",
    "PROPERTIES",
    "ID",
    "LABEL",
    "RANK",
    "DATETIME",
    "TIMESTAMP",
    "STRING",
    "INT",
    "INTEGER",
    "FLOAT",
    "DOUBLE",
    "BOOL",
    "BOOLEAN",
    "LIST",
    "MAP",
    "SET",
    "IF",
    "COMMENT",
    "DEFAULT",
    "PARTITION_NUM",
    "REPLICA_FACTOR",
    "VID_TYPE",
    "TTL_DURATION",
    "TTL_COL",
];

const META_COMMANDS: &[&str] = &[
    "\\connect",
    "\\c",
    "\\disconnect",
    "\\conninfo",
    "\\show_spaces",
    "\\l",
    "\\show_tags",
    "\\dt",
    "\\show_edges",
    "\\de",
    "\\show_indexes",
    "\\di",
    "\\show_users",
    "\\du",
    "\\show_functions",
    "\\df",
    "\\describe",
    "\\d",
    "\\describe_edge",
    "\\format",
    "\\pager",
    "\\timing",
    "\\x",
    "\\set",
    "\\unset",
    "\\i",
    "\\ir",
    "\\o",
    "\\!",
    "\\help",
    "\\?",
    "\\version",
    "\\copyright",
    "\\q",
    "\\quit",
    "\\begin",
    "\\commit",
    "\\rollback",
];

#[derive(Debug)]
pub struct StringCandidate {
    display: String,
    replacement: String,
}

impl Candidate for StringCandidate {
    fn display(&self) -> &str {
        &self.display
    }

    fn replacement(&self) -> &str {
        &self.replacement
    }
}

#[derive(Debug)]
pub struct GraphDBCompleter {
    keywords: Vec<String>,
    meta_commands: Vec<String>,
}

impl Default for GraphDBCompleter {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphDBCompleter {
    pub fn new() -> Self {
        Self {
            keywords: GQL_KEYWORDS.iter().map(|s| s.to_string()).collect(),
            meta_commands: META_COMMANDS.iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl Completer for GraphDBCompleter {
    type Candidate = StringCandidate;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> Result<(usize, Vec<StringCandidate>)> {
        let line_to_pos = &line[..pos];

        if line_to_pos.starts_with('\\') {
            let partial = line_to_pos.trim_start_matches('\\');
            let completions: Vec<StringCandidate> = self
                .meta_commands
                .iter()
                .filter(|cmd| cmd.trim_start_matches('\\').starts_with(partial))
                .map(|cmd| StringCandidate {
                    display: cmd.clone(),
                    replacement: cmd[1..].to_string(),
                })
                .collect();

            let start = if partial.is_empty() {
                pos
            } else {
                pos - partial.len() - 1
            };
            return Ok((start, completions));
        }

        let last_word = get_last_word(line_to_pos);
        if last_word.is_empty() {
            return Ok((pos, Vec::new()));
        }

        let completions: Vec<StringCandidate> = self
            .keywords
            .iter()
            .filter(|kw| kw.starts_with(&last_word.to_uppercase()))
            .map(|kw| StringCandidate {
                display: kw.clone(),
                replacement: kw[last_word.len()..].to_string(),
            })
            .collect();

        let start = pos - last_word.len();
        Ok((start, completions))
    }
}

impl Hinter for GraphDBCompleter {
    type Hint = String;

    fn hint(&self, _line: &str, _pos: usize, _ctx: &rustyline::Context<'_>) -> Option<String> {
        None
    }
}

impl Highlighter for GraphDBCompleter {}

impl Validator for GraphDBCompleter {}

impl Helper for GraphDBCompleter {}

fn get_last_word(input: &str) -> String {
    let trimmed = input.trim_end();
    if trimmed.is_empty() {
        return String::new();
    }

    let word_start = trimmed
        .char_indices()
        .rev()
        .find(|(_, c)| {
            c.is_whitespace()
                || *c == '('
                || *c == '['
                || *c == '{'
                || *c == ','
                || *c == ':'
                || *c == '='
        })
        .map(|(i, _)| i + 1)
        .unwrap_or(0);

    trimmed[word_start..].to_string()
}

//! Lexer implementation for the query parser
//!
//! This module implements a lexical analyzer that converts input query strings into tokens.

use crate::query::parser::ast::Position;
use crate::query::parser::{Token, TokenKind};

#[derive(Clone)]
pub struct Lexer {
    input: Vec<char>,
    position: usize,
    read_position: usize,
    ch: Option<char>,
    line: usize,
    column: usize,
    current_token: Token,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        let mut lexer = Lexer {
            input: input.chars().collect(),
            position: 0,
            read_position: 0,
            ch: None,
            line: 1,
            column: 0,
            current_token: Token::new(TokenKind::Eof, String::new(), 0, 0),
        };
        lexer.read_char();
        lexer.current_token = lexer.next_token();
        lexer
    }

    fn read_char(&mut self) {
        if self.read_position >= self.input.len() {
            self.ch = None;
        } else {
            self.ch = Some(self.input[self.read_position]);
        }

        // Update position and column
        if self.ch == Some('\n') {
            self.line += 1;
            self.column = 0;
        } else {
            self.column += 1;
        }

        self.position = self.read_position;
        self.read_position += 1;
    }

    fn peek_char(&self) -> Option<char> {
        if self.read_position >= self.input.len() {
            None
        } else {
            Some(self.input[self.read_position])
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.ch {
            if ch == ' ' || ch == '\t' || ch == '\r' || ch == '\n' {
                self.read_char();
            } else {
                break;
            }
        }
    }

    fn read_identifier(&mut self) -> String {
        let start_position = self.position;
        while let Some(ch) = self.ch {
            if ch.is_alphanumeric() || ch == '_' {
                self.read_char();
            } else {
                break;
            }
        }
        self.input[start_position..self.position].iter().collect()
    }

    fn read_number(&mut self) -> String {
        let start_position = self.position;
        let mut has_decimal = false;
        let mut has_exponent = false;

        while let Some(ch) = self.ch {
            if ch.is_ascii_digit() {
                self.read_char();
            } else if ch == '.' && !has_decimal && !has_exponent {
                if self
                    .peek_char()
                    .map_or(false, |c| c.is_ascii_digit())
                {
                    has_decimal = true;
                    self.read_char();
                } else {
                    break;
                }
            } else if (ch == 'e' || ch == 'E') && !has_exponent {
                has_exponent = true;
                self.read_char();
                if self.ch == Some('+') || self.ch == Some('-') {
                    self.read_char();
                }
            } else {
                break;
            }
        }
        self.input[start_position..self.position].iter().collect()
    }

    fn read_string(&mut self) -> String {
        let quote = self.ch.unwrap();
        self.read_char(); // Skip opening quote
        let mut result = String::new();
        let start_line = self.line;
        let start_column = self.column;

        while let Some(ch) = self.ch {
            if ch == '\\' {
                // 转义序列处理
                self.read_char(); // Skip backslash
                match self.ch {
                    Some('n') => result.push('\n'),
                    Some('t') => result.push('\t'),
                    Some('r') => result.push('\r'),
                    Some('\\') => result.push('\\'),
                    Some('"') => result.push('"'),
                    Some('\'') => result.push('\''),
                    Some('0') => result.push('\0'),
                    Some('\n') => {
                        // 行继续符，忽略换行符及其前导空白
                        self.read_char();
                        while let Some(c) = self.ch {
                            if c == ' ' || c == '\t' {
                                self.read_char();
                            } else {
                                break;
                            }
                        }
                        continue;
                    }
                    Some('u') => {
                        // Unicode 转义 \uXXXX
                        self.read_char();
                        let mut unicode_seq = String::new();
                        for _ in 0..4 {
                            if let Some(c) = self.ch {
                                if c.is_ascii_hexdigit() {
                                    unicode_seq.push(c);
                                    self.read_char();
                                } else {
                                    break;
                                }
                            }
                        }
                        if !unicode_seq.is_empty() {
                            if let Ok(code_point) = u32::from_str_radix(&unicode_seq, 16) {
                                if let Some(ch) = char::from_u32(code_point) {
                                    result.push(ch);
                                }
                            }
                        }
                        continue;
                    }
                    Some('x') => {
                        // 十六进制转义 \xHH
                        self.read_char();
                        let mut hex_seq = String::new();
                        for _ in 0..2 {
                            if let Some(c) = self.ch {
                                if c.is_ascii_hexdigit() {
                                    hex_seq.push(c);
                                    self.read_char();
                                } else {
                                    break;
                                }
                            }
                        }
                        if !hex_seq.is_empty() {
                            if let Ok(byte) = u8::from_str_radix(&hex_seq, 16) {
                                result.push(byte as char);
                            }
                        }
                        continue;
                    }
                    _ => {
                        // 未知转义序列，保留反杠和字符
                        result.push('\\');
                        if let Some(c) = self.ch {
                            result.push(c);
                        }
                    }
                }
                self.read_char();
            } else if ch == quote {
                // 结束引号
                self.read_char();
                return result;
            } else if ch == '\n' {
                // 未闭合的字符串，遇到换行
                panic!("Unterminated string literal at line {}", start_line);
            } else {
                result.push(ch);
                self.read_char();
            }
        }

        // 未闭合的字符串
        panic!("Unterminated string literal at line {}", start_line);
    }

    fn peek_next_word(&self) -> String {
        let mut temp_lexer = self.clone();
        temp_lexer.skip_whitespace();
        temp_lexer.read_identifier()
    }

    fn skip_next_word(&mut self) {
        self.skip_whitespace();
        while let Some(ch) = self.ch {
            if ch.is_whitespace() {
                break;
            }
            self.read_char();
        }
    }

    pub fn skip_comment(&mut self) {
        if self.ch == Some('/') {
            match self.peek_char() {
                Some('/') => {
                    self.read_char(); // Skip first /
                    self.read_char(); // Skip second /
                    while let Some(ch) = self.ch {
                        if ch == '\n' {
                            break;
                        }
                        self.read_char();
                    }
                }
                Some('*') => {
                    self.read_char(); // Skip first /
                    self.read_char(); // Skip *
                    loop {
                        match self.ch {
                            Some('*') => {
                                if self.peek_char() == Some('/') {
                                    self.read_char(); // Skip *
                                    self.read_char(); // Skip /
                                    break;
                                } else {
                                    self.read_char();
                                }
                            }
                            Some('\n') => {
                                self.read_char();
                            }
                            Some(_) => {
                                self.read_char();
                            }
                            None => {
                                panic!("Unterminated multi-line comment");
                            }
                        }
                    }
                }
                _ => {}
            }
        } else if self.ch == Some('-') {
            if self.peek_char() == Some('-') {
                self.read_char(); // Skip first -
                self.read_char(); // Skip second -
                while let Some(ch) = self.ch {
                    if ch == '\n' {
                        break;
                    }
                    self.read_char();
                }
            }
        }
    }

    fn read_raw_string(&mut self) -> String {
        self.read_char(); // Skip opening quote
        self.read_char(); // Skip second quote
        self.read_char(); // Skip third quote
        let start_position = self.position;

        while let Some(ch) = self.ch {
            if ch == '"' {
                if self.peek_char() == Some('"')
                    && self.input[self.read_position + 1..]
                        .first()
                        .map_or(false, |c| *c == '"')
                {
                    // End of raw string
                    self.read_char();
                    self.read_char();
                    self.read_char();
                    return self.input[start_position..self.position - 3]
                        .iter()
                        .collect();
                }
            }
            self.read_char();
        }

        panic!("Unterminated raw string literal");
    }

    fn lookup_keyword(&self, identifier: &str) -> TokenKind {
        match identifier.to_uppercase().as_str() {
            "CREATE" => TokenKind::Create,
            "MATCH" => TokenKind::Match,
            "RETURN" => TokenKind::Return,
            "WHERE" => TokenKind::Where,
            "DELETE" => TokenKind::Delete,
            "UPDATE" => TokenKind::Update,
            "INSERT" => TokenKind::Insert,
            "UPSERT" => TokenKind::Upsert,
            "FROM" => TokenKind::From,
            "TO" => TokenKind::To,
            "AS" => TokenKind::As,
            "WITH" => TokenKind::With,
            "YIELD" => TokenKind::Yield,
            "GO" => TokenKind::Go,
            "OVER" => TokenKind::Over,
            "STEPS" | "STEP" => TokenKind::Step,
            "UPTO" => TokenKind::Upto,
            "LIMIT" => TokenKind::Limit,
            "ASC" => TokenKind::Asc,
            "DESC" => TokenKind::Desc,
            "ORDER" => TokenKind::Order,
            "BY" => TokenKind::By,
            "SKIP" => TokenKind::Skip,
            "UNWIND" => TokenKind::Unwind,
            "OPTIONAL" => TokenKind::Optional,
            "DISTINCT" => TokenKind::Distinct,
            "ALL" => TokenKind::All,
            "NULL" => TokenKind::Null,
            "IS" => TokenKind::Is,
            "NOT" => TokenKind::Not,
            "AND" => TokenKind::And,
            "OR" => TokenKind::Or,
            "XOR" => TokenKind::Xor,
            "CONTAINS" => TokenKind::Contains,
            "STARTS" | "STARTS WITH" => TokenKind::StartsWith,
            "ENDS" | "ENDS WITH" => TokenKind::EndsWith,
            "CASE" => TokenKind::Case,
            "WHEN" => TokenKind::When,
            "THEN" => TokenKind::Then,
            "ELSE" => TokenKind::Else,
            "END" => TokenKind::End,
            "UNION" => TokenKind::Union,
            "INTERSECT" => TokenKind::Intersect,
            "GROUP" => TokenKind::Group,
            "BETWEEN" => TokenKind::Between,
            "ADMIN" => TokenKind::Admin,
            "EDGE" => TokenKind::Edge,
            "EDGES" => TokenKind::Edges,
            "VERTEX" => TokenKind::Vertex,
            "VERTICES" => TokenKind::Vertices,
            "TAG" => TokenKind::Tag,
            "TAGS" => TokenKind::Tags,
            "INDEX" => TokenKind::Index,
            "INDEXES" => TokenKind::Indexes,
            "LOOKUP" => TokenKind::Lookup,
            "FIND" => TokenKind::Find,
            "PATH" => TokenKind::Path,
            "SHORTEST" => TokenKind::Shortest,
            "NOLOOP" => TokenKind::NoLoop,
            "ALLSHORTESTPATHS" => TokenKind::AllShortestPaths,
            "SUBGRAPH" => TokenKind::Subgraph,
            "BOTH" => TokenKind::Both,
            "OUT" => TokenKind::Out,
            "IN" => TokenKind::In,
            "NO" => TokenKind::No,
            "OVERWRITE" => TokenKind::Overwrite,
            "SHOW" => TokenKind::Show,
            "ADD" => TokenKind::Add,
            "DROP" => TokenKind::Drop,
            "REMOVE" => TokenKind::Remove,
            "IF" => TokenKind::If,
            "EXISTS" => TokenKind::Exists,
            "CHANGE" => TokenKind::Change,
            "GRANT" => TokenKind::Grant,
            "REVOKE" => TokenKind::Revoke,
            "ON" => TokenKind::On,
            "OF" => TokenKind::Of,
            "GET" => TokenKind::Get,
            "SET" => TokenKind::Set,
            "HOST" => TokenKind::Host,
            "HOSTS" => TokenKind::Hosts,
            "SPACE" => TokenKind::Space,
            "SPACES" => TokenKind::Spaces,
            "USER" => TokenKind::User,
            "USERS" => TokenKind::Users,
            "PASSWORD" => TokenKind::Password,
            "ROLE" => TokenKind::Role,
            "ROLES" => TokenKind::Roles,
            "GOD" => TokenKind::God,
            "DBA" => TokenKind::Dba,
            "GUEST" => TokenKind::Guest,
            "COMMENT" => TokenKind::Comment,
            "CHARSET" => TokenKind::Charset,
            "COLLATE" => TokenKind::Collate,
            "COLLATION" => TokenKind::Collation,
            "VID_TYPE" => TokenKind::VIdType,
            "PARTITION_NUM" => TokenKind::PartitionNum,
            "REPLICA_FACTOR" => TokenKind::ReplicaFactor,
            "REBUILD" => TokenKind::Rebuild,
            "BOOL" => TokenKind::Bool,
            "INT" => TokenKind::Int,
            "INT8" => TokenKind::Int8,
            "INT16" => TokenKind::Int16,
            "INT32" => TokenKind::Int32,
            "INT64" => TokenKind::Int64,
            "FLOAT" => TokenKind::Float,
            "DOUBLE" => TokenKind::Double,
            "STRING" => TokenKind::String,
            "FIXED_STRING" => TokenKind::FixedString,
            "TIMESTAMP" => TokenKind::Timestamp,
            "DATE" => TokenKind::Date,
            "TIME" => TokenKind::Time,
            "DATETIME" => TokenKind::Datetime,
            "DURATION" => TokenKind::Duration,
            "GEOGRAPHY" => TokenKind::Geography,
            "POINT" => TokenKind::Point,
            "LINESTRING" => TokenKind::Linestring,
            "POLYGON" => TokenKind::Polygon,
            "LIST" => TokenKind::List,
            "MAP" => TokenKind::Map,
            "DOWNLOAD" => TokenKind::Download,
            "HDFS" => TokenKind::HDFS,
            "UUID" => TokenKind::UUID,
            "CONFIGS" => TokenKind::Configs,
            "FORCE" => TokenKind::Force,
            "PART" => TokenKind::Part,
            "PARTS" => TokenKind::Parts,
            "DATA" => TokenKind::Data,
            "LEADER" => TokenKind::Leader,
            "JOBS" => TokenKind::Jobs,
            "JOB" => TokenKind::Job,
            "BIDIRECT" => TokenKind::Bidirect,
            "STATS" => TokenKind::Stats,
            "STATUS" => TokenKind::Status,
            "RECOVER" => TokenKind::Recover,
            "EXPLAIN" => TokenKind::Explain,
            "PROFILE" => TokenKind::Profile,
            "FORMAT" => TokenKind::Format,
            "ATOMIC_EDGE" => TokenKind::AtomicEdge,
            "DEFAULT" => TokenKind::Default,
            "FLUSH" => TokenKind::Flush,
            "COMPACT" => TokenKind::Compact,
            "SUBMIT" => TokenKind::Submit,
            "ASCENDING" => TokenKind::Ascending,
            "DESCENDING" => TokenKind::Descending,
            "FETCH" => TokenKind::Fetch,
            "PROP" => TokenKind::Prop,
            "BALANCE" => TokenKind::Balance,
            "STOP" => TokenKind::Stop,
            "REVERT" => TokenKind::Revert,
            "USE" => TokenKind::Use,
            "SETLIST" => TokenKind::SetList,
            "CLEAR" => TokenKind::Clear,
            "MERGE" => TokenKind::Merge,
            "DIVIDE" => TokenKind::Divide,
            "RENAME" => TokenKind::Rename,
            "LOCAL" => TokenKind::Local,
            "SESSIONS" => TokenKind::Sessions,
            "SESSION" => TokenKind::Session,
            "SAMPLE" => TokenKind::Sample,
            "QUERIES" => TokenKind::Queries,
            "QUERY" => TokenKind::Query,
            "KILL" => TokenKind::Kill,
            "TOP" => TokenKind::Top,
            "TEXT" => TokenKind::Text,
            "SEARCH" => TokenKind::Search,
            "CLIENT" => TokenKind::Client,
            "CLIENTS" => TokenKind::Clients,
            "SIGN" => TokenKind::Sign,
            "SERVICE" => TokenKind::Service,

            // 扩展的关键词
            "COUNT" => TokenKind::Count,
            "SUM" => TokenKind::Sum,
            "AVG" => TokenKind::Avg,
            "MIN" => TokenKind::Min,
            "MAX" => TokenKind::Max,
            "SOURCE" => TokenKind::Source,
            "DESTINATION" => TokenKind::Destination,
            "RANK" => TokenKind::Rank,
            "INPUT" => TokenKind::Input,
            _ => TokenKind::Identifier(identifier.to_string()),
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        // Skip comments
        while self.ch == Some('/') || self.ch == Some('-') {
            if self.ch == Some('/') {
                match self.peek_char() {
                    Some('/') | Some('*') => {
                        self.skip_comment();
                        self.skip_whitespace();
                    }
                    _ => break,
                }
            } else if self.ch == Some('-') {
                if self.peek_char() == Some('-') {
                    self.skip_comment();
                    self.skip_whitespace();
                } else {
                    break;
                }
            }
        }

        let token = match self.ch {
            Some('=') => {
                if self.peek_char() == Some('=') {
                    self.read_char(); // Skip the next '='
                    Token::new(TokenKind::Eq, "==".to_string(), self.line, self.column)
                } else {
                    Token::new(TokenKind::Assign, "=".to_string(), self.line, self.column)
                }
            }
            Some('+') => Token::new(TokenKind::Plus, "+".to_string(), self.line, self.column),
            Some('-') => {
                if self.peek_char() == Some('>') {
                    self.read_char(); // Skip the '>'
                    Token::new(TokenKind::Arrow, "->".to_string(), self.line, self.column)
                } else {
                    Token::new(TokenKind::Minus, "-".to_string(), self.line, self.column)
                }
            }
            Some('*') => {
                if self.peek_char() == Some('*') {
                    self.read_char(); // Skip the second '*'
                    Token::new(TokenKind::Exp, "**".to_string(), self.line, self.column)
                } else {
                    Token::new(TokenKind::Star, "*".to_string(), self.line, self.column)
                }
            }
            Some('/') => Token::new(TokenKind::Div, "/".to_string(), self.line, self.column),
            Some('%') => Token::new(TokenKind::Mod, "%".to_string(), self.line, self.column),
            Some('!') => {
                if self.peek_char() == Some('=') {
                    self.read_char(); // Skip the '='
                    Token::new(TokenKind::Ne, "!=".to_string(), self.line, self.column)
                } else {
                    Token::new(TokenKind::NotOp, "!".to_string(), self.line, self.column)
                }
            }
            Some('<') => {
                if self.peek_char() == Some('-') {
                    self.read_char(); // Skip the '-'
                    Token::new(
                        TokenKind::BackArrow,
                        "<-".to_string(),
                        self.line,
                        self.column,
                    )
                } else if self.peek_char() == Some('=') {
                    self.read_char(); // Skip the '='
                    Token::new(TokenKind::Le, "<=".to_string(), self.line, self.column)
                } else {
                    Token::new(TokenKind::Lt, "<".to_string(), self.line, self.column)
                }
            }
            Some('>') => {
                if self.peek_char() == Some('=') {
                    self.read_char(); // Skip the '='
                    Token::new(TokenKind::Ge, ">=".to_string(), self.line, self.column)
                } else {
                    Token::new(TokenKind::Gt, ">".to_string(), self.line, self.column)
                }
            }
            Some('~') => {
                if self.peek_char() == Some('=') {
                    self.read_char(); // Skip the '='
                    Token::new(TokenKind::Regex, "=~".to_string(), self.line, self.column)
                } else {
                    // Handle other cases or return error
                    self.read_char();
                    Token::new(TokenKind::NotOp, "~".to_string(), self.line, self.column)
                }
            }
            Some('(') => Token::new(TokenKind::LParen, "(".to_string(), self.line, self.column),
            Some(')') => Token::new(TokenKind::RParen, ")".to_string(), self.line, self.column),
            Some('[') => Token::new(TokenKind::LBracket, "[".to_string(), self.line, self.column),
            Some(']') => Token::new(TokenKind::RBracket, "]".to_string(), self.line, self.column),
            Some('{') => Token::new(TokenKind::LBrace, "{".to_string(), self.line, self.column),
            Some('}') => Token::new(TokenKind::RBrace, "}".to_string(), self.line, self.column),
            Some(',') => Token::new(TokenKind::Comma, ",".to_string(), self.line, self.column),
            Some('.') => {
                if self.peek_char() == Some('.') {
                    self.read_char(); // Skip the second '.'
                    Token::new(TokenKind::DotDot, "..".to_string(), self.line, self.column)
                } else {
                    Token::new(TokenKind::Dot, ".".to_string(), self.line, self.column)
                }
            }
            Some(':') => Token::new(TokenKind::Colon, ":".to_string(), self.line, self.column),
            Some(';') => Token::new(
                TokenKind::Semicolon,
                ";".to_string(),
                self.line,
                self.column,
            ),
            Some('?') => Token::new(TokenKind::QMark, "?".to_string(), self.line, self.column),
            Some('|') => Token::new(TokenKind::Pipe, "|".to_string(), self.line, self.column),
            Some('@') => Token::new(TokenKind::At, "@".to_string(), self.line, self.column),
            Some('$') => {
                // Check for special property identifiers like $$, $^, $-
                match self.peek_char() {
                    Some('$') => {
                        self.read_char(); // Read the next '$'
                        Token::new(TokenKind::DstRef, "$$".to_string(), self.line, self.column)
                    }
                    Some('^') => {
                        self.read_char(); // Read the '^'
                        Token::new(TokenKind::SrcRef, "$^".to_string(), self.line, self.column)
                    }
                    Some('-') => {
                        self.read_char(); // Read the '-'
                        Token::new(
                            TokenKind::InputRef,
                            "$-".to_string(),
                            self.line,
                            self.column,
                        )
                    }
                    _ => Token::new(TokenKind::Dollar, "$".to_string(), self.line, self.column),
                }
            }
            Some('"') | Some('\'') => {
                let literal = self.read_string();
                Token::new(
                    TokenKind::StringLiteral(literal.clone()),
                    literal.clone(),
                    self.line,
                    self.column,
                )
            }
            Some(ch) if ch.is_ascii_digit() => {
                let literal = self.read_number();
                if literal.contains('.') || literal.contains('e') || literal.contains('E') {
                    let float_val: f64 = literal.parse().unwrap_or(0.0);
                    Token::new(
                        TokenKind::FloatLiteral(float_val),
                        literal,
                        self.line,
                        self.column,
                    )
                } else {
                    let int_val: i64 = literal.parse().unwrap_or(0);
                    Token::new(
                        TokenKind::IntegerLiteral(int_val),
                        literal,
                        self.line,
                        self.column,
                    )
                }
            }
            Some(ch) if ch.is_alphabetic() || ch == '_' => {
                let literal = self.read_identifier();

                // Check for special property identifiers
                match literal.as_str() {
                    "_id" => Token::new(TokenKind::IdProp, literal, self.line, self.column),
                    "_type" => Token::new(TokenKind::TypeProp, literal, self.line, self.column),
                    "_src" => Token::new(TokenKind::SrcIdProp, literal, self.line, self.column),
                    "_dst" => Token::new(TokenKind::DstIdProp, literal, self.line, self.column),
                    "_rank" => Token::new(TokenKind::RankProp, literal, self.line, self.column),
                    _ => {
                        // Check if it's a multi-word keyword that needs to be looked ahead
                        let token_kind = self.lookup_keyword(&literal);
                        match token_kind {
                            // For certain keywords, check if the next token would make it a multi-word keyword
                            TokenKind::Not => {
                                // Check if the next word is "IN"
                                if self.peek_next_word() == "IN" {
                                    // Skip the next word and return "NOT IN"
                                    self.skip_next_word();
                                    Token::new(
                                        TokenKind::NotIn,
                                        "NOT IN".to_string(),
                                        self.line,
                                        self.column,
                                    )
                                } else {
                                    Token::new(token_kind, literal, self.line, self.column)
                                }
                            }
                            TokenKind::Is => {
                                // Check if the next word is "NULL" or "NOT NULL", etc.
                                match self.peek_next_word().as_str() {
                                    "NULL" => {
                                        self.skip_next_word();
                                        Token::new(
                                            TokenKind::IsNull,
                                            "IS NULL".to_string(),
                                            self.line,
                                            self.column,
                                        )
                                    }
                                    "NOT" => {
                                        // Check if after "NOT" comes "NULL"
                                        if self.peek_word_after_next() == "NULL" {
                                            self.skip_next_word(); // skip "NOT"
                                            self.skip_next_word(); // skip "NULL"
                                            Token::new(
                                                TokenKind::IsNotNull,
                                                "IS NOT NULL".to_string(),
                                                self.line,
                                                self.column,
                                            )
                                        } else if self.peek_word_after_next() == "EMPTY" {
                                            self.skip_next_word(); // skip "NOT"
                                            self.skip_next_word(); // skip "EMPTY"
                                            Token::new(
                                                TokenKind::IsNotEmpty,
                                                "IS NOT EMPTY".to_string(),
                                                self.line,
                                                self.column,
                                            )
                                        } else {
                                            Token::new(token_kind, literal, self.line, self.column)
                                        }
                                    }
                                    "EMPTY" => {
                                        self.skip_next_word();
                                        Token::new(
                                            TokenKind::IsEmpty,
                                            "IS EMPTY".to_string(),
                                            self.line,
                                            self.column,
                                        )
                                    }
                                    _ => Token::new(token_kind, literal, self.line, self.column),
                                }
                            }
                            _ => Token::new(token_kind, literal, self.line, self.column),
                        }
                    }
                }
            }
            None => Token::new(TokenKind::Eof, "".to_string(), self.line, self.column),
            Some(other) => {
                // Handle unexpected characters
                let unexpected = other.to_string();
                self.read_char();
                Token::new(
                    TokenKind::Identifier(unexpected.clone()),
                    unexpected,
                    self.line,
                    self.column,
                )
            }
        };

        // Only advance the position for single-character tokens
        // For multi-word tokens, identifiers and literals, we've already advanced the lexer position
        // However, we need to ensure we're positioned at the start of the next token
        match token.kind {
            TokenKind::Identifier(_)
            | TokenKind::StringLiteral(_)
            | TokenKind::IntegerLiteral(_)
            | TokenKind::FloatLiteral(_)
            | TokenKind::BooleanLiteral(_)
            | TokenKind::Count // 关键字已经通过 read_identifier 推进了位置
            | TokenKind::Sum
            | TokenKind::Avg
            | TokenKind::Min
            | TokenKind::Max => {
                // For identifiers, literals and keywords, we've already advanced past them
                // No need to call read_char()
            }
            _ => {
                // For single-character tokens, advance to next character
                if !self.is_multitoken_keyword(&token) {
                    self.read_char();
                }
            }
        }
        token
    }

    fn peek_word_after_next(&self) -> String {
        let mut temp_lexer = self.clone();
        temp_lexer.skip_whitespace();
        temp_lexer.next_token();
        temp_lexer.next_token();
        let next_token = temp_lexer.next_token();
        match next_token.kind {
            TokenKind::Identifier(s) => s,
            _ => next_token.lexeme,
        }
    }

    fn is_multitoken_keyword(&self, token: &Token) -> bool {
        matches!(
            token.kind,
            TokenKind::NotIn
                | TokenKind::IsNull
                | TokenKind::IsNotNull
                | TokenKind::IsEmpty
                | TokenKind::IsNotEmpty
        )
    }

    /// 获取当前位置
    pub fn current_position(&self) -> Position {
        Position::new(self.line, self.column)
    }

    /// 检查是否在文件末尾
    pub fn is_at_end(&self) -> bool {
        self.ch.is_none()
    }

    /// 查看下一个令牌
    pub fn peek(&mut self) -> Result<Token, String> {
        Ok(self.current_token.clone())
    }

    /// 推进到下一个字符
    pub fn advance(&mut self) {
        self.current_token = self.next_token();
    }

    /// 检查当前令牌是否匹配类型
    pub fn check(&mut self, kind: TokenKind) -> bool {
        self.current_token.kind == kind
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_token(token: &Token, expected_kind: TokenKind, expected_lexeme: &str) {
        assert_eq!(
            token.kind, expected_kind,
            "Token kind mismatch. Expected: {:?}, Got: {:?}",
            expected_kind, token.kind
        );
        assert_eq!(
            token.lexeme, expected_lexeme,
            "Lexeme mismatch for token {:?}. Expected: '{}', Got: '{}'",
            expected_kind, expected_lexeme, token.lexeme
        );
    }

    #[test]
    fn test_simple_identifiers() {
        let input = "CREATE MATCH RETURN";
        let mut lexer = Lexer::new(input);

        // Test individual tokens using the lexer API
        println!("First token: {:?} - '{}'", lexer.current_token.kind, lexer.current_token.lexeme);
        assert_eq!(lexer.current_token.kind, TokenKind::Create);
        assert_eq!(lexer.current_token.lexeme, "CREATE");

        lexer.advance();
        println!("Second token: {:?} - '{}'", lexer.current_token.kind, lexer.current_token.lexeme);
        assert_eq!(lexer.current_token.kind, TokenKind::Match);
        assert_eq!(lexer.current_token.lexeme, "MATCH");

        lexer.advance();
        println!("Third token: {:?} - '{}'", lexer.current_token.kind, lexer.current_token.lexeme);
        assert_eq!(lexer.current_token.kind, TokenKind::Return);
        assert_eq!(lexer.current_token.lexeme, "RETURN");

        lexer.advance();
        println!("Fourth token: {:?} - '{}'", lexer.current_token.kind, lexer.current_token.lexeme);
        assert_eq!(lexer.current_token.kind, TokenKind::Eof);
    }

    #[test]
    fn test_operators() {
        let input = "= == ! != < <= > >=";
        let mut lexer = Lexer::new(input);

        assert_eq!(lexer.current_token.kind, TokenKind::Assign);
        lexer.advance();
        assert_eq!(lexer.current_token.kind, TokenKind::Eq);
        lexer.advance();
        assert_eq!(lexer.current_token.kind, TokenKind::NotOp);
        lexer.advance();
        assert_eq!(lexer.current_token.kind, TokenKind::Ne);
        lexer.advance();
        assert_eq!(lexer.current_token.kind, TokenKind::Lt);
        lexer.advance();
        assert_eq!(lexer.current_token.kind, TokenKind::Le);
        lexer.advance();
        assert_eq!(lexer.current_token.kind, TokenKind::Gt);
        lexer.advance();
        assert_eq!(lexer.current_token.kind, TokenKind::Ge);
    }

    #[test]
    fn test_literals() {
        let input = "42 3.14 \"hello\" true false";
        let mut lexer = Lexer::new(input);

        assert_eq!(lexer.current_token.kind, TokenKind::IntegerLiteral(42));
        lexer.advance();
        assert_eq!(lexer.current_token.kind, TokenKind::FloatLiteral(3.14));
        lexer.advance();
        assert_eq!(
            lexer.current_token.kind,
            TokenKind::StringLiteral("hello".to_string())
        );
        lexer.advance();
        assert_eq!(lexer.current_token.kind, TokenKind::Identifier("true".to_string()));
        lexer.advance();
        assert_eq!(lexer.current_token.kind, TokenKind::Identifier("false".to_string()));
    }

    #[test]
    fn test_arrows() {
        let input = "-> <-";
        let mut lexer = Lexer::new(input);

        assert_eq!(lexer.current_token.kind, TokenKind::Arrow);
        lexer.advance();
        assert_eq!(lexer.current_token.kind, TokenKind::BackArrow);
    }

    #[test]
    fn test_special_properties() {
        let input = "_id _type _src _dst _rank";
        let mut lexer = Lexer::new(input);

        assert_token(&lexer.current_token, TokenKind::IdProp, "_id");
        lexer.advance();
        assert_token(&lexer.current_token, TokenKind::TypeProp, "_type");
        lexer.advance();
        assert_token(&lexer.current_token, TokenKind::SrcIdProp, "_src");
        lexer.advance();
        assert_token(&lexer.current_token, TokenKind::DstIdProp, "_dst");
        lexer.advance();
        assert_token(&lexer.current_token, TokenKind::RankProp, "_rank");
    }

    #[test]
    fn test_graph_reference_identifiers() {
        let input = "$$ $^ $-";
        let mut lexer = Lexer::new(input);

        assert_token(&lexer.current_token, TokenKind::DstRef, "$$");
        lexer.advance();
        assert_token(&lexer.current_token, TokenKind::SrcRef, "$^");
        lexer.advance();
        assert_token(&lexer.current_token, TokenKind::InputRef, "$-");
    }

    #[test]
    fn test_aggregation_functions() {
        let input = "COUNT SUM AVG MIN MAX";
        let mut lexer = Lexer::new(input);

        assert_token(&lexer.current_token, TokenKind::Count, "COUNT");
        lexer.advance();
        assert_token(&lexer.current_token, TokenKind::Sum, "SUM");
        lexer.advance();
        assert_token(&lexer.current_token, TokenKind::Avg, "AVG");
        lexer.advance();
        assert_token(&lexer.current_token, TokenKind::Min, "MIN");
        lexer.advance();
        assert_token(&lexer.current_token, TokenKind::Max, "MAX");
    }

    #[test]
    fn test_new_keywords() {
        let input = "SOURCE DESTINATION RANK INPUT";
        let mut lexer = Lexer::new(input);

        assert_token(&lexer.current_token, TokenKind::Source, "SOURCE");
        lexer.advance();
        assert_token(&lexer.current_token, TokenKind::Destination, "DESTINATION");
        lexer.advance();
        assert_token(&lexer.current_token, TokenKind::Rank, "RANK");
        lexer.advance();
        assert_token(&lexer.current_token, TokenKind::Input, "INPUT");
    }

    #[test]
    fn test_basic_functionality() {
        // Test that basic functionality still works after our enhancements
        let input = "CREATE (n:Person {name: 'John'}) RETURN n.name";
        let mut lexer = Lexer::new(input);

        assert_token(&lexer.current_token, TokenKind::Create, "CREATE");
        lexer.advance();
        assert_token(&lexer.current_token, TokenKind::LParen, "(");
        lexer.advance();
        assert_token(
            &lexer.current_token,
            TokenKind::Identifier("n".to_string()),
            "n",
        );
        lexer.advance();
        assert_token(&lexer.current_token, TokenKind::Colon, ":");
        lexer.advance();
        assert_token(
            &lexer.current_token,
            TokenKind::Identifier("Person".to_string()),
            "Person",
        );
    }

    #[test]
    fn test_count_function() {
        let mut lexer = Lexer::new("COUNT(x)");
        assert_token(&lexer.current_token, TokenKind::Count, "COUNT");
        lexer.advance();
        assert_token(&lexer.current_token, TokenKind::LParen, "(");
        lexer.advance();
        assert_token(&lexer.current_token, TokenKind::Identifier("x".to_string()), "x");
        lexer.advance();
        assert_token(&lexer.current_token, TokenKind::RParen, ")");
    }

    #[test]
    fn test_string_escape_sequences() {
        let input = r#""hello\nworld\t!""#;
        let mut lexer = Lexer::new(input);
        let token = lexer.current_token.clone();
        match token.kind {
            TokenKind::StringLiteral(content) => {
                assert!(content.contains('\n'), "Should contain newline escape");
                assert!(content.contains('\t'), "Should contain tab escape");
            }
            _ => panic!("Expected StringLiteral, got {:?}", token.kind),
        }
    }

    #[test]
    fn test_string_unicode_escape() {
        let input = r#""\u0041""#;
        let mut lexer = Lexer::new(input);
        let token = lexer.current_token.clone();
        match token.kind {
            TokenKind::StringLiteral(content) => {
                assert_eq!(content, "A", "Unicode escape \\u0041 should produce 'A'");
            }
            _ => panic!("Expected StringLiteral, got {:?}", token.kind),
        }
    }

    #[test]
    fn test_single_line_comment() {
        let input = "CREATE -- this is a comment\nMATCH";
        let mut lexer = Lexer::new(input);
        assert_token(&lexer.current_token, TokenKind::Create, "CREATE");
        lexer.advance();
        assert_token(&lexer.current_token, TokenKind::Match, "MATCH");
    }

    #[test]
    fn test_multi_line_comment() {
        let input = "CREATE /* multi line\ncomment */ MATCH";
        let mut lexer = Lexer::new(input);
        assert_token(&lexer.current_token, TokenKind::Create, "CREATE");
        lexer.advance();
        assert_token(&lexer.current_token, TokenKind::Match, "MATCH");
    }

    #[test]
    fn test_scientific_notation() {
        let input = "1.5e10 2.5E-3 1e+5";
        let mut lexer = Lexer::new(input);

        assert_eq!(lexer.current_token.kind, TokenKind::FloatLiteral(1.5e10));
        lexer.advance();
        assert_eq!(lexer.current_token.kind, TokenKind::FloatLiteral(2.5e-3));
        lexer.advance();
        assert_eq!(lexer.current_token.kind, TokenKind::FloatLiteral(1e5));
    }

    #[test]
    fn test_comment_in_query() {
        let input = r#"
            CREATE (n:Person {name: "John"})
            -- Add comment here
            /* Another comment */
            RETURN n.name
        "#;
        let mut lexer = Lexer::new(input);
        assert_token(&lexer.current_token, TokenKind::Create, "CREATE");
        lexer.advance();
        assert_token(&lexer.current_token, TokenKind::LParen, "(");
        lexer.advance();
        assert_token(
            &lexer.current_token,
            TokenKind::Identifier("n".to_string()),
            "n",
        );
    }
}

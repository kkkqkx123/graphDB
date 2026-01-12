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
        while let Some(ch) = self.ch {
            if ch.is_ascii_digit() {
                self.read_char();
            } else {
                break;
            }
        }
        self.input[start_position..self.position].iter().collect()
    }

    fn read_string(&mut self) -> String {
        self.read_char(); // Skip opening quote
        let start_position = self.position;

        while let Some(ch) = self.ch {
            if ch == '"' || ch == '\'' {
                break;
            }
            self.read_char();
        }

        let result: String = self.input[start_position..self.position].iter().collect();
        self.read_char(); // Skip closing quote
        result
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
                if self.ch == Some('.') && self.peek_char().map_or(false, |c| c.is_ascii_digit()) {
                    // This is a float
                    self.read_char(); // Skip the '.'
                    let float_literal = format!("{}.{}", literal, self.read_number());
                    let float_val: f64 = float_literal.parse().unwrap_or(0.0);
                    Token::new(
                        TokenKind::FloatLiteral(float_val),
                        float_literal,
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
        if !self.is_multitoken_keyword(&token)
            && !matches!(
                token.kind,
                TokenKind::Identifier(_)
                    | TokenKind::StringLiteral(_)
                    | TokenKind::IntegerLiteral(_)
                    | TokenKind::FloatLiteral(_)
                    | TokenKind::BooleanLiteral(_)
            )
        {
            self.read_char();
        }
        token
    }
    // Helper methods for multi-word token detection
    fn peek_next_word(&self) -> String {
        // Create a temporary lexer to peek ahead
        let mut temp_lexer = Lexer::new(&self.get_remaining_input());
        // Skip the current token by reading and discarding it
        temp_lexer.next_token();
        // Get the next token and return its lexeme if it's an identifier
        let next_token = temp_lexer.next_token();
        match next_token.kind {
            TokenKind::Identifier(s) => s,
            _ => next_token.lexeme,
        }
    }

    fn peek_word_after_next(&self) -> String {
        // Create a temporary lexer to peek ahead
        let mut temp_lexer = Lexer::new(&self.get_remaining_input());
        // Skip the current and next token
        temp_lexer.next_token();
        temp_lexer.next_token();
        // Get the token after that and return its lexeme if it's an identifier
        let next_token = temp_lexer.next_token();
        match next_token.kind {
            TokenKind::Identifier(s) => s,
            _ => next_token.lexeme,
        }
    }

    fn skip_next_word(&mut self) {
        // Skip whitespace
        self.skip_whitespace();
        // Read an identifier or keyword
        if let Some(ch) = self.ch {
            if ch.is_alphabetic() || ch == '_' {
                // Skip the identifier
                while let Some(inner_ch) = self.ch {
                    if inner_ch.is_alphanumeric() || inner_ch == '_' {
                        self.read_char();
                    } else {
                        break;
                    }
                }
            }
        }
    }

    fn get_remaining_input(&self) -> String {
        // Get the remaining input from current position
        self.input[self.position..].iter().collect()
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

        let tokens: Vec<Token> = std::iter::from_fn(|| {
            let token = lexer.next_token();
            if token.kind == TokenKind::Eof {
                None
            } else {
                Some(token)
            }
        })
        .collect();

        assert_eq!(tokens[0].kind, TokenKind::Create);
        assert_eq!(tokens[0].lexeme, "CREATE");
        assert_eq!(tokens[1].kind, TokenKind::Match);
        assert_eq!(tokens[1].lexeme, "MATCH");
        assert_eq!(tokens[2].kind, TokenKind::Return);
        assert_eq!(tokens[2].lexeme, "RETURN");
    }

    #[test]
    fn test_operators() {
        let input = "= == ! != < <= > >=";
        let mut lexer = Lexer::new(input);

        let tokens: Vec<Token> = std::iter::from_fn(|| {
            let token = lexer.next_token();
            if token.kind == TokenKind::Eof {
                None
            } else {
                Some(token)
            }
        })
        .collect();

        assert_eq!(tokens[0].kind, TokenKind::Assign);
        assert_eq!(tokens[1].kind, TokenKind::Eq);
        assert_eq!(tokens[2].kind, TokenKind::NotOp);
        assert_eq!(tokens[3].kind, TokenKind::Ne);
        assert_eq!(tokens[4].kind, TokenKind::Lt);
        assert_eq!(tokens[5].kind, TokenKind::Le);
        assert_eq!(tokens[6].kind, TokenKind::Gt);
        assert_eq!(tokens[7].kind, TokenKind::Ge);
    }

    #[test]
    fn test_literals() {
        let input = "42 3.14 \"hello\" true false";
        let mut lexer = Lexer::new(input);

        let tokens: Vec<Token> = std::iter::from_fn(|| {
            let token = lexer.next_token();
            if token.kind == TokenKind::Eof {
                None
            } else {
                Some(token)
            }
        })
        .collect();

        assert_eq!(tokens[0].kind, TokenKind::IntegerLiteral(42));
        assert_eq!(tokens[1].kind, TokenKind::FloatLiteral(3.14));
        assert_eq!(
            tokens[2].kind,
            TokenKind::StringLiteral("hello".to_string())
        );
        assert_eq!(tokens[3].kind, TokenKind::Identifier("true".to_string()));
        assert_eq!(tokens[4].kind, TokenKind::Identifier("false".to_string()));
    }

    #[test]
    fn test_arrows() {
        let input = "-> <-";
        let mut lexer = Lexer::new(input);

        let tokens: Vec<Token> = std::iter::from_fn(|| {
            let token = lexer.next_token();
            if token.kind == TokenKind::Eof {
                None
            } else {
                Some(token)
            }
        })
        .collect();

        assert_eq!(tokens[0].kind, TokenKind::Arrow);
        assert_eq!(tokens[1].kind, TokenKind::BackArrow);
    }

    #[test]
    fn test_special_properties() {
        let input = "_id _type _src _dst _rank";
        let mut lexer = Lexer::new(input);

        assert_token(&lexer.next_token(), TokenKind::IdProp, "_id");
        assert_token(&lexer.next_token(), TokenKind::TypeProp, "_type");
        assert_token(&lexer.next_token(), TokenKind::SrcIdProp, "_src");
        assert_token(&lexer.next_token(), TokenKind::DstIdProp, "_dst");
        assert_token(&lexer.next_token(), TokenKind::RankProp, "_rank");
        assert_token(&lexer.next_token(), TokenKind::Eof, "");
    }

    #[test]
    fn test_graph_reference_identifiers() {
        let input = "$$ $^ $-";
        let mut lexer = Lexer::new(input);

        assert_token(&lexer.next_token(), TokenKind::DstRef, "$$");
        assert_token(&lexer.next_token(), TokenKind::SrcRef, "$^");
        assert_token(&lexer.next_token(), TokenKind::InputRef, "$-");
        assert_token(&lexer.next_token(), TokenKind::Eof, "");
    }

    #[test]
    fn test_aggregation_functions() {
        let input = "COUNT SUM AVG MIN MAX";
        let mut lexer = Lexer::new(input);

        assert_token(&lexer.next_token(), TokenKind::Count, "COUNT");
        assert_token(&lexer.next_token(), TokenKind::Sum, "SUM");
        assert_token(&lexer.next_token(), TokenKind::Avg, "AVG");
        assert_token(&lexer.next_token(), TokenKind::Min, "MIN");
        assert_token(&lexer.next_token(), TokenKind::Max, "MAX");
        assert_token(&lexer.next_token(), TokenKind::Eof, "");
    }

    #[test]
    fn test_new_keywords() {
        let input = "SOURCE DESTINATION RANK INPUT";
        let mut lexer = Lexer::new(input);

        assert_token(&lexer.next_token(), TokenKind::Source, "SOURCE");
        assert_token(&lexer.next_token(), TokenKind::Destination, "DESTINATION");
        assert_token(&lexer.next_token(), TokenKind::Rank, "RANK");
        assert_token(&lexer.next_token(), TokenKind::Input, "INPUT");
        assert_token(&lexer.next_token(), TokenKind::Eof, "");
    }

    #[test]
    fn test_basic_functionality() {
        // Test that basic functionality still works after our enhancements
        let input = "CREATE (n:Person {name: 'John'}) RETURN n.name";
        let mut lexer = Lexer::new(input);

        assert_token(&lexer.next_token(), TokenKind::Create, "CREATE");
        assert_token(&lexer.next_token(), TokenKind::LParen, "(");
        assert_token(
            &lexer.next_token(),
            TokenKind::Identifier("n".to_string()),
            "n",
        );
        assert_token(&lexer.next_token(), TokenKind::Colon, ":");
        assert_token(
            &lexer.next_token(),
            TokenKind::Identifier("Person".to_string()),
            "Person",
        );
    }
}

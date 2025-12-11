//! Lexer implementation for the query parser
//!
//! This module implements a lexical analyzer that converts input query strings into tokens.

use crate::query::parser::core::token::{Token, TokenKind};

pub struct Lexer {
    input: Vec<char>,
    position: usize,      // Current position in input
    read_position: usize, // Next position to read
    ch: Option<char>,     // Current character
    line: usize,          // Current line number
    column: usize,        // Current column number
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
        };
        lexer.read_char();
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

    fn read_float(&mut self) -> String {
        let start_position = self.position;
        while let Some(ch) = self.ch {
            if ch.is_ascii_digit() || ch == '.' {
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
            Some('*') => Token::new(TokenKind::Star, "*".to_string(), self.line, self.column),
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
            Some('$') => Token::new(TokenKind::Dollar, "$".to_string(), self.line, self.column),
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
                let token_kind = self.lookup_keyword(&literal);
                Token::new(token_kind, literal, self.line, self.column)
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

        self.read_char();
        token
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}

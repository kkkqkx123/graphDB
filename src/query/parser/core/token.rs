//! Token definitions for the query parser
//!
//! This module defines the lexical tokens used by the parser.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Position {
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

impl Span {
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    pub fn from_tokens(start_line: usize, start_col: usize, end_line: usize, end_col: usize) -> Self {
        Self {
            start: Position::new(start_line, start_col),
            end: Position::new(end_line, end_col),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    Create,
    Match,
    Return,
    Where,
    Delete,
    Update,
    Insert,
    Upsert,
    From,
    To,
    As,
    With,
    Yield,
    Go,
    Over,
    Step,
    Upto,
    Limit,
    Asc,
    Desc,
    Order,
    By,
    Skip,
    Unwind,
    Optional,
    Distinct,
    All,
    Null,
    Is,
    Not,
    And,
    Or,
    Xor,
    Contains,
    StartsWith,
    EndsWith,
    Case,
    When,
    Then,
    Else,
    End,
    Union,
    Intersect,
    Group,
    Between,
    Admin,
    Edge,
    Edges,
    Vertex,
    Vertices,
    Tag,
    Tags,
    Index,
    Indexes,
    Lookup,
    Find,
    Path,
    Shortest,
    NoLoop,
    AllShortestPaths,
    Subgraph,
    Both,
    Out,
    In,
    No,
    Overwrite,
    Show,
    Add,
    Drop,
    Remove,
    If,
    Exists,
    Change,
    Grant,
    Revoke,
    On,
    Of,
    Get,
    Set,
    Host,
    Hosts,
    Space,
    Spaces,
    User,
    Users,
    Password,
    Role,
    Roles,
    God,
    AdminRole,
    Dba,
    Guest,
    Comment,
    Charset,
    Collate,
    Collation,
    VIdType,
    PartitionNum,
    ReplicaFactor,
    Rebuild,
    Bool,
    Int,
    Int8,
    Int16,
    Int32,
    Int64,
    Float,
    Double,
    String,
    FixedString,
    Timestamp,
    Date,
    Time,
    Datetime,
    Duration,
    Geography,
    Point,
    Linestring,
    Polygon,
    List,
    Map,
    Download,
    HDFS,
    UUID,
    Configs,
    Force,
    Part,
    Parts,
    Data,
    Leader,
    Jobs,
    Job,
    Bidirect,
    Stats,
    Status,
    Recover,
    Explain,
    Profile,
    Format,
    AtomicEdge,
    Default,
    Flush,
    Compact,
    Submit,
    Ascending,
    Descending,
    Fetch,
    Prop,
    Balance,
    Stop,
    Revert,
    Use,
    SetList,
    Clear,
    Merge,
    Divide,
    Rename,
    Local,
    Sessions,
    Session,
    Sample,
    Queries,
    Query,
    Kill,
    Top,
    Text,
    Search,
    Client,
    Clients,
    Sign,
    Service,

    // 扩展的关键词
    Count,
    Sum,
    Avg,
    Min,
    Max,
    NotIn,
    IsNull,
    IsNotNull,
    IsEmpty,
    IsNotEmpty,
    Outbound,
    Inbound,
    Source,
    Destination,
    Rank,
    Input,
    FindPath,

    // Literals
    Identifier(String),
    StringLiteral(String),
    IntegerLiteral(i64),
    FloatLiteral(f64),
    BooleanLiteral(bool),

    // Operators
    Plus,   // +
    Minus,  // -
    Star,   // *
    Div,    // /
    Mod,    // %
    Exp,    // **
    Eq,     // ==
    Assign, // =
    Ne,     // !=
    Lt,     // <
    Le,     // <=
    Gt,     // >
    Ge,     // >=
    Regex,  // =~
    NotOp,  // !

    // Delimiters
    LParen,     // (
    RParen,     // )
    LBracket,   // [
    RBracket,   // ]
    LBrace,     // {
    RBrace,     // }
    Comma,      // ,
    Dot,        // .
    DotDot,     // ..
    Colon,      // :
    Semicolon,  // ;
    QMark,      // ?
    Question,   // ? (别名)
    Pipe,       // |
    Arrow,      // ->
    BackArrow,  // <-
    RightArrow, // -> (别名)
    LeftArrow,  // <- (别名)
    At,         // @
    Dollar,     // $

    // Special properties
    IdProp,    // _id
    TypeProp,  // _type
    SrcIdProp, // _src
    DstIdProp, // _dst
    RankProp,  // _rank

    // Graph reference identifiers
    DstRef,   // $$
    SrcRef,   // $^
    InputRef, // $-

    // End of input
    Eof,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Identifier(s) => write!(f, "{}", s),
            TokenKind::StringLiteral(s) => write!(f, "\"{}\"", s),
            TokenKind::IntegerLiteral(n) => write!(f, "{}", n),
            TokenKind::FloatLiteral(n) => write!(f, "{}", n),
            TokenKind::BooleanLiteral(b) => write!(f, "{}", b),
            _ => write!(f, "{:?}", self),
        }
    }
}

impl Token {
    pub fn new(kind: TokenKind, lexeme: String, line: usize, column: usize) -> Self {
        Token {
            kind,
            lexeme,
            line,
            column,
        }
    }
}

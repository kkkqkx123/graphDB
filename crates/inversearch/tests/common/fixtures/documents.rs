//! 测试文档数据固件
//!
//! 提供各种测试场景所需的文档数据

#![allow(dead_code)]

/// 测试文档结构
#[derive(Debug, Clone, Copy)]
pub struct TestDocument {
    pub id: u64,
    pub content: &'static str,
}

/// 编程语言相关文档 - 用于基本功能测试
pub const PROGRAMMING_DOCS: &[TestDocument] = &[
    TestDocument {
        id: 1,
        content: "Rust is a systems programming language focused on safety and performance",
    },
    TestDocument {
        id: 2,
        content: "Python is a high-level programming language known for its simplicity",
    },
    TestDocument {
        id: 3,
        content: "JavaScript is the programming language of the web",
    },
    TestDocument {
        id: 4,
        content: "Go is a statically typed compiled programming language",
    },
    TestDocument {
        id: 5,
        content: "TypeScript is a typed superset of JavaScript that compiles to plain JavaScript",
    },
    TestDocument {
        id: 6,
        content: "Java is a class-based object-oriented programming language",
    },
    TestDocument {
        id: 7,
        content: "C++ is a general-purpose programming language with imperative and object-oriented features",
    },
    TestDocument {
        id: 8,
        content: "Ruby is a dynamic open source programming language with a focus on simplicity",
    },
];

/// 中文文档 - 用于 CJK 字符集测试
pub const CHINESE_DOCS: &[TestDocument] = &[
    TestDocument {
        id: 100,
        content: "Rust是一种系统编程语言，专注于安全和性能",
    },
    TestDocument {
        id: 101,
        content: "Python是一种高级编程语言，以简洁著称",
    },
    TestDocument {
        id: 102,
        content: "JavaScript是Web的编程语言",
    },
    TestDocument {
        id: 103,
        content: "搜索引擎是用于检索信息的系统",
    },
    TestDocument {
        id: 104,
        content: "倒排索引是一种常用的数据结构",
    },
];

/// 日文文档 - 用于日文测试
pub const JAPANESE_DOCS: &[TestDocument] = &[
    TestDocument {
        id: 200,
        content: "Rustはシステムプログラミング言語です",
    },
    TestDocument {
        id: 201,
        content: "Pythonは高水準プログラミング言語です",
    },
    TestDocument {
        id: 202,
        content: "JavaScriptはウェブのプログラミング言語です",
    },
    TestDocument {
        id: 203,
        content: "検索エンジンは情報を検索するシステムです",
    },
];

/// 韩文文档 - 用于韩文测试
pub const KOREAN_DOCS: &[TestDocument] = &[
    TestDocument {
        id: 300,
        content: "Rust는 시스템 프로그래밍 언어입니다",
    },
    TestDocument {
        id: 301,
        content: "Python은 고급 프로그래밍 언어입니다",
    },
    TestDocument {
        id: 302,
        content: "JavaScript는 웹의 프로그래밍 언어입니다",
    },
];

/// 混合语言文档 - 用于多语言测试
pub const MIXED_LANG_DOCS: &[TestDocument] = &[
    TestDocument {
        id: 400,
        content: "Rust编程语言は高性能です",
    },
    TestDocument {
        id: 401,
        content: "Python 프로그래밍は簡単です",
    },
    TestDocument {
        id: 402,
        content: "Search engine 검색エンジン",
    },
];

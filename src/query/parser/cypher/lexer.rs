//! Cypher词法分析器

/// Cypher词法分析器
#[derive(Debug)]
pub struct CypherLexer {
    input: String,
    position: usize,
    tokens: Vec<Token>,
}

/// 词法标记
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub value: String,
    pub position: usize,
}

/// 标记类型
#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Keyword,        // MATCH, RETURN, CREATE, etc.
    Identifier,     // 变量名、标签名、类型名
    LiteralString,  // 字符串字面量
    LiteralNumber,  // 数字字面量
    LiteralBoolean, // 布尔字面量
    Operator,       // +, -, *, /, =, <, >, etc.
    Punctuation,    // (, ), [, ], {, }, :, ,, ;, .
    Whitespace,     // 空格、制表符、换行符
    Comment,        // 注释
    EOF,            // 文件结束
}

impl CypherLexer {
    /// 创建新的Cypher词法分析器
    pub fn new(input: String) -> Self {
        Self {
            input,
            position: 0,
            tokens: Vec::new(),
        }
    }

    /// 词法分析
    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        self.tokens.clear();

        while !self.is_eof() {
            let token = self.next_token()?;
            self.tokens.push(token);
        }

        // 添加EOF标记
        self.tokens.push(Token {
            token_type: TokenType::EOF,
            value: "".to_string(),
            position: self.position,
        });

        Ok(self.tokens.clone())
    }

    /// 获取下一个标记
    fn next_token(&mut self) -> Result<Token, String> {
        self.skip_whitespace();

        if self.is_eof() {
            return Ok(Token {
                token_type: TokenType::EOF,
                value: "".to_string(),
                position: self.position,
            });
        }

        let ch = self
            .peek_char()
            .expect("Lexer should have a character to peek at");
        let position = self.position;

        match ch {
            // 字符串字面量
            '"' => {
                let value = self.read_string()?;
                Ok(Token {
                    token_type: TokenType::LiteralString,
                    value,
                    position,
                })
            }

            // 数字字面量
            '0'..='9' => {
                let value = self.read_number()?;
                Ok(Token {
                    token_type: TokenType::LiteralNumber,
                    value,
                    position,
                })
            }

            // 标识符或关键字
            'a'..='z' | 'A'..='Z' | '_' => {
                let value = self.read_identifier()?;
                let token_type = if Self::is_keyword(&value) {
                    TokenType::Keyword
                } else {
                    TokenType::Identifier
                };
                Ok(Token {
                    token_type,
                    value,
                    position,
                })
            }

            // 标点符号
            '(' | ')' | '[' | ']' | '{' | '}' | ':' | ',' | ';' | '.' => {
                let value = self.read_punctuation()?;
                Ok(Token {
                    token_type: TokenType::Punctuation,
                    value,
                    position,
                })
            }

            // 注释
            '/' if self.peek_next_char() == Some('/') => {
                let value = self.read_comment()?;
                Ok(Token {
                    token_type: TokenType::Comment,
                    value,
                    position,
                })
            }

            // 操作符
            '+' | '-' | '*' | '/' | '=' | '<' | '>' | '!' | '|' | '&' => {
                let value = self.read_operator()?;
                Ok(Token {
                    token_type: TokenType::Operator,
                    value,
                    position,
                })
            }

            _ => Err(format!("无法识别的字符: '{}'", ch)),
        }
    }

    /// 读取字符串字面量
    fn read_string(&mut self) -> Result<String, String> {
        self.expect_char('"')?;
        let mut string = String::new();

        while let Some(ch) = self.peek_char() {
            if ch == '"' {
                break;
            }
            if ch == '\\' {
                self.consume_char();
                if let Some(escaped) = self.parse_escape_sequence()? {
                    string.push(escaped);
                }
            } else {
                string.push(ch);
                self.consume_char();
            }
        }

        self.expect_char('"')?;
        Ok(string)
    }

    /// 解析转义序列
    fn parse_escape_sequence(&mut self) -> Result<char, String> {
        match self.peek_char() {
            Some('n') => {
                self.consume_char();
                Ok('\n')
            }
            Some('t') => {
                self.consume_char();
                Ok('\t')
            }
            Some('r') => {
                self.consume_char();
                Ok('\r')
            }
            Some('\\') => {
                self.consume_char();
                Ok('\\')
            }
            Some('"') => {
                self.consume_char();
                Ok('"')
            }
            Some('\'') => {
                self.consume_char();
                Ok('\'')
            }
            Some('b') => {
                self.consume_char();
                Ok('\x08')
            }
            Some('f') => {
                self.consume_char();
                Ok('\x0c')
            }
            Some('u') => self.parse_unicode_escape(),
            Some('U') => self.parse_unicode_escape_long(),
            Some(c) if c.is_digit(10) => self.parse_octal_escape(),
            _ => Err(format!(
                "无效的转义序列: '\\{}'",
                self.peek_char().unwrap_or(' ')
            )),
        }
    }

    /// 解析 Unicode 转义序列（4位十六进制）
    fn parse_unicode_escape(&mut self) -> Result<char, String> {
        self.expect_char('u')?;
        let mut code = String::new();
        for _ in 0..4 {
            if let Some(ch) = self.peek_char() {
                if ch.is_ascii_hexdigit() {
                    code.push(ch);
                    self.consume_char();
                } else {
                    return Err(format!("期望十六进制数字，得到 '{}'", ch));
                }
            } else {
                return Err("意外的文件结束在 Unicode 转义序列中".to_string());
            }
        }
        let code_point = u32::from_str_radix(&code, 16)
            .map_err(|e| format!("无效的 Unicode 码点: {}", e))?;
        char::from_u32(code_point)
            .ok_or_else(|| format!("无效的 Unicode 码点: 0x{}", code))
    }

    /// 解析长 Unicode 转义序列（8位十六进制）
    fn parse_unicode_escape_long(&mut self) -> Result<char, String> {
        self.expect_char('U')?;
        let mut code = String::new();
        for _ in 0..8 {
            if let Some(ch) = self.peek_char() {
                if ch.is_ascii_hexdigit() {
                    code.push(ch);
                    self.consume_char();
                } else {
                    return Err(format!("期望十六进制数字，得到 '{}'", ch));
                }
            } else {
                return Err("意外的文件结束在 Unicode 转义序列中".to_string());
            }
        }
        let code_point = u32::from_str_radix(&code, 16)
            .map_err(|e| format!("无效的 Unicode 码点: {}", e))?;
        char::from_u32(code_point)
            .ok_or_else(|| format!("无效的 Unicode 码点: 0x{}", code))
    }

    /// 解析八进制转义序列
    fn parse_octal_escape(&mut self) -> Result<char, String> {
        let mut octal = String::new();
        for _ in 0..3 {
            if let Some(ch) = self.peek_char() {
                if ch.is_digit(8) {
                    octal.push(ch);
                    self.consume_char();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        if octal.is_empty() {
            return Err("期望八进制数字".to_string());
        }
        let code = u8::from_str_radix(&octal, 8)
            .map_err(|e| format!("无效的八进制转义序列: {}", e))?;
        Ok(code as char)
    }

    /// 读取数字字面量
    fn read_number(&mut self) -> Result<String, String> {
        let mut number = String::new();

        // 检查十六进制数字
        if self.peek_char() == Some('0') && self.peek_next_char() == Some('x') {
            self.consume_char(); // '0'
            self.consume_char(); // 'x'
            while let Some(ch) = self.peek_char() {
                if ch.is_ascii_hexdigit() {
                    number.push(ch);
                    self.consume_char();
                } else {
                    break;
                }
            }
            return Ok(format!("0x{}", number));
        }

        // 检查二进制数字
        if self.peek_char() == Some('0') && self.peek_next_char() == Some('b') {
            self.consume_char(); // '0'
            self.consume_char(); // 'b'
            while let Some(ch) = self.peek_char() {
                if ch == '0' || ch == '1' {
                    number.push(ch);
                    self.consume_char();
                } else {
                    break;
                }
            }
            return Ok(format!("0b{}", number));
        }

        // 普通数字（可能包含小数点和科学计数法）
        let mut has_decimal = false;
        let mut has_exponent = false;

        while let Some(ch) = self.peek_char() {
            if ch.is_digit(10) {
                number.push(ch);
                self.consume_char();
            } else if ch == '.' && !has_decimal && !has_exponent {
                number.push(ch);
                has_decimal = true;
                self.consume_char();
            } else if (ch == 'e' || ch == 'E') && !has_exponent {
                number.push(ch);
                has_exponent = true;
                self.consume_char();

                // 检查指数符号
                if let Some(next_ch) = self.peek_char() {
                    if next_ch == '+' || next_ch == '-' {
                        number.push(next_ch);
                        self.consume_char();
                    }
                }
            } else {
                break;
            }
        }

        Ok(number)
    }

    /// 读取标识符
    fn read_identifier(&mut self) -> Result<String, String> {
        let mut identifier = String::new();

        while let Some(ch) = self.peek_char() {
            if ch.is_alphanumeric() || ch == '_' {
                identifier.push(ch);
                self.consume_char();
            } else {
                break;
            }
        }

        Ok(identifier)
    }

    /// 读取标点符号
    fn read_punctuation(&mut self) -> Result<String, String> {
        let ch = self
            .peek_char()
            .expect("Lexer should have a character to peek at");
        self.consume_char();
        Ok(ch.to_string())
    }

    /// 读取操作符
    fn read_operator(&mut self) -> Result<String, String> {
        let mut operator = String::new();
        let first_char = self
            .peek_char()
            .expect("Lexer should have a character to peek at");
        operator.push(first_char);
        self.consume_char();

        // 检查多字符操作符
        if let Some(next_char) = self.peek_char() {
            match (first_char, next_char) {
                ('=', '=') | ('!', '=') | ('<', '=') | ('>', '=') | ('|', '|') | ('&', '&') => {
                    operator.push(next_char);
                    self.consume_char();
                }
                ('-', '>') | ('<', '-') => {
                    operator.push(next_char);
                    self.consume_char();
                }
                _ => {}
            }
        }

        Ok(operator)
    }

    /// 读取注释
    fn read_comment(&mut self) -> Result<String, String> {
        self.expect_char('/')?;

        if self.peek_char() == Some('/') {
            // 单行注释
            self.expect_char('/')?;
            let mut comment = String::new();
            while let Some(ch) = self.peek_char() {
                if ch == '\n' {
                    break;
                }
                comment.push(ch);
                self.consume_char();
            }
            Ok(comment)
        } else if self.peek_char() == Some('*') {
            // 多行注释
            self.expect_char('*')?;
            let mut comment = String::new();
            let mut nesting = 1;

            while nesting > 0 {
                if let Some(ch) = self.peek_char() {
                    if ch == '/' && self.peek_next_char() == Some('*') {
                        self.consume_char(); // '/'
                        self.consume_char(); // '*'
                        nesting += 1;
                        comment.push_str("/*");
                    } else if ch == '*' && self.peek_next_char() == Some('/') {
                        self.consume_char(); // '*'
                        self.consume_char(); // '/'
                        nesting -= 1;
                        if nesting > 0 {
                            comment.push_str("*/");
                        }
                    } else {
                        comment.push(ch);
                        self.consume_char();
                    }
                } else {
                    return Err("意外的文件结束在多行注释中".to_string());
                }
            }
            Ok(comment)
        } else {
            Err("期望注释标记（// 或 /*）".to_string())
        }
    }

    /// 跳过空白字符
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek_char() {
            if ch.is_whitespace() {
                self.consume_char();
            } else {
                break;
            }
        }
    }

    /// 检查是否为关键字
    fn is_keyword(word: &str) -> bool {
        let keywords = vec![
            // Cypher 关键字
            "MATCH", "RETURN", "CREATE", "DELETE", "SET", "REMOVE", "MERGE", "WITH", "UNWIND",
            "CALL", "WHERE", "ORDER", "BY", "SKIP", "LIMIT", "DISTINCT", "AS", "AND", "OR", "NOT",
            "TRUE", "FALSE", "NULL", "ON", "DETACH", "START", "END", "CONTAINS",
            "STARTS", "ENDS", "IN", "IS", "ALL", "ANY", "NONE", "SINGLE", "OPTIONAL",
            
            // NGQL 关键字
            "GO", "FROM", "OVER", "REVERSELY", "UPTO", "STEPS", "SAMPLE", "YIELD",
            "LOOKUP", "ON", "WHERE", "FETCH", "PROP", "VERTEX", "VERTICES", "EDGE", "EDGES",
            "FIND", "PATH", "SHORTEST", "ALLSHORTESTPATHS", "NOLOOP",
            "USE", "SPACE", "DESCRIBE", "DESC", "SHOW", "TAG", "TAGS",
            "INDEX", "INDEXES", "REBUILD", "DROP", "IF", "EXISTS",
            "INSERT", "UPDATE", "UPSERT", "VALUES", "VALUE",
            "EXPLAIN", "PROFILE", "FORMAT",
            
            // 集合操作
            "UNION", "INTERSECT", "MINUS",
            
            // 管道操作
            "PIPE",
        ];

        keywords.contains(&word.to_uppercase().as_str())
    }

    // 辅助方法
    fn peek_char(&self) -> Option<char> {
        self.input.chars().nth(self.position)
    }

    fn peek_next_char(&self) -> Option<char> {
        self.input.chars().nth(self.position + 1)
    }

    fn consume_char(&mut self) {
        self.position += 1;
    }

    fn expect_char(&mut self, expected: char) -> Result<(), String> {
        if self.peek_char() == Some(expected) {
            self.consume_char();
            Ok(())
        } else {
            Err(format!("期望字符 '{}'", expected))
        }
    }

    fn is_eof(&self) -> bool {
        self.position >= self.input.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_simple_match() {
        let input = "MATCH (n:Person)".to_string();
        let mut lexer = CypherLexer::new(input);

        let result = lexer.tokenize();
        assert!(result.is_ok());

        let tokens = result.expect("Lexer should return valid tokens");
        assert_eq!(tokens.len(), 7); // MATCH, (, n, :, Person, ), EOF

        assert_eq!(tokens[0].token_type, TokenType::Keyword);
        assert_eq!(tokens[0].value, "MATCH");

        assert_eq!(tokens[1].token_type, TokenType::Punctuation);
        assert_eq!(tokens[1].value, "(");

        assert_eq!(tokens[2].token_type, TokenType::Identifier);
        assert_eq!(tokens[2].value, "n");

        assert_eq!(tokens[3].token_type, TokenType::Punctuation);
        assert_eq!(tokens[3].value, ":");

        assert_eq!(tokens[4].token_type, TokenType::Identifier);
        assert_eq!(tokens[4].value, "Person");

        assert_eq!(tokens[5].token_type, TokenType::Punctuation);
        assert_eq!(tokens[5].value, ")");
    }

    #[test]
    fn test_tokenize_string_literal() {
        let input = "\"Hello, World!\"".to_string();
        let mut lexer = CypherLexer::new(input);

        let result = lexer.tokenize();
        assert!(result.is_ok());

        let tokens = result.expect("Lexer should return valid tokens");
        assert_eq!(tokens.len(), 2); // 字符串和EOF

        assert_eq!(tokens[0].token_type, TokenType::LiteralString);
        assert_eq!(tokens[0].value, "Hello, World!");
    }

    #[test]
    fn test_tokenize_number_literal() {
        let input = "123 45.67".to_string();
        let mut lexer = CypherLexer::new(input);

        let result = lexer.tokenize();
        assert!(result.is_ok());

        let tokens = result.expect("Lexer should return valid tokens");
        assert_eq!(tokens.len(), 3); // 123, 45.67, EOF

        assert_eq!(tokens[0].token_type, TokenType::LiteralNumber);
        assert_eq!(tokens[0].value, "123");

        assert_eq!(tokens[1].token_type, TokenType::LiteralNumber);
        assert_eq!(tokens[1].value, "45.67");
    }

    #[test]
    fn test_tokenize_operators() {
        let input = "= == != < <= > >= + - * /".to_string();
        let mut lexer = CypherLexer::new(input);

        let result = lexer.tokenize();
        assert!(result.is_ok());

        let tokens = result.expect("Lexer should return valid tokens");
        assert_eq!(tokens.len(), 12); // 11个操作符 + EOF

        assert_eq!(tokens[0].token_type, TokenType::Operator);
        assert_eq!(tokens[0].value, "=");

        assert_eq!(tokens[1].token_type, TokenType::Operator);
        assert_eq!(tokens[1].value, "==");

        assert_eq!(tokens[2].token_type, TokenType::Operator);
        assert_eq!(tokens[2].value, "!=");
    }

    #[test]
    fn test_tokenize_relationship_pattern() {
        let input = "(a)-[:FRIENDS_WITH]->(b)".to_string();
        let mut lexer = CypherLexer::new(input);

        let result = lexer.tokenize();
        assert!(result.is_ok());

        let tokens = result.expect("Lexer should return valid tokens");
        assert_eq!(tokens.len(), 13); // (, a, ), -, [, :, FRIENDS_WITH, ], ->, (, b, ), EOF

        assert_eq!(tokens[2].token_type, TokenType::Punctuation);
        assert_eq!(tokens[2].value, ")");

        assert_eq!(tokens[3].token_type, TokenType::Operator);
        assert_eq!(tokens[3].value, "-");

        assert_eq!(tokens[4].token_type, TokenType::Punctuation);
        assert_eq!(tokens[4].value, "[");

        assert_eq!(tokens[5].token_type, TokenType::Punctuation);
        assert_eq!(tokens[5].value, ":");

        assert_eq!(tokens[6].token_type, TokenType::Identifier);
        assert_eq!(tokens[6].value, "FRIENDS_WITH");

        assert_eq!(tokens[7].token_type, TokenType::Punctuation);
        assert_eq!(tokens[7].value, "]");

        assert_eq!(tokens[8].token_type, TokenType::Operator);
        assert_eq!(tokens[8].value, "->");
    }

    #[test]
    fn test_tokenize_complex_query() {
        let input = "MATCH (n:Person {name: \"Alice\", age: 25}) RETURN n.name, n.age".to_string();
        let mut lexer = CypherLexer::new(input);

        let result = lexer.tokenize();
        assert!(result.is_ok());

        let tokens = result.expect("Lexer should return valid tokens");
        assert!(tokens.len() > 10);

        // 检查关键字
        assert_eq!(tokens[0].token_type, TokenType::Keyword);
        assert_eq!(tokens[0].value, "MATCH");

        // 检查字符串字面量
        let string_token = tokens
            .iter()
            .find(|t| t.value == "Alice")
            .expect("Should find Alice token");
        assert_eq!(string_token.token_type, TokenType::LiteralString);

        // 检查数字字面量
        let number_token = tokens
            .iter()
            .find(|t| t.value == "25")
            .expect("Should find number token");
        assert_eq!(number_token.token_type, TokenType::LiteralNumber);
    }

    #[test]
    fn test_tokenize_comment() {
        let input = "MATCH (n) // 这是一个注释\nRETURN n".to_string();
        let mut lexer = CypherLexer::new(input);

        let result = lexer.tokenize();
        assert!(result.is_ok());

        let tokens = result.expect("Lexer should return valid tokens");

        // 查找注释标记
        let comment_token = tokens.iter().find(|t| t.token_type == TokenType::Comment);
        assert!(comment_token.is_some());
        assert_eq!(
            comment_token.expect("Should find comment token").value,
            " 这是一个注释"
        );
    }

    #[test]
    fn test_tokenize_invalid_character() {
        let input = "MATCH (n@Person)".to_string();
        let mut lexer = CypherLexer::new(input);

        let result = lexer.tokenize();
        assert!(result.is_err());
    }
}

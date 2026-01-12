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
            string.push(ch);
            self.consume_char();
        }

        self.expect_char('"')?;
        Ok(string)
    }

    /// 读取数字字面量
    fn read_number(&mut self) -> Result<String, String> {
        let mut number = String::new();

        while let Some(ch) = self.peek_char() {
            if ch.is_digit(10) || ch == '.' {
                number.push(ch);
                self.consume_char();
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
            "MATCH", "RETURN", "CREATE", "DELETE", "SET", "REMOVE", "MERGE", "WITH", "UNWIND",
            "CALL", "WHERE", "ORDER", "BY", "SKIP", "LIMIT", "DISTINCT", "AS", "AND", "OR", "NOT",
            "TRUE", "FALSE", "NULL", "ON", "CREATE", "MATCH", "DETACH", "START", "END", "CONTAINS",
            "STARTS", "ENDS", "IN", "IS", "ALL", "ANY", "NONE", "SINGLE",
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

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Literals
    Number,
    Identifier,
    
    // Arithmetic operators
    Plus,
    Minus,
    Multiply,
    Divide,
    
    // Boolean operators
    Equal,          // ==
    NotEqual,       // !=
    Less,           // <
    Greater,        // >
    LessEqual,      // <=
    GreaterEqual,   // >=
    
    // Assignment
    Assign,         // =
    
    // Delimiters
    LParen,
    RParen,
    LBrace,
    RBrace,
    Semicolon,
    
    // Keywords
    Print,
    If,
    Else,
    
    // Special
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, line: usize, column: usize) -> Self {
        Token {
            token_type,
            lexeme,
            line,
            column,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:?} '{}' [{}:{}]",
            self.token_type, self.lexeme, self.line, self.column
        )
    }
}

pub struct Lexer {
    source: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(source: String) -> Self {
        Lexer {
            source: source.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
        }
    }

    fn current(&self) -> char {
        if self.position >= self.source.len() {
            '\0'
        } else {
            self.source[self.position]
        }
    }

    fn peek(&self, offset: usize) -> char {
        let pos = self.position + offset;
        if pos >= self.source.len() {
            '\0'
        } else {
            self.source[pos]
        }
    }

    fn advance(&mut self) {
        if self.position < self.source.len() {
            if self.source[self.position] == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            self.position += 1;
        }
    }

    fn skip_whitespace(&mut self) {
        while self.current().is_whitespace() {
            self.advance();
        }
    }

    fn number(&mut self) -> Token {
        let start_col = self.column;
        let mut num = String::new();

        while self.current().is_numeric() || self.current() == '.' {
            num.push(self.current());
            self.advance();
        }

        Token::new(TokenType::Number, num, self.line, start_col)
    }

    fn identifier(&mut self) -> Token {
        let start_col = self.column;
        let mut id = String::new();

        while self.current().is_alphanumeric() || self.current() == '_' {
            id.push(self.current());
            self.advance();
        }

        let token_type = match id.as_str() {
            "print" => TokenType::Print,
            "if" => TokenType::If,
            "else" => TokenType::Else,
            _ => TokenType::Identifier,
        };

        Token::new(token_type, id, self.line, start_col)
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();

        while self.position < self.source.len() {
            self.skip_whitespace();
            if self.position >= self.source.len() {
                break;
            }

            let ch = self.current();
            let col = self.column;

            if ch.is_numeric() {
                tokens.push(self.number());
            } else if ch.is_alphabetic() || ch == '_' {
                tokens.push(self.identifier());
            } else {
                match ch {
                    '+' => {
                        tokens.push(Token::new(TokenType::Plus, "+".to_string(), self.line, col));
                        self.advance();
                    }
                    '-' => {
                        tokens.push(Token::new(TokenType::Minus, "-".to_string(), self.line, col));
                        self.advance();
                    }
                    '*' => {
                        tokens.push(Token::new(TokenType::Multiply, "*".to_string(), self.line, col));
                        self.advance();
                    }
                    '/' => {
                        tokens.push(Token::new(TokenType::Divide, "/".to_string(), self.line, col));
                        self.advance();
                    }
                    '=' => {
                        if self.peek(1) == '=' {
                            tokens.push(Token::new(TokenType::Equal, "==".to_string(), self.line, col));
                            self.advance();
                            self.advance();
                        } else {
                            tokens.push(Token::new(TokenType::Assign, "=".to_string(), self.line, col));
                            self.advance();
                        }
                    }
                    '!' => {
                        if self.peek(1) == '=' {
                            tokens.push(Token::new(TokenType::NotEqual, "!=".to_string(), self.line, col));
                            self.advance();
                            self.advance();
                        } else {
                            return Err(format!("Error léxico: carácter '!' inesperado en línea {}", self.line));
                        }
                    }
                    '<' => {
                        if self.peek(1) == '=' {
                            tokens.push(Token::new(TokenType::LessEqual, "<=".to_string(), self.line, col));
                            self.advance();
                            self.advance();
                        } else {
                            tokens.push(Token::new(TokenType::Less, "<".to_string(), self.line, col));
                            self.advance();
                        }
                    }
                    '>' => {
                        if self.peek(1) == '=' {
                            tokens.push(Token::new(TokenType::GreaterEqual, ">=".to_string(), self.line, col));
                            self.advance();
                            self.advance();
                        } else {
                            tokens.push(Token::new(TokenType::Greater, ">".to_string(), self.line, col));
                            self.advance();
                        }
                    }
                    '(' => {
                        tokens.push(Token::new(TokenType::LParen, "(".to_string(), self.line, col));
                        self.advance();
                    }
                    ')' => {
                        tokens.push(Token::new(TokenType::RParen, ")".to_string(), self.line, col));
                        self.advance();
                    }
                    '{' => {
                        tokens.push(Token::new(TokenType::LBrace, "{".to_string(), self.line, col));
                        self.advance();
                    }
                    '}' => {
                        tokens.push(Token::new(TokenType::RBrace, "}".to_string(), self.line, col));
                        self.advance();
                    }
                    ';' => {
                        tokens.push(Token::new(TokenType::Semicolon, ";".to_string(), self.line, col));
                        self.advance();
                    }
                    _ => {
                        return Err(format!(
                            "Error léxico: carácter '{}' en línea {}",
                            ch, self.line
                        ));
                    }
                }
            }
        }

        tokens.push(Token::new(TokenType::Eof, "".to_string(), self.line, self.column));
        Ok(tokens)
    }
}

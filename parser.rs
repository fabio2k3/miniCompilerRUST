use crate::lexer::{Token, TokenType};
use std::fmt;

// AST Node definitions
#[derive(Debug, Clone)]
pub enum ASTNode {
    Number(f64),
    Identifier(String),
    BinaryOp {
        op: String,
        left: Box<ASTNode>,
        right: Box<ASTNode>,
    },
    Assignment {
        variable: String,
        expression: Box<ASTNode>,
    },
    Print {
        expression: Box<ASTNode>,
    },
    If {
        condition: Box<ASTNode>,
        then_block: Vec<ASTNode>,
        else_block: Option<Vec<ASTNode>>,
    },
}

impl ASTNode {
    pub fn print_tree(&self, indent: usize) {
        let spaces = " ".repeat(indent);
        match self {
            ASTNode::Number(val) => println!("{}Number: {}", spaces, val),
            ASTNode::Identifier(name) => println!("{}Identifier: {}", spaces, name),
            ASTNode::BinaryOp { op, left, right } => {
                println!("{}BinaryOp: {}", spaces, op);
                left.print_tree(indent + 2);
                right.print_tree(indent + 2);
            }
            ASTNode::Assignment { variable, expression } => {
                println!("{}Assignment: {}", spaces, variable);
                expression.print_tree(indent + 2);
            }
            ASTNode::Print { expression } => {
                println!("{}Print:", spaces);
                expression.print_tree(indent + 2);
            }
            ASTNode::If { condition, then_block, else_block } => {
                println!("{}If:", spaces);
                println!("{}Condition:", spaces);
                condition.print_tree(indent + 2);
                println!("{}Then:", spaces);
                for stmt in then_block {
                    stmt.print_tree(indent + 2);
                }
                if let Some(else_stmts) = else_block {
                    println!("{}Else:", spaces);
                    for stmt in else_stmts {
                        stmt.print_tree(indent + 2);
                    }
                }
            }
        }
    }

    pub fn get_node_type(&self) -> &str {
        match self {
            ASTNode::Number(_) => "Number",
            ASTNode::Identifier(_) => "Identifier",
            ASTNode::BinaryOp { .. } => "BinaryOp",
            ASTNode::Assignment { .. } => "Assignment",
            ASTNode::Print { .. } => "Print",
            ASTNode::If { .. } => "If",
        }
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, position: 0 }
    }

    fn current(&self) -> &Token {
        if self.position >= self.tokens.len() {
            &self.tokens[self.tokens.len() - 1]
        } else {
            &self.tokens[self.position]
        }
    }

    fn peek(&self, offset: usize) -> &Token {
        let pos = self.position + offset;
        if pos >= self.tokens.len() {
            &self.tokens[self.tokens.len() - 1]
        } else {
            &self.tokens[pos]
        }
    }

    fn advance(&mut self) {
        if self.position < self.tokens.len() - 1 {
            self.position += 1;
        }
    }

    fn match_token(&mut self, token_type: TokenType) -> bool {
        if self.current().token_type == token_type {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, token_type: TokenType, message: &str) -> Result<(), String> {
        if self.current().token_type != token_type {
            return Err(format!(
                "Error sintaxis: {} en línea {}",
                message,
                self.current().line
            ));
        }
        self.advance();
        Ok(())
    }

    // factor ::= NUMBER | IDENTIFIER | '(' expression ')'
    fn factor(&mut self) -> Result<ASTNode, String> {
        let tok = self.current().clone();

        match &tok.token_type {
            TokenType::Number => {
                self.advance();
                Ok(ASTNode::Number(
                    tok.lexeme.parse::<f64>().map_err(|_| {
                        format!("Error: no se pudo parsear número '{}'", tok.lexeme)
                    })?,
                ))
            }
            TokenType::Identifier => {
                self.advance();
                Ok(ASTNode::Identifier(tok.lexeme.clone()))
            }
            TokenType::LParen => {
                self.advance();
                let expr = self.expression()?;
                self.expect(TokenType::RParen, "esperaba ')'")?;
                Ok(expr)
            }
            _ => Err(format!("Error sintaxis en línea {}", tok.line)),
        }
    }

    // term ::= factor (('*' | '/') factor)*
    fn term(&mut self) -> Result<ASTNode, String> {
        let mut left = self.factor()?;

        while matches!(
            self.current().token_type,
            TokenType::Multiply | TokenType::Divide
        ) {
            let op = self.current().lexeme.clone();
            self.advance();
            let right = self.factor()?;
            left = ASTNode::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    // arithmetic ::= term (('+' | '-') term)*
    fn arithmetic(&mut self) -> Result<ASTNode, String> {
        let mut left = self.term()?;

        while matches!(self.current().token_type, TokenType::Plus | TokenType::Minus) {
            let op = self.current().lexeme.clone();
            self.advance();
            let right = self.term()?;
            left = ASTNode::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    // comparison ::= arithmetic (('==' | '!=' | '<' | '>' | '<=' | '>=') arithmetic)?
    fn comparison(&mut self) -> Result<ASTNode, String> {
        let mut left = self.arithmetic()?;

        if matches!(
            self.current().token_type,
            TokenType::Equal
                | TokenType::NotEqual
                | TokenType::Less
                | TokenType::Greater
                | TokenType::LessEqual
                | TokenType::GreaterEqual
        ) {
            let op = self.current().lexeme.clone();
            self.advance();
            let right = self.arithmetic()?;
            left = ASTNode::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    // expression ::= comparison
    fn expression(&mut self) -> Result<ASTNode, String> {
        self.comparison()
    }

    // block ::= '{' statement* '}'
    fn block(&mut self) -> Result<Vec<ASTNode>, String> {
        self.expect(TokenType::LBrace, "esperaba '{'")?;
        let mut statements = Vec::new();

        while self.current().token_type != TokenType::RBrace
            && self.current().token_type != TokenType::Eof
        {
            statements.push(self.statement()?);
        }

        self.expect(TokenType::RBrace, "esperaba '}'")?;
        Ok(statements)
    }

    // statement ::= assignment | print | if_statement
    fn statement(&mut self) -> Result<ASTNode, String> {
        match &self.current().token_type {
            TokenType::Print => {
                self.advance();
                self.expect(TokenType::LParen, "esperaba '(' después de print")?;
                let expr = self.expression()?;
                self.expect(TokenType::RParen, "esperaba ')'")?;
                self.expect(TokenType::Semicolon, "esperaba ';'")?;
                Ok(ASTNode::Print {
                    expression: Box::new(expr),
                })
            }
            TokenType::If => {
                self.advance();
                self.expect(TokenType::LParen, "esperaba '(' después de if")?;
                let condition = self.expression()?;
                self.expect(TokenType::RParen, "esperaba ')'")?;

                let then_block = self.block()?;

                let else_block = if self.match_token(TokenType::Else) {
                    Some(self.block()?)
                } else {
                    None
                };

                Ok(ASTNode::If {
                    condition: Box::new(condition),
                    then_block,
                    else_block,
                })
            }
            TokenType::Identifier => {
                if self.peek(1).token_type == TokenType::Assign {
                    let var_name = self.current().lexeme.clone();
                    self.advance();
                    self.advance();
                    let expr = self.expression()?;
                    self.expect(TokenType::Semicolon, "esperaba ';'")?;
                    Ok(ASTNode::Assignment {
                        variable: var_name,
                        expression: Box::new(expr),
                    })
                } else {
                    Err(format!(
                        "Error sintaxis en línea {}",
                        self.current().line
                    ))
                }
            }
            _ => Err(format!(
                "Error sintaxis en línea {}",
                self.current().line
            )),
        }
    }

    pub fn parse(&mut self) -> Result<Vec<ASTNode>, String> {
        let mut statements = Vec::new();

        while self.current().token_type != TokenType::Eof {
            statements.push(self.statement()?);
        }

        Ok(statements)
    }
}

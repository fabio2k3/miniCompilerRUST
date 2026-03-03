use crate::parser::ASTNode;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SymbolTable {
    symbols: HashMap<String, String>,
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            symbols: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, symbol_type: String) {
        self.symbols.insert(name, symbol_type);
    }

    pub fn is_defined(&self, name: &str) -> bool {
        self.symbols.contains_key(name)
    }

    pub fn get_type(&self, name: &str) -> Result<String, String> {
        self.symbols
            .get(name)
            .cloned()
            .ok_or_else(|| format!("Error semántico: variable '{}' no definida", name))
    }

    pub fn print(&self) {
        for (name, symbol_type) in &self.symbols {
            println!("  {} : {}", name, symbol_type);
        }
    }

    pub fn get_symbols(&self) -> &HashMap<String, String> {
        &self.symbols
    }
}

pub struct SemanticAnalyzer {
    symbol_table: SymbolTable,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        SemanticAnalyzer {
            symbol_table: SymbolTable::new(),
        }
    }

    fn analyze_expression(&self, node: &ASTNode) -> Result<String, String> {
        match node {
            ASTNode::Number(_) => Ok("number".to_string()),
            ASTNode::Identifier(name) => {
                if !self.symbol_table.is_defined(name) {
                    return Err(format!(
                        "Error semántico: variable '{}' no definida",
                        name
                    ));
                }
                self.symbol_table.get_type(name)
            }
            ASTNode::BinaryOp { op, left, right } => {
                let left_type = self.analyze_expression(left)?;
                let right_type = self.analyze_expression(right)?;

                // Arithmetic operators require numbers
                if matches!(op.as_str(), "+" | "-" | "*" | "/") {
                    if left_type != "number" || right_type != "number" {
                        return Err(
                            "Error semántico: operación aritmética requiere números".to_string()
                        );
                    }
                    return Ok("number".to_string());
                }

                // Comparison operators require numbers but return boolean
                if matches!(op.as_str(), "==" | "!=" | "<" | ">" | "<=" | ">=") {
                    if left_type != "number" || right_type != "number" {
                        return Err(
                            "Error semántico: comparación requiere números".to_string()
                        );
                    }
                    return Ok("boolean".to_string());
                }

                Err(format!("Error semántico: operador desconocido '{}'", op))
            }
            _ => Err("Error semántico: nodo desconocido en expresión".to_string()),
        }
    }

    fn analyze_statement(&mut self, node: &ASTNode) -> Result<(), String> {
        match node {
            ASTNode::Assignment { variable, expression } => {
                let expr_type = self.analyze_expression(expression)?;
                self.symbol_table.define(variable.clone(), expr_type);
                Ok(())
            }
            ASTNode::Print { expression } => {
                self.analyze_expression(expression)?;
                Ok(())
            }
            ASTNode::If {
                condition,
                then_block,
                else_block,
            } => {
                // Check that condition is a boolean expression
                let cond_type = self.analyze_expression(condition)?;
                if cond_type != "boolean" {
                    return Err(
                        "Error semántico: la condición del if debe ser booleana".to_string()
                    );
                }

                // Analyze then block
                for stmt in then_block {
                    self.analyze_statement(stmt)?;
                }

                // Analyze else block if present
                if let Some(else_stmts) = else_block {
                    for stmt in else_stmts {
                        self.analyze_statement(stmt)?;
                    }
                }

                Ok(())
            }
            _ => Err("Error semántico: tipo de statement desconocido".to_string()),
        }
    }

    pub fn analyze(&mut self, statements: &[ASTNode]) -> Result<(), String> {
        for stmt in statements {
            self.analyze_statement(stmt)?;
        }
        Ok(())
    }

    pub fn print_symbol_table(&self) {
        self.symbol_table.print();
    }

    pub fn get_symbol_table(&self) -> &SymbolTable {
        &self.symbol_table
    }
}

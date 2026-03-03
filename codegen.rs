use crate::parser::ASTNode;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Instruction {
    pub op: String,
    pub arg1: String,
    pub arg2: String,
    pub result: String,
}

impl Instruction {
    pub fn to_string(&self) -> String {
        if self.op == "=" {
            format!("{} = {}", self.result, self.arg1)
        } else if self.op == "print" {
            format!("print {}", self.arg1)
        } else if self.op == "if_false" {
            format!("if_false {} goto {}", self.arg1, self.result)
        } else if self.op == "goto" {
            format!("goto {}", self.result)
        } else if self.op == "label" {
            format!("{}:", self.result)
        } else {
            format!("{} = {} {} {}", self.result, self.arg1, self.op, self.arg2)
        }
    }
}

pub struct CodeGenerator {
    instructions: Vec<Instruction>,
    temp_counter: usize,
    label_counter: usize,
    memory: HashMap<String, f64>,
}

impl CodeGenerator {
    pub fn new() -> Self {
        CodeGenerator {
            instructions: Vec::new(),
            temp_counter: 0,
            label_counter: 0,
            memory: HashMap::new(),
        }
    }

    fn new_temp(&mut self) -> String {
        let temp = format!("t{}", self.temp_counter);
        self.temp_counter += 1;
        temp
    }

    fn new_label(&mut self) -> String {
        let label = format!("L{}", self.label_counter);
        self.label_counter += 1;
        label
    }

    fn generate_expression(&mut self, node: &ASTNode) -> String {
        match node {
            ASTNode::Number(val) => val.to_string(),
            ASTNode::Identifier(name) => name.clone(),
            ASTNode::BinaryOp { op, left, right } => {
                let left_result = self.generate_expression(left);
                let right_result = self.generate_expression(right);
                let temp = self.new_temp();

                self.instructions.push(Instruction {
                    op: op.clone(),
                    arg1: left_result,
                    arg2: right_result,
                    result: temp.clone(),
                });

                temp
            }
            _ => String::new(),
        }
    }

    fn generate_statement(&mut self, node: &ASTNode) {
        match node {
            ASTNode::Assignment { variable, expression } => {
                let expr_result = self.generate_expression(expression);
                self.instructions.push(Instruction {
                    op: "=".to_string(),
                    arg1: expr_result,
                    arg2: String::new(),
                    result: variable.clone(),
                });
            }
            ASTNode::Print { expression } => {
                let expr_result = self.generate_expression(expression);
                self.instructions.push(Instruction {
                    op: "print".to_string(),
                    arg1: expr_result,
                    arg2: String::new(),
                    result: String::new(),
                });
            }
            ASTNode::If {
                condition,
                then_block,
                else_block,
            } => {
                let cond_result = self.generate_expression(condition);
                let else_label = self.new_label();
                let end_label = self.new_label();

                // If condition is false, jump to else or end
                self.instructions.push(Instruction {
                    op: "if_false".to_string(),
                    arg1: cond_result,
                    arg2: String::new(),
                    result: else_label.clone(),
                });

                // Generate then block
                for stmt in then_block {
                    self.generate_statement(stmt);
                }

                // If there's an else block, jump to end after then block
                if else_block.is_some() {
                    self.instructions.push(Instruction {
                        op: "goto".to_string(),
                        arg1: String::new(),
                        arg2: String::new(),
                        result: end_label.clone(),
                    });
                }

                // Else label
                self.instructions.push(Instruction {
                    op: "label".to_string(),
                    arg1: String::new(),
                    arg2: String::new(),
                    result: else_label,
                });

                // Generate else block if present
                if let Some(else_stmts) = else_block {
                    for stmt in else_stmts {
                        self.generate_statement(stmt);
                    }
                }

                // End label
                self.instructions.push(Instruction {
                    op: "label".to_string(),
                    arg1: String::new(),
                    arg2: String::new(),
                    result: end_label,
                });
            }
            _ => {}
        }
    }

    pub fn generate(&mut self, statements: &[ASTNode]) -> String {
        self.instructions.clear();
        self.temp_counter = 0;
        self.label_counter = 0;

        for stmt in statements {
            self.generate_statement(stmt);
        }

        let mut output = String::new();
        for (i, inst) in self.instructions.iter().enumerate() {
            output.push_str(&format!("{}: {}\n", i + 1, inst.to_string()));
        }
        output
    }

    fn get_value(&self, operand: &str) -> Result<f64, String> {
        if let Some(&value) = self.memory.get(operand) {
            Ok(value)
        } else {
            operand
                .parse::<f64>()
                .map_err(|_| format!("Error: no se puede obtener valor de '{}'", operand))
        }
    }

    fn evaluate_comparison(&self, op: &str, left: f64, right: f64) -> f64 {
        let result = match op {
            "==" => left == right,
            "!=" => left != right,
            "<" => left < right,
            ">" => left > right,
            "<=" => left <= right,
            ">=" => left >= right,
            _ => false,
        };
        if result { 1.0 } else { 0.0 }
    }

    pub fn execute(&mut self) -> Result<(), String> {
        self.memory.clear();
        let mut pc = 0; // Program counter
        let mut labels: HashMap<String, usize> = HashMap::new();

        // First pass: collect labels
        for (i, inst) in self.instructions.iter().enumerate() {
            if inst.op == "label" {
                labels.insert(inst.result.clone(), i);
            }
        }

        // Second pass: execute instructions
        while pc < self.instructions.len() {
            let inst = &self.instructions[pc].clone();

            match inst.op.as_str() {
                "=" => {
                    let value = self.get_value(&inst.arg1)?;
                    self.memory.insert(inst.result.clone(), value);
                    pc += 1;
                }
                "print" => {
                    let value = self.get_value(&inst.arg1)?;
                    println!("{}", value);
                    pc += 1;
                }
                "+" | "-" | "*" | "/" => {
                    let left = self.get_value(&inst.arg1)?;
                    let right = self.get_value(&inst.arg2)?;

                    let result = match inst.op.as_str() {
                        "+" => left + right,
                        "-" => left - right,
                        "*" => left * right,
                        "/" => {
                            if right == 0.0 {
                                return Err("Error: División por cero".to_string());
                            }
                            left / right
                        }
                        _ => 0.0,
                    };

                    self.memory.insert(inst.result.clone(), result);
                    pc += 1;
                }
                "==" | "!=" | "<" | ">" | "<=" | ">=" => {
                    let left = self.get_value(&inst.arg1)?;
                    let right = self.get_value(&inst.arg2)?;
                    let result = self.evaluate_comparison(&inst.op, left, right);
                    self.memory.insert(inst.result.clone(), result);
                    pc += 1;
                }
                "if_false" => {
                    let condition = self.get_value(&inst.arg1)?;
                    if condition == 0.0 {
                        // Jump to label
                        if let Some(&target) = labels.get(&inst.result) {
                            pc = target;
                        } else {
                            return Err(format!("Error: etiqueta '{}' no encontrada", inst.result));
                        }
                    } else {
                        pc += 1;
                    }
                }
                "goto" => {
                    if let Some(&target) = labels.get(&inst.result) {
                        pc = target;
                    } else {
                        return Err(format!("Error: etiqueta '{}' no encontrada", inst.result));
                    }
                }
                "label" => {
                    pc += 1;
                }
                _ => {
                    return Err(format!("Error: operación desconocida '{}'", inst.op));
                }
            }
        }

        Ok(())
    }
}

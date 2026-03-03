mod lexer;
mod parser;
mod semantic;
mod codegen;

use lexer::Lexer;
use parser::Parser;
use semantic::SemanticAnalyzer;
use codegen::CodeGenerator;
use std::io::{self, Write};

struct REPL {
    semantic: SemanticAnalyzer,
    codegen: CodeGenerator,
    all_statements: Vec<parser::ASTNode>,
}

impl REPL {
    fn new() -> Self {
        REPL {
            semantic: SemanticAnalyzer::new(),
            codegen: CodeGenerator::new(),
            all_statements: Vec::new(),
        }
    }

    fn print_banner(&self) {
        println!("\n╔════════════════════════════════════════╗");
        println!("║   MINI COMPILADOR INTERACTIVO (REPL)   ║");
        println!("║         Rust Edition v1.0              ║");
        println!("╚════════════════════════════════════════╝\n");
        println!("Comandos especiales:");
        println!("  :help    - Ayuda");
        println!("  :vars    - Ver variables");
        println!("  :clear   - Limpiar variables");
        println!("  :exit    - Salir\n");
        println!("Ejemplos:");
        println!("  x = 5 + 3;");
        println!("  print(x);");
        println!("  if (x > 5) {{ print(1); }} else {{ print(0); }}\n");
    }

    fn print_help(&self) {
        println!("\n=== SINTAXIS ===\n");
        println!("Asignación:  variable = expresion;");
        println!("Print:       print(expresion);");
        println!("If:          if (condicion) {{ statements }} else {{ statements }}");
        println!("\nOperadores aritméticos: + - * /");
        println!("Operadores de comparación: == != < > <= >=");
        println!("\nEjemplos:");
        println!("  x = 10;");
        println!("  y = x * 2 + 5;");
        println!("  print(y);");
        println!("  if (y > 20) {{");
        println!("    print(1);");
        println!("  }} else {{");
        println!("    print(0);");
        println!("  }}\n");
    }

    fn show_variables(&self) {
        println!("\n=== VARIABLES ===");
        let symbols = self.semantic.get_symbol_table().get_symbols();
        if symbols.is_empty() {
            println!("  (ninguna)");
        } else {
            for (name, symbol_type) in symbols {
                println!("  {} : {}", name, symbol_type);
            }
        }
        println!();
    }

    fn clear_variables(&mut self) {
        self.semantic = SemanticAnalyzer::new();
        self.codegen = CodeGenerator::new();
        self.all_statements.clear();
        println!("\nVariables limpiadas\n");
    }

    fn process_command(&mut self, input: &str) -> bool {
        let cmd = input.trim();

        if cmd.is_empty() {
            return true;
        }

        match cmd {
            ":help" | ":h" => {
                self.print_help();
                true
            }
            ":vars" | ":v" => {
                self.show_variables();
                true
            }
            ":clear" | ":c" => {
                self.clear_variables();
                true
            }
            ":exit" | ":quit" | ":q" => false,
            _ => true,
        }
    }

    fn evaluate_line(&mut self, line: &str) {
        // Handle commands
        if line.trim().starts_with(':') {
            if !self.process_command(line) {
                println!("\n¡Adiós!\n");
                std::process::exit(0);
            }
            return;
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            return;
        }

        // Check if the line ends with semicolon or closing brace
        if !trimmed.ends_with(';') && !trimmed.ends_with('}') {
            eprintln!("Error: falta ';' al final o sintaxis incompleta");
            return;
        }

        match self.try_evaluate(line) {
            Ok(_) => {}
            Err(e) => eprintln!("{}", e),
        }
    }

    fn try_evaluate(&mut self, line: &str) -> Result<(), String> {
        let mut lexer = Lexer::new(line.to_string());
        let tokens = lexer.tokenize()?;

        let mut parser = Parser::new(tokens);
        let statements = parser.parse()?;

        self.semantic.analyze(&statements)?;

        // Add new statements to accumulated statements
        for stmt in statements {
            self.all_statements.push(stmt);
        }

        // Generate code for ALL statements (needed for cross-references)
        self.codegen.generate(&self.all_statements);
        
        // Execute the generated code
        self.codegen.execute()?;

        Ok(())
    }

    pub fn run(&mut self) {
        self.print_banner();

        loop {
            print!(">>> ");
            io::stdout().flush().unwrap();

            let mut line = String::new();
            match io::stdin().read_line(&mut line) {
                Ok(0) => {
                    println!("\n\n¡Adiós!\n");
                    break;
                }
                Ok(_) => {
                    self.evaluate_line(&line);
                }
                Err(e) => {
                    eprintln!("Error leyendo entrada: {}", e);
                    break;
                }
            }
        }
    }
}

fn main() {
    let mut repl = REPL::new();
    repl.run();
}
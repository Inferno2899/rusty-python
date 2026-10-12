mod frontend;
mod runtime;
mod vm;

use frontend::Lexer;
use frontend::Parser;
use vm::{Compiler, Machine};

fn main() -> Result<(), String> {
    let source = std::fs::read_to_string("examples/example.py").map_err(|e| e.to_string())?;

    let tokens = Lexer::new().tokenize(source)?;

    // println!("{tokens:?}");

    let mut parser = Parser::new(&tokens);
    let program = parser.parse_program()?;

    // println!("{program:?}");

    let compiler = Compiler::new();
    let compiled_program = compiler.compile(&program)?;

    println!("{compiled_program:?}");

    let mut machine = Machine::new();

    machine.execute(compiled_program)?;

    Ok(())
}

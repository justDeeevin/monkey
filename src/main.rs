mod ast;
mod eval;
mod lexer;
mod object;
mod parser;
mod token;

use ast::Program;
use eval::eval;
use object::Environment;
use rustyline::error::ReadlineError;

fn main() {
    println!("Monkey REPL");
    println!("Ctrl-D to exit");
    let mut rl = rustyline::DefaultEditor::new().unwrap();
    let mut env = Environment::default();
    loop {
        match rl.readline(">> ") {
            Ok(line) => {
                let _ = rl.add_history_entry(&line);
                let program = match line.parse::<Program>() {
                    Ok(program) => program,
                    Err(e) => {
                        eprintln!("{e}");
                        continue;
                    }
                };
                let eval = match eval(&program, &mut env) {
                    Ok(eval) => eval,
                    Err(e) => {
                        eprintln!("{e}");
                        continue;
                    }
                };
                println!("{}", eval)
            }
            Err(ReadlineError::Interrupted) => {}
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(e) => {
                eprintln!("Error: {e:?}");
                break;
            }
        }
    }
}

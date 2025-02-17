mod ast;
mod lexer;
mod parser;
mod token;

use ast::Program;
use rustyline::error::ReadlineError;

fn main() {
    let mut rl = rustyline::DefaultEditor::new().unwrap();
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
                println!("{program}");
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
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

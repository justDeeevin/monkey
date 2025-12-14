use crate::eval::Environment;
use rustyline::error::ReadlineError;

mod ast;
mod eval;
mod lexer;
mod object;
mod parser;
mod token;

fn main() {
    if let Some(file) = std::env::args().nth(1) {
        let contents = std::fs::read_to_string(file).unwrap();
        let program = match parser::parse(&contents) {
            Ok(program) => program,
            Err(errors) => {
                for error in errors {
                    error.report(&contents);
                }
                return;
            }
        };
        if let Err(errors) = Environment::default().eval_program(program) {
            for error in errors {
                error.report(&contents);
            }
            return;
        };
        return;
    }

    println!("Monkey REPL");
    println!("Ctrl-D to exit");
    let mut rl = rustyline::DefaultEditor::new().unwrap();

    let mut env = Environment::default();

    loop {
        match rl.readline(">> ") {
            Ok(line) => {
                let _ = rl.add_history_entry(&line);
                let line = line.leak().trim();
                let program = match parser::parse(line) {
                    Ok(program) => program,
                    Err(errors) => {
                        for error in errors {
                            error.report(line);
                        }
                        continue;
                    }
                };
                let eval = match env.eval_program(program) {
                    Ok(eval) => eval,
                    Err(errors) => {
                        for error in errors {
                            error.report(line);
                        }
                        continue;
                    }
                };
                println!("{eval}");
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

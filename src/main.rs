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
        println!("{program}");
        return;
    }

    println!("Monkey REPL");
    println!("Ctrl-D to exit");
    let mut rl = rustyline::DefaultEditor::new().unwrap();

    loop {
        match rl.readline(">> ") {
            Ok(line) => {
                let _ = rl.add_history_entry(&line);
                let (debug, line) = match line.trim().strip_prefix('?') {
                    Some(line) => (true, line),
                    None => (false, line.as_str()),
                };
                let program = match parser::parse(line) {
                    Ok(program) => program,
                    Err(errors) => {
                        for error in errors {
                            error.report(line);
                        }
                        continue;
                    }
                };
                if debug {
                    println!("{program:#?}");
                } else {
                    println!("{program}");
                }
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

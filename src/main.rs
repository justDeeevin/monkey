use crate::lexer::Lexer;
use rustyline::error::ReadlineError;

mod ast;
mod lexer;
mod parser;
mod token;

fn main() {
    println!("Monkey REPL");
    println!("Ctrl-D to exit");
    let mut rl = rustyline::DefaultEditor::new().unwrap();
    loop {
        match rl.readline(">> ") {
            Ok(line) => {
                let _ = rl.add_history_entry(&line);
                for token in Lexer::new(&line) {
                    println!("{token:?}");
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

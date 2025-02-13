mod lexer;
mod token;

use lexer::Lexer;
use rustyline::error::ReadlineError;

fn main() {
    let mut rl = rustyline::DefaultEditor::new().unwrap();
    loop {
        match rl.readline(">> ") {
            Ok(line) => {
                let _ = rl.add_history_entry(&line);
                dbg!(Lexer::new(line).collect::<Vec<_>>());
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

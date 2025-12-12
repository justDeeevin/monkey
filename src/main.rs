use rustyline::error::ReadlineError;

mod lexer;
mod token;

fn main() {
    println!("Monkey REPL");
    println!("Ctrl-D to exit");
    let mut rl = rustyline::DefaultEditor::new().unwrap();
    loop {
        match rl.readline(">> ") {
            Ok(line) => {
                let _ = rl.add_history_entry(&line);
                println!("{line}")
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

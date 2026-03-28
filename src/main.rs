mod ast;
mod cli;
mod parse;

use parse::parse_program;
use rustyline::error::ReadlineError;

fn main() {
    let args = cli::parse();
    if let Some(file) = args.file {
        let contents = std::fs::read_to_string(file).unwrap();
        let program = parse_program(&contents).unwrap();
        dbg!(&program);
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
                let program = parse_program(&line).unwrap();
                dbg!(&program);
                println!("{program}");
            }
            Err(ReadlineError::Eof) => {
                println!("Ctrl-D");
                break;
            }
            Err(ReadlineError::Interrupted) => {}
            Err(err) => {
                println!("Error: {err}");
                break;
            }
        }
    }
}

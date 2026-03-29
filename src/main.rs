mod ast;
mod cli;
mod eval;
mod intrinsic;
mod parse;
mod value;

use eval::Environment;
use parse::parse_program;
use rustyline::error::ReadlineError;
use value::Value;

fn main() {
    let args = cli::parse();
    if let Some(file) = args.file {
        let contents = std::fs::read_to_string(file).unwrap();
        let program = parse_program(&contents).unwrap();
        eprintln!("{program}");
        match Environment::default().eval(program).unwrap() {
            Value::Null => {}
            value => println!("{value}"),
        }
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
                let program = parse_program(line.leak().trim()).unwrap();
                let value = env.eval(program).unwrap();
                println!("{value}");
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

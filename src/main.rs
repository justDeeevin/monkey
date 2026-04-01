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
        match Environment::default().eval(program) {
            Err(e) => e.report(&contents),
            Ok(Value::Null) => {}
            Ok(value) => println!("{value}"),
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
                let line = line.leak().trim();
                let program = parse_program(line).unwrap();
                let value = match env.eval(program) {
                    Ok(value) => value,
                    Err(e) => {
                        e.report(line);
                        continue;
                    }
                };
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

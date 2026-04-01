mod ast;
mod cli;
mod eval;
mod intrinsic;
mod parse;
mod value;

use eval::{Environment, Error};
use parse::{parse_program, report_errors};
use rustyline::error::ReadlineError;
use value::Value;

fn main() {
    let args = cli::parse();
    if let Some(file) = args.file {
        let contents = std::fs::read_to_string(file).unwrap();
        let parse_result = parse_program(&contents);
        if parse_result.has_errors() {
            report_errors(parse_result.into_errors(), &contents);
            return;
        }
        let Some(program) = parse_result.into_output() else {
            return;
        };
        eprintln!("{program}");
        match Environment::default().eval(program) {
            Err(e) => Error::report(e, &contents),
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
                let parse_result = parse_program(line);
                if parse_result.has_errors() {
                    report_errors(parse_result.into_errors(), line);
                    continue;
                }
                let Some(program) = parse_result.into_output() else {
                    continue;
                };
                let value = match env.eval(program) {
                    Ok(value) => value,
                    Err(e) => {
                        Error::report(e, line);
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

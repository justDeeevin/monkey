use std::rc::Rc;

use cli::Backend;
use compiler::Compiler;
use eval::Environment;
use rustyline::error::ReadlineError;
use vm::VM;

use crate::object::{CompiledFunction, Object};

mod ast;
mod cli;
mod code;
mod compiler;
mod eval;
mod lexer;
mod object;
mod parser;
mod token;
mod vm;

fn main() {
    let args = cli::parse();
    if let Some(file) = args.file {
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
        match args.backend {
            Backend::Otf => {
                if let Err(error) = Environment::default().eval_program(program) {
                    error.report(&contents);
                };
            }
            Backend::Vm => match VM::new(Compiler::default().compile(program)).run() {
                Err(error) => {
                    error.report(&contents);
                }
                Ok(out) if out != Object::Null => println!("{out}"),
                Ok(_) => {}
            },
        }
        return;
    }

    println!("Monkey REPL");
    println!("Ctrl-D to exit");
    let mut rl = rustyline::DefaultEditor::new().unwrap();

    let mut env = Environment::default();
    let mut vm = VM::default();
    let mut compiler = Compiler::default();

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
                let eval = match args.backend {
                    Backend::Otf => match env.eval_program(program) {
                        Ok(eval) => eval,
                        Err(error) => {
                            error.report(line);
                            continue;
                        }
                    },
                    Backend::Vm => {
                        let program = compiler.compile(program);
                        vm.frames[0].function = Rc::new(CompiledFunction {
                            ops: program.ops,
                            params: Rc::new([]),
                        });
                        vm.frames[0].ip = 0;
                        vm.constants = program.constants.clone();
                        match vm.run() {
                            Ok(eval) => eval,
                            Err(error) => {
                                error.report(line);
                                continue;
                            }
                        }
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

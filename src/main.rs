mod chunk;
mod compiler;
mod op;
mod scanner;
mod value;
mod vm;

use crate::vm::InterpretError;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        repl();
    } else if args.len() == 2 {
        run_file(&args[1]);
    } else {
        eprintln!("Usage: lox [path]");
        process::exit(64);
    }
}

fn repl() {
    println!("Welcome to lox!");
    loop {
        print!("> ");

        // Have to flush or the prompt never gets printed.
        io::stdout().flush().unwrap();

        let mut buffer = String::new();
        let result = io::stdin().read_line(&mut buffer);
        match result {
            Ok(_) => {
                vm::interpret(&buffer);
            }
            Err(_) => {
                println!("");
            }
        }
    }
}

fn run_file(path: &str) {
    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(e) => {
            eprintln!("Error opening file '{}': {}", path, e);
            process::exit(74);
        }
    };

    match vm::interpret(&source) {
        Ok(_) => {}
        Err(InterpretError::Compile) => process::exit(65),
        Err(InterpretError::Runtime) => process::exit(70),
    }
}

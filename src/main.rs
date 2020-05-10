mod chunk;
mod compiler;
mod object;
mod op;
mod scanner;
mod value;
mod vm;

use crate::vm::InterpretError;
use std::collections::HashMap;
use std::env;
use std::error::Error;
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
    fn read_line(prompt: &str) -> Result<String, Box<dyn Error>> {
        print!("{} ", prompt);

        // Have to flush or the prompt never gets printed.
        io::stdout().flush().unwrap();
        let mut buffer = String::new();
        let _result = io::stdin().read_line(&mut buffer)?;

        Ok(buffer)
    }

    println!("Welcome to lox!");
    let mut globals = HashMap::new();
    loop {
        let result = read_line(">").map(|line| vm::interpret(&line, &mut globals));

        if result.is_err() {
            eprintln!("{}", result.err().unwrap());
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

    let mut globals = HashMap::new();
    match vm::interpret(&source, &mut globals) {
        Ok(_) => {}
        Err(InterpretError::Compile) => process::exit(65),
        Err(InterpretError::Runtime) => process::exit(70),
    }
}

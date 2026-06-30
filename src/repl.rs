use std::collections::HashMap;
use std::io::{self, BufRead, Write};

use crate::eval::expr;

pub fn run() {
    let mut vars = HashMap::new();
    let parser = expr::ProgramParser::new();
    let stdin = io::stdin();

    loop {
        print!("bonk> ");
        io::stdout().flush().unwrap();

        let mut line = String::new();
        if stdin.lock().read_line(&mut line).unwrap() == 0 {
            break;
        }
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        match parser.parse(line) {
            Ok(ast) => match ast.eval(&mut vars) {
                Ok(val) => println!("{val}"),
                Err(e) => eprintln!("Runtime error: {e}"),
            },
            Err(_) if !line.ends_with("please") => {
                eprintln!("Parse error: you forgot to say please")
            }
            Err(e) => eprintln!("Parse error: {e}"),
        }
    }
}

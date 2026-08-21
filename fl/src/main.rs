use std::{
    env,
    error::Error,
    fs,
    io::{self, BufRead, Write},
};

use fl::{interpreter::Interpreter, parser::parse};

fn main() -> Result<(), Box<dyn Error>> {
    let args = env::args().collect::<Vec<String>>();

    if args.len() > 1 {
        let filename = &args[1];
        run_file(filename)?;
    } else {
        repl()?;
    }

    Ok(())
}

fn repl() -> Result<(), Box<dyn Error>> {
    println!("repl v1.0");
    println!("type expressions to evaluate, or 'quit' to exit.\n");

    let mut interpreter = Interpreter::new();
    let (stdin, mut stdout) = (io::stdin(), io::stdout());

    loop {
        print!(">>> ");
        stdout.flush()?;

        let mut input = String::new();
        let mut line = String::new();

        if stdin.lock().read_line(&mut line)? == 0 {
            break;
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            break;
        }

        if ["exit", "quit"].contains(&trimmed) {
            println!("Goodbye!");
            break;
        }

        input.push_str(&line);

        while bracket_depth(input.as_ref()) > 0 {
            print!("... ");
            stdout.flush()?;

            line.clear();
            if stdin.lock().read_line(&mut line).unwrap() == 0 {
                break;
            }

            input.push_str(&line);
        }

        let input = input.trim();

        match parse(input) {
            Ok(program) => match interpreter.run(&program) {
                Ok(ret) => println!("{}", ret),
                Err(err) => println!("runtime input {}", err),
            },
            Err(err) => println!("parse input {}", err),
        }

        stdout.flush()?;
    }

    Ok(())
}

fn run_file(filename: &str) -> Result<(), Box<dyn Error>> {
    let source = match fs::read_to_string(filename) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", filename, e);
            return Err(e.into());
        }
    };

    match fl::run(&source) {
        Ok(value) => println!("{}", value),
        Err(e) => {
            eprintln!("Error: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}

fn bracket_depth(s: &str) -> usize {
    let mut depth = 0;
    let mut in_string = false;
    let mut prev_char = ' ';

    for c in s.chars() {
        if c == '"' && prev_char != '\\' {
            in_string = !in_string
        }

        if !in_string {
            match c {
                '(' | '[' | '{' => depth += 1,
                ')' | ']' | '}' => depth -= 1,
                _ => {}
            }
        }

        prev_char = c;
    }

    depth
}

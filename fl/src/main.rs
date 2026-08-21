use std::{
    error::Error,
    fs,
    io::{self, BufRead, Write},
    time::Instant,
};

use clap::Parser;
use fl::{interpreter::Interpreter, parser::parse};

#[derive(Parser)]
#[command(name = "fl", version, about = "A small interpreted language")]
struct Cli {
    /// Source file to execute (omit for REPL)
    file: Option<String>,

    /// Print diagnostic information (parse time, AST, etc.)
    #[arg(short, long)]
    verbose: bool,

    /// Parse the source and print the AST, then exit
    #[arg(long)]
    emit_ast: bool,

    /// Parse the source without executing
    #[arg(long)]
    parse_only: bool,

    /// Print every expression as it is evaluated
    #[arg(short, long)]
    trace: bool,

    /// Suppress REPL banner and prompts
    #[arg(short, long)]
    quiet: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    if let Some(filename) = &cli.file {
        run_file(filename, &cli)?;
    } else {
        repl(&cli)?;
    }

    Ok(())
}

fn repl(cli: &Cli) -> Result<(), Box<dyn Error>> {
    if !cli.quiet {
        println!("repl v1.0");
        println!("type expressions to evaluate, or 'quit' to exit.\n");
    }

    let mut interpreter = Interpreter::new();
    interpreter.set_trace(cli.trace);
    let (stdin, mut stdout) = (io::stdin(), io::stdout());

    loop {
        if !cli.quiet {
            print!(">>> ");
            stdout.flush()?;
        }

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
            if !cli.quiet {
                println!("Goodbye!");
            }
            break;
        }

        input.push_str(&line);

        while bracket_depth(input.as_ref()) > 0 {
            if !cli.quiet {
                print!("... ");
                stdout.flush()?;
            }

            line.clear();
            if stdin.lock().read_line(&mut line).unwrap() == 0 {
                break;
            }

            input.push_str(&line);
        }

        let input = input.trim();

        let start = Instant::now();
        match parse(input) {
            Ok(program) => {
                if cli.verbose {
                    eprintln!("parsed in {:?}", start.elapsed());
                }
                if cli.emit_ast {
                    println!("{:#?}", program);
                } else if !cli.parse_only
                    && let Err(e) = interpreter.run(&program)
                {
                    eprintln!("runtime error: {}", e);
                }
            }

            Err(err) => eprintln!("parse error: {}", err),
        }

        stdout.flush()?;
    }

    Ok(())
}

fn run_file(filename: &str, cli: &Cli) -> Result<(), Box<dyn Error>> {
    let source = match fs::read_to_string(filename) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", filename, e);
            return Err(e.into());
        }
    };

    let start = Instant::now();
    let program = match parse(&source) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("parse error: {}", e);
            return Err(e.into());
        }
    };

    if cli.verbose {
        eprintln!("parsed in {:?}", start.elapsed());
    }

    if cli.emit_ast {
        println!("{:#?}", program);
        return Ok(());
    }

    if cli.parse_only {
        return Ok(());
    }

    let mut interpreter = Interpreter::new();
    interpreter.set_trace(cli.trace);

    let run_start = Instant::now();
    if let Err(e) = interpreter.run(&program) {
        eprintln!("runtime error: {}", e);
        return Err(e.into());
    }

    if cli.verbose {
        eprintln!("executed in {:?}", run_start.elapsed());
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

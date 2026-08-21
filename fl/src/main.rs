use std::{
    error::Error,
    fs,
    io::{self, BufRead, Write},
    time::Instant,
};

use clap::Parser;
use fl::{interpreter::Interpreter, parser::parse, preprocessor::Preprocessor};

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
        let raw = fs::read_to_string(filename).map_err(|e| -> Box<dyn Error> {
            eprintln!("Error reading file '{}': {}", filename, e);
            e.into()
        })?;
        let mut proc = Preprocessor::default();
        let mut source = String::new();
        for line in raw.lines() {
            if Preprocessor::is_macro_def(line) {
                proc.define(line);
            } else {
                source.push_str(line);
                source.push('\n');
            }
        }
        proc.preprocess(&mut source);
        let mut interpreter = Interpreter::new();
        interpreter.set_trace(cli.trace);
        run_from_source(&source, &cli, &mut interpreter)?;
    } else {
        repl(&cli)?;
    }

    Ok(())
}

fn run_from_source(
    source: &str,
    cli: &Cli,
    interpreter: &mut Interpreter,
) -> Result<(), Box<dyn Error>> {
    let start = Instant::now();
    let program = match parse(source) {
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

fn repl(cli: &Cli) -> Result<(), Box<dyn Error>> {
    if !cli.quiet {
        println!("repl v1.0");
        println!("type expressions to evaluate, or 'quit' to exit.\n");
    }

    let mut interpreter = Interpreter::new();
    interpreter.set_trace(cli.trace);
    let mut proc = Preprocessor::default();
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

        if Preprocessor::is_macro_def(input) {
            proc.define(input);
        } else {
            let mut source = input.to_string();
            proc.preprocess(&mut source);
            if let Err(e) = run_from_source(&source, cli, &mut interpreter) {
                let _ = e;
            }
        }

        stdout.flush()?;
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

use calculator::{
    Result,
    compile::{Compile, interpreter::Interpreter, jit::Jit, vm::VM},
};
use rustyline::{DefaultEditor, error::ReadlineError};

#[derive(Clone, Copy, PartialEq)]
enum Backend {
    Interpreter,
    Vm,
    Jit,
}

impl Backend {
    fn name(self) -> &'static str {
        match self {
            Backend::Interpreter => "interpreter",
            Backend::Vm => "vm",
            Backend::Jit => "jit",
        }
    }

    fn parse(s: &str) -> Option<Self> {
        match s {
            "interpreter" | "i" => Some(Backend::Interpreter),
            "vm" | "v" => Some(Backend::Vm),
            "jit" | "j" => Some(Backend::Jit),
            _ => None,
        }
    }
}

fn main() -> Result<()> {
    let mut rl = DefaultEditor::new()?;
    let mut backend = Backend::Interpreter;

    println!("calculator repl");
    println!("commands: use <backend> (interpreter, vm, jit), :q to quit");
    println!("current backend: {}\n", backend.name());

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                if line == ":q" || line == ":quit" {
                    break;
                }

                if let Some(rest) = line.strip_prefix("use ") {
                    let name = rest.trim();
                    match Backend::parse(name) {
                        Some(b) => {
                            backend = b;
                            println!("switched to {}", backend.name());
                        }
                        None => println!(
                            "unknown backend '{}'. available: interpreter (i), vm (v), jit (j)",
                            name
                        ),
                    }
                    continue;
                }

                let source = format!("{};", line);
                match backend {
                    Backend::Interpreter => {
                        let res = Interpreter::from_source(&source);
                        println!("{:?}", res);
                    }
                    Backend::Vm => {
                        let res = VM::from_source(&source);
                        println!("{:?}", res);
                    }
                    Backend::Jit => {
                        let res = Jit::from_source(&source);
                        println!("{:?}", res);
                    }
                }
            }

            Err(ReadlineError::Interrupted | ReadlineError::Eof) => {
                println!("Interrupted");
                break;
            }

            Err(err) => println!("an error has occured! {:?}", err),
        }
    }

    Ok(())
}

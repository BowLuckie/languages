use std::fmt;
use std::path::PathBuf;

use calculator::compile::Compile;
use clap::{Parser, ValueEnum};

#[derive(ValueEnum, Clone)]
enum Backend {
    Interpreter,
    Vm,
    Jit,
}

impl fmt::Display for Backend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Backend::Interpreter => write!(f, "interpreter"),
            Backend::Vm => write!(f, "vm"),
            Backend::Jit => write!(f, "jit"),
        }
    }
}

#[derive(Parser)]
#[command(version, about = "A calculator with multiple compilation backends")]
struct Cli {
    /// Which backend to use
    #[arg(short, long, default_value_t = Backend::Interpreter)]
    backend: Backend,

    /// Input .calc file
    file: PathBuf,
}

fn main() {
    let cli = Cli::parse();

    let source = std::fs::read_to_string(&cli.file).unwrap_or_else(|err| {
        eprintln!("failed to read '{}': {}", cli.file.display(), err);
        std::process::exit(1);
    });

    match cli.backend {
        Backend::Interpreter => {
            let res = calculator::compile::interpreter::Interpreter::from_source(&source);
            println!("result: {:?}", res);
        }

        Backend::Vm => {
            let mut vm = calculator::compile::vm::VM::from_source(&source);
            println!("instructions: {:02X?}", vm.bytecode.instructions);
            println!("constant table: {:?}", vm.bytecode.constant);
            vm.run();
            println!("result: {:?}", vm.result());
        }

        Backend::Jit => {
            let res = calculator::compile::jit::Jit::from_source(&source);
            println!("result: {:?}", res);
        }
    }
}

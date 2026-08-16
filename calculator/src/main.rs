use calculator::compile::{Compile, interpreter::Interpreter};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("No input file was provided");
        std::process::exit(-1);
    }

    let res = Interpreter::from_source(&std::fs::read_to_string(&args[1]).unwrap());
    println!("result: {:?}", res);
}

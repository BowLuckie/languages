use std::{
    env,
    fs::{self},
};

fn main() {
    let filename = env::args().collect::<Vec<String>>()[1].to_string();
    let source = match fs::read_to_string(&filename) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", filename, e);
            std::process::exit(1);
        }
    };

    match fl::run(&source) {
        Ok(value) => println!("{}", value),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
